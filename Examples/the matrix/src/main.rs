#![allow(dead_code)]

use std::time::Instant;

use wgpu_app::*;

use crate::matrix::Vertex;
use crate::push_const::MvpPushConst;

mod fps;
mod matrix;
mod push_const;

struct App {
    fps_counter: fps::FpsCounter,

    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    matrix: matrix::Matrix,

    vb: Vec<Vertex>,
    ib: Vec<u16>,
}

impl App {
    fn new(app_context: &AppContext) -> Self {
        let vertex_buffer_layout = [wgpu::VertexBufferLayout {
            array_stride: Vertex::size_in_bytes() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 2,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 2,
                },
            ],
        }];

        let vertex_buffer = app_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * 1024 * 5,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = app_context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: 1024 * 1024 * 5,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            app_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                    label: None,
                });

        let pipeline_layout =
            app_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX,
                        range: 0..MvpPushConst::size_in_bytes(),
                    }],
                    label: None,
                });

        let screen_shader = app_context
            .device
            .create_shader_module(wgpu::include_wgsl!("../assets/shader.wgsl"));

        let render_pipeline =
            app_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &screen_shader,
                        entry_point: Some("vs_main"),
                        buffers: &vertex_buffer_layout,
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &screen_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: app_context.surface_config.format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    operation: wgpu::BlendOperation::Add,
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                },
                                alpha: wgpu::BlendComponent {
                                    operation: wgpu::BlendOperation::Add,
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        cull_mode: None,
                        front_face: wgpu::FrontFace::Ccw,
                        topology: wgpu::PrimitiveTopology::TriangleList,

                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        let img =
            imaginarium::image::Image::read_file("./Examples/the matrix/assets/ascii_texture.png")
                .unwrap()
                .convert(imaginarium::color_format::ColorFormat::GRAY_U8)
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
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            label: None,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        app_context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            img.bytes.as_slice(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(img.desc.stride()),
                rows_per_image: Some(img.desc.height()),
            },
            wgpu::Extent3d {
                width: img.desc.width(),
                height: img.desc.height(),
                depth_or_array_layers: 1,
            },
        );

        let sampler = app_context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            border_color: None,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = app_context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
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

        Self {
            fps_counter: fps::FpsCounter::new(),
            render_pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            matrix: matrix::Matrix::new(),
            vb: vec![],
            ib: vec![],
        }
    }
}

impl WgpuApp for App {
    fn window_event(&mut self, _app_context: &AppContext, event: WindowEvent) -> EventResult {
        match event {
            WindowEvent::Resized(_new_size) => EventResult::Redraw,

            _ => EventResult::Continue,
        }
    }

    fn render(
        &mut self,
        app_context: &AppContext,
        surface_view: &wgpu::TextureView,
    ) -> EventResult {
        let time = (Instant::now() - app_context.start_time).as_secs_f32();
        self.matrix.update(time);

        let mvp = if app_context.surface_config.width > app_context.surface_config.height {
            let aspect = (app_context.surface_config.height as f32
                / app_context.surface_config.width as f32
                - 1.0)
                / 2.0;
            glam::Mat4::orthographic_rh(0.0, 1.0, 0.0 - aspect, 1.0 + aspect, 0.0, 1.0)
        } else {
            let aspect = (app_context.surface_config.width as f32
                / app_context.surface_config.height as f32
                - 1.0)
                / 2.0;
            glam::Mat4::orthographic_rh(0.0 - aspect, 1.0 + aspect, 0.0, 1.0, 0.0, 1.0)
        };
        let pc = MvpPushConst { mvp };

        self.matrix.geometry(&mut self.vb, &mut self.ib);
        app_context
            .queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vb));
        app_context
            .queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.ib));

        let mut encoder = app_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, pc.as_bytes());
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw_indexed(0..self.ib.len() as u32, 0, 0..1);
        }

        app_context.queue.submit([encoder.finish()]);

        if self.fps_counter.update() {
            println!("FPS: {}", self.fps_counter.get_fps());
        }

        EventResult::Redraw
    }
}

fn main() {
    wgpu_app::run(|app_context: &AppContext| Box::new(App::new(app_context)));
}
