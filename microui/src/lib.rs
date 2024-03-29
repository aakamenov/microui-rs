#![feature(new_uninit)]
#![feature(variant_count)]

pub mod const_vec;
pub mod widget;
mod container;
mod text_buf;
mod geometry;
mod style;
mod id;

pub use geometry::*;
pub use style::*;
pub use id::Id;
pub use text_buf::TextBuf;
pub use widget::{textbox, dropdown, *};
pub use container::*;

use std::{ptr, cmp, mem, ops::Range, hash::Hash};

use const_vec::{ConstVec, ConstStr};

pub const COMMAND_LIST_SIZE: usize = 4096;
pub const ROOT_LIST_SIZE: usize = 32;
pub const CONTAINER_STACK_SIZE: usize = 32;
pub const CLIP_STACK_SIZE: usize = 32;
pub const ID_STACK_SIZE: usize = 32;
pub const LAYOUT_STACK_SIZE: usize = 16;
pub const CONTAINER_POOL_SIZE: usize = 48;
pub const TREENODE_POOL_SIZE: usize = 48;
pub const MAX_WIDTHS: usize = 16;
pub const MAX_FMT: usize = 127;
pub const MAX_TEXT_STORE: usize = 1024;

pub type DrawFrameFn = fn(ctx: &mut Context, rect: Rect, color_id: WidgetColor);

pub type LayoutWidths = [i32; MAX_WIDTHS];
type FrameIdx = u64;

macro_rules! impl_flags {
    ($visibility:vis $state:ident, $variants:ty, $size:ty) => {
        #[derive(Clone, Copy, Default, PartialEq, Debug)]
        $visibility struct $state($size);

        impl $state {
            #[inline(always)]
            pub fn is_set(&self, btn: $variants) -> bool {
                let btn = btn as $size;
                self.0 & btn == btn
            }
        
            #[inline(always)]
            #[allow(dead_code)]
            pub fn is_unset(&self, btn: $variants) -> bool {
                !self.is_set(btn)
            }
        
            #[inline(always)]
            pub fn set(&mut self, btn: $variants) {
                self.0 |= btn as $size;
            }
            
            #[inline(always)]
            pub fn unset(&mut self, btn: $variants) {
                self.0 &= !(btn as $size);
            }
        }
    };
}

pub struct Context {
    pub draw_frame: DrawFrameFn,
    pub style: Style,
    font_handler: Box<dyn TextSizeHandler>,
    cursor_icon: Option<CursorIcon>,
    hover_id: Option<Id>,
    focus_id: Option<Id>,
    last_id: Option<Id>,
    last_rect: Rect,
    last_zindex: isize,
    updated_focus: bool,
    frame: FrameIdx,
    hover_root: Option<usize>,
    next_hover_root: Option<usize>,
    scroll_target: Option<usize>,
    number_edit_buf: ConstStr<MAX_FMT>,
    number_edit_id: Option<Id>,
    command_list: ConstVec<Command, COMMAND_LIST_SIZE>,
    root_list: ConstVec<usize, ROOT_LIST_SIZE>,
    container_stack: ConstVec<usize, CONTAINER_STACK_SIZE>,
    clip_stack: ConstVec<Rect, CLIP_STACK_SIZE>,
    id_stack: ConstVec<Id, ID_STACK_SIZE>,
    layout_stack: ConstVec<Layout, LAYOUT_STACK_SIZE>,
    container_pool: ConstVec<PoolItem, CONTAINER_POOL_SIZE>,
    containers: ConstVec<Container, CONTAINER_POOL_SIZE>,
    treenode_pool: ConstVec<PoolItem, TREENODE_POOL_SIZE>,
    mouse_pos: Vec2,
    last_mouse_pos: Vec2,
    mouse_delta: Vec2,
    scroll_delta: Vec2,
    mouse_down: MouseState,
    mouse_pressed: MouseState,
    key_down: ModKeyState,
    key_pressed: ModKeyState,
    text_input: ConstStr<MAX_TEXT_STORE>
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum Icon {
    None,
    Close,
    Check,
    Collapsed,
    Expanded,
    Resize
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum CursorIcon {
    Hand,
    Text,
    Drag,
    Resize
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Response {
    pub active: bool,
    pub submit: bool,
    pub change: bool
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u16)]
pub enum ContainerOption {
    AlignCenter = 1 << 0,
    AlignRight = 1 << 1,
    NoInteract = 1 << 2,
    NoFrame = 1 << 3,
    NoResize = 1 << 4,
    NoScroll = 1 << 5,
    NoClose = 1 << 6,
    NoTitle = 1 << 7,
    HoldFocus = 1 << 8,
    AutoSize = 1 << 9,
    Popup = 1 << 10,
    Closed = 1 << 11
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum MouseButton {
    Left = 1 << 0,
    Right = 1 << 1,
    Middle = 1 << 2
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum ModKey {
    Shift = 1 << 0,
    Ctrl = 1 << 1,
    Alt = 1 << 2,
    Backspace = 1 << 3,
    Return = 1 << 4
}

impl_flags!(pub ContainerOptions, ContainerOption, u16);
impl_flags!(MouseState, MouseButton, u8);
impl_flags!(ModKeyState, ModKey, u8);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct FontId(pub u32);

#[derive(Clone, Default)]
pub struct Layout {
    body: Rect,
    next: Rect,
    pos: Vec2,
    size: Vec2,
    max: Vec2,
    widths: LayoutWidths,
    items: usize,
    item_index: usize,
    next_row: i32,
    next_type: Option<LayoutType>,
    indent: i32
}

#[derive(Clone, Copy)]
pub enum LayoutType {
    Relative,
    Absolute
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Container {
    pub rect: Rect,
    pub body: Rect,
    pub content_size: Vec2,
    pub scroll: Vec2,
    pub zindex: isize,
    pub open: bool,
    head: Option<usize>,
    tail: Option<usize>
}

pub trait TextSizeHandler {
    fn text_width(&self, id: FontId, text: &str) -> i32;
    fn text_height(&self, id: FontId) -> i32;
}

pub trait CommandHandler {
    fn clip_cmd(&mut self, rect: Rect);
    fn rect_cmd(&mut self, rect: Rect, color: Color);
    fn text_cmd(
        &mut self,
        font: FontId,
        pos: Vec2,
        color: Color,
        text: String
    );
    fn icon_cmd(
        &mut self,
        id: Icon,
        rect: Rect,
        color: Color
    );
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct WidgetInteraction {
    options: ContainerOptions,
    cursor: Option<CursorIcon>,
    retain_cursor_focus: bool
}

#[derive(Debug)]
enum Command {
    Jump(usize),
    Clip(Rect),
    Rect {
        rect: Rect,
        color: Color
    },
    Text {
        font: FontId,
        pos: Vec2,
        color: Color,
        text: String
    },
    Icon {
        id: Icon,
        rect: Rect,
        color: Color
    }
}

#[derive(Clone, Copy, Default)]
struct PoolItem {
    id: Id,
    last_update: FrameIdx
}

pub fn draw_frame(ctx: &mut Context, rect: Rect, color_id: WidgetColor) {
    ctx.draw_rect(rect, ctx.style.colors[color_id]);

    if matches!(
        color_id,
        WidgetColor::ScrollBase |
        WidgetColor::ScrollThumb |
        WidgetColor::TitleBackground
    ) {
        return;
    }

    let border_color = ctx.style.colors[WidgetColor::Border];
    if border_color.a != 0 {
        ctx.draw_box(rect.expand(1), border_color);
    }
}

impl Context {
    pub fn new(font_handler: impl TextSizeHandler + 'static) -> Box<Self> {
        let mut ctx = Box::<Self>::new_zeroed();
        let ctx_ptr = ctx.as_mut_ptr();

        // Fields with types that need Drop cannot be written to
        // through a &mut reference! We need to use addr_of_mut!
        // https://lucumr.pocoo.org/2022/1/30/unsafe-rust/
        unsafe {
            ptr::addr_of_mut!(
                (*ctx_ptr).font_handler
            ).write(Box::new(font_handler));
        }

        let mut ptr = unsafe { &mut *ctx_ptr };
        ptr.cursor_icon = None;
        ptr.draw_frame = draw_frame;
        ptr.style = Style::default();
        ptr.hover_id = None;
        ptr.focus_id = None;
        ptr.last_id = None;
        ptr.last_rect = Rect::default();
        ptr.last_zindex = 0;
        ptr.updated_focus = false;
        ptr.frame = 0;
        ptr.hover_root = None;
        ptr.next_hover_root = None;
        ptr.scroll_target = None;
        ptr.number_edit_id = None;
        ptr.mouse_pos = Vec2::ZERO;
        ptr.last_mouse_pos = Vec2::ZERO;
        ptr.mouse_delta = Vec2::ZERO;
        ptr.scroll_delta = Vec2::ZERO;
        ptr.mouse_down = MouseState::default();
        ptr.mouse_pressed = MouseState::default();
        ptr.key_down = ModKeyState::default();
        ptr.key_pressed = ModKeyState::default();

        ptr.containers.init_default();
        ptr.container_pool.init_default();
        ptr.treenode_pool.init_default();

        unsafe {
            ctx.assume_init()
        }
    }

    pub fn begin(&mut self) {
        self.command_list.clear();
        self.root_list.clear();
        self.cursor_icon = None;
        self.scroll_target = None;
        self.hover_root = self.next_hover_root.take();
        self.mouse_delta.x = self.mouse_pos.x - self.last_mouse_pos.x;
        self.mouse_delta.y = self.mouse_pos.y - self.last_mouse_pos.y;
        self.frame += 1;
    }

    pub fn end(&mut self) {
        assert_eq!(self.container_stack.len(), 0, "You probably forgot to call end_window() or end_popup()");
        assert_eq!(self.clip_stack.len(), 0);
        assert_eq!(self.id_stack.len(), 0);
        assert_eq!(self.layout_stack.len(), 0);

        if let Some(index) = self.scroll_target {
            self.containers[index].scroll.x += self.scroll_delta.x;
            self.containers[index].scroll.y += self.scroll_delta.y;
        }

        if !self.updated_focus {
            self.focus_id = None;
        }
        self.updated_focus = false;

        // Bring hover root to front if mouse was pressed
        if let Some(index) = self.next_hover_root {
            if self.mouse_any_pressed() {
                let container = &mut self.containers[index];

                if container.zindex < self.last_zindex &&
                    container.zindex >= 0
                {
                    // Bring to front
                    self.last_zindex += 1;
                    container.zindex = self.last_zindex;
                }
            }
        }

        self.key_pressed = ModKeyState::default();
        self.mouse_pressed = MouseState::default();
        self.scroll_delta = Vec2::ZERO;
        self.last_mouse_pos = self.mouse_pos;
        self.text_input.clear();

        self.root_list.sort_unstable_by(|a, b| {
            let a_zindex = self.containers[*a].zindex;
            let b_zindex = self.containers[*b].zindex;

            a_zindex.cmp(&b_zindex)
        });

        for i in 0..self.root_list.len() {
            let cnt_idx = self.root_list[i];

            // If this is the first container then make the first command jump to it.
            // Otherwise set the previous container's tail to jump to this one.
            if i == 0 {
                if let Some(cmd) = self.command_list.first_mut() {
                    if let Command::Jump(dst) = cmd {
                        *dst = self.containers[cnt_idx].head.unwrap() + 1;
                    } else {
                        panic!("Widgets must be drawn inside of a window or a popup.")
                    }
                }
            } else {
                let prev_idx = self.root_list[i - 1];
                let tail_idx = self.containers[prev_idx].tail.unwrap();

                match &mut self.command_list[tail_idx] {
                    Command::Jump(dst) => {
                        *dst = self.containers[cnt_idx].head.unwrap() + 1;
                    },
                    _ => unreachable!()
                }
            }

            // Make the last container's tail jump to the end of command list.
            if i == self.root_list.len() - 1 {
                let tail_idx = self.containers[cnt_idx].tail.unwrap();
                let commands_len = self.command_list.len();

                assert!(tail_idx < commands_len);

                match &mut self.command_list[tail_idx] {
                    Command::Jump(dst) => {
                        *dst = commands_len;
                    },
                    _ => unreachable!()
                }
            }
        }
    }

    pub fn handle_commands(&mut self, handler: &mut impl CommandHandler) {
        let mut i = 0;

        while i < self.command_list.len() {
            let cmd = unsafe {
                self.command_list.read_at(i)  
            };

            match cmd {
                Command::Clip(rect) => handler.clip_cmd(rect),
                Command::Rect { rect, color } => handler.rect_cmd(rect, color),
                Command::Icon { id, rect, color } => handler.icon_cmd(id, rect, color),
                Command::Text { font, pos, color, text } => handler.text_cmd(font, pos, color, text),
                Command::Jump(dst) => {
                    i = dst;

                    continue;
                }
            }

            i += 1;
        }

        // We must set the length to zero because we are doing a bitwise copy
        // of each command which possibly includes strings. By doing this we
        // are effectively transferring the ownership to the handler which will
        // take care of freeing them. Otherwise, begin() will attempt to drop
        // those strings the next frame which would result in a double-free.
        unsafe {
            self.command_list.set_len(0);
        }
    }

    #[inline]
    pub fn cursor_icon(&self) -> Option<CursorIcon> {
        self.cursor_icon
    }

    #[inline]
    pub fn current_frame(&self) -> FrameIdx {
        self.frame
    }

    #[inline]
    pub fn is_focused(&self, id: Id) -> bool {
        self.focus_id.map_or(false, |x| x == id)
    }

    #[inline]
    pub fn is_hovered(&self, id: Id) -> bool {
        self.hover_id.map_or(false, |x| x == id)
    }

    #[inline]
    pub fn set_focus(&mut self, id: Option<Id>) {
        self.focus_id = id;
        self.updated_focus = true;
    }

    #[inline]
    pub fn create_id(&mut self, item: &impl Hash) -> Id {
        let entropy = self.id_stack.last().copied().unwrap_or(Id::default());

        let id = Id::new(item, entropy.0);
        self.last_id = Some(id);

        id
    }

    pub fn push_id(&mut self, item: &impl Hash) -> Id {
        let id = self.create_id(item);
        self.id_stack.push(id);

        id
    }

    #[inline]
    pub fn pop_id(&mut self) -> Option<Id> {
        self.id_stack.pop()
    }

    #[inline]
    pub fn last_id(&self) -> Option<Id> {
        self.last_id
    }

    #[inline]
    pub fn last_rect(&self) -> Rect {
        self.last_rect
    }

    #[inline]
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

    #[inline]
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    #[inline]
    pub fn mouse_pos(&self) -> Vec2 {
        self.mouse_pos
    }

    #[inline]
    pub fn last_mouse_pos(&self) -> Vec2 {
        self.last_mouse_pos
    }

    #[inline]
    pub fn mouse_any_pressed(&self) -> bool {
        self.mouse_pressed != MouseState::default()
    }

    #[inline]
    pub fn mouse_any_down(&self) -> bool {
        self.mouse_down != MouseState::default()
    }

    #[inline]
    pub fn mouse_pressed(&self, btn: MouseButton) -> bool {
        self.mouse_pressed.is_set(btn)
    }

    #[inline]
    pub fn mouse_down(&self, btn: MouseButton) -> bool {
        self.mouse_down.is_set(btn)
    }

    #[inline]
    pub fn key_down(&self, key: ModKey) -> bool {
        self.key_down.is_set(key)
    }

    #[inline]
    pub fn key_pressed(&self, key: ModKey) -> bool {
        self.key_pressed.is_set(key)
    }

    #[inline]
    pub fn key_up(&self, key: ModKey) -> bool {
        self.key_down.is_unset(key)
    }

    #[inline]
    pub fn check_clip(&self, rect: Rect) -> Clip {
        if let Some(last) = self.clip_stack.last() {
            last.clip(rect)
        } else {
            Clip::None
        }
    }

    #[inline]
    pub fn push_clip_rect(&mut self, rect: Rect) {
        let last = self.clip_rect();
        self.clip_stack.push(rect.intersect(last));
    }

    #[inline]
    pub fn pop_clip_rect(&mut self) -> Option<Rect> {
        self.clip_stack.pop()
    }

    #[inline]
    pub fn set_clip(&mut self, rect: Rect) {
        self.command_list.push(Command::Clip(rect));
    }

    #[inline]
    pub fn clip_rect(&self) -> Rect {
        *self.clip_stack.last().unwrap()
    }
}

//============================================================================
// Draw
//============================================================================

impl Context {
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        let rect = rect.intersect(self.clip_rect());

        if rect.w > 0 && rect.h > 0 {
            self.command_list.push(Command::Rect {
                rect,
                color
            });
        }
    }

    #[inline]
    pub fn draw_box(&mut self, r: Rect, color: Color) {
        self.draw_rect(rect(r.x + 1, r.y, r.w - 2, 1), color);
        self.draw_rect(rect(r.x + 1, r.y + r.h - 1, r.w - 2, 1), color);
        self.draw_rect(rect(r.x, r.y, 1, r.h), color);
        self.draw_rect(rect(r.x + r.w - 1, r.y, 1, r.h), color);
    }

    pub fn draw_text(&mut self, font: FontId, text: impl Into<String>, pos: Vec2, color: Color) {
        let text: String = text.into();

        let rect = rect(
            pos.x,
            pos.y,
            self.font_handler.text_width(font, &text),
            self.font_handler.text_height(font)
        );

        let clip = self.check_clip(rect);
        match clip {
            Clip::None => {},
            Clip::All => { return; },
            Clip::Part => self.set_clip(self.clip_rect())
        }

        self.command_list.push(Command::Text {
            font,
            pos,
            color,
            text
        });

        // Reset clipping if it was set.
        if !matches!(clip, Clip::None) {
            self.set_clip(Rect::UNCLIPPED);
        }
    }

    pub fn draw_icon(&mut self, id: Icon, rect: Rect, color: Color) {
        let clip = self.check_clip(rect);
        match clip {
            Clip::None => {},
            Clip::All => { return; },
            Clip::Part => self.set_clip(self.clip_rect())
        }

        self.command_list.push(Command::Icon {
            id,
            rect,
            color
        });

        // Reset clipping if it was set.
        if !matches!(clip, Clip::None) {
            self.set_clip(Rect::UNCLIPPED);
        }
    }
}

//============================================================================
// Container
//============================================================================

impl Context {
    #[inline]
    pub fn current_container_index(&self) -> Option<usize> {
        self.container_stack.last().copied()
    }

    #[inline]
    pub fn current_container(&self) -> &Container {
        let index = self.current_container_index().unwrap();

        &self.containers[index]
    }

    #[inline]
    pub fn current_container_mut(&mut self) -> &mut Container {
        let index = self.current_container_index().unwrap();

        &mut self.containers[index]
    }

    #[inline]
    pub fn container_index_by_name(
        &mut self,
        name: &str,
        options: ContainerOptions
    ) -> Option<usize> {
        let id = self.create_id(&name);

        self.get_container(id, options)
    }

    #[inline(always)]
    pub fn container(&self, index: usize) -> &Container {
        &self.containers[index]
    }

    #[inline(always)]
    pub fn container_mut(&mut self, index: usize) -> &mut Container {
        &mut self.containers[index]
    }

    #[inline(always)]
    pub fn containers_len(&self) -> usize {
        self.containers.len()
    }

    #[inline]
    pub fn bring_to_front(&mut self, index: usize) {
        self.last_zindex += 1;
        self.containers[index].zindex = self.last_zindex;
    }

    fn get_container(&mut self, id: Id, options: ContainerOptions) -> Option<usize> {
        let index = self.container_pool.find_by_id(id);

        if let Some(index) = index {
            if self.containers[index].open || options.is_unset(ContainerOption::Closed) {
                self.container_pool[index].last_update = self.frame;
            }

            return Some(index);
        }

        if options.is_set(ContainerOption::Closed) {
            return None;
        }

        if let Some(index) = self.init_container_pool(id) {
            let container = &mut self.containers[index];
            *container = Container::default();
            container.open = true;

            // Bring to front
            self.last_zindex += 1;
            container.zindex = self.last_zindex;

            Some(index)
        } else {
            None
        }
    }

    #[inline]
    fn pop_container(&mut self) {
        if let Some(layout) = self.layout_stack.pop() {
            if let Some(index) = self.current_container_index() {
                self.containers[index].content_size.x = layout.max.x - layout.body.x;
                self.containers[index].content_size.y = layout.max.y - layout.body.y;
            }
        }

        self.container_stack.pop();
        self.pop_id();
    }
}

//============================================================================
// Pool
//============================================================================

impl Context {
    #[inline]
    pub fn init_treenode_pool(&mut self, id: Id) -> Option<usize> {
        self.treenode_pool.init(id, self.frame)
    }

    #[inline]
    pub fn init_container_pool(&mut self, id: Id) -> Option<usize> {
        self.container_pool.init(id, self.frame)
    }
}

impl<const N: usize> ConstVec<PoolItem, N> {
    fn init(&mut self, id: Id, frame: FrameIdx) -> Option<usize> {
        let mut index = None;
        let mut f = frame;

        for (i, item) in self.iter().enumerate() {
            if item.last_update < f {
                f = item.last_update;
                index = Some(i);
            }
        }

        if let Some(i) = index {
            self[i].id = id;
            self[i].last_update = frame;

            index
        } else {
            None
        }
    }

    #[inline]
    fn find_by_id(&self, id: Id) -> Option<usize> {
        self.iter().position(|x| x.id == id)
    }
}

//============================================================================
// Input
//============================================================================

impl Context {
    #[inline]
    pub fn input_mouse_move(&mut self, pos: Vec2) {
        self.mouse_pos = pos;
    }

    #[inline]
    pub fn input_mouse_down(&mut self, pos: Vec2, btn: MouseButton) {
        self.input_mouse_move(pos);
        self.mouse_down.set(btn);
        self.mouse_pressed.set(btn);
    }

    #[inline]
    pub fn input_mouse_up(&mut self, pos: Vec2, btn: MouseButton) {
        self.input_mouse_move(pos);
        self.mouse_down.unset(btn);
    }

    #[inline]
    pub fn input_scroll(&mut self, delta: Vec2) {
        self.scroll_delta.x += delta.x;
        self.scroll_delta.y += delta.y;
    }

    #[inline]
    pub fn input_key_down(&mut self, key: ModKey) {
        self.key_down.set(key);
        self.key_pressed.set(key);
    }

    #[inline]
    pub fn input_key_up(&mut self, key: ModKey) {
        self.key_down.unset(key);
    }

    /// The maximum size of the backing store is [`MAX_TEXT_STORE`].
    /// Returns the number of bytes written.
    #[inline]
    pub fn input_text(&mut self, text: &str) -> usize {
        self.text_input.push_str(text)
    }
}

//============================================================================
// Layout
//============================================================================

impl Context {
    #[inline]
    pub fn layout_begin_column(&mut self) {
        let next = self.layout_next();
        self.push_layout(next, Vec2::ZERO);
    }

    pub fn layout_end_column(&mut self) {
        let b = self.layout_stack.pop().unwrap();
        let a = self.layout_stack.last_mut().unwrap();
        
        a.pos.x = cmp::max(a.pos.x, b.pos.x + b.body.x - a.body.x);
        a.next_row = cmp::max(a.next_row, b.next_row + b.body.y - a.body.y);
        a.max.x = cmp::max(a.max.x, b.max.x);
        a.max.y = cmp::max(a.max.y, b.max.y);
    }

    #[inline]
    pub fn layout_row(&mut self, widths: &[i32], height: i32) {
        let layout = self.layout_stack.last_mut().unwrap();
        layout.row(widths, height);
    }

    #[inline]
    pub fn layout_row_items(&mut self, items: usize, height: i32) {
        let layout = self.layout_stack.last_mut().unwrap();
        layout.row_items(items, height);
    }

    #[inline]
    pub fn layout_set_next(&mut self, rect: Rect, ty: LayoutType) {
        let layout = self.layout_stack.last_mut().unwrap();
        layout.set_next(rect, ty);
    }

    pub fn layout_next(&mut self) -> Rect {
        let layout = self.layout_stack.last_mut().unwrap();

        let mut result = if layout.next_type.is_some() {
            let ty = layout.next_type.take().unwrap();
            let result = layout.next;

            if let LayoutType::Absolute = ty {
                self.last_rect = result;

                return result;
            }

            result
        } else {
            if layout.item_index == layout.items {
                layout.row_items(layout.items, layout.size.y);
            }

            let mut result = rect(
                layout.pos.x,
                layout.pos.y,
                if layout.items > 0 {
                    layout.widths[layout.item_index]
                } else {
                    layout.size.x
                },
                layout.size.y
            );

            let style = &self.style;

            if result.w == 0 {
                result.w = style.size.x + style.padding as i32 * 2;
            }

            if result.h == 0 {
                result.h = style.size.y + style.padding as i32 * 2;
            }

            if result.w < 0 {
                result.w += layout.body.w - result.x + 1;
            }

            if result.h < 0 {
                result.h += layout.body.h - result.y + 1;
            }

            layout.item_index += 1;

            result
        };

        let spacing = (self.style.spacing) as i32;
        layout.pos.x += result.w + spacing;
        layout.next_row = cmp::max(layout.next_row, result.y + result.h + spacing);

        result.x += layout.body.x;
        result.y += layout.body.y;

        layout.max.x = cmp::max(layout.max.x, result.x + result.w);
        layout.max.y = cmp::max(layout.max.y, result.y + result.h);

        self.last_rect = result;

        result
    }

    fn push_layout(&mut self, body: Rect, scroll: Vec2) {
        let mut layout = Layout {
            body: rect(body.x - scroll.x, body.y - scroll.y, body.w, body.h),
            max: vec2(-0x1000000, -0x1000000),
            ..Layout::default()
        };
        layout.row(&[0], 0);

        self.layout_stack.push(layout);
    }
}

impl Layout {
    #[inline]
    pub fn width(&mut self, width: i32) {
        self.size.x = width
    }

    #[inline]
    pub fn height(&mut self, height: i32) {
        self.size.y = height
    }

    pub fn row(&mut self, widths: &[i32], height: i32) {
        if !widths.is_empty() {
            assert!(widths.len() <= MAX_WIDTHS);

            unsafe {
                ptr::copy(
                    widths.as_ptr(),
                    self.widths.as_mut_ptr(),
                    widths.len()
                );
            }
        }

        self.row_items(widths.len(), height);
    }

    #[inline]
    pub fn row_items(&mut self, items: usize, height: i32) {
        self.items = items;
        self.pos = vec2(self.indent, self.next_row);
        self.size.y = height;
        self.item_index = 0;
    }

    #[inline]
    pub fn set_next(&mut self, rect: Rect, ty: LayoutType) {
        self.next = rect;
        self.next_type = Some(ty);
    }
}

//============================================================================
// Widgets
//============================================================================

impl Context {
    #[inline(always)]
    pub fn w(&mut self, widget: impl Widget) -> Response {
        widget.draw(self)
    }

    /// `color_id` must be either WidgetColor::Button or WidgetColor::Base.
    pub fn draw_widget_frame(
        &mut self,
        id: Id,
        rect: Rect,
        color_id: WidgetColor,
        options: ContainerOptions
    ) {
        if options.is_set(ContainerOption::NoFrame) {
            return;
        }

        assert!(matches!(color_id, WidgetColor::Button | WidgetColor::Base));

        let color_id = if self.is_focused(id) {
            2
        } else if self.is_hovered(id) {
            1
        } else {
            0
        } + color_id as u8;

        (self.draw_frame)(self, rect, unsafe { mem::transmute(color_id) });
    }

    /// Returns the [`Rect`] of the measured text with clipping **taken into account**.
    pub fn draw_widget_text(
        &mut self,
        text: impl Into<String>,
        rect: Rect,
        color_id: WidgetColor,
        options: ContainerOptions
    ) -> Rect {
        let text: String = text.into();

        let font = self.style.font;
        let width = self.font_handler.text_width(font, &text);
        let height = self.font_handler.text_height(font);

        self.push_clip_rect(rect);

        let mut pos = vec2(0, rect.y + (rect.h - height) / 2);
        pos.x = if options.is_set(ContainerOption::AlignCenter) {
            rect.x + (rect.w - width) / 2
        } else if options.is_set(ContainerOption::AlignRight) {
            rect.x + rect.w - width - self.style.padding as i32
        } else {
            rect.x + self.style.padding as i32
        };

        self.draw_text(
            font,
            text,
            pos,
            self.style.colors[color_id]
        );

        self.pop_clip_rect();

        let x = cmp::max(rect.x, pos.x);
        let y = cmp::max(rect.y, pos.y);
        let width = cmp::min(width, rect.w);
        let height = cmp::min(height, rect.h);

        crate::rect(x, y, width, height)
    }

    pub fn is_mouse_over(&self, rect: Rect) -> bool {
        rect.overlaps(self.mouse_pos) &&
            self.clip_rect().overlaps(self.mouse_pos) &&
            self.in_hover_root()
    }

    pub fn update_widget(&mut self, id: Id, rect: Rect, interact: WidgetInteraction) {
        let currently_focused = self.is_focused(id);

        if currently_focused {
            self.updated_focus = true;
        }

        if interact.options.is_set(ContainerOption::NoInteract) {
            return;
        }

        let mouse_over = self.is_mouse_over(rect);

        if mouse_over && !self.mouse_any_down() {
            self.hover_id = Some(id);
        }

        if currently_focused {
            if self.mouse_any_pressed() && !mouse_over {
                self.set_focus(None);
            }

            if !self.mouse_any_down() && interact.options.is_unset(ContainerOption::HoldFocus) {
                self.set_focus(None);
            }
        }

        if self.is_hovered(id) {
            if self.mouse_any_pressed() {
                self.set_focus(Some(id));
            } else if !mouse_over {
                self.hover_id = None;
            }
        }

        if interact.cursor.is_some() {
            // We don't want to change the cursor if another widget
            // wants to retain its cursor while focused.
            let hovered = self.is_hovered(id) && self.cursor_icon.is_none();

            if (self.is_focused(id) && interact.retain_cursor_focus) || hovered {
                self.cursor_icon = interact.cursor;
            } 
        }
    }

    pub fn text(&mut self, text: impl Into<String>) {
        let text: String = text.into();
        let color = self.style.colors[WidgetColor::Text];

        self.layout_begin_column();

        let font = self.style.font;

        let height = self.font_handler.text_height(font);
        self.layout_row(&[-1], height);

        let mut slice = &text[..];

        while slice.len() > 0 {
            let mut w = 0;
            let mut start = 0;
            let mut end = slice.len();
            let rect = self.layout_next();

            for (i, c) in slice.char_indices().filter(|x| x.1 == ' ' || x.1 == '\n') {
                let word = &slice[start..i];
                w += self.font_handler.text_width(font, word);

                if w > rect.w && start != 0 {
                    end = start;
                    break;
                }

                w += self.font_handler.text_width(font, &slice[i..i+1]);

                if c == '\n' {
                    end = i + 1;
                    break;
                }

                start = i + 1;
            }

            self.draw_text(
                font,
                &slice[..end],
                vec2(rect.x, rect.y),
                color
            );

            slice = &slice[end..];
        }

        self.layout_end_column();
    }

    /// Shorthand for `Label::new(text)`.
    #[inline]
    pub fn label(&mut self, text: impl Into<String>) {
        Label::new(text).draw(self);
    }

    /// Shorthand for `ClickableLabel::new(text)`.
    #[inline]
    pub fn clickable_label(&mut self, text: impl Into<String>) -> bool {
        ClickableLabel::new(text).draw(self).submit
    }

    /// Shorthand for `Button::new(text)`.
    #[inline]
    pub fn button(&mut self, text: impl Into<String>) -> bool {
        Button::new(text).draw(self).submit
    }

    /// Shorthand for `Checkbox::new(label, &mut checked)`.
    /// 
    /// Returns `true` if the checked state changed.
    #[inline]
    pub fn checkbox(&mut self, label: impl Into<String>, checked: &mut bool) -> bool {
        Checkbox::new(label, checked).draw(self).change
    }

    /// Shorthand for `Textbox::new(buf)`
    #[inline]
    pub fn textbox(&mut self, buf: &mut impl TextBuf) -> Response {
        TextBox::new(buf).draw(self)
    }

    /// Shorthand for `Slider::new(&mut value, range)`.
    /// 
    /// Returns `true` if the value changed.
    #[inline]
    pub fn slider(
        &mut self,
        value: &mut f64,
        range: Range<f64>
    ) -> bool {
        Slider::new(value, range).draw(self).change
    }

    /// Shorthand for `DragValue::new(&mut value, step)`.
    /// 
    /// Returns `true` if the value changed.
    #[inline]
    pub fn drag_value(
        &mut self,
        value: &mut f64,
        step: f64
    ) -> bool {
        DragValue::new(value, step).draw(self).change
    }

    /// Shorthand for `Dropdown::new(&mut state, items)`.
    /// 
    /// Returns `true` if a value was selected.
    #[inline]
    pub fn dropdown<T: AsRef<str>>(
        &mut self,
        state: &mut dropdown::State,
        items: &[T]
    ) -> bool {
        Dropdown::new(state, items).draw(self).submit
    }

    #[inline]
    pub fn header(
        &mut self,
        label: impl Into<String>,
        expanded: bool
    ) -> bool {
        self.header_impl(label, false, expanded)
    }

    pub fn begin_window(
        &mut self,
        title: impl Into<String>,
        mut rect: Rect,
        options: ContainerOptions
    ) -> bool {
        let title: String = title.into();
        assert!(!title.is_empty(), "Window title string is empty.");

        let id = self.create_id(&title);
        let cnt_idx = self.get_container(id, options);

        if cnt_idx.is_none() {
            return false;
        }

        let cnt_idx = cnt_idx.unwrap();

        if !self.containers[cnt_idx].open {
            return false;
        }

        self.id_stack.push(id);

        if self.containers[cnt_idx].rect.w == 0 {
            self.containers[cnt_idx].rect = rect;
        }

        self.begin_root_container(cnt_idx);

        rect = self.containers[cnt_idx].rect;
        let mut body = rect;

        if options.is_unset(ContainerOption::NoFrame) {
            (self.draw_frame)(self, rect, WidgetColor::WindowBackground);
        }

        // Title bar
        if options.is_unset(ContainerOption::NoTitle) {
            let mut title_rect = rect;
            title_rect.h = self.style.title_height as i32;

            (self.draw_frame)(self, title_rect, WidgetColor::TitleBackground);

            // Title text
            let id = self.create_id(&"!title");
            self.update_widget(id, title_rect, WidgetInteraction::from(options));
            self.draw_widget_text(title, title_rect, WidgetColor::TitleText, options);

            if self.is_focused(id) && self.mouse_down.is_set(MouseButton::Left) {
                self.containers[cnt_idx].rect.x += self.mouse_delta.x;
                self.containers[cnt_idx].rect.y += self.mouse_delta.y;
            }

            body.y += title_rect.h;
            body.h -= title_rect.h;

            // Close button
            if options.is_unset(ContainerOption::NoClose) {
                let id = self.create_id(&"!close");
                let r = Rect {
                    x: title_rect.x + title_rect.w - title_rect.h,
                    y: title_rect.y,
                    w: title_rect.h,
                    h: title_rect.h
                };

                title_rect.w -= r.w;

                self.draw_icon(Icon::Close, r, self.style.colors[WidgetColor::TitleText]);
                self.update_widget(
                    id,
                    r,
                    WidgetInteraction::from(options).cursor(CursorIcon::Hand)
                );

                if self.mouse_pressed.is_set(MouseButton::Left) && self.is_focused(id) {
                    self.containers[cnt_idx].open = false;
                }
            }
        }

        if options.is_unset(ContainerOption::NoResize) {
            let sz = self.style.footer_height as i32;
            let id = self.create_id(&"!resize");
            let r = Rect {
                x: rect.x + rect.w - sz,
                y: rect.y + rect.h - sz,
                w: sz,
                h: sz
            };

            self.draw_icon(Icon::Resize, r, self.style.colors[WidgetColor::Text]);
            self.update_widget(
                id,
                r,
                WidgetInteraction::from(options)
                    .cursor(CursorIcon::Resize)
                    .retain_cursor_focus()
            );

            if self.is_focused(id) && self.mouse_down.is_set(MouseButton::Left) {
                let cnt_rect = self.containers[cnt_idx].rect;

                self.containers[cnt_idx].rect.w = cmp::max(96, cnt_rect.w + self.mouse_delta.x);
                self.containers[cnt_idx].rect.h = cmp::max(64, cnt_rect.h + self.mouse_delta.y);
            }

            body.h -= sz;
        }

        self.push_container_body(cnt_idx, body, options);

        if options.is_set(ContainerOption::AutoSize) {
            let r = self.layout_stack.last().unwrap().body;
            let cnt_rect = self.containers[cnt_idx].rect;
            let content_size = self.containers[cnt_idx].content_size;

            self.containers[cnt_idx].rect.w = content_size.x + (cnt_rect.w - r.w);
            self.containers[cnt_idx].rect.h = content_size.y + (cnt_rect.h - r.h);
        }

        // Close if this is a popup window and elsewhere was clicked.
        if options.is_set(ContainerOption::Popup) &&
            self.mouse_any_pressed() &&
            self.hover_root.map_or(false, |x| x != cnt_idx)
        {
            self.containers[cnt_idx].open = false;
        }

        self.push_clip_rect(self.containers[cnt_idx].body);

        return true;
    }

    #[inline]
    pub fn end_window(&mut self) {
        self.pop_clip_rect();
        self.end_root_container();
    }

    pub fn begin_panel(
        &mut self,
        name: impl Into<String>,
        options: ContainerOptions
    ) -> bool {
        let name: String = name.into();
        assert!(!name.is_empty(), "Panel name string is empty.");

        let id = self.push_id(&name);
        let cnt_idx = self.get_container(id, options);

        if cnt_idx.is_none() {
            return false;
        }

        let cnt_idx = cnt_idx.unwrap();

        let rect = self.layout_next();
        self.containers[cnt_idx].rect = rect;

        if options.is_unset(ContainerOption::NoFrame) {
            (self.draw_frame)(self, rect, WidgetColor::PanelBackground);
        }

        self.container_stack.push(cnt_idx);
        self.push_container_body(cnt_idx, rect, options);
        self.push_clip_rect(self.containers[cnt_idx].body);

        return true;
    }

    #[inline]
    pub fn end_panel(&mut self) {
        self.pop_clip_rect();
        self.pop_container();
    }

    fn header_impl(
        &mut self,
        label: impl Into<String>,
        is_treenode: bool,
        expanded: bool
    ) -> bool {
        let label: String = label.into();
        let id = self.create_id(&label);

        let index = self.treenode_pool.find_by_id(id);
        let mut active = index.is_some();

        let expanded = if expanded {
            !active
        } else {
            active
        };

        self.layout_row(&[-1], 0);

        let mut r = self.layout_next();
        self.update_widget(id, r, WidgetInteraction::default());

        if self.mouse_pressed.is_set(MouseButton::Left) && self.is_focused(id) {
            active = !active;
        }

        if let Some(index) = index {
            if active {
                self.treenode_pool[index].last_update = self.frame;
            } else {
                self.treenode_pool[index] = PoolItem::default();
            }
        } else if active {
            self.init_treenode_pool(id);
        }

        if is_treenode && self.is_focused(id) {
            (self.draw_frame)(self, r, WidgetColor::ButtonHover);
        } else {
            self.draw_widget_frame(id, r, WidgetColor::Button, ContainerOptions::default());
        }

        self.draw_icon(
            if expanded {
                Icon::Expanded
            } else {
                Icon::Collapsed
            },
            rect(r.x, r.y, r.h, r.h),
            self.style.colors[WidgetColor::Text]
        );

        let padding = self.style.padding as i32; 
        r.x += r.h - padding;
        r.w -= r.h - padding;

        self.draw_widget_text(label, r, WidgetColor::Text, ContainerOptions::default());

        if expanded {
            true
        } else {
            false
        }
    }

    fn scrollbars(
        &mut self,
        cnt_idx: usize,
        body: &mut Rect,
    ) {
        let scrollbar_size = self.style.scrollbar_size as i32;
        let padding = self.style.padding as i32;

        let mut content_size = self.containers[cnt_idx].content_size;
        content_size.x += padding * 2;
        content_size.y += padding * 2;

        self.push_clip_rect(*body);

        let container = &self.containers[cnt_idx];
        // Resize body to make room for scrollbars.
        if content_size.y > container.body.h {
            body.w -= scrollbar_size;
        }

        if content_size.x > container.body.w {
            body.h -= scrollbar_size;
        }

        self.scrollbar_v(cnt_idx, body, content_size, "!scrollbarv");
        self.scrollbar_h(cnt_idx, body, content_size, "!scrollbarh");

        self.pop_clip_rect();
    }

    fn push_container_body(
        &mut self,
        cnt_idx: usize,
        mut body: Rect,
        options: ContainerOptions
    ) {
        if options.is_unset(ContainerOption::NoScroll) {
            self.scrollbars(cnt_idx, &mut body);
        }

        self.push_layout(
            body.expand(-(self.style.padding as i32)),
            self.containers[cnt_idx].scroll
        );
        self.containers[cnt_idx].body = body;
    }

    fn begin_root_container(&mut self, cnt_idx: usize) {
        self.container_stack.push(cnt_idx);

        // Push container to roots list and push head command.
        self.root_list.push(cnt_idx);
        self.command_list.push(Command::Jump(0));

        self.containers[cnt_idx].head = Some(self.command_list.len() - 1);

        // Set as hover root if the mouse is overlapping this container
        // and it has a higher zindex than the current hover root.
        if self.containers[cnt_idx].rect.overlaps(self.mouse_pos) &&
            self.next_hover_root.map_or(
                true,
                |x| self.containers[cnt_idx].zindex > self.containers[x].zindex
            )
        {
            self.next_hover_root = Some(cnt_idx);
        }

        // Clipping is reset here in case a root-container is made within
        // another root-containers's begin/end block; this prevents the inner
        // root-container being clipped to the outer.
        self.clip_stack.push(Rect::UNCLIPPED);
    }

    fn end_root_container(&mut self) {
        // Push tail 'goto' jump command and set head 'skip' command.
        // The final steps on initing these are done in end()
        let index = self.current_container_index().unwrap();
        self.command_list.push(Command::Jump(0));

        self.containers[index].tail = Some(self.command_list.len() - 1);

        let head = self.containers[index].head.expect("Called end_root_container() before begin_root_container()");
        let commands_len = self.command_list.len();

        match &mut self.command_list[head] {
            Command::Jump(dst) => {
                *dst = commands_len;
            },
            _ => unreachable!()
        }

        self.pop_clip_rect();
        self.pop_container();
    }

    fn in_hover_root(&self) -> bool {
        if self.hover_root.is_none() {
            return false;
        }

        let hover_root = self.hover_root.as_ref().unwrap();

        for index in self.container_stack.iter().rev() {
            if index == hover_root {
                return true;
            }

            // Only root containers have their `head` field set
            // so stop searching if we've reached the current root container
            if self.containers[*index].head.is_some() {
                return false;
            }
        }

        false
    }
}

macro_rules! scrollbar {
    ($name:ident, $x:ident, $y:ident, $w:ident, $h:ident) => {
        fn $name(
            &mut self,
            cnt_idx: usize,
            body: &mut Rect,
            content_size: Vec2,
            id_str: &'static str,
        ) {
            let maxscroll = content_size.$y - body.$h;

            if maxscroll > 0 && body.$h > 0 {
                let id = self.create_id(&id_str);

                let mut base = *body;
                base.$x = body.$x + body.$w;
                base.$w = self.style.scrollbar_size as i32;

                self.update_widget(id, base, WidgetInteraction::default());

                if self.is_focused(id) && self.mouse_down.is_set(MouseButton::Left) {
                    self.containers[cnt_idx].scroll.$y += self.mouse_delta.$y * content_size.$y / base.$h;
                }

                self.containers[cnt_idx].scroll.$y = self.containers[cnt_idx].scroll.$y.clamp(0, maxscroll);

                (self.draw_frame)(self, base, WidgetColor::ScrollBase);

                let mut thumb = base;
                thumb.$h = cmp::max(self.style.thumb_size as i32, base.$h * body.$h / content_size.$y);
                thumb.$y += self.containers[cnt_idx].scroll.$y * (base.$h - thumb.$h) / maxscroll;

                (self.draw_frame)(self, thumb, WidgetColor::ScrollThumb);

                // Set this as the scroll_target (will get scrolled on mousewheel)
                // if the mouse is over it
                if self.is_mouse_over(*body) {
                    self.scroll_target = Some(cnt_idx);
                }
            } else {
                self.containers[cnt_idx].scroll.$y = 0;
            }
        }
    };
}

impl Context {
    scrollbar!(scrollbar_v, x, y, w, h);
    scrollbar!(scrollbar_h, y, x, h, w);
}

impl WidgetInteraction {
    #[inline]
    pub fn cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = Some(cursor);

        self
    }

    /// The cursor should remain even if the widget is not being
    /// hovered as long as it has focus. Only has effect if [`WidgetInteraction::cursor`]
    /// was set.
    /// 
    /// For example, this should be set for widgets like window resize
    /// or drag value because the mouse can leave the widget bounds while
    /// dragging but we still want to display the cursor icon since the
    /// action is still in progress.
    /// 
    /// But for textboxes we don't want to retain cursor focus because
    /// even if the textbox has focus we only want to display the text
    /// cursor only when we are hovering over it.
    #[inline]
    pub fn retain_cursor_focus(mut self) -> Self {
        self.retain_cursor_focus = true;

        self
    }

    #[inline]
    pub fn no_interact(mut self) -> Self {
        self.options.set(ContainerOption::NoInteract);

        self
    }

    #[inline]
    pub fn hold_focus(mut self) -> Self {
        self.options.set(ContainerOption::HoldFocus);

        self
    }
}

impl From<ContainerOptions> for WidgetInteraction {
    fn from(options: ContainerOptions) -> Self {
        Self {
            options,
            cursor: None,
            retain_cursor_focus: false
        }
    }
}
