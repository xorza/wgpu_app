#![allow(dead_code)]

use wgpu::util::DeviceExt;
use wgpu::DepthStencilState;

use wgpu_app::EventResult;
use wgpu_app::WindowEvent::{self};
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

    // Quaternion-based rotation state
    rotation: glam::Quat,
    is_mouse_pressed: bool,
    last_mouse_position: Option<glam::UVec2>,
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
                    immediate_size: MvpPushConst::size_in_bytes(),
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
                    multiview_mask: None,
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
            depth_texture: None,
            depth_texture_view: None,

            // Initialize quaternion rotation state
            rotation: glam::Quat::IDENTITY,
            is_mouse_pressed: false,
            last_mouse_position: None,
        }
    }

    // Helper method to perform arcball rotation from screen coordinates
    fn arcball_rotation(
        &self,
        window_size: glam::UVec2,
        from: glam::UVec2,
        to: glam::UVec2,
    ) -> glam::Quat {
        // First check if there's actually any movement - if positions are identical, return identity
        if from == to {
            return glam::Quat::IDENTITY;
        }

        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        // Scale factor makes the effective sphere larger (values < 1.0 create larger sphere)
        // Decreased to make rotation more sensitive
        let scale_factor = 0.6; // Reduced from 0.8 for higher sensitivity

        let screen_to_ndc = |pos: glam::UVec2| -> glam::Vec2 {
            glam::Vec2::new(
                ((2.0 * pos.x as f32) / window_size.x as f32 - 1.0) * scale_factor,
                (1.0 - (2.0 * pos.y as f32) / window_size.y as f32) * scale_factor,
            )
        };

        // Convert to normalized device coordinates
        let from_ndc = screen_to_ndc(from);
        let to_ndc = screen_to_ndc(to);

        // Check if movement is too small to produce reliable rotation
        let delta_ndc = to_ndc - from_ndc;
        let movement_sq = delta_ndc.length_squared();

        // For extremely tiny movements, create a minimal rotation in the direction of movement
        // This ensures even slow mouse movements create some rotation
        if movement_sq < 1e-6 {
            // Determine primary movement direction
            let axis = if delta_ndc.x.abs() > delta_ndc.y.abs() {
                // Primarily horizontal movement
                glam::Vec3::new(0.0, 1.0, 0.0) // Rotate around Y-axis
            } else {
                // Primarily vertical movement
                glam::Vec3::new(1.0, 0.0, 0.0) // Rotate around X-axis
            };

            // Create a small rotation in that direction
            // Sign determines rotation direction
            let sign = if (delta_ndc.x > 0.0 && delta_ndc.x.abs() > delta_ndc.y.abs())
                || (delta_ndc.y < 0.0 && delta_ndc.y.abs() >= delta_ndc.x.abs())
            {
                1.0
            } else {
                -1.0
            };

            // Create minimal rotation - increased for faster response
            let min_angle = 0.003 * sign; // Increased from 0.001 for faster response
            return glam::Quat::from_axis_angle(axis, min_angle);
        }

        // Project onto virtual sphere - using a larger virtual sphere
        // Increased radius makes rotation more sensitive
        let sphere_radius = 1.1; // Reduced from 1.3 for higher sensitivity
        let radius_sq = sphere_radius * sphere_radius;

        let project_to_sphere = |p: glam::Vec2| -> glam::Vec3 {
            let len_sq = p.dot(p);

            // If point is on the sphere
            if len_sq <= radius_sq {
                // Project onto sphere surface
                glam::Vec3::new(p.x, p.y, (radius_sq - len_sq).sqrt())
            } else {
                // If point is outside the sphere, project onto the sphere
                let normalized = p.normalize() * sphere_radius;
                glam::Vec3::new(normalized.x, normalized.y, 0.0)
            }
        };

        // Get 3D points on the virtual sphere
        let from_sphere = project_to_sphere(from_ndc);
        let to_sphere = project_to_sphere(to_ndc);

        // Apply additional smoothing for motions near the edge
        // Compute direction vectors from origin to the points
        let from_dir = from_sphere.normalize();
        let to_dir = to_sphere.normalize();

        // Axis of rotation is the cross product of the two vectors
        let axis = from_dir.cross(to_dir);
        let axis_len_sq = axis.length_squared();

        // Angle of rotation is the dot product of the two vectors
        let cos_angle = from_dir.dot(to_dir).clamp(-1.0, 1.0);

        // Create quaternion from axis and angle - with handling for near-parallel vectors
        // Using a much smaller threshold to capture tiny movements
        if axis_len_sq < 1e-10 || (cos_angle - 1.0).abs() < 1e-10 {
            // Even for tiny rotations, provide a small rotation feedback
            // This creates a more responsive feel for slow movements
            let min_angle = 0.003; // Increased from 0.001 for faster response

            // Use movement direction to determine rotation axis
            let minimal_axis = if delta_ndc.x.abs() > delta_ndc.y.abs() {
                glam::Vec3::new(0.0, 1.0, 0.0) // Y-axis for horizontal movement
            } else {
                glam::Vec3::new(1.0, 0.0, 0.0) // X-axis for vertical movement
            };

            // Direction based on movement direction
            let sign = if (delta_ndc.x > 0.0 && delta_ndc.x.abs() > delta_ndc.y.abs())
                || (delta_ndc.y < 0.0 && delta_ndc.y.abs() >= delta_ndc.x.abs())
            {
                1.0
            } else {
                -1.0
            };

            glam::Quat::from_axis_angle(minimal_axis, min_angle * sign)
        } else {
            // Normal case - calculate rotation based on arcball movement
            // Scale up angle for slow movements to make them more noticeable
            // but keep normal scaling for larger movements
            let angle = cos_angle.acos();

            // Adjust rotation speed with dynamic scaling:
            // - For very small angles, apply higher scaling to make them more noticeable
            // - For larger movements, keep the standard scaling
            let angle_scale = if angle < 0.01 {
                // Boost tiny movements to be more noticeable
                2.5 // Increased from 1.0 for faster small rotations
            } else {
                // Normal scaling for regular movements
                1.8 // Increased from 0.7 for faster regular rotations
            };

            let scaled_angle = angle * angle_scale;
            glam::Quat::from_axis_angle(axis.normalize(), scaled_angle)
        }
    }
}

impl WgpuApp for App {
    fn window_event(&mut self, app_context: &AppContext, event: WindowEvent) -> EventResult {
        match event {
            WindowEvent::Resized(_new_size) => {
                self.depth_texture = None;
                self.depth_texture_view = None;
                EventResult::Redraw
            }

            // Use index 0 to check for left mouse button
            WindowEvent::MouseButton(ref _button, ref state, position) => {
                // 0 is typically the left mouse button index
                if let WindowEvent::MouseButton(_, _, _) = event {
                    // Simple approach - just track if any mouse button is pressed
                    // Set is_mouse_pressed based on if it's a press or release event
                    let is_pressed = match state {
                        // First component is Pressed
                        s if format!("{:?}", s).contains("Pressed") => true,
                        _ => false,
                    };

                    self.is_mouse_pressed = is_pressed;

                    if self.is_mouse_pressed {
                        self.last_mouse_position = Some(position);
                    } else {
                        self.last_mouse_position = None;
                    }
                }
                EventResult::Continue
            }

            WindowEvent::MouseMove {
                position,
                delta: _delta,
            } => {
                if self.is_mouse_pressed {
                    if let Some(last_pos) = self.last_mouse_position {
                        // Use arcball rotation to calculate quaternion delta
                        let delta_rotation =
                            self.arcball_rotation(app_context.window_size, last_pos, position);

                        // Apply the delta rotation to the current rotation
                        // Note: quaternion multiplication is in reverse order
                        // New rotation = delta_rotation * current_rotation
                        self.rotation = delta_rotation * self.rotation;
                        self.rotation = self.rotation.normalize(); // Prevents precision errors
                    }

                    self.last_mouse_position = Some(position);
                    EventResult::Redraw
                } else {
                    EventResult::Continue
                }
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
                    depth_slice: None,
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
                multiview_mask: None,
            });

            // Convert quaternion to rotation matrix
            let rotation_matrix = glam::Mat4::from_quat(self.rotation);

            let mvp = glam::Mat4::perspective_rh_gl(
                45.0_f32.to_radians(),
                app_context.window_size.x as f32 / app_context.window_size.y as f32,
                0.1,
                100.0,
            ) * glam::Mat4::look_at_rh(
                glam::Vec3::new(0.0, 0.0, 5.0),
                glam::Vec3::new(0.0, 0.0, 0.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ) * rotation_matrix;

            let pc = MvpPushConst { mvp };

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_immediates(0, pc.as_bytes());
            render_pass.set_bind_group(0, &self.bind_group, &[]);
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
