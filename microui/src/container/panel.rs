use crate::{Context, ContainerOptions, ContainerOption};

pub struct Panel {
    name: String,
    options: ContainerOptions
}

impl Panel {
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            options: ContainerOptions::default()
        }
    }

    #[inline]
    pub fn no_frame(mut self) -> Self {
        self.options.set(ContainerOption::NoFrame);

        self
    }

    #[inline]
    pub fn no_scroll(mut self) -> Self {
        self.options.set(ContainerOption::NoScroll);

        self
    }

    #[inline]
    pub fn show(self, ctx: &mut Context, contents: impl FnOnce(&mut Context)) {
        if ctx.begin_panel(self.name, self.options) {
            contents(ctx);
            ctx.end_panel();
        }
    }
}
