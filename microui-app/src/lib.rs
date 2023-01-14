use std::time::{Instant, Duration};

use microui::{Context, TextSizeHandler, MouseButton, ModKey, Color, Vec2, vec2};
use winit::{
    event::{
        Event, WindowEvent, ElementState, MouseScrollDelta,
        VirtualKeyCode, MouseButton as WinitMouseBtn
    },
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::PhysicalSize
};

pub use microui;
pub use winit;

pub trait App {
    fn init(&mut self) { }
    fn frame(&mut self, ctx: &mut Context, shell: &mut Shell);
}

pub trait MicrouiRenderer {
    type TextSizeHandler: TextSizeHandler;

    fn init(
        window_builder: WindowBuilder,
        event_loop: &EventLoop<()>
    ) -> Self;
    fn window(&self) -> &Window;
    fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: f64);
    fn render(&mut self, ctx: &mut Context, clear_color: Option<Color>);
    fn text_size_handler(&self) -> Self::TextSizeHandler;
}

#[derive(Clone)]
pub struct Shell {
    clear_color: Option<Color>,
    screen_size: Vec2
}

pub fn run<Renderer: MicrouiRenderer + 'static>(mut app: Box<dyn App>) {
    let event_loop = EventLoop::new();
    let mut renderer = Renderer::init(
        WindowBuilder::new().with_transparent(true),
        &event_loop
    );

    let mut ctx = Context::new(renderer.text_size_handler());

    let mut mouse_pos = Vec2::ZERO;
    let mut render_delta = Instant::now();

    let mut current_scale_factor = renderer.window().scale_factor();
    let size = renderer.window().inner_size().to_logical::<i32>(current_scale_factor);
    let mut shell = Shell::new(vec2(size.width, size.height));

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

                let size = new_inner_size.to_logical::<i32>(current_scale_factor);
                shell.screen_size = vec2(size.width, size.height);

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
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        let speed = 30.0f32;
                        ctx.input_scroll(vec2(-(x * speed) as i32, -(y * speed) as i32));
                    }
                    _ => unimplemented!()
                }
            }
            WindowEvent::ReceivedCharacter(c) => {
                // Winit also sends non-text characters here.
                if c.is_alphanumeric() || c.is_ascii_punctuation() {
                    let mut buf = [0; 4];
                    let text = c.encode_utf8(&mut buf);

                    ctx.input_text(&text[0..c.len_utf8()]);
                }
            },
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    let key = match key {
                        VirtualKeyCode::LShift | VirtualKeyCode::RShift => Some(ModKey::Shift),
                        VirtualKeyCode::LControl | VirtualKeyCode::RControl => Some(ModKey::Ctrl),
                        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => Some(ModKey::Alt),
                        VirtualKeyCode::Back => Some(ModKey::Backspace),
                        VirtualKeyCode::Return => Some(ModKey::Return),
                        _ => None
                    };

                    if let Some(key) = key {
                        match input.state {
                            ElementState::Pressed => ctx.input_key_down(key),
                            ElementState::Released => ctx.input_key_up(key)
                        }
                    }
                }
            }
            _ => {}
        },
        Event::RedrawRequested(id) if id == renderer.window().id() => {
            ctx.begin();
            app.frame(&mut ctx, &mut shell);
            ctx.end();

            renderer.render(&mut ctx, shell.clear_color.take());

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

impl Shell {
    #[inline]
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = Some(color);
    }

    #[inline]
    pub fn screen_size(&self) -> Vec2 {
        self.screen_size
    }

    #[inline]
    fn new(screen_size: Vec2) -> Self {
        Self {
            clear_color: Some(Color::rgb(90, 95, 100)),
            screen_size
        }
    }
}
