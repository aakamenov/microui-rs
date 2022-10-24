use microui_wgpu::{App, run, microui::*};

struct Demo;

impl App for Demo {
    fn frame(&mut self, ctx: &mut Context) {
        if ctx.begin_window(
            "window",
            rect(10, 10, 600, 800),
            ContainerOptions::default()
        ) {
            ctx.layout_row(&[100, -110, -1], 0);
    
            ctx.label("Buttons");
            ctx.button("Button 1", Icon::None, None);
            ctx.button("Button 2", Icon::None, None);
    
            ctx.end_window();
        }
    }
}

fn main() {
    run(Box::new(Demo));
}
