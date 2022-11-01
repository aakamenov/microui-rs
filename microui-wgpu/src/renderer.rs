use std::{mem, num::NonZeroU64, ops::Range};

use microui::{Context, CommandHandler, Font, Icon, Color, Rect, Vec2};
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use winit::{
    window::Window,
    event::WindowEvent,
    dpi::PhysicalSize
};
use bytemuck::{Pod, Zeroable};
use pollster::FutureExt;

pub struct Renderer {
    scale_factor: f64,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    screen_size_bind_group: wgpu::BindGroup,
    screen_size_buffer: wgpu::Buffer
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [i32; 2],
    color: [u8; 4]
}

struct Painter<'a> {
    draw_calls: Vec<MicrouiDrawCall>,
    vertices: &'a mut Vec<Vertex>,
    indices: &'a mut Vec<u32>,
    current_quad: u32,
    last_index: u32
}

enum MicrouiDrawCall {
    Mesh(Range<u32>),
    Clip(Rect),
    Text { }
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )
        .block_on()
        .unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        )
        .block_on()
        .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(
            wgpu::include_wgsl!("../shaders/microui.wgsl")
        );

        let screen_size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("microui screen size buffer"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("microui screen suze buffer layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(8)
                    },
                    count: None
                }]
            }
        );

        let screen_size_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("microui bindgroup"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &screen_size_buffer,
                        offset: 0,
                        size: None
                    }),
                }],
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("microui pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[]
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("microui render pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Sint32x2
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[i32; 2]>() as u64,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Unorm8x4
                            }
                        ]
                    }]
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL
                    })]
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Front),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None
            }
        );

        let instance = Self {
            surface,
            device,
            queue,
            config,
            scale_factor: window.scale_factor(),
            pipeline,
            vertices: vec![],
            indices: vec![],
            screen_size_buffer,
            screen_size_bind_group
        };
        instance.write_screen_size_buffer(size);

        instance
    }

    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.config.width, self.config.height)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>, scale_factor: Option<f64>) {
        if let Some(scale_factor) = scale_factor {
            self.scale_factor = scale_factor;
        }

        if size.width == 0 || size.height == 0 {
            return;
        }

        self.write_screen_size_buffer(size);

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {

    }

    pub fn render(&mut self, ctx: &mut Box<Context>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("microui_command encoder")
        });

        let mut painter = Painter::new(&mut self.vertices, &mut self.indices);
        ctx.handle_commands(&mut painter);
        let calls = painter.finish();

        let index_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("microui_index_buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX
        });

        let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("microui_vertex_buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("microui_render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_bind_group(0, &self.screen_size_bind_group, &[]);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for call in calls {
            match call {
                MicrouiDrawCall::Mesh(range) => render_pass.draw_indexed(
                    range,
                    0,
                    0..1
                ),
                MicrouiDrawCall::Clip(rect) => render_pass.set_scissor_rect(
                    rect.x as u32,
                    rect.y as u32,
                    rect.w as u32,
                    rect.h as u32
                ),
                MicrouiDrawCall::Text {  } => todo!()
            }
        }

        drop(render_pass);

        self.queue.submit([encoder.finish()]);
        output.present();
    
        Ok(())
    }

    fn write_screen_size_buffer(&self, size: PhysicalSize<u32>) {
        let logical_size = size.to_logical::<f32>(self.scale_factor);

        self.queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(
                &[logical_size.width as f32, logical_size.height as f32]
            )
        );
    }
}

impl<'a> Painter<'a> {
    fn new(
        vertices: &'a mut Vec<Vertex>,
        indices: &'a mut Vec<u32>
    ) -> Self {
        vertices.clear();
        indices.clear();

        Self {
            draw_calls: vec![],
            vertices,
            indices,
            current_quad: 0,
            last_index: 0
        }
    }

    fn write_mesh_call(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        self.current_quad = 0;

        let len = self.indices.len() as u32;
        self.draw_calls.push(MicrouiDrawCall::Mesh(
            self.last_index..len
        ));

        self.last_index = len;
    }

    #[inline]
    fn finish(mut self) -> Vec<MicrouiDrawCall> {
        self.write_mesh_call();
        self.draw_calls
    }
}

impl<'a> CommandHandler for Painter<'a> {
    #[inline]
    fn clip_cmd(&mut self, rect: Rect) {
        self.write_mesh_call();
        self.draw_calls.push(MicrouiDrawCall::Clip(rect));
    }

    fn rect_cmd(&mut self, rect: Rect, color: Color) {
        self.vertices.extend(&[
            Vertex {
                position: [rect.x, rect.y],
                color: [color.r, color.g, color.b, color.a]
            },
            Vertex {
                position: [rect.x + rect.w, rect.y],
                color: [color.r, color.g, color.b, color.a]
            },
            Vertex {
                position: [rect.x + rect.w, rect.y + rect.h],
                color: [color.r, color.g, color.b, color.a]
            },
            Vertex {
                position: [rect.x, rect.y + rect.h],
                color: [color.r, color.g, color.b, color.a]
            },
        ]);

        self.indices.extend(&[
            self.current_quad * 4 + 0,
            self.current_quad * 4 + 1,
            self.current_quad * 4 + 2,
            self.current_quad * 4 + 0,
            self.current_quad * 4 + 2,
            self.current_quad * 4 + 3,
        ]);

        self.current_quad += 1;
    }

    fn text_cmd(
        &mut self,
        font: Font,
        pos: Vec2,
        color: Color,
        text: String
    ) {
        //self.write_mesh_call();
    }

    fn icon_cmd(
        &mut self,
        id: Icon,
        rect: Rect,
        color: Color
    ) {
        //self.write_mesh_call();
    }
}

unsafe impl Zeroable for Vertex {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

unsafe impl Pod for Vertex { }
