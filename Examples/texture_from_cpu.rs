use wgpu_app::event::{Event, EventResult};
use wgpu_app::fullscreen_texture::FullScreenTexture;
use wgpu_app::math::Vec2u32;
use wgpu_app::wgpu_app::{Runtime, WgpuApp};

struct App {
    fullscreen_texture: FullScreenTexture,
    window_size: Vec2u32,
}

type UserRuntime<'a> = Runtime<'a, <App as WgpuApp>::UserEventType>;

impl WgpuApp for App {
    type UserEventType = ();

    fn new(runtime: &UserRuntime) -> Self {
        let fullscreen_texture = FullScreenTexture::new(
            &runtime.device,
            runtime.surface_config.format,
            runtime.window_size,
        );
        Self::update_texture(runtime, &fullscreen_texture);

        Self {
            fullscreen_texture,
            window_size: runtime.window_size,
        }
    }

    fn update(
        &mut self,
        runtime: &UserRuntime,
        event: Event<Self::UserEventType>,
    ) -> EventResult {
        match event {
            Event::Init => EventResult::Continue,
            Event::WindowClose => EventResult::Exit,
            Event::Resized(new_size) => {
                if new_size == self.window_size {
                    return EventResult::Continue;
                }

                self.window_size = new_size;
                self.fullscreen_texture
                    .resize_window(&runtime.device, new_size);
                Self::update_texture(runtime, &self.fullscreen_texture);

                EventResult::Redraw
            }
            _ => EventResult::Continue,
        }
    }

    fn render(&mut self, runtime: &UserRuntime, surface_view: &wgpu::TextureView) {
        self.fullscreen_texture
            .render(&runtime.device, &runtime.queue, surface_view);
    }
}

impl App {
    fn update_texture(runtime: &Runtime<()>, fullscreen_texture: &FullScreenTexture) {
        let mut bytes = vec![255u8; (4 * runtime.window_size.x * runtime.window_size.y) as usize];
        for i in 0..runtime.window_size.x {
            for j in 0..runtime.window_size.y {
                let index = (4 * (i + j * runtime.window_size.x)) as usize;
                let i_normalized = (i as f32 / runtime.window_size.x as f32 * 255.0) as u8;
                let j_normalized = (j as f32 / runtime.window_size.y as f32 * 255.0) as u8;

                bytes[index..index + 4].copy_from_slice(&[i_normalized, j_normalized, 0, 255]);
            }
        }

        runtime.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: fullscreen_texture.get_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            bytes.as_slice(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * runtime.window_size.x),
                rows_per_image: Some(runtime.window_size.y),
            },
            wgpu::Extent3d {
                width: runtime.window_size.x,
                height: runtime.window_size.y,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn main() {
    wgpu_app::wgpu_app::run::<App>("Hello, world!");
}
