use std::num::NonZeroU32;

use microui_app::{
    MicrouiRenderer,
    microui::{
        Context, CommandHandler, TextSizeHandler,
        FontId, Icon, Color, Rect, Vec2
    },
    winit::{
        event_loop::EventLoop,
        window::{Window, WindowBuilder},
        dpi::PhysicalSize
    }
};

use femtovg::{
    Canvas, TextContext, Baseline, Align, Paint, Path,
    FontId as FemtovgFontId, Color as FemtovgColor,
    renderer::OpenGl
};

use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    surface::{SurfaceAttributesBuilder, Surface, WindowSurface},
    prelude::*
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;

const DEFAULT_FONT: &[u8] = include_bytes!("../../fonts/ProggyClean.ttf");
const FONT_SIZE_PT: f32 = 16.0;

pub struct Renderer {
    window: Window,
    ctx: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    canvas: Canvas<OpenGl>,
    text_context: TextContext,
    font_id: FemtovgFontId,
    clear_color: FemtovgColor
}

#[derive(Clone)]
pub struct FemtovgTextSizeHandler {
    ctx: TextContext,
    font_id: FemtovgFontId
}

impl MicrouiRenderer for Renderer {
    type TextSizeHandler = FemtovgTextSizeHandler;

    fn init(
        window_builder: WindowBuilder,
        event_loop: &EventLoop<()>
    ) -> Self {
        let template = ConfigTemplateBuilder::new()

            .prefer_hardware_accelerated(Some(true))
            .with_alpha_size(8);

        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (window, gl_config) = display_builder.build(&event_loop, template, |configs| {
            // Find the config with the least number of samples, so our triangle will be smooth.
            configs.reduce(|accum, config| {
                let transparency_check = config.supports_transparency().unwrap_or(false) &
                    !accum.supports_transparency().unwrap_or(false);

                if transparency_check || config.num_samples() < accum.num_samples() {
                    config
                } else {
                    accum
                }
            }).unwrap()
        }).unwrap();

        let window = window.unwrap();
        let window_handle = Some(window.raw_window_handle());
        let display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .build(window_handle);

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(window_handle);

        let mut not_current_gl_context = Some(unsafe {
            display.create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    display.create_context(
                        &gl_config,
                        &fallback_context_attributes
                    ).expect("Couldn't create OpenGL context.")
                })
        });

        let (width, height): (u32, u32) = window.inner_size().into();
        let window_handle = window.raw_window_handle();

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            gl_config.display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let ctx = not_current_gl_context.take()
            .unwrap()
            .make_current(&surface)
            .unwrap();

        let renderer = unsafe {
            OpenGl::new_from_function_cstr(|s|
                display.get_proc_address(s) as *const _
            )
        }.expect("Couldn't create renderer.");

        let text_context = TextContext::default();
        let mut canvas = Canvas::new_with_text_context(renderer, text_context.clone()).unwrap();
        let font_id = canvas.add_font_mem(DEFAULT_FONT).unwrap();

        canvas.set_size(width, height, window.scale_factor() as f32);

        let renderer = Renderer {
            window,
            ctx,
            surface,
            canvas,
            text_context,
            font_id,
            clear_color: FemtovgColor::black()
        };

        renderer
    }

    #[inline]
    fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: f64) {
        self.surface.resize(
            &self.ctx,
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap()
        );

        self.canvas.set_size(size.width, size.height, scale_factor as f32);
    }

    #[inline]
    fn window(&self) -> &Window {
        &self.window
    }

    #[inline]
    fn render(&mut self, ctx: &mut Context, clear_color: Option<Color>) {
        if let Some(color) = clear_color {
            self.clear_color = FemtovgColor::rgba(color.r, color.g, color.b, color.a);
        }

        let size = self.window.inner_size();
        self.canvas.clear_rect(0, 0, size.width, size.height, self.clear_color);

        ctx.handle_commands(self);

        self.canvas.flush();
        self.surface.swap_buffers(&self.ctx).unwrap();
    }

    #[inline]
    fn text_size_handler(&self) -> Self::TextSizeHandler {
        FemtovgTextSizeHandler {
            ctx: self.text_context.clone(),
            font_id: self.font_id
        }
    }
}

impl CommandHandler for Renderer {
    #[inline]
    fn clip_cmd(&mut self, rect: Rect) {
        self.canvas.scissor(rect.x as f32, rect.y as f32, rect.w as f32, rect.h as f32);
    }

    #[inline]
    fn rect_cmd(&mut self, rect: Rect, color: Color) {
        let mut path = Path::default();
        path.rect(rect.x as f32, rect.y as f32, rect.w as f32, rect.h as f32);

        let paint = Paint::default().with_color(
            FemtovgColor::rgba(color.r, color.g, color.b, color.a)
        );

        self.canvas.fill_path(&mut path, &paint);
    }

    #[inline]
    fn text_cmd(
        &mut self,
        _font: FontId,
        pos: Vec2,
        color: Color,
        text: String
    ) {
        let paint = Paint::default()
            .with_font(&[self.font_id])
            .with_font_size(FONT_SIZE_PT)
            .with_text_baseline(Baseline::Top)
            .with_color(
                FemtovgColor::rgba(color.r, color.g, color.b, color.a)
            );

        self.canvas.fill_text(pos.x as f32, pos.y as f32, text, &paint).unwrap();
    }

    #[inline]
    fn icon_cmd(
        &mut self,
        id: Icon,
        rect: Rect,
        color: Color
    ) {
        let text = match id {
            Icon::Close => "",
            Icon::Resize => "樂",
            Icon::Check => "",
            Icon::Collapsed => "",
            Icon::Expanded => "",
            Icon::None => return
        };

        let paint = Paint::default()
            .with_font(&[self.font_id])
            .with_font_size(FONT_SIZE_PT)
            .with_text_baseline(Baseline::Top)
            .with_text_align(Align::Center)
            .with_color(
                FemtovgColor::rgba(color.r, color.g, color.b, color.a)
            );
            
        let metrics = self.canvas.measure_text(0., 0., text, &paint).unwrap();

        let x = rect.x as f32 + (rect.w as f32 - metrics.width()) / 2.;
        let y = rect.y as f32 + (rect.h as f32 - metrics.height()) / 2.;

        self.canvas.fill_text(x, y, text, &paint).unwrap();
    }
}

impl TextSizeHandler for FemtovgTextSizeHandler {
    #[inline]
    fn text_width(&self, _id: FontId, text: &str) -> i32 {
        let paint = Paint::default()
            .with_font(&[self.font_id])
            .with_font_size(FONT_SIZE_PT);

        let metrics = self.ctx.measure_text(0., 0., text, &paint).unwrap();

        metrics.width() as i32
    }

    #[inline]
    fn text_height(&self, _id: FontId) -> i32 {
        let paint = Paint::default()
            .with_font(&[self.font_id])
            .with_font_size(FONT_SIZE_PT);

        let metrics = self.ctx.measure_font(&paint).unwrap();
        
        metrics.height() as i32
    }
}
