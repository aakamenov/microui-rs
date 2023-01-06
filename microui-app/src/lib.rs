use std::time::{Instant, Duration};

use microui::{Context, TextSizeHandler, MouseButton, Vec2, vec2};
use winit::{
    event::{Event, WindowEvent, ElementState, MouseButton as WinitMouseBtn},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::PhysicalSize
};

pub use microui;
pub use winit;

pub trait App {
    fn init(&mut self) { }
    fn frame(&mut self, ctx: &mut Context);
}

pub trait MicrouiRenderer: Sized {
    type TextSizeHandler: TextSizeHandler;

    fn init(
        window_builder: WindowBuilder,
        event_loop: &EventLoop<()>
    ) -> Self;
    fn window(&self) -> &Window;
    fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: f64);
    fn render(&mut self, ctx: &mut Context);
    fn text_size_handler(&self) -> Self::TextSizeHandler;
}

pub fn run<Renderer: MicrouiRenderer + 'static>(mut app: Box<dyn App>) {
    let event_loop = EventLoop::new();
    let mut renderer = Renderer::init(
        WindowBuilder::new(),
        &event_loop
    );

    let mut ctx = Context::new(renderer.text_size_handler());

    let mut mouse_pos = Vec2::ZERO;
    let mut render_delta = Instant::now();

    let mut current_scale_factor = renderer.window().scale_factor();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id
        } if window_id == renderer.window().id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size, current_scale_factor);
            }
            WindowEvent::ScaleFactorChanged {
                new_inner_size,
                scale_factor
            } => {
                current_scale_factor = *scale_factor;
                renderer.resize(**new_inner_size, current_scale_factor);
            },
            WindowEvent::CursorMoved { position, .. } => {
                let position = position.to_logical::<i32>(current_scale_factor);
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
        Event::RedrawRequested(id) if id == renderer.window().id() => {
            ctx.begin();
            app.frame(&mut ctx);
            ctx.end();

            renderer.render(&mut ctx);

            render_delta = Instant::now();
        },
        Event::MainEventsCleared => {
            // Cap to 60 FPS
            if render_delta.elapsed() >= Duration::from_millis(16) {
                renderer.window().request_redraw();
            }
        }
        _ => {}
    });
}
