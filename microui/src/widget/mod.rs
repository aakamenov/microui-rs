pub mod textbox;
mod button;
mod label;
mod checkbox;
mod slider;
mod drag_value;

pub use button::*;
pub use label::*;
pub use checkbox::*;
pub use textbox::TextBox;
pub use slider::*;
pub use drag_value::DragValue;

use crate::{Context, Response, ContainerOption};

pub trait Widget {
    fn draw(self, ctx: &mut Context) -> Response;
}

#[derive(Clone, Copy, Debug)]
pub enum HorizontalAlign {
    Left,
    Right,
    Center
}

impl Into<Option<ContainerOption>> for HorizontalAlign {
    #[inline]
    fn into(self) -> Option<ContainerOption> {
        match self {
            Self::Left => None,
            Self::Center => Some(ContainerOption::AlignCenter),
            Self::Right => Some(ContainerOption::AlignRight)
        }
    }
}

impl Default for HorizontalAlign {
    #[inline]
    fn default() -> Self {
        Self::Left
    }
}
