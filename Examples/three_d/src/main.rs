#![allow(dead_code)]

use std::time::Instant;

use wgpu::util::DeviceExt;
use wgpu::DepthStencilState;

use wgpu_app::*;

use crate::geometry::Cube;
use crate::push_const::MvpPushConst;

mod fps;
mod geometry;
mod push_const;

struct App {
    fps_counter: fps::FpsCounter,

    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,
}

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

        let vertex_buffer =
            app_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    contents: cube_geometry.vertex_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                    label: Some("Vertex Buffer"),
                });

        let index_buffer =
            app_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: cube_geometry.index_bytes(),
                    usage: wgpu::BufferUsages::INDEX,
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
                        front_face: wgpu::FrontFace::Ccw,
                        topology: wgpu::PrimitiveTopology::TriangleList,

                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        let img =
            imaginarium::image::Image::read_file("./Examples/three_d/assets/Screenshot_01.png")
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
            depth_texture: None,
            depth_texture_view: None,
        }
    }
}

impl WgpuApp for App {
    fn window_event(&mut self, _app_context: &AppContext, event: WindowEvent) -> EventResult {
        match event {
            WindowEvent::Resized(_new_size) => {
                self.depth_texture = None;
                self.depth_texture_view = None;
                EventResult::Redraw
            }

            _ => EventResult::Continue,
        }
    }

    fn render(
        &mut self,
        app_context: &AppContext,
        surface_view: &wgpu::TextureView,
    ) -> EventResult {
        if self.depth_texture_view.is_none() {
            let depth_texture_extent = wgpu::Extent3d {
                width: app_context.window_size.x,
                height: app_context.window_size.y,
                depth_or_array_layers: 1,
            };
            let depth_texture = app_context.device.create_texture(&wgpu::TextureDescriptor {
                size: depth_texture_extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
                label: None,
            });
            let depth_texture_view =
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.depth_texture = Some(depth_texture);
            self.depth_texture_view = Some(depth_texture_view);
        }
        let depth_texture_view = self.depth_texture_view.as_ref().unwrap();

        let time = (Instant::now() - app_context.start_time).as_secs_f32();

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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mvp = glam::Mat4::perspective_rh_gl(
                45.0_f32.to_radians(),
                app_context.window_size.x as f32 / app_context.window_size.y as f32,
                0.1,
                100.0,
            ) * glam::Mat4::look_at_rh(
                glam::Vec3::new(0.0, 0.0, 5.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ) * glam::Mat4::from_rotation_x(time)
                * glam::Mat4::from_rotation_y(time * 0.3);
            let pc = MvpPushConst { mvp };

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, pc.as_bytes());
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            // render_pass.draw(0..Cube::vertex_count(), 0..1);
            render_pass.draw_indexed(0..Cube::index_count(), 0, 0..1);
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
