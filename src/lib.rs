#![allow(dead_code)]

pub use events::{EventResult, WindowEvent};
pub use wgpu_app::run;
pub use wgpu_app::{AppContext, UserEventType, WgpuApp};

mod events;
mod wgpu_app;
