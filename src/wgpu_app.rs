use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use pollster::FutureExt;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

use crate::events::{EventResult, WindowEvent};
use crate::math::Vec2u32;

#[derive(Debug)]
pub struct AppContext<'window> {
    pub window: Arc<Window>,

    pub surface: wgpu::Surface<'window>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub window_size: Vec2u32,
    pub mouse_position: Vec2u32,

    pub start_time: Instant,

    is_redrawing: bool,
    is_resizing: bool,
}


pub struct UserEventType {}


pub trait WgpuApp {
    fn window_event(&mut self, app_context: &AppContext, event: WindowEvent) -> EventResult;
    // fn update(&mut self, event: Self::UserEventType) -> EventResult;
    fn render(&mut self, app_context: &AppContext, surface_texture_view: &wgpu::TextureView);
}

struct AppState<'window> {
    event_loop_proxy: EventLoopProxy<UserEventType>,

    main_window_context: Option<AppContext<'window>>,

    start_time: Instant,

    app: Option<Box<dyn WgpuApp>>,
    app_ctor: fn(&AppContext) -> Box<dyn WgpuApp>,
}


impl<'window> ApplicationHandler<UserEventType> for AppState<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.main_window_context.is_some() {
            panic!("Resumed called twice");
        }

        let window_attr = Window::default_attributes()
            .with_title("title");
        let window = Arc::new(event_loop.create_window(window_attr).unwrap());

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
                dxil_path: None,
                dxc_path: None,
            },
            gles_minor_version: Default::default(),
            flags: Default::default(),
        });

        // let surface = unsafe {
        //     let target = wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap();
        //     instance.create_surface_unsafe(target).unwrap()
        // };
        let surface = instance.create_surface(window.clone()).unwrap();


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
            max_push_constant_size: 64,
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

        self.main_window_context = Some(AppContext {
            window: window.clone(),
            surface,
            surface_config,
            device,
            queue,
            mouse_position: Vec2u32::new(0, 0),
            window_size: Vec2u32::new(size.width, size.height),
            is_redrawing: false,
            is_resizing: false,
            start_time: self.start_time,
        });

        let app = (self.app_ctor)(self.main_window_context.as_ref().unwrap());
        self.app = Some(app);

        window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _user_event: UserEventType) {
        //...
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                self.redraw();
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(_new_size) => {
                let app_context = self.main_window_context.as_mut().unwrap();
                let window_size = physical_size_to_vec2u32(app_context.window.inner_size());
                if window_size == app_context.window_size {
                    return;
                }
                app_context.is_resizing = true;
            }

            _ => {
                let window_context = self.main_window_context.as_mut().unwrap();
                let event = WindowEvent::convert_event(&event, &mut window_context.mouse_position);
                if !matches!(event, WindowEvent::Unknown) {
                    self.app.as_mut().unwrap().window_event(window_context, event);
                }
            }
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: DeviceId, _event: DeviceEvent) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let window_context = self.main_window_context.as_mut().unwrap();

        let resize_result =
            if window_context.is_resizing {
                window_context.is_resizing = false;

                let window_size = physical_size_to_vec2u32(window_context.window.inner_size());
                window_context.window_size = window_size;
                window_context.surface_config.width = window_size.x;
                window_context.surface_config.height = window_size.y;
                window_context.surface.configure(&window_context.device, &window_context.surface_config);

                self.app.as_mut().unwrap().window_event(window_context, WindowEvent::Resized(window_size))
            } else {
                EventResult::Continue
            };

        let finish_redraw_result =
            if window_context.is_redrawing {
                window_context.is_redrawing = false;

                if let Some(error) = window_context.device.pop_error_scope().block_on() {
                    panic!("Device error: {:?}", error);
                }

                self.app.as_mut().unwrap().window_event(window_context, WindowEvent::RedrawFinished)
            } else { EventResult::Continue };

        match (resize_result, finish_redraw_result) {
            (EventResult::Exit, _) | (_, EventResult::Exit) => {
                event_loop.exit();
            }
            (EventResult::Redraw, _) | (_, EventResult::Redraw) => {
                window_context.window.request_redraw();
            }

            _ => {}
        }
    }
}

impl<'window> AppState<'window> {
    fn redraw(&mut self) {
        let app_context = self.main_window_context.as_mut().unwrap();
        let surface = &app_context.surface;

        app_context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        app_context.is_redrawing = true;

        let surface_texture = surface
            .get_current_texture()
            .unwrap_or_else(|_| {
                surface
                    .configure(&app_context.device, &app_context.surface_config);
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture.")
            });
        let surface_texture_view =
            surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(app_context.surface_config.format),
                    ..wgpu::TextureViewDescriptor::default()
                });

        self.app.as_mut().unwrap().render(app_context, &surface_texture_view);

        surface_texture.present();
    }
}

pub fn run(app_ctor: fn(&AppContext) -> Box<dyn WgpuApp>) {
    let event_loop: EventLoop<UserEventType> = EventLoop::<UserEventType>::with_user_event()
        .build()
        .unwrap();
    let mut app_state = AppState {
        event_loop_proxy: event_loop.create_proxy(),
        main_window_context: None,
        start_time: Instant::now(),
        app: None,
        app_ctor,
    };
    event_loop.run_app(&mut app_state).unwrap();
}

fn physical_size_to_vec2u32(size: winit::dpi::PhysicalSize<u32>) -> Vec2u32 {
    Vec2u32::new(size.width, size.height)
}
