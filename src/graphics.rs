#![allow(dead_code)]


use bytemuck;
use winit::{window::Window, event::{WindowEvent, ElementState, MouseButton}};
use wgpu::{self, TextureViewDescriptor, TextureAspect, RenderPipeline, ColorTargetState, MultisampleState, util::{DeviceExt, BufferInitDescriptor}};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window)}.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions{ 
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface), 
                force_fallback_adapter: false,
            }
        ).await.unwrap();

        //println!("{:?}", adapter.features());
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            }, 
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync, // Maybe switch to PresentMode::Fifo for Vsync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor { 
                label: Some("Shader"), 
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()) 
            }
        );
        let pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader, 
                entry_point: "vs_main", 
                buffers: &[
                    Vertex::desc(),
                ], 
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: "fs_main", 
                targets: &[Some(ColorTargetState{
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES_2),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(
            &BufferInitDescriptor{
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(INDICES_2),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = INDICES_2.len() as u32;

        Self{
            surface,
            device,
            queue,
            config,
            size,
            window,
            vertex_buffer,
            render_pipeline,
            clear_color: wgpu::Color { 
                r: 0.1, g: 0.2, b: 0.3, a: 1.0 
            },
            num_indices,
            index_buffer,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
         //for some reason rust-analyzer doesnt recognize TextureViewDescriptor::default
        let desc = TextureViewDescriptor{
            label: None,
            format: None,
            dimension: None,
            aspect: TextureAspect::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };
        let view = output.texture.create_view(&desc);
        drop(desc);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder")
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(self.clear_color), 
                    store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        drop(render_pass);
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match *event {
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left , ..} => {
                self.clear_color = wgpu::Color { 
                    r: 0.5, g: 0.1, b: 0.1, a: 1.0 
                }
            },
            _ => (),
        }
        false
    }

    pub fn update(&mut self) {
        todo!()
    }

}

#[repr(C)]
#[derive(Copy,Clone,Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex{
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -0.1, 0.0], color: [0.4, 0.0, 0.4] }, // A
    Vertex { position: [-0.16, -0.1, 0.0], color: [0.4, 0.0, 0.4] }, // B 
    Vertex { position: [-1.0, 0.2, 0.0], color: [0.4, 0.0, 0.4] }, // C 

    Vertex { position: [-1.0, -0.8, 0.0], color: [0.4, 0.0, 0.4] }, // D
    Vertex { position: [-0.16, -0.8, 0.0], color: [0.4, 0.0, 0.4] }, // E
    Vertex { position: [-1.0, -1.0, 0.0], color: [0.4, 0.0, 0.4] }, // F


    Vertex { position: [1.0, -0.1, 0.0], color: [0.4, 0.0, 0.4] }, // G
    Vertex { position: [0.16, -0.1, 0.0], color: [0.4, 0.0, 0.4] }, // H
    Vertex { position: [1.0, 0.2, 0.0], color: [0.4, 0.0, 0.4] }, // I

    Vertex { position: [1.0, -0.8, 0.0], color: [0.4, 0.0, 0.4] }, // J
    Vertex { position: [0.16, -0.8, 0.0], color: [0.4, 0.0, 0.4] }, // K
    Vertex { position: [1.0, -1.0, 0.0], color: [0.4, 0.0, 0.4] }, // L


    Vertex { position: [0.16, -1.0, 0.0], color: [0.6, 0.52, 0.8] }, // M
    Vertex { position: [-0.16, -1.0, 0.0], color: [0.6, 0.52, 0.8] }, // N
    ];

const INDICES: &[u16] = &[
    0, 1, 2,
    3, 4, 0,
    1, 0, 4, 
    4, 3, 5,

    7, 6, 8,
    10, 9, 6,
    6, 7, 10, 
    8, 10, 11,

    4, 5, 13,
    10, 12, 11,

    13, 12, 10,
    10, 4, 13,
];

const VERTICES_2: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
];

const INDICES_2: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];