pub use microui_app::{microui, App};

mod renderer;

#[inline]
pub fn run(app: Box<dyn App>) {
    microui_app::run::<renderer::Renderer>(app)
}
