use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

// Naive mouse state.
#[derive(Debug, Default)]
pub struct MouseState {
    pub position: [f32; 2],
    pub position_delta: [f32; 2],
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub scroll_delta: f32,
}

impl MouseState {
    pub fn pressed(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.left,
            MouseButton::Right => self.right,
            MouseButton::Middle => self.middle,
            _ => false,
        }
    }

    // Called on Event::RedrawRequested since it's the start of a new frame.
    pub fn clear_deltas(&mut self) {
        self.position_delta = [0.0, 0.0];
        self.scroll_delta = 0.0;
    }

    // Called on relevant window events.
    pub fn on_window_event(&mut self, window_event: &WindowEvent) {
        self.position_delta = [0.0, 0.0];
        self.scroll_delta = 0.0;
        match *window_event {
            WindowEvent::MouseInput { button, state, .. } => match button {
                MouseButton::Left => self.left = state == ElementState::Pressed,
                MouseButton::Right => self.right = state == ElementState::Pressed,
                MouseButton::Middle => self.middle = state == ElementState::Pressed,
                _ => (),
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.position_delta = [
                    position.x as f32 - self.position[0],
                    position.y as f32 - self.position[1],
                ];
                self.position = [position.x as f32, position.y as f32];
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    // native mode: line delta should be 1 or -1
                    MouseScrollDelta::LineDelta(_, y) => self.scroll_delta = y,
                    // wasm: pixel delta is around 100 * display_scale
                    MouseScrollDelta::PixelDelta(pos) => self.scroll_delta = pos.y as f32 / 100.0,
                }
            }
            _ => (),
        }
    }
}
