use wgpu::{LoadOp, Operations, StoreOp};

use wgpu_app::events::{EventResult, WindowEvent};
use wgpu_app::wgpu_app::{AppContext, WgpuApp};

struct App {}


impl App {
    fn new(_app_context: &AppContext) -> Self {
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