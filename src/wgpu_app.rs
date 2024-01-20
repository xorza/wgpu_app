use std::default::Default;
use std::fmt::Debug;
use std::time::Instant;

use bytemuck::Zeroable;
use pollster::FutureExt;
use winit::event_loop::{EventLoop as WinitEventLoop, EventLoopBuilder};

use crate::event::{convert_event, Event, EventLoop, EventResult};
use crate::math::Vec2u32;

pub trait WgpuApp: 'static {
    type UserEventType: Send + Debug + 'static;

    fn new(runtime: &Runtime<Self::UserEventType>) -> Self;

    fn update(
        &mut self,
        runtime: &Runtime<Self::UserEventType>,
        event: Event<Self::UserEventType>,
    ) -> EventResult;

    fn render(&mut self, runtime: &Runtime<Self::UserEventType>, surface_view: &wgpu::TextureView);
}

pub struct Runtime<'a, UserEventType: 'static> {
    pub window: &'a winit::window::Window,
    pub event_loop: EventLoop<UserEventType>,
    // instance: wgpu::Instance,
    // size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    // adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub start: Instant,

    pub window_size: Vec2u32,
    pub mouse_position: Vec2u32,
}

pub fn run<AppType: WgpuApp>(title: &str) {
    // setup

    let event_loop: WinitEventLoop<AppType::UserEventType> =
        EventLoopBuilder::<AppType::UserEventType>::with_user_event()
            .build()
            .unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        // .with_inner_size(LogicalSize::new(1024, 512))
        .build(&event_loop)
        .expect("Failed to create window.");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
            dxil_path: None,
            dxc_path: None,
        },
        gles_minor_version: Default::default(),
        flags: Default::default(),
    });
    let size = window.inner_size();
    let surface = instance.create_surface(&window).unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .block_on()
        .expect("No suitable GPU adapters found on the system.");

    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
    let required_limits = wgpu::Limits {
        max_push_constant_size: 1024,
        ..Default::default()
    }
        .using_resolution(adapter.limits());

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::PUSH_CONSTANTS,
                required_limits,
            },
            None,
        )
        .block_on()
        .expect("Unable to find a suitable GPU adapter.");

    let mut surface_config = surface
        .get_default_config(&adapter, size.width, size.height)
        .expect("Surface isn't supported by the adapter.");
    surface_config.format = surface_config.format.add_srgb_suffix();
    surface_config.view_formats.push(surface_config.format);
    surface.configure(&device, &surface_config);

    // run
    let mut is_redrawing = false;
    let mut is_resizing = false;

    let mut runtime = Runtime {
        window: &window,
        event_loop: EventLoop {
            event_loop_proxy: event_loop.create_proxy(),
        },
        surface,
        surface_config,
        device,
        queue,
        start: Instant::now(),
        mouse_position: Vec2u32::zeroed(),
        window_size: Vec2u32::new(size.width, size.height),
    };

    let mut app = AppType::new(&runtime);
    match app.update(&runtime, Event::Init) {
        EventResult::Continue => {}
        EventResult::Redraw => {}
        EventResult::Exit => return,
    }

    event_loop.run(move |event, target| {
        let mut result: EventResult = EventResult::Continue;

        match event {
            winit::event::Event::AboutToWait => {
                if is_resizing {
                    is_resizing = false;

                    let window_size = Vec2u32::new(
                        runtime.window.inner_size().width.max(1),
                        runtime.window.inner_size().height.max(1),
                    );
                    runtime.window_size = window_size;
                    runtime.surface_config.width = window_size.x;
                    runtime.surface_config.height = window_size.y;
                    runtime.surface
                        .configure(&runtime.device, &runtime.surface_config);

                    result = app.update(&runtime, Event::Resized(window_size));
                } else if is_redrawing {
                    is_redrawing = false;

                    if let Some(error) = runtime.device.pop_error_scope().block_on() {
                        panic!("Device error: {:?}", error);
                    }

                    result = app.update(&runtime, Event::RedrawFinished);
                }
            }
            winit::event::Event::WindowEvent {
                event:
                winit::event::WindowEvent::RedrawRequested,
                ..
            } => {
                assert!(!is_redrawing);
                runtime
                    .device
                    .push_error_scope(wgpu::ErrorFilter::Validation);
                is_redrawing = true;

                let surface_texture = runtime
                    .surface
                    .get_current_texture()
                    .unwrap_or_else(|_| {
                        runtime.surface
                            .configure(&runtime.device, &runtime.surface_config);
                        runtime.surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture.")
                    });
                let surface_texture_view =
                    surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor {
                            format: Some(runtime.surface_config.format),
                            ..wgpu::TextureViewDescriptor::default()
                        });

                app.render(&runtime, &surface_texture_view);

                surface_texture.present();
            }

            winit::event::Event::WindowEvent {
                event:
                winit::event::WindowEvent::Resized(size),
                ..
            } => {
                if size.width != runtime.window_size.x || size.height != runtime.window_size.y {
                    is_resizing = true;
                }
            }
            winit::event::Event::WindowEvent { event, .. } => {
                let event = convert_event(event, &mut runtime.mouse_position);
                if !matches!(event, Event::Unknown) {
                    result = app.update(&runtime, event);
                }
            }
            winit::event::Event::UserEvent(event) => {
                result = app.update(&runtime, Event::Custom(event));
            }

            _ => {}
        }

        match result {
            EventResult::Continue => {}
            EventResult::Redraw => runtime.window.request_redraw(),
            EventResult::Exit => target.exit(),
        }
    }).expect("TODO: panic message");
}
