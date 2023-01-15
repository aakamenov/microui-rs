use crate::{
    Context, ContainerOptions, ContainerOption, MouseButton,
    WidgetColor, WidgetInteraction, CursorIcon, Response
};
use super::{Widget, HorizontalAlign, textbox};

pub struct DragValue<'a> {
    value: &'a mut f64,
    step: f64,
    options: ContainerOptions
}

impl<'a> DragValue<'a> {
    #[inline]
    pub fn new(value: &'a mut f64, step: f64) -> Self {
        Self {
            value,
            step,
            options: ContainerOptions(ContainerOption::AlignCenter as u16)
        }
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

    #[inline]
    pub fn no_frame(mut self) -> Self {
        self.options.set(ContainerOption::NoFrame);

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

impl<'a> Widget for DragValue<'a> {
    fn draw(self, ctx: &mut Context) -> Response {
        let mut resp = Response::default();

        let id = ctx.create_id(&(self.value as *const f64));
        let base = ctx.layout_next();
        let last = *self.value;

        if textbox::number(ctx, self.value, base, id) {
            return resp;
        }

        ctx.update_widget(
            id,
            base,
            WidgetInteraction::from(self.options)
                .cursor(CursorIcon::Drag)
                .retain_cursor_focus()
        );

        if ctx.is_focused(id) && ctx.mouse_down(MouseButton::Left) {
            *self.value += ctx.mouse_delta().x as f64 * self.step;
        }

        if *self.value != last {
            resp.change = true;
        }

        ctx.draw_widget_frame(id, base, WidgetColor::Base, self.options);

        let text = format!("{:.2}", *self.value);
        ctx.draw_widget_text(text, base, WidgetColor::Text, self.options);

        resp
    }
}
