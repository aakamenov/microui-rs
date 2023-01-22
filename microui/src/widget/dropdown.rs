use std::cmp;

use crate::{
    Context, ContainerOptions, ContainerOption, WidgetInteraction,
    MouseButton, Response, WidgetColor, Vec2, rect
};
use super::{Widget, HorizontalAlign, Button};

pub struct Dropdown<'a> {
    state: &'a mut State,
    body: Button,
    content_options: ContainerOptions,
    visible_items: u8
}

#[derive(Clone, PartialEq, Debug)]
pub struct State {
    pub is_open: bool,
    pub entries: Vec<String>,
    pub index: usize
}

impl<'a> Dropdown<'a> {
    pub fn new(state: &'a mut State) -> Self {
        assert!(!state.entries.is_empty());
        assert!(state.index <= state.entries.len());

        let label = &state.entries[state.index];
        let body = Button::new(label);

        Self {
            state,
            body,
            content_options: ContainerOptions::default(),
            visible_items: 3
        }
    }

    /// The number of entries that are visible at once.
    /// Default is `3` although it's always capped to the
    /// maximux number of entries.
    #[inline]
    pub fn visible_items(mut self, count: u8) -> Self {
        self.visible_items = count;

        self
    }

    #[inline]
    pub fn content_align(mut self, align: HorizontalAlign) -> Self {
        match align {
            HorizontalAlign::Left => {}
            HorizontalAlign::Center => self.content_options.set(ContainerOption::AlignCenter),
            HorizontalAlign::Right => self.content_options.set(ContainerOption::AlignRight)
        }

        self
    }

    #[inline]
    pub fn no_interact(mut self) -> Self {
        self.body = self.body.no_interact();

        self
    }

    #[inline]
    pub fn no_frame(mut self) -> Self {
        self.body = self.body.no_frame();

        self
    }
}

impl<'a> Widget for Dropdown<'a> {
    fn draw(self, ctx: &mut Context) -> Response {
        let btn_resp = self.body.draw(ctx);
        let mut resp = Response::default();

        if btn_resp.submit {
            resp.change = true;
            self.state.toggle();
        }

        if !self.state.is_open {
            return Response::default();
        }

        let name = format!("{:p}", self.state.entries.as_ptr());
        let id = ctx.create_id(&name);

        if let Some(cnt_idx) = ctx.get_container_impl(id, ContainerOptions::default()) {
            let last = ctx.last_rect;
            let items = cmp::min(self.visible_items as usize, self.state.entries.len());
            let rect = rect(last.x, last.y + last.h, last.w, last.h * items as i32);
    
            if btn_resp.submit {
                ctx.bring_to_front(cnt_idx);
                
                // Set as hover root so popup isn't closed in begin_window()
                ctx.hover_root = Some(cnt_idx);
                ctx.next_hover_root = Some(cnt_idx);
    
                // Open, position below the button and reset scroll
                let container = ctx.get_container_mut(cnt_idx);
                container.open = true;
                container.rect = rect;
                container.body = rect;
                container.scroll = Vec2::ZERO;
            }
            
            let mut options = ContainerOptions::default();
            options.set(ContainerOption::Popup);
            options.set(ContainerOption::NoResize);
            options.set(ContainerOption::NoTitle);
    
            let padding = ctx.style.padding;
            ctx.style.padding = 0;
    
            if ctx.begin_window(name, rect, options) {
                ctx.style.padding = padding;
                ctx.layout_row(&[-1], 0);
    
                let spacing = ctx.style.spacing;
                ctx.style.spacing = 0;
    
                for (i, option) in self.state.entries.iter().enumerate() {
                    if dropdown_entry(ctx, option, self.content_options) {
                        self.state.index = i;
                        resp.submit = true;
                    }
                }
    
                ctx.style.spacing = spacing;
                ctx.end_window();
            }
    
            // Close if a value was selected or there was a
            // click outside of the dropdown area.
            if resp.submit || !ctx.containers[cnt_idx].open {
                ctx.containers[cnt_idx].open = false;
                resp.change = true;
                self.state.toggle();
            }
        }

        resp
    }
}

impl State {
    #[inline]
    pub fn new(entries: Vec<String>) -> Self {
        assert!(!entries.is_empty());

        Self {
            entries,
            is_open: false,
            index: 0
        }
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    #[inline]
    pub fn selected(&self) -> &str {
        &self.entries[self.index]
    }
}

fn dropdown_entry(ctx: &mut Context, text: &str, options: ContainerOptions) -> bool {
    let mut resp = false;
    let id = ctx.create_id(&text);

    let rect = ctx.layout_next();
    ctx.update_widget(id, rect, WidgetInteraction::default());

    if ctx.mouse_pressed(MouseButton::Left) && ctx.is_focused(id) {
        resp = true;
    }

    let color = if ctx.is_hovered(id) {
        WidgetColor::BaseHover
    } else {
        WidgetColor::WindowBackground
    };

    ctx.draw_rect(rect, ctx.style.colors[color]);
    ctx.draw_widget_text(text, rect, WidgetColor::Text, options);

    resp
}