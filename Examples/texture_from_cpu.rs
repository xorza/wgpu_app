use wgpu_app::events::{EventResult, WindowEvent};
use wgpu_app::fullscreen_texture::FullScreenTexture;
use wgpu_app::math::Vec2u32;
use wgpu_app::wgpu_app::{AppContext, WgpuApp};

struct App {
    fullscreen_texture: FullScreenTexture,
    window_size: Vec2u32,
}

impl App {
    fn new(app_context: &AppContext) -> Self {
        let fullscreen_texture = FullScreenTexture::new(
            &app_context.device,
            app_context.surface_config.format,
            app_context.window_size,
        );
        Self::update_texture(app_context, &fullscreen_texture);

        Self {
            fullscreen_texture,
            window_size: app_context.window_size,
        }
    }
}

impl WgpuApp for App {
    fn window_event(&mut self, app_context: &AppContext, event: WindowEvent) -> EventResult {
        match event {
            WindowEvent::Resized(new_size) => {
                if new_size == self.window_size {
                    return EventResult::Continue;
                }

                self.window_size = new_size;
                self.fullscreen_texture
                    .resize_window(&app_context.device, new_size);
                Self::update_texture(app_context, &self.fullscreen_texture);

                EventResult::Redraw
            }
            _ => { EventResult::Continue }
        }
    }

    fn render(&mut self, app_context: &AppContext, surface_view: &wgpu::TextureView) {
        self.fullscreen_texture.render(&app_context.device, &app_context.queue, surface_view);
    }
}

impl App {
    fn update_texture(app_context: &AppContext, fullscreen_texture: &FullScreenTexture) {
        let mut bytes = vec![255u8; (4 * app_context.window_size.x * app_context.window_size.y) as usize];
        for i in 0..app_context.window_size.x {
            for j in 0..app_context.window_size.y {
                let index = (4 * (i + j * app_context.window_size.x)) as usize;
                let i_normalized = (i as f32 / app_context.window_size.x as f32 * 255.0) as u8;
                let j_normalized = (j as f32 / app_context.window_size.y as f32 * 255.0) as u8;

                bytes[index..index + 4].copy_from_slice(&[i_normalized, j_normalized, 0, 255]);
            }
        }

        app_context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: fullscreen_texture.get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            bytes.as_slice(),
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
    }
}

fn main() {
    wgpu_app::wgpu_app::run(
        |app_context: &AppContext| Box::new(App::new(app_context))
    );
}
