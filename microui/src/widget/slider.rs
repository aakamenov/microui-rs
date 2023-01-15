use std::ops::Range;

use crate::{
    Context, ContainerOptions, ContainerOption, MouseButton,
    WidgetInteraction, WidgetColor, Response, rect
};
use super::{Widget, HorizontalAlign, textbox};

#[derive(Debug)]
pub struct Slider<'a> {
    value: &'a mut f64,
    range: Range<f64>,
    step: Option<f64>,
    options: ContainerOptions
}

impl<'a> Slider<'a> {
    #[inline]
    pub fn new(value: &'a mut f64, range: Range<f64>) -> Self {
        Self {
            value,
            range,
            step: None,
            options: ContainerOptions(ContainerOption::AlignCenter as u16)
        }
    }

    #[inline]
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);

        self
    }

    #[inline]
    pub fn no_frame(mut self) -> Self {
        self.options.set(ContainerOption::NoFrame);

        self
    }

    #[inline]
    pub fn no_interact(mut self) -> Self {
        self.options.set(ContainerOption::NoInteract);

        self
    }

    #[inline]
    pub fn align(mut self, align: HorizontalAlign) -> Self {
        match align {
            HorizontalAlign::Left => self.options.unset(ContainerOption::AlignCenter),
            HorizontalAlign::Center => {}
            HorizontalAlign::Right => self.options.set(ContainerOption::AlignRight)
        }

        self
    }
}

impl<'a> Widget for Slider<'a> {
    fn draw(self, ctx: &mut Context) -> Response {
        let mut resp = Response::default();

        let last = *self.value;
        let mut v = last;
        let id = ctx.create_id(&(self.value as *const f64));
        let base = ctx.layout_next();

        if textbox::number(ctx, &mut v, base, id) {
            return Response::default();
        }

        ctx.update_widget(id, base, WidgetInteraction::from(self.options));

        if ctx.is_focused(id) && ctx.mouse_down.is_set(MouseButton::Left) {
            v = self.range.start + (ctx.mouse_pos().x - base.x) as f64 * (self.range.end - self.range.start) / base.w as f64;

            if let Some(step) = self.step {
                v = ((v + step / 2f64) / step) * step;
            }
        }

        v = v.clamp(self.range.start, self.range.end);
        *self.value = v;

        if last != v {
            resp.change = true;
        }

        ctx.draw_widget_frame(id, base, WidgetColor::Base, self.options);

        let w = ctx.style.thumb_size as i32;
        let x = ((v - self.range.start) * (base.w - w) as f64 / (self.range.end - self.range.start)) as i32;

        let thumb = rect(base.x + x, base.y, w, base.h);
        ctx.draw_widget_frame(id, thumb, WidgetColor::Button, self.options);

        let text = format!("{:.2}", v);
        ctx.draw_widget_text(text, base, WidgetColor::Text, self.options);

        resp
    }
}
