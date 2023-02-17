use std::{cmp, borrow::Cow};

use crate::{
    Context, ContainerOptions, ContainerOption, WidgetInteraction,
    MouseButton, Response, WidgetColor, Vec2, rect
};
use super::{Widget, HorizontalAlign, Button};

pub struct Dropdown<'a, T: AsRef<str>> {
    state: &'a mut State,
    items: &'a [T],
    placeholder: Option<Cow<'a, str>>,
    body: Button,
    content_options: ContainerOptions,
    visible_items: u8
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct State {
    pub is_open: bool,
    pub index: Option<usize>
}

impl<'a, T: AsRef<str>> Dropdown<'a, T> {
    pub fn new(state: &'a mut State, items: &'a [T]) -> Self {
        if let Some(index) = state.index {
            assert!(index <= items.len());
        }

        Self {
            state,
            items,
            placeholder: None,
            body: Button::empty(),
            content_options: ContainerOptions::default(),
            visible_items: 3
        }
    }

    /// Set the text that is shown when no value is selected.
    /// `always_show` determines whether to show this text
    /// even when there is a value currently selected.
    #[inline]
    pub fn placeholder_text(
        mut self,
        text: impl Into<Cow<'a, str>>,
        always_show: bool
    ) -> Self {
        self.placeholder = Some(if always_show {
            text.into()
        } else {
            match self.state.index {
                Some(index) => Cow::Borrowed(
                    self.items[index].as_ref()
                ),
                None => text.into()
            }
        });

        self
    }

    /// The number of entries that are visible at once.
    /// Default is `3` although it's always capped to the
    /// maximum number of entries.
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
}

impl<'a, T: AsRef<str>> Widget for Dropdown<'a, T> {
    fn draw(self, ctx: &mut Context) -> Response {
        let ptr = &self as *const _;

        let label = if let Some(text) = self.placeholder {
            text.into_owned()
        } else {
            match self.state.index {
                Some(index) => self.items[index].as_ref().into(),
                None => String::new()
            }
        };

        let btn_resp = self.body.text(label).draw(ctx);
        let mut resp = Response::default();

        if btn_resp.submit {
            self.state.toggle();
            resp.active = self.state.is_open;
            resp.change = true;
        }

        if !self.state.is_open {
            return resp;
        }

        let name = format!("!dropdown{:p}", ptr);
        let id = ctx.create_id(&name);

        if let Some(cnt_idx) = ctx.get_container(id, ContainerOptions::default()) {
            let last = ctx.last_rect;
            let items = cmp::min(self.visible_items as usize, self.items.len());
            let rect = rect(last.x, last.y + last.h, last.w, last.h * items as i32);
    
            if btn_resp.submit {
                ctx.bring_to_front(cnt_idx);
                
                // Set as hover root so popup isn't closed in begin_window()
                ctx.hover_root = Some(cnt_idx);
                ctx.next_hover_root = Some(cnt_idx);
    
                // Open, position below the button and reset scroll
                let container = ctx.container_mut(cnt_idx);
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
    
                for (i, option) in self.items.iter().enumerate() {
                    if dropdown_entry(ctx, i, option.as_ref(), self.content_options) {
                        self.state.index = Some(i);
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
                self.state.toggle();

                resp.change = true;
                resp.active = false;
            }
        }

        resp
    }
}

impl State {
    #[inline]
    pub fn with_selection(selected: usize) -> Self {
        Self {
            is_open: false,
            index: Some(selected)
        }
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }
}

fn dropdown_entry(ctx: &mut Context, index: usize, text: &str, options: ContainerOptions) -> bool {
    let mut resp = false;
    let id = ctx.create_id(&[text.as_ptr() as usize, index]);

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
