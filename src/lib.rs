#![allow(dead_code)]

pub use events::{EventResult, WindowEvent};
pub use wgpu_app::{AppContext, UserEventType, WgpuApp};
pub use wgpu_app::run;

mod events;
mod wgpu_app;

