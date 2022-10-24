pub use microui;

use microui::{Context, Font};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod renderer;

use renderer::Renderer;

pub trait App {
    fn init(&mut self) { }
    fn frame(&mut self, ctx: &mut Context);
}

fn text_width(font: &Font, text: &str) -> u16 {
    10
}

fn text_height(font: &Font) -> u16 {
    10
}

pub fn run(mut app: Box<dyn App>) {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut context = Context::new(text_width, text_height);
    let mut renderer = Renderer::new(&window);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(**new_inner_size);
            }
            _ => {}
        },
        Event::RedrawRequested(id) if id == window.id() => {
            context.begin();
            app.frame(&mut context);
            context.end();

            renderer.update();

            match renderer.render(&mut context) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
