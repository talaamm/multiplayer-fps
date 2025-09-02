use macroquad::prelude::*;
use crate::level::Level;
// use crate::player::RemotePlayer;
use crate::input::InputState;

// ---------- Config ----------
const MOVE_SPEED: f32 = 5.0; // cells/sec
const PLAYER_RADIUS: f32 = 0.10; // radius in cells for collision - very small to allow close combat

// ---------- Collision helpers ----------
fn solid_at(level: &Level, p: Vec2) -> bool {
    let xi = p.x.floor() as i32;
    let yi = p.y.floor() as i32;
    !level.is_walkable(xi, yi) // Not walkable = solid
}

fn collides_circle_grid(level: &Level, pos: Vec2) -> bool {
    let r = PLAYER_RADIUS;
    let checks = [
        vec2(pos.x - r, pos.y - r),
        vec2(pos.x + r, pos.y - r),
        vec2(pos.x - r, pos.y + r),
        vec2(pos.x + r, pos.y + r),
    ];
    for c in checks.iter() {
        if solid_at(level, *c) {
            return true;
        }
    }
    false
}

// Check if position collides with other players (for close combat)
// fn collides_with_players(pos: Vec2, others: &[RemotePlayer], my_id: Option<u64>) -> bool {
//     let min_distance = 0.25; // Minimum distance between players (reduced for closer combat)
    
//     for other in others {
//         // Skip self
//         if let Some(id) = my_id {
//             // We don't have player IDs in RemotePlayer, so we'll use a different approach
//             // For now, we'll allow very close proximity
//             continue;
//         }
        
//         let distance = (pos - other.pos).length();
//         if distance < min_distance {
//             return true;
//         }
//     }
//     false
// }

// ---------- Movement with robust collision ----------
pub fn move_player(level: &Level, player: &mut crate::player::Player, input: &InputState, dt: f32) {
    // rotate (mouse input is already in radians, no need for ROT_SPEED)
    player.dir += input.rot;

    // move vector in facing basis
    let f = vec2(player.dir.cos(), player.dir.sin());
    let r = vec2(-f.y, f.x);
    let wish = f * input.forward + r * input.strafe;

    if wish.length_squared() > 1e-6 {
        // normalize + scale, clamp step to avoid tunneling on very low FPS
        let mut step = wish.normalize() * MOVE_SPEED * dt;
        let max_step = 0.35; // fraction of a cell per frame
        let len = step.length();
        if len > max_step {
            step *= max_step / len;
        }

        // --- move X axis ---
        let try_pos_x = vec2(player.pos.x + step.x, player.pos.y);
        if !collides_circle_grid(level, try_pos_x) {
            player.pos.x = try_pos_x.x;
        }

        // --- move Y axis ---
        let try_pos_y = vec2(player.pos.x, player.pos.y + step.y);
        if !collides_circle_grid(level, try_pos_y) {
            player.pos.y = try_pos_y.y;
        }
    }
}
