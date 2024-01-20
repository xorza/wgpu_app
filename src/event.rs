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
pub enum Event<UserEvent> {
    Init,
    Resized(Vec2u32),
    WindowClose,
    RedrawFinished,
    MouseWheel(Vec2u32, f32),
    MouseMove { position: Vec2u32, delta: Vec2i32 },
    MouseButton(MouseButtons, ElementState, Vec2u32),
    Custom(UserEvent),
    Unknown,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum EventResult {
    Continue,
    Redraw,
    Exit,
}

#[derive(Debug, Clone)]
pub struct EventLoop<UserEventType: 'static> {
    pub(crate) event_loop_proxy: winit::event_loop::EventLoopProxy<UserEventType>,
}

impl From<winit::event::ElementState> for ElementState {
    fn from(value: winit::event::ElementState) -> Self {
        match value {
            winit::event::ElementState::Pressed => ElementState::Pressed,
            winit::event::ElementState::Released => ElementState::Released,
        }
    }
}
impl From<winit::event::MouseButton> for MouseButtons {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => MouseButtons::Left,
            winit::event::MouseButton::Right => MouseButtons::Right,
            winit::event::MouseButton::Middle => MouseButtons::Middle,
            winit::event::MouseButton::Back => MouseButtons::Back,
            winit::event::MouseButton::Forward => MouseButtons::Forward,
            winit::event::MouseButton::Other(other) => MouseButtons::Other(other as u8),
        }
    }
}

impl<UserEventType: Send + 'static> EventLoop<UserEventType> {
    pub fn send_event(&self, event: UserEventType) -> anyhow::Result<()> {
        self.event_loop_proxy
            .send_event(event)
            .map_err(|_| anyhow::anyhow!("Failed to send event to event loop."))
    }
}

pub(crate) fn convert_event<UserEvent>(
    event: winit::event::WindowEvent,
    mouse_position: &mut Vec2u32,
) -> Event<UserEvent> {
    match event {
        winit::event::WindowEvent::Resized(size) => {
            Event::Resized(Vec2u32::new(size.width.max(1), size.height.max(1)))
        }
        winit::event::WindowEvent::Focused(_is_focused) => Event::Unknown,
        winit::event::WindowEvent::CursorEntered { .. } => Event::Unknown,
        winit::event::WindowEvent::CursorLeft { .. } => Event::Unknown,
        winit::event::WindowEvent::CursorMoved {
            position: _position,
            ..
        } => {
            let prev_pos = *mouse_position;
            let new_pos = Vec2u32::new(_position.x as u32, _position.y as u32);
            *mouse_position = new_pos;

            Event::MouseMove {
                position: new_pos,
                delta: Vec2i32::from(new_pos) - Vec2i32::from(prev_pos),
            }
        }
        winit::event::WindowEvent::Occluded(_is_occluded) => Event::Unknown,
        winit::event::WindowEvent::MouseInput { state, button, .. } => Event::MouseButton(
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
                Event::MouseWheel(mouse_position.clone(), l2)
            }
            winit::event::MouseScrollDelta::PixelDelta(pix) => {
                println!("PIXEL DELTA: {:?}", pix);
                Event::Unknown
            }
        },
        winit::event::WindowEvent::CloseRequested => Event::WindowClose,
        winit::event::WindowEvent::Moved(_position) => Event::Unknown,
        _ => Event::Unknown,
    }
}
