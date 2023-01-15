use std::{cmp, fmt::Write};

use crate::{
    Context, ContainerOptions, ContainerOption, MouseButton,
    CursorIcon, WidgetInteraction, ModKey, Id, Rect, Response,
    WidgetColor, TextBuf, vec2, rect
};
use super::Widget;

pub enum TextBoxBuf<'a> {
    Text(&'a mut dyn TextBuf),
    Numeric
}

pub struct TextBox<'a, T: TextBuf> {
    buf: &'a mut T,
    options: ContainerOptions
}

impl<'a, T: TextBuf> TextBox<'a , T> {
    #[inline]
    pub fn new(buf: &'a mut T) -> Self {
        Self {
            buf,
            options: ContainerOptions::default()
        }
    }

    #[inline]
    pub fn no_frame(mut self) -> Self {
        self.options.set(ContainerOption::NoFrame);

        self
    }
}

pub fn raw(
    ctx: &mut Context,
    buf: TextBoxBuf,
    id: Id,
    r: Rect,
    options: ContainerOptions
) -> Response {
    let mut resp = Response::default();

    let mut opts_copy = options;
    opts_copy.set(ContainerOption::HoldFocus);

    ctx.update_widget(
        id,
        r,
        WidgetInteraction::from(opts_copy).cursor(CursorIcon::Text)
    );

    let text: String = if ctx.is_focused(id) {
        // Handle text input
        let input = ctx.text_input.as_str();
        let buf = match buf {
            TextBoxBuf::Text(buf) => buf,
            TextBoxBuf::Numeric => &mut ctx.number_edit_buf as &mut dyn TextBuf
        };

        if buf.push_str(input) > 0 {
            resp.change = true;
        }

        if ctx.key_pressed.is_set(ModKey::Backspace) {
            buf.pop_char();
            resp.change = true;
        }

        let text = buf.as_str().into();

        if ctx.key_pressed.is_set(ModKey::Return) {
            ctx.set_focus(None);
            resp.submit = true;
        }

        text
    } else {
        let buf = match buf {
            TextBoxBuf::Text(buf) => buf,
            TextBoxBuf::Numeric => &mut ctx.number_edit_buf as &mut dyn TextBuf
        };

        buf.as_str().into()
    };

    ctx.draw_widget_frame(id, r, WidgetColor::Base, options);

    if ctx.is_focused(id) {
        let color = ctx.style.colors[WidgetColor::Text];

        let font = ctx.style.font;
        let textw = ctx.font_handler.text_width(font, &text);
        let texth = ctx.font_handler.text_height(font);

        let offset = r.w - ctx.style.padding as i32 - textw - 1;
        let textx = r.x + cmp::min(offset, ctx.style.padding as i32);
        let texty = r.y + (r.h - texth) / 2;

        ctx.push_clip_rect(r);
        ctx.draw_text(font, text, vec2(textx, texty), color);
        ctx.draw_rect(rect(textx + textw, texty, 1, texth), color);
        ctx.pop_clip_rect();
    } else {
        ctx.draw_widget_text(text, r, WidgetColor::Text, options);
    }

    resp
}

pub fn number(
    ctx: &mut Context,
    value: &mut f64,
    rect: Rect,
    id: Id
) -> bool {
    if ctx.mouse_pressed.is_set(MouseButton::Left) &&
        ctx.key_down.is_set(ModKey::Shift) &&
        ctx.is_hovered(id)
    {
        ctx.number_edit_id = Some(id);
        ctx.number_edit_buf.clear();

        let _ = write!(
            &mut ctx.number_edit_buf,
            "{:.2}",
            value
        );
    }

    if ctx.number_edit_id.map_or(false, |x| x == id) {
        let resp = raw(
            ctx,
            TextBoxBuf::Numeric,
            id,
            rect,
            ContainerOptions::default()
        );

        if resp.submit || !ctx.is_focused(id) {
            if let Ok(val) = ctx.number_edit_buf.as_str().parse::<f64>() {
                *value = val;
            }

            ctx.number_edit_id = None;
        } else {
            return true;
        }
    }

    false
}

impl<'a, T: TextBuf> Widget for TextBox<'a, T> {
    #[inline]
    fn draw(self, ctx: &mut Context) -> Response {
        let id = ctx.create_id(&self.buf.as_str().as_ptr());
        let rect = ctx.layout_next();

        raw(ctx, TextBoxBuf::Text(self.buf), id, rect, self.options)
    }
}
