use glam::{IVec2, UVec2};

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
    Resized(UVec2),
    WindowClose,
    RedrawFinished,
    MouseWheel(UVec2, f32),
    MouseMove { position: UVec2, delta: IVec2 },
    MouseButton(MouseButtons, ElementState, UVec2),
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
        mouse_position: &mut Option<UVec2>,
    ) -> WindowEvent {
        match event {
            winit::event::WindowEvent::Resized(size) => {
                WindowEvent::Resized(UVec2::new(size.width.max(1), size.height.max(1)))
            }
            winit::event::WindowEvent::Focused(_is_focused) => WindowEvent::Unknown,
            winit::event::WindowEvent::CursorEntered { .. } => WindowEvent::Unknown,
            winit::event::WindowEvent::CursorLeft { .. } => WindowEvent::Unknown,
            winit::event::WindowEvent::CursorMoved {
                position,
                ..
            } => {
                let new_pos = UVec2::new(position.x as u32, position.y as u32);
                let delta = match mouse_position {
                    Some(prev_pos) => IVec2::try_from(new_pos).unwrap() - IVec2::try_from(*prev_pos).unwrap(),
                    None => IVec2::ZERO,
                };
                *mouse_position = Some(new_pos);

                WindowEvent::MouseMove {
                    position: new_pos,
                    delta,
                }
            }
            winit::event::WindowEvent::Occluded(_is_occluded) => WindowEvent::Unknown,
            winit::event::WindowEvent::MouseInput { state, button, .. } => WindowEvent::MouseButton(
                MouseButtons::from(button),
                ElementState::from(state),
                mouse_position.unwrap_or(UVec2::ZERO),
            ),
            winit::event::WindowEvent::MouseWheel {
                delta,
                phase: _phase,
                ..
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_l1, l2) => {
                    WindowEvent::MouseWheel(mouse_position.unwrap_or(UVec2::ZERO), *l2)
                }
                winit::event::MouseScrollDelta::PixelDelta(_pix) => {
                    WindowEvent::Unknown
                }
            },
            winit::event::WindowEvent::CloseRequested => WindowEvent::WindowClose,
            winit::event::WindowEvent::Moved(_position) => WindowEvent::Unknown,
            _ => WindowEvent::Unknown,
        }
    }
}

