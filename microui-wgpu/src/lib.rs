pub use microui_app::{microui, App, Shell};

mod renderer;

#[inline]
pub fn run(app: Box<dyn App>) {
    microui_app::run::<renderer::Renderer>(app)
}
