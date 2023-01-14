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

use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use femtovg::{
    Canvas, TextContext, Baseline, Align, Paint, Path,
    FontId as FemtovgFontId, Color as FemtovgColor,
    renderer::OpenGl
};

const DEFAULT_FONT: &[u8] = include_bytes!("../../fonts/ProggyClean.ttf");
const FONT_SIZE_PT: f32 = 16.0;

pub struct Renderer {
    ctx: ContextWrapper<PossiblyCurrent, Window>,
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
        let ctx_builder = ContextBuilder::new()
            .with_vsync(false)
            .build_windowed(window_builder, &event_loop)
            .unwrap();

        let ctx = unsafe { ctx_builder.make_current().unwrap() };
        let renderer = unsafe {
            OpenGl::new_from_function(|s| ctx.get_proc_address(s) as *const _)
        }.unwrap();

        let text_context = TextContext::default();
        let mut canvas = Canvas::new_with_text_context(renderer, text_context.clone()).unwrap();
        let font_id = canvas.add_font_mem(DEFAULT_FONT).unwrap();

        let size = ctx.window().inner_size();
        canvas.set_size(size.width, size.height, ctx.window().scale_factor() as f32);

        let renderer = Renderer {
            ctx,
            canvas,
            text_context,
            font_id,
            clear_color: FemtovgColor::black()
        };

        renderer
    }

    #[inline]
    fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: f64) {
        self.ctx.resize(size);
        self.canvas.set_size(size.width, size.height, scale_factor as f32);
    }

    #[inline]
    fn window(&self) -> &Window {
        self.ctx.window()
    }

    #[inline]
    fn render(&mut self, ctx: &mut Context, clear_color: Option<Color>) {
        if let Some(color) = clear_color {
            self.clear_color = FemtovgColor::rgba(color.r, color.g, color.b, color.a);
        }

        let size = self.ctx.window().inner_size();
        self.canvas.clear_rect(0, 0, size.width, size.height, self.clear_color);

        ctx.handle_commands(self);

        self.canvas.flush();
        self.ctx.swap_buffers().unwrap();
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

        self.canvas.fill_path(&mut path, paint);
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

        self.canvas.fill_text(pos.x as f32, pos.y as f32, text, paint).unwrap();
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
            
        let metrics = self.canvas.measure_text(0., 0., text, paint).unwrap();

        let x = rect.x as f32 + (rect.w as f32 - metrics.width()) / 2.;
        let y = rect.y as f32 + (rect.h as f32 - metrics.height()) / 2.;

        self.canvas.fill_text(x, y, text, paint).unwrap();
    }
}

impl TextSizeHandler for FemtovgTextSizeHandler {
    #[inline]
    fn text_width(&self, _id: FontId, text: &str) -> i32 {
        let paint = Paint::default()
            .with_font(&[self.font_id])
            .with_font_size(FONT_SIZE_PT);

        let metrics = self.ctx.measure_text(0., 0., text, paint).unwrap();

        metrics.width() as i32
    }

    #[inline]
    fn text_height(&self, _id: FontId) -> i32 {
        let paint = Paint::default()
            .with_font(&[self.font_id])
            .with_font_size(FONT_SIZE_PT);

        let metrics = self.ctx.measure_font(paint).unwrap();
        
        metrics.height() as i32
    }
}
