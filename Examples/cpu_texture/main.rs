use fullscreen_texture::FullScreenTexture;
use wgpu_app::*;

mod fullscreen_texture;
mod screen_rect;

struct App {
    fullscreen_texture: FullScreenTexture,
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
        }
    }
}

impl WgpuApp for App {
    fn window_event(&mut self, app_context: &AppContext, event: WindowEvent) -> EventResult {
        match event {
            WindowEvent::Resized(new_size) => {
                self.fullscreen_texture.resize_window(&app_context.device, new_size);
                Self::update_texture(app_context, &self.fullscreen_texture);

                EventResult::Redraw
            }

            _ => { EventResult::Continue }
        }
    }

    fn render(&mut self, app_context: &AppContext, surface_view: &wgpu::TextureView) -> EventResult {
        let mut command_encoder =
            app_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.fullscreen_texture.render(&mut command_encoder, surface_view);

        app_context.queue.submit([command_encoder.finish()]);

        EventResult::Continue
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
    wgpu_app::run(
        |app_context: &AppContext| Box::new(App::new(app_context))
    );
}
