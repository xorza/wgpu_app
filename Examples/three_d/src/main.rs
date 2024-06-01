mod geometry;

use std::borrow::Cow;
use wgpu::{LoadOp, Operations, StoreOp};
use wgpu::util::DeviceExt;

use wgpu_app::events::{EventResult, WindowEvent};
use wgpu_app::wgpu_app::{AppContext, WgpuApp};
use crate::geometry::Cube;

struct App {}


impl App {
    fn new(app_context: &AppContext) -> Self {
        let cube_geometry = Cube::default();

        let vertex_buffer_layout = [wgpu::VertexBufferLayout {
            array_stride: Cube::vertex_size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let cube_buf = app_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: cube_geometry.vertex_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
            label: None,
        });



        let sampler = app_context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            border_color: None,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });


        let bind_group_layout = app_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let pipeline_layout = app_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
            label: None,
        });

        let screen_shader = app_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed("")), //include_str!("screen_shader.wgsl")
        });

        let screen_pipeline = app_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &screen_shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layout,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &screen_shader,
                entry_point: "fs_main",
                targets: &[Some(app_context.surface_config.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                front_face: wgpu::FrontFace::Cw,
                topology: wgpu::PrimitiveTopology::TriangleStrip,

                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });



        let img = imaginarium::image::Image::read_file("./Examples/three_d/assets/Screenshot_01.png")
            .unwrap()
            .convert(imaginarium::color_format::ColorFormat::RGBA_U8)
            .unwrap();


        let texture_extent = wgpu::Extent3d {
            width: img.desc.width(),
            height: img.desc.height(),
            depth_or_array_layers: 1,
        };

        let texture = app_context.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            label: None,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = app_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        app_context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            img.bytes.as_slice(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * app_context.window_size.x),
                rows_per_image: Some(app_context.window_size.y),
            },
            wgpu::Extent3d {
                width: app_context.window_size.x,
                height: app_context.window_size.y,
                depth_or_array_layers: 1,
            },
        );

        Self {}
    }
}


impl WgpuApp for App {
    fn window_event(&mut self, _app_context: &AppContext, event: WindowEvent) -> EventResult {
        match event {
            WindowEvent::Resized(_new_size) => {
                EventResult::Redraw
            }

            _ => { EventResult::Continue }
        }
    }

    fn render(&mut self, app_context: &AppContext, surface_view: &wgpu::TextureView) -> EventResult {
        let mut encoder =
            app_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });


        app_context.queue.submit([encoder.finish()]);
        
        EventResult::Redraw
    }
}


fn main() {
    wgpu_app::wgpu_app::run(
        |app_context: &AppContext| Box::new(App::new(app_context))
    );
}