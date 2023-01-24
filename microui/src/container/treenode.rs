use crate::Context;

pub struct Treenode {
    label: String,
    expanded: bool
}

impl Treenode {
    #[inline]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            expanded: false
        }
    }

    #[inline]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;

        self
    }

    #[inline]
    pub fn show(self, ctx: &mut Context, contents: impl FnOnce(&mut Context)) {
        if ctx.header_impl(self.label, true, self.expanded) {
            if let Some(layout) = ctx.layout_stack.last_mut() {
                layout.indent += ctx.style.indent as i32;
                ctx.id_stack.push(ctx.last_id.unwrap_or_default());
            }

            contents(ctx);

            if let Some(layout) = ctx.layout_stack.last_mut() {
                layout.indent -= ctx.style.indent as i32;
                ctx.pop_id();
            }
        }
    }
}
