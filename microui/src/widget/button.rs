use crate::{
    Context, ContainerOptions, ContainerOption, MouseButton,
    Icon, WidgetInteraction, WidgetColor, Response
};
use super::{Widget, HorizontalAlign};

#[derive(Clone, PartialEq, Debug)]
pub struct Button {
    content: Content,
    options: ContainerOptions
}

#[derive(Clone, PartialEq, Debug)]
enum Content {
    Text(String),
    Icon(Icon)
}

impl Button {
    #[inline]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content: Content::Text(text.into()),
            options: ContainerOptions(ContainerOption::AlignCenter as u16)
        }
    }

    #[inline]
    pub fn icon(icon: Icon) -> Self {
        Self {
            content: Content::Icon(icon),
            options: ContainerOptions(ContainerOption::AlignCenter as u16)
        }
    }

    #[inline]
    pub fn empty() -> Self {
        Self {
            content: Content::Icon(Icon::None),
            options: ContainerOptions::default()
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

impl Widget for Button {
    fn draw(self, ctx: &mut Context) -> Response {
        let mut resp = Response::default();

        let id = match &self.content {
            Content::Text(text) => ctx.create_id(text),
            Content::Icon(icon) => ctx.create_id(&(*icon as u8 as *const u8))
        };

        let rect = ctx.layout_next();
        ctx.update_widget(id, rect, WidgetInteraction::from(self.options));

        if ctx.mouse_pressed(MouseButton::Left) && ctx.is_focused(id) {
            resp.submit = true;
        }

        ctx.draw_widget_frame(id, rect, WidgetColor::Button, self.options);

        match self.content {
            Content::Text(text) => {
                ctx.draw_widget_text(text, rect, WidgetColor::Text, self.options);
            },
            Content::Icon(icon) => {
                if !matches!(icon, Icon::None) {
                    ctx.draw_icon(icon, rect, ctx.style.colors[WidgetColor::Text]);
                }
            }
        }

        resp
    }
}
