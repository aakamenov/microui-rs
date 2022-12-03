pub use microui;

use microui::{Context, Font, MouseButton, Vec2, vec2};
use winit::{
    event::{Event, WindowEvent, ElementState, MouseButton as WinitMouseBtn},
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

    let mut ctx = Context::new(text_width, text_height);
    let mut renderer = Renderer::new(&window);

    let mut mouse_pos = Vec2::ZERO;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size, None);
            }
            WindowEvent::ScaleFactorChanged {
                new_inner_size,
                scale_factor
            } => {
                renderer.resize(**new_inner_size, Some(*scale_factor));
            },
            WindowEvent::CursorMoved { position, .. } => {
                let position = position.to_logical::<i32>(renderer.scale_factor);
                mouse_pos = vec2(position.x, position.y);
                
                ctx.input_mouse_move(mouse_pos);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    WinitMouseBtn::Left => Some(MouseButton::Left),
                    WinitMouseBtn::Right => Some(MouseButton::Right),
                    WinitMouseBtn::Middle => Some(MouseButton::Middle),
                    WinitMouseBtn::Other(_) => None
                };

                if let Some(button) = button {
                    match state {
                        ElementState::Pressed => ctx.input_mouse_down(mouse_pos, button),
                        ElementState::Released => ctx.input_mouse_up(mouse_pos, button),
                    }
                }
            }
            _ => {}
        },
        Event::RedrawRequested(id) if id == window.id() => {
            ctx.begin();
            app.frame(&mut ctx);
            ctx.end();

            renderer.update();

            match renderer.render(&mut ctx) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size(), None),
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
