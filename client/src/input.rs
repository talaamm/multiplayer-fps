use macroquad::prelude::*;

// ---------- Input ----------
#[derive(Default)]
pub struct InputState {
    pub forward: f32,
    pub strafe: f32,
    pub rot: f32,
    pub shoot: bool,
}

pub fn gather_input(mouse_captured: bool) -> InputState {
    let mut s = InputState::default();

    // Movement keys (should work regardless of mouse capture)
    if is_key_down(KeyCode::W) {
        s.forward += 1.0;
    }
    if is_key_down(KeyCode::S) {
        s.forward -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        s.strafe += 1.0;
    }
    if is_key_down(KeyCode::A) {
        s.strafe -= 1.0;
    }

    // Shooting
    s.shoot = is_mouse_button_down(MouseButton::Left);

    // Mouse rotation (only when captured)
    if mouse_captured {
        let mouse_delta = mouse_delta_position();
        s.rot = -mouse_delta.x * 0.3; // MOUSE_SENSITIVITY
    }

    s
}
