#![feature(new_uninit)]

pub mod const_vec;
mod geometry;
mod style;
mod id;

pub use geometry::*;
pub use style::*;
pub use id::Id;

use std::{ptr, cmp, hash::Hash};

use geometry::{Rect, Vec2, vec2, rect};
use style::{Style, Color, WidgetColor};
use const_vec::ConstVec;

pub const COMMAND_LIST_SIZE: usize = 256 * 1024;
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
pub type TextWidthFn = fn(Font, &str) -> u16;
pub type TextHeightFn = fn(Font) -> u16;

pub type FrameIdx = u64;
pub type LayoutWidths = [i32; MAX_WIDTHS];

macro_rules! impl_flags {
    ($visibility:vis $state:ident, $variants:ty, $size:ty) => {
        #[derive(Clone, Copy, Default, PartialEq)]
        $visibility struct $state($size);

        impl $state {
            #[inline(always)]
            pub fn is_set(&self, btn: $variants) -> bool {
                let btn = btn as $size;
                self.0 & btn == btn
            }
        
            #[inline(always)]
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
    text_width: TextWidthFn,
    text_height: TextHeightFn,
    style: Style,
    hover_id: Option<Id>,
    focus_id: Option<Id>,
    last_id: Option<Id>,
    last_rect: Rect,
    last_zindex: isize,
    updated_focus: bool,
    frame: FrameIdx,
    hover_root: Option<Container>,
    next_hover_root: Option<Container>,
    scroll_target: Option<Container>,
    number_edit_buf: [u8; MAX_FMT],
	number_edit_len: usize,
	number_edit_id: Option<Id>,
    command_list: ConstVec<Command, COMMAND_LIST_SIZE>,
    root_list: ConstVec<Container, ROOT_LIST_SIZE>,
    container_stack: ConstVec<Container, CONTAINER_STACK_SIZE>,
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
    mouse_released: MouseState,
    key_down: ModKeyState,
    key_pressed: ModKeyState,
    text_input: ConstVec<u8, MAX_TEXT_STORE>
}

pub enum Icon {
    None,
    Close,
    Check,
    Collapsed,
    Expanded,
    Resize
}

pub enum Response {
    Active,
    Submit,
    Change
}

#[derive(Clone, Copy, PartialEq)]
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
	Closed = 1 << 11,
	Expanded = 1 << 12,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MouseButton {
    Left = 1 << 0,
    Right = 1 << 1,
    Middle = 1 << 2
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ModKey {
    Shift = 1 << 0,
    Ctrl = 1 << 1,
    Alt = 1 << 2,
    Backspace = 1 << 3,
    Enter = 1 << 4
}

impl_flags!(pub ContainerOptions, ContainerOption, u16);
impl_flags!(MouseState, MouseButton, u8);
impl_flags!(ModKeyState, ModKey, u8);

pub struct Font;

pub enum Command {
    Jump(*const Self),
    Clip(Rect),
    Rect {
        rect: Rect,
        color: Color
    },
    Text {
        font: Font,
        pos: Vec2,
        color: Color,
        text: String
    },
    Icon {
        rect: Rect,
        color: Color
    }
}

#[derive(Default)]
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

#[derive(Clone)]
pub struct Container {
    pub rect: Rect,
    pub body: Rect,
    pub content_size: Vec2,
    pub scroll: Vec2,
    pub zindex: isize,
    pub open: bool,
    head: *const Command,
    tail: *const Command
}

#[derive(Clone, Copy, Default)]
struct PoolItem {
    id: Id,
    last_update: FrameIdx
}

fn draw_frame(ctx: &mut Context, rect: Rect, color_id: WidgetColor) {
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
    pub fn new(text_width: TextWidthFn, text_height: TextHeightFn) -> Box<Self> {
        let mut c = Box::<Self>::new_uninit();
        let mut ptr = unsafe { &mut *c.as_mut_ptr() };

        ptr.draw_frame = draw_frame;
        ptr.text_width = text_width;
        ptr.text_height = text_height;
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
        ptr.number_edit_len = 0;
        ptr.number_edit_id = None;
        ptr.mouse_pos = Vec2::ZERO;
        ptr.last_mouse_pos = Vec2::ZERO;
        ptr.mouse_delta = Vec2::ZERO;
        ptr.scroll_delta = Vec2::ZERO;
        ptr.mouse_down = MouseState::default();
        ptr.mouse_pressed = MouseState::default();
        ptr.mouse_released = MouseState::default();
        ptr.key_down = ModKeyState::default();
        ptr.key_pressed = ModKeyState::default();
        
        unsafe {
            c.assume_init()
        }
    }

    pub fn begin(&mut self) {
        self.command_list.clear();
        self.root_list.clear();
        self.scroll_target = None;
        self.hover_root = self.next_hover_root.take();
        self.mouse_delta.x = self.mouse_pos.x - self.last_mouse_pos.x;
        self.mouse_delta.y = self.mouse_pos.y - self.last_mouse_pos.y;
        self.frame += 1;
    }

    pub fn end(&mut self) {
        assert_eq!(self.container_stack.len(), 0);
        assert_eq!(self.clip_stack.len(), 0);
        assert_eq!(self.id_stack.len(), 0);
        assert_eq!(self.layout_stack.len(), 0);

        if let Some(target) = &mut self.scroll_target {
            target.scroll.x += self.scroll_delta.x;
            target.scroll.y += self.scroll_delta.y;
        }

        if !self.updated_focus {
            self.focus_id = None;
        }
        self.updated_focus = false;

        // Bring hover root to front if mouse was pressed
        if self.mouse_pressed() &&
            self.next_hover_root.as_ref().map_or(
                false,
                |x| x.zindex < self.last_zindex && x.zindex >= 0
            )
        {
            // Bring to front
            self.last_zindex += 1;
            self.next_hover_root.as_mut().unwrap().zindex = self.last_zindex
        }

        self.key_pressed = ModKeyState::default();
        self.mouse_pressed = MouseState::default();
        self.mouse_released = MouseState::default();
        self.scroll_delta = Vec2::ZERO;
        self.last_mouse_pos = self.mouse_pos;
        self.text_input.clear();

        self.root_list.sort(|a, b| a.zindex.cmp(&b.zindex));

        for i in 0..self.root_list.len() {
            // If this is the first container then make the first command jump to it.
		    // Otherwise set the previous container's tail to jump to this one.
            if i == 0 {
                if let Some(cmd) = self.command_list.first_mut() {
                    if let Command::Jump(dst) = cmd {
                        unsafe {
                            *dst = self.root_list[i].head.offset(1)
                        }
                    } else {
                        panic!("Widgets must be drawn inside of a window or a popup.")
                    }
                }
            } else {
                unsafe {
                    self.root_list[i - 1].tail = self.root_list[i].head.offset(1)
                }
            }

            // Make the last container's tail jump to the end of command list.
            if i == self.root_list.len() - 1 {
                unsafe {
                    self.root_list[i].tail = self.command_list.ptr_at(self.command_list.len())
                }
            }
        }
    }

    pub fn commands(&self) -> impl Iterator<Item = &Command> {
        self.command_list.iter().map(|x| {
            if let Command::Jump(dst) = x {
                //wicked
                unsafe { &**dst }
            } else {
                x
            }
        })
    }

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

    #[inline]
    pub fn set_focus(&mut self, id: Id) {
        self.focus_id = Some(id);
        self.updated_focus = true;
    }

    #[inline(always)]
    pub fn get_id(&self, item: impl Hash) -> Id {
        Id::new(item, self.id_stack.len() as u64)
    }

    #[inline]
    pub fn mouse_pressed(&self) -> bool {
        self.mouse_pressed != MouseState::default()
    }

    #[inline]
    pub fn mouse_released(&self) -> bool {
        self.mouse_released != MouseState::default()
    }

    #[inline]
    pub fn mouse_down(&self) -> bool {
        self.mouse_down != MouseState::default()
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
    fn clip_rect(&self) -> Rect {
        *self.clip_stack.last().unwrap()
    }
}

//============================================================================
// Container
//============================================================================

impl Context {
    #[inline]
    pub fn get_current_container(&self) -> Option<&Container> {
        self.container_stack.last()
    }

    #[inline]
    pub fn get_container_by_name(
        &mut self,
        name: &str,
        options: Option<ContainerOptions>
    ) -> Option<&mut Container> {
        let id = self.get_id(name);

        self.get_container(id, options.unwrap_or_default())
    }

    fn get_container(&mut self, id: Id, options: ContainerOptions) -> Option<&mut Container> {
        let index = self.container_pool.find_by_id(id);

        if let Some(index) = index {
            if self.containers[index].open || options.is_unset(ContainerOption::Closed) {
                self.container_pool[index].last_update = self.frame;
            }

            return Some(&mut self.containers[index]);
        }

        if options.is_set(ContainerOption::Closed) {
            return None;
        }

        if let Some(index) = self.container_pool.init(id, self.frame) {
            let container = &mut self.containers[index];
            *container = Container::default();
            container.open = true;

            // Bring to front
            self.last_zindex += 1;
            container.zindex = self.last_zindex;

            Some(container)
        } else {
            None
        }
    }

    #[inline]
    fn pop_container(&mut self) {
        self.container_stack.pop();
        self.layout_stack.pop();
        self.id_stack.pop();
    }
}

impl Default for Container {
    fn default() -> Self {
        Self {
            head: ptr::null(),
            tail: ptr::null(),
            rect: Rect::default(),
            body: Rect::default(),
            content_size: Vec2::ZERO,
            scroll: Vec2::default(),
            zindex: 0,
            open: false
        }
    }
}

//============================================================================
// Pool
//============================================================================

impl<const N: usize> ConstVec<PoolItem, N> {
    pub fn init(&mut self, id: Id, frame: FrameIdx) -> Option<usize> {
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
    pub fn find_by_id(&self, id: Id) -> Option<usize> {
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
        self.mouse_down.set(btn);
    }

    #[inline]
    pub fn input_mouse_up(&mut self, pos: Vec2, btn: MouseButton) {
        self.input_mouse_move(pos);
        self.mouse_down.unset(btn);
        self.mouse_released.set(btn);
    }

    #[inline]
    pub fn input_scroll(&mut self, delta: Vec2) {
        self.scroll_delta = delta;
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

    /// Panics if the `text` length **in bytes** is longer than [`MAX_TEXT_STORE`].
    pub fn input_text(&mut self, text: &str) {
        let bytes = text.as_bytes();
        assert!(bytes.len() <= self.text_input.capacity());

        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.text_input.ptr_at_mut(0),
                bytes.len()
            )
        }
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

        result.x = layout.body.x;
        result.y = layout.body.y;

        layout.max.x = cmp::max(layout.max.x, result.x + result.w);
        layout.max.y = cmp::max(layout.max.y, result.y + result.h);

        self.last_rect = result;

        result
    }

    fn push_layout(&mut self, body: Rect, scroll: Vec2) {
        let mut layout = Layout {
            body: rect(body.x - scroll.y, body.y - scroll.y, body.w, body.h),
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

        self.items = widths.len();
        self.pos = vec2(self.indent, self.next_row);
        self.size.y = height;
        self.item_index = 0;
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
