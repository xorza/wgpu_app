use std::borrow::Cow;

use wgpu::util::DeviceExt;

use crate::math::Vec2u32;
use crate::render_pods::ScreenRect;

pub struct FullScreenTexture {
    window_size: Vec2u32,

    screen_rect_buf: wgpu::Buffer,
    screen_pipeline: wgpu::RenderPipeline,

    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    bind_group: TextureBindGroup,
}

struct TextureBindGroup {
    pub texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}


impl FullScreenTexture {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window_size: Vec2u32,
    ) -> Self {
        let vertex_buffer_layout = [wgpu::VertexBufferLayout {
            array_stride: ScreenRect::vert_size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let screen_rect_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: ScreenRect::default().as_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
            label: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            border_color: None,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
                label: None,
            });

        let screen_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("screen_shader.wgsl"))),
        });

        let screen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &screen_shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: &screen_shader,
                entry_point: "fs_main",
                targets: &[
                    Some(surface_format.into()),
                ],
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

        let bind_group = TextureBindGroup::new(
            device,
            &bind_group_layout,
            &sampler,
            window_size,
        );

        Self {
            bind_group,
            sampler,
            bind_group_layout,
            window_size,
            screen_rect_buf,
            screen_pipeline,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_view: &wgpu::TextureView,
    ) {
        let mut command_encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = command_encoder
                .begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: surface_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: true,
                                },
                            }),
                        ],
                        depth_stencil_attachment: None,
                    }
                );

            render_pass.set_pipeline(&self.screen_pipeline);
            render_pass.set_vertex_buffer(0, self.screen_rect_buf.slice(..));

            render_pass.set_bind_group(0, &self.bind_group.bind_group, &[]);
            render_pass.draw(0..ScreenRect::vert_count(), 0..1);
        }

        queue.submit(Some(command_encoder.finish()));
    }

    pub fn resize_window(&mut self, device: &wgpu::Device, window_size: Vec2u32) {
        self.window_size = window_size;

        self.bind_group = TextureBindGroup::new(
            device,
            &self.bind_group_layout,
            &self.sampler,
            window_size,
        );
    }

    pub fn get_texture(&self) -> &wgpu::Texture {
        &self.bind_group.texture
    }
}

impl TextureBindGroup {
    pub fn new(
        device: &wgpu::Device,
        bindind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        window_size: Vec2u32) -> Self {
        let texture_extent = wgpu::Extent3d {
            width: window_size.x,
            height: window_size.y,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            label: None,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bindind_group_layout,
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
            texture,
            bind_group,
        }
    }
}
