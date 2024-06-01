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
            WindowEvent::RedrawFinished => {
                EventResult::Redraw
            }

            _ => { EventResult::Continue }
        }
    }

    fn render(&mut self, app_context: &AppContext, _surface_view: &wgpu::TextureView) {
        let mut command_encoder =
            app_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });


        app_context.queue.submit([command_encoder.finish()]);
    }
}


fn main() {
    wgpu_app::wgpu_app::run(
        |app_context: &AppContext| Box::new(App::new(app_context))
    );
}