use crate::{
    Context, Response, ContainerOptions, MouseButton,
    WidgetColor, WidgetInteraction, CursorIcon, rect
};
use super::{Widget, HorizontalAlign};

#[derive(Clone, PartialEq, Debug)]
pub struct Label {
    text: String,
    options: ContainerOptions
}

#[derive(Clone, PartialEq, Debug)]
pub struct ClickableLabel {
    text: String,
    options: ContainerOptions
}

impl Label {
    #[inline]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            options: ContainerOptions::default()
        }
    }

    #[inline]
    pub fn align(mut self, align: HorizontalAlign) -> Self {
        if let Some(option) = align.into() {
            self.options.set(option);
        }

        self
    }
}

impl Widget for Label {
    #[inline]
    fn draw(self, ctx: &mut Context) -> Response {
        let layout = ctx.layout_next();
        ctx.draw_widget_text(
            self.text,
            layout,
            WidgetColor::Text,
            self.options
        );

        Response::default()
    }
}

impl ClickableLabel {
    #[inline]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            options: ContainerOptions::default()
        }
    }

    #[inline]
    pub fn align(mut self, align: HorizontalAlign) -> Self {
        if let Some(option) = align.into() {
            self.options.set(option);
        }

        self
    }
}

impl Widget for ClickableLabel {
    fn draw(self, ctx: &mut Context) -> Response {
        let id = ctx.create_id(&self.text);

        let layout = ctx.layout_next();
        ctx.update_widget(
            id,
            layout,
            WidgetInteraction::from(self.options)
                .cursor(CursorIcon::Hand)
        );

        let text_rect = ctx.draw_widget_text(
            self.text,
            layout,
            WidgetColor::Text,
            self.options
        );

        if ctx.is_hovered(id) {
            ctx.draw_rect(
                rect(text_rect.x, text_rect.y + text_rect.h, text_rect.w, 1),
                ctx.style.colors[WidgetColor::Text]
            );
        }

        let mut resp = Response::default();
        if ctx.mouse_pressed(MouseButton::Left) && ctx.is_focused(id) {
            resp.submit = true;
        }

        resp
    }
}
