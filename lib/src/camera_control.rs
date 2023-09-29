use winit::event::MouseButton;

use crate::mouse_input::MouseState;

// Naive look-at camera.
// This version removes the use of quaternion to avoid adding a dependency.
// To avoid having to do linear algebra ourselves, most computations are done in the shader.
// This is sub-optimal. Improving this is left as an exercise to the reader.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraLookAt {
    // Object the camera is looking at.
    pub center: [f32; 3],
    // Angle around the object on the horizontal plane, in radians.
    pub angle: f32,
    // Height between -1 and 1, 0 is flat, 1 is zenith, -1 is nadir
    pub height: f32,
    // Distance from center
    pub distance: f32,
}

impl Default for CameraLookAt {
    fn default() -> Self {
        // See object in 0,0,0 from the front top left
        CameraLookAt {
            center: [0.0, 0.0, 0.0],
            angle: 2.0 * std::f32::consts::FRAC_PI_3,
            height: 0.3,
            distance: f32::sqrt(72.0),
        }
    }
}

impl CameraLookAt {
    /// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
    pub fn update(&mut self, mouse_state: &MouseState, window_size: [f32; 2]) {
        // change input mapping for orbit and panning here
        let orbit_button = MouseButton::Right;
        let translation_button = MouseButton::Middle;

        if mouse_state.position_delta[0] != 0.0 || mouse_state.position_delta[1] != 0.0 {
            if mouse_state.pressed(orbit_button) {
                let delta_x =
                    mouse_state.position_delta[0] / window_size[0] * std::f32::consts::PI * 2.0;
                let delta_y = mouse_state.position_delta[1] / window_size[1] * std::f32::consts::PI;
                self.angle += delta_x;
                self.height += delta_y;
                self.height = self
                    .height
                    .max(-std::f32::consts::FRAC_PI_2 + 0.001)
                    .min(std::f32::consts::FRAC_PI_2 - 0.001);
            }

            if mouse_state.pressed(translation_button) {
                let dir = [self.angle.cos(), self.angle.sin()];
                let translation_dir = [-dir[1], dir[0]];
                let translation_weight =
                    mouse_state.position_delta[0] / window_size[0] * self.distance;

                self.center[0] += translation_dir[0] * translation_weight;
                self.center[2] += translation_dir[1] * translation_weight;
                self.center[1] += mouse_state.position_delta[1] / window_size[1] * self.distance;
            }
        }

        if mouse_state.scroll_delta != 0.0 {
            self.distance -= mouse_state.scroll_delta * self.distance * 0.2;
            // Don't allow zoom to reach 0 or 1e6 to avoid getting stuck / in float precision issue realm.
            self.distance = self.distance.max(0.05).min(1e6);
        }
    }
}
