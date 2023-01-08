use crate::{Context, Response, ContainerOptions, WidgetColor};
use super::{Widget, HorizontalAlign};

#[derive(Clone, PartialEq, Debug)]
pub struct Label {
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
