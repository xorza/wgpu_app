use crate::math::{Vec2i32, Vec2u32};

#[derive(PartialEq, Debug, Clone)]
pub enum MouseButtons {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u8),
}

#[derive(PartialEq, Debug, Clone)]
pub enum ElementState {
    Pressed,
    Released,
}

#[derive(PartialEq, Debug, Clone)]
pub enum WindowEvent {
    Resized(Vec2u32),
    WindowClose,
    RedrawFinished,
    MouseWheel(Vec2u32, f32),
    MouseMove { position: Vec2u32, delta: Vec2i32 },
    MouseButton(MouseButtons, ElementState, Vec2u32),
    Unknown,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum EventResult {
    Continue,
    Redraw,
    Exit,
}

impl From<&winit::event::ElementState> for ElementState {
    fn from(value: &winit::event::ElementState) -> Self {
        match value {
            winit::event::ElementState::Pressed => ElementState::Pressed,
            winit::event::ElementState::Released => ElementState::Released,
        }
    }
}
impl From<&winit::event::MouseButton> for MouseButtons {
    fn from(value: &winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => MouseButtons::Left,
            winit::event::MouseButton::Right => MouseButtons::Right,
            winit::event::MouseButton::Middle => MouseButtons::Middle,
            winit::event::MouseButton::Back => MouseButtons::Back,
            winit::event::MouseButton::Forward => MouseButtons::Forward,
            winit::event::MouseButton::Other(other) => MouseButtons::Other(*other as u8),
        }
    }
}

impl WindowEvent {
    pub(crate) fn convert_event(
        event: &winit::event::WindowEvent,
        mouse_position: &mut Vec2u32,
    ) -> WindowEvent {
        match event {
            winit::event::WindowEvent::Resized(size) => {
                WindowEvent::Resized(Vec2u32::new(size.width.max(1), size.height.max(1)))
            }
            winit::event::WindowEvent::Focused(_is_focused) => WindowEvent::Unknown,
            winit::event::WindowEvent::CursorEntered { .. } => WindowEvent::Unknown,
            winit::event::WindowEvent::CursorLeft { .. } => WindowEvent::Unknown,
            winit::event::WindowEvent::CursorMoved {
                position: _position,
                ..
            } => {
                let prev_pos = *mouse_position;
                let new_pos = Vec2u32::new(_position.x as u32, _position.y as u32);
                *mouse_position = new_pos;

                WindowEvent::MouseMove {
                    position: new_pos,
                    delta: Vec2i32::from(new_pos) - Vec2i32::from(prev_pos),
                }
            }
            winit::event::WindowEvent::Occluded(_is_occluded) => WindowEvent::Unknown,
            winit::event::WindowEvent::MouseInput { state, button, .. } => WindowEvent::MouseButton(
                MouseButtons::from(button),
                ElementState::from(state),
                mouse_position.clone(),
            ),
            winit::event::WindowEvent::MouseWheel {
                delta,
                phase: _phase,
                ..
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_l1, l2) => {
                    WindowEvent::MouseWheel(mouse_position.clone(), *l2)
                }
                winit::event::MouseScrollDelta::PixelDelta(pix) => {
                    println!("PIXEL DELTA: {:?}", pix);
                    WindowEvent::Unknown
                }
            },
            winit::event::WindowEvent::CloseRequested => WindowEvent::WindowClose,
            winit::event::WindowEvent::Moved(_position) => WindowEvent::Unknown,
            _ => WindowEvent::Unknown,
        }
    }
}

