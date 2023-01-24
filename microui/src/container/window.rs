use crate::{Context, Rect, Vec2, ContainerOptions, ContainerOption, vec2};

pub struct Window {
    title: String,
    rect: Rect,
    options: ContainerOptions,
    min_size: Option<Vec2>,
    max_size: Option<Vec2>
}

impl Window {
    #[inline]
    pub fn new(title: impl Into<String>, rect: Rect) -> Self {
        Self {
            title: title.into(),
            rect,
            options: ContainerOptions::default(),
            min_size: None,
            max_size: None
        }
    }

    #[inline]
    pub fn min_size(mut self, size: Vec2) -> Self {
        self.min_size = Some(size);

        self
    }

    #[inline]
    pub fn max_size(mut self, size: Vec2) -> Self {
        self.max_size = Some(size);

        self
    }

    #[inline]
    pub fn no_frame(mut self) -> Self {
        self.options.set(ContainerOption::NoFrame);

        self
    }

    #[inline]
    pub fn no_title_bar(mut self) -> Self {
        self.options.set(ContainerOption::NoTitle);

        self
    }

    #[inline]
    pub fn no_interact(mut self) -> Self {
        self.options.set(ContainerOption::NoInteract);

        self
    }

    #[inline]
    pub fn no_close(mut self) -> Self {
        self.options.set(ContainerOption::NoClose);

        self
    }

    #[inline]
    pub fn no_resize(mut self) -> Self {
        self.options.set(ContainerOption::NoResize);

        self
    }

    #[inline]
    pub fn no_scroll(mut self) -> Self {
        self.options.set(ContainerOption::NoScroll);

        self
    }

    #[inline]
    pub fn auto_size(mut self) -> Self {
        self.options.set(ContainerOption::AutoSize);

        self
    }

    /// Will close the window if the mouse clicks outside of it.
    #[inline]
    pub fn popup(mut self) -> Self {
        self.options.set(ContainerOption::Popup);

        self
    }

    pub fn show(self, ctx: &mut Context, contents: impl FnOnce(&mut Context)) {
        if ctx.begin_window(self.title, self.rect, self.options) {
            if self.min_size.is_some() || self.max_size.is_some() {
                let index = ctx.current_container_index().unwrap();
                let container = ctx.get_container_mut(index);
    
                let min = self.min_size.unwrap_or(Vec2::ZERO);
                let max = self.max_size.unwrap_or(vec2(i32::MAX, i32::MAX));

                container.rect.w = container.rect.w.clamp(min.x, max.x);
                container.rect.h = container.rect.h.clamp(min.y, max.y);
            }

            contents(ctx);
            ctx.end_window()
        }
    }
}
