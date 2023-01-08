use crate::{Context, Response, ContainerOptions, Icon, WidgetColor, MouseButton, rect};
use super::Widget;

#[derive(Debug)]
pub struct Checkbox<'a> {
    label: String,
    checked: &'a mut bool
}

impl<'a> Checkbox<'a> {
    #[inline]
    pub fn new(label: impl Into<String>, checked: &'a mut bool) -> Self {
        Self {
            label: label.into(),
            checked
        }
    }
}

impl<'a> Widget for Checkbox<'a> {
    fn draw(self, ctx: &mut Context) -> Response {
        let mut resp = Response::default();

        let id = ctx.create_id(&(self.checked as *const bool));
        let r = ctx.layout_next();
        let frame = rect(r.x, r.y, r.h, r.h);

        ctx.update_widget(id, r, ContainerOptions::default());

        if ctx.mouse_released.is_set(MouseButton::Left) && ctx.is_hovered(id) {
            resp.change = true;
            *self.checked = !*self.checked;
        }

        ctx.draw_widget_frame(id, frame, WidgetColor::Base, ContainerOptions::default());

        if *self.checked {
            ctx.draw_icon(Icon::Check, frame, ctx.style.colors[WidgetColor::Text]);
        }

        let r = rect(r.x + frame.w, r.y, r.w - frame.w, r.h);
        ctx.draw_widget_text(self.label, r, WidgetColor::Text, ContainerOptions::default());

        resp
    }
}
