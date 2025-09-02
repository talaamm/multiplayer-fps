use macroquad::prelude::*;
use crate::level::Level;
use crate::player::{Player, RemotePlayer, draw_player_with_skin};

// ---------- Config ----------
const FOV_DEG: f32 = 70.0;
const RENDER_SCALE: f32 = 1.0;

// ---------- Bullet ----------
#[derive(Clone, Copy)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    // pub angle: f32,
    // pub lifetime: f32,
}

// ---------- Minimap ----------
pub fn draw_minimap(level: &Level, player: &Player, others: &[RemotePlayer], bullets: &[Bullet]) {
    let map_scale = 4.0;
    let pad = 8.0;
    let w = level.w as f32 * map_scale;
    let h = level.h as f32 * map_scale;

    draw_rectangle(
        pad - 2.0,
        pad - 2.0,
        w + 4.0,
        h + 4.0,
        Color::from_rgba(0, 0, 0, 160),
    );

    // ✅ PATH (t==0) = WHITE, WALL (t==1) = DARKGREEN, SPAWN (t==2) = BLUE, COVER (t==3) = BROWN
    for y in 0..level.h {
        for x in 0..level.w {
            let t = level.tiles[y * level.w + x];
            let c = match t {
                1 => DARKGREEN, // Wall
                2 => BLUE,      // Spawn point
                3 => BROWN,     // Cover
                _ => WHITE,     // Path
            };
            draw_rectangle(
                pad + x as f32 * map_scale,
                pad + y as f32 * map_scale,
                map_scale,
                map_scale,
                c,
            );
        }
    }

    // player
    let px = pad + player.pos.x * map_scale;
    let py = pad + player.pos.y * map_scale;
    draw_circle(px, py, 1.5, YELLOW); // Smaller player dot

    // facing line
    let p2 = vec2(px, py) + vec2(player.dir.cos(), player.dir.sin()) * 6.0; // Shorter facing line
    draw_line(px, py, p2.x, p2.y, 1.0, ORANGE);

    // other players + facing arrows
    for rp in others.iter() {
        let ox = pad + rp.pos.x * map_scale;
        let oy = pad + rp.pos.y * map_scale;
        draw_circle(ox, oy, 1.5, RED); // Smaller other player dots
        // facing arrow
        let ax = rp.angle.cos() as f32;
        let ay = rp.angle.sin() as f32;
        let tip = vec2(ox, oy) + vec2(ax, ay) * (4.0); // Shorter facing arrows
        draw_line(ox, oy, tip.x, tip.y, 1.0, ORANGE);
    }

    // bullets
    for bullet in bullets {
        let bx = pad + bullet.x * map_scale;
        let by = pad + bullet.y * map_scale;
        draw_circle(bx, by, 1.0, ORANGE);
    }
}

// ---------- Raycasting (DDA) ----------
pub fn draw_world(level: &Level, player: &Player, others: &[RemotePlayer], bullets: &[Bullet]) {
    let (sw, sh) = (
        screen_width() * RENDER_SCALE,
        screen_height() * RENDER_SCALE,
    );
    let fov = FOV_DEG.to_radians();
    let half_fov = fov * 0.5;
    let num_cols = sw as i32;

    // Sky & floor
    draw_rectangle(0.0, 0.0, sw, sh * 0.5, Color::from_rgba(30, 30, 50, 255));
    draw_rectangle(
        0.0,
        sh * 0.5,
        sw,
        sh * 0.5,
        Color::from_rgba(25, 35, 25, 255),
    );

    // Depth buffer per column for occlusion (z-buffer)
    let mut zbuffer = vec![f32::INFINITY; num_cols as usize];

    for col in 0..num_cols {
        let colf = col as f32;
        let cam_x = (2.0 * colf / sw - 1.0) * (half_fov).tan();
        let ray_dir = vec2(player.dir.cos(), player.dir.sin())
            + vec2(-player.dir.sin(), player.dir.cos()) * cam_x;

        // DDA setup
        let mut map_x = player.pos.x.floor() as i32;
        let mut map_y = player.pos.y.floor() as i32;

        let delta_dist = vec2(
            if ray_dir.x.abs() < 1e-6 {
                1e30
            } else {
                (1.0 / ray_dir.x).abs()
            },
            if ray_dir.y.abs() < 1e-6 {
                1e30
            } else {
                (1.0 / ray_dir.y).abs()
            },
        );

        let step_x = if ray_dir.x < 0.0 { -1 } else { 1 };
        let step_y = if ray_dir.y < 0.0 { -1 } else { 1 };

        let mut side_dist = vec2(
            if ray_dir.x < 0.0 {
                (player.pos.x - map_x as f32) * delta_dist.x
            } else {
                ((map_x as f32 + 1.0) - player.pos.x) * delta_dist.x
            },
            if ray_dir.y < 0.0 {
                (player.pos.y - map_y as f32) * delta_dist.y
            } else {
                ((map_y as f32 + 1.0) - player.pos.y) * delta_dist.y
            },
        );

        // DDA loop
        let mut side = 0; // 0: x hit, 1: y hit
        let mut hit = false;
        for _ in 0..1024 {
            if side_dist.x < side_dist.y {
                side_dist.x += delta_dist.x;
                map_x += step_x;
                side = 0;
            } else {
                side_dist.y += delta_dist.y;
                map_y += step_y;
                side = 1;
            }

            let tile = level.at(map_x, map_y);
            if tile == 1 || tile == 2 || tile == 3 {
                // Wall, spawn point, or cover
                hit = true;
                break;
            }
        }
        if !hit {
            continue;
        }

        // Perp distance to avoid fisheye
        let perp_dist = if side == 0 {
            (map_x as f32 - player.pos.x + (1 - step_x) as f32 / 2.0) / ray_dir.x
        } else {
            (map_y as f32 - player.pos.y + (1 - step_y) as f32 / 2.0) / ray_dir.y
        }
        .abs()
        .max(0.0001);

        let line_h = (sh / perp_dist).min(sh);
        let y0 = (sh * 0.5 - line_h * 0.5).max(0.0);
        let y1 = (y0 + line_h).min(sh);

        // Different colors for different tile types
        let tile = level.at(map_x, map_y);
        let base = match tile {
            2 => { // Spawn point - blue
                if side == 0 {
                    Color::from_rgba(100, 100, 255, 255)
                } else {
                    Color::from_rgba(80, 80, 200, 255)
                }
            }
            3 => { // Cover - brown
                if side == 0 {
                    Color::from_rgba(139, 69, 19, 255)
                } else {
                    Color::from_rgba(101, 67, 33, 255)
                }
            }
            _ => { // Wall - gray with side-based shading
                if side == 0 {
                    Color::from_rgba(190, 190, 200, 255)
                } else {
                    Color::from_rgba(120, 120, 140, 255)
                }
            }
        };

        draw_line(colf, y0, colf, y1, 1.0, base);

        // store depth
        zbuffer[col as usize] = perp_dist;
    }

    // Enhanced billboard rendering for other players with skins
    for rp in others.iter() {
        let to = rp.pos - player.pos;
        let dir = vec2(player.dir.cos(), player.dir.sin());
        let right = vec2(-dir.y, dir.x);
        let depth = to.dot(dir);
        if depth <= 0.05 {
            continue;
        }
        let lateral = to.dot(right);
        let fov = FOV_DEG.to_radians();
        let half = (fov * 0.5).tan();
        let screen_x = (0.5 + (lateral / depth) / (2.0 * half)) * sw;
        let perp = depth.max(0.0001);
        let sprite_h = (sh / perp).clamp(12.0, sh * 0.8);
        let sprite_w = sprite_h * 0.35; // aspect ratio of a person
        let y0 = sh * 0.5 - sprite_h * 0.5;
        let y1 = y0 + sprite_h;
        let x0 = (screen_x - sprite_w * 0.5).max(0.0);
        let x1 = (screen_x + sprite_w * 0.5).min(sw);
        // occlusion test using center column under the sprite
        let col = (screen_x.round() as i32).clamp(0, num_cols - 1) as usize;
        let occluded = perp >= zbuffer[col] - 0.001;
        if x1 > 0.0 && x0 < sw && !occluded {
            // Draw player with skin
            draw_player_with_skin(screen_x, y0, sprite_w, sprite_h, rp.skin, rp.angle, sh, perp);
            
            // name tag above
            let name_y = (y0 - 12.0).max(0.0);
            let tw = measure_text(&rp.name, None, 14, 1.0);
            draw_text(
                &rp.name,
                (screen_x - tw.width * 0.5).max(0.0),
                name_y,
                14.0,
                WHITE,
            );
        }
    }

    // Render bullets as tiny dots
    for bullet in bullets {
        let to = vec2(bullet.x, bullet.y) - player.pos;
        let dir = vec2(player.dir.cos(), player.dir.sin());
        let right = vec2(-dir.y, dir.x);
        let depth = to.dot(dir);
        if depth <= 0.05 {
            continue;
        }
        let lateral = to.dot(right);
        let fov = FOV_DEG.to_radians();
        let half = (fov * 0.5).tan();
        let screen_x = (0.5 + (lateral / depth) / (2.0 * half)) * sw;
        let perp = depth.max(0.0001);
        let sprite_size = (sh / perp).clamp(1.0, sh * 0.02); // Much smaller size - tiny dots
        let y0 = sh * 0.5 - sprite_size * 0.5;
        let x0 = screen_x - sprite_size * 0.5;
        let col = (screen_x.round() as i32).clamp(0, num_cols - 1) as usize;
        let occluded = perp >= zbuffer[col] - 0.001;
        if x0 > 0.0 && x0 < sw && y0 > 0.0 && y0 < sh && !occluded {
            draw_circle(
                screen_x,
                sh * 0.5,
                sprite_size * 0.5,
                Color::from_rgba(255, 255, 0, 255), // Bright yellow for better visibility
            );
        }
    }
}

// ---------- Screen Flash ----------
pub fn draw_screen_flash(flash_timer: f32) {
    if flash_timer > 0.0 {
        let alpha = (flash_timer / 0.1).min(1.0); // Flash duration: 0.1 seconds
        let flash_color = Color::from_rgba(255, 255, 255, (alpha * 100.0) as u8); // Semi-transparent white
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), flash_color);
    }
}

// ---------- Crosshair ----------
pub fn draw_crosshair() {
    let center_x = screen_width() * 0.5;
    let center_y = screen_height() * 0.5;
    let size = 8.0;
    let thickness = 2.0;
    
    // Draw crosshair lines
    // Horizontal line
    draw_rectangle(
        center_x - size,
        center_y - thickness * 0.5,
        size * 2.0,
        thickness,
        WHITE,
    );
    
    // Vertical line
    draw_rectangle(
        center_x - thickness * 0.5,
        center_y - size,
        thickness,
        size * 2.0,
        WHITE,
    );
    
    // Draw center dot
    draw_circle(center_x, center_y, 1.0, WHITE);
}

// ---------- HUD ----------
pub fn draw_hud(
    level: &Level,
    rtt_ms: Option<u64>,
    username: &str,
    player_count: usize,
    mouse_captured: bool,
    player: &Player,
    // has_moved_locally: bool,
    map_change_mode: bool,
) {
    let fps = macroquad::time::get_fps();
    let ping_txt = match rtt_ms {
        Some(v) => format!("{} ms", v),
        None => "--".to_string(),
    };
    let txt = format!(
        "Ping: {ping_txt}   Players: {player_count}\nUser: {username}   Map: {}\nWASD move, Mouse look, Left Click shoot, F1 change map",
        level.name
    );
    let fpstxt = format!("FPS: {fps}");
    draw_text(&txt, 10.0, screen_height() - 60.0, 20.0, WHITE);
    draw_text(&fpstxt, 10.0, screen_height() - 40.0, 20.0, WHITE);

    // Health and ammo display
    let health_color = if player.health > 50 { GREEN } else if player.health > 25 { YELLOW } else { RED };
    let health_txt = format!("Health: {}", player.health);
    draw_text(&health_txt, 10.0, screen_height() - 80.0, 20.0, health_color);
    
    let ammo_txt = format!("Ammo: {}", player.ammo);
    draw_text(&ammo_txt, 10.0, screen_height() - 100.0, 20.0, WHITE);
    
    let stats_txt = format!("Kills: {} | Deaths: {}", player.kills, player.deaths);
    draw_text(&stats_txt, 10.0, screen_height() - 120.0, 16.0, WHITE);

    // Debug: Show key states
    let w_pressed = if is_key_down(KeyCode::W) { "W" } else { " " };
    let a_pressed = if is_key_down(KeyCode::A) { "A" } else { " " };
    let s_pressed = if is_key_down(KeyCode::S) { "S" } else { " " };
    let d_pressed = if is_key_down(KeyCode::D) { "D" } else { " " };
    let shoot_pressed = if is_mouse_button_down(MouseButton::Left) { "SHOOT" } else { "     " };
    let debug_txt = format!(
        "Keys: [{}][{}][{}][{}] [{}]",
        w_pressed, a_pressed, s_pressed, d_pressed, shoot_pressed
    );
    draw_text(&debug_txt, 10.0, screen_height() - 140.0, 16.0, YELLOW);

    // Debug: Show mouse capture state
    let mouse_state = if mouse_captured { "CAPTURED" } else { "FREE" };
    draw_text(
        &format!("Mouse: {}", mouse_state),
        10.0,
        screen_height() - 160.0,
        16.0,
        YELLOW,
    );

    // Debug: Show player position
    draw_text(
        &format!("Pos: ({:.1}, {:.1})", player.pos.x, player.pos.y),
        10.0,
        screen_height() - 180.0,
        16.0,
        YELLOW,
    );

    // Show map change mode status
    if map_change_mode {
        draw_text(
            "MAP CHANGE MODE ACTIVE",
            10.0,
            screen_height() - 200.0,
            16.0,
            Color::from_rgba(255, 255, 0, 255), // Bright yellow
        );
    }
}
