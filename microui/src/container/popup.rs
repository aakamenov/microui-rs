use crate::{Context, ContainerOptions, ContainerOption, Rect, rect};

pub struct Popup {
    name: String
}

impl Popup {
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Call this when some action occurs (i.e when pressing a button)
    /// to cause the popup to be shown.
    pub fn open(&self, ctx: &mut Context) {
        let id = ctx.create_id(&self.name);

        if let Some(cnt_idx) = ctx.get_container(
            id,
            ContainerOptions::default()
        ) {
            // Set as hover root so popup isn't closed in begin_window()
            ctx.hover_root = Some(cnt_idx);
            ctx.next_hover_root = Some(cnt_idx);

            // Position at mouse cursor, open and bring to front.
            ctx.containers[cnt_idx].rect = rect(ctx.mouse_pos.x, ctx.mouse_pos.y, 1, 1);
            ctx.containers[cnt_idx].open = true;

            ctx.bring_to_front(cnt_idx);
        }
    }

    /// This must be called unconditionally just like a [`Window`].
    /// However, it will only actually show anything if [`Popup::open`] was
    /// called prior to this.
    #[inline]
    pub fn show(self, ctx: &mut Context, contents: impl FnOnce(&mut Context)) {
        let mut options = ContainerOptions::default();
        options.set(ContainerOption::Popup);
        options.set(ContainerOption::AutoSize);
        options.set(ContainerOption::NoResize);
        options.set(ContainerOption::NoScroll);
        options.set(ContainerOption::NoTitle);
        options.set(ContainerOption::Closed);

        if ctx.begin_window(self.name, Rect::default(), options) {
            contents(ctx);
            ctx.end_window();
        }
    }
}
