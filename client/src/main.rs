use macroquad::prelude::*;
use protocol;
mod network;

// ---------- Local Level definition (client-side single-player prototype) ----------
#[derive(Clone, Debug)]
struct Level {
    w: usize,
    h: usize,
    tiles: Vec<u8>, // 0 = floor/path, 1 = wall
}

impl Level {
    fn new(w: usize, h: usize, tiles: Vec<u8>) -> Self { Self { w, h, tiles } }

    // Safe cell access: out-of-bounds are treated as walls
    fn at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 { return 1; }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h { return 1; }
        self.tiles[y * self.w + x]
    }

    // Three preset mazes with increasing complexity (roughly mirrors server difficulty)
    fn preset(id: u8) -> Self {
        match id {
            1 => Self::preset_small_open(),
            2 => Self::preset_medium_grid(),
            3 => Self::preset_large_maze(),
            _ => Self::preset_small_open(),
        }
    }

    fn preset_small_open() -> Self {
        let w = 15usize; let h = 15usize;
        let mut tiles = vec![1u8; w * h];
        // carve outer ring and some paths (L-shape + branches)
        let mut carve = |x: usize, y: usize| tiles[y * w + x] = 0;
        for x in 1..=13 { carve(x, 1); }
        for y in 1..=13 { carve(13, y); }
        for x in 3..12 { carve(x, 5); carve(x, 9); }
        for y in 3..12 { carve(5, y); carve(9, y); }
        for (x,y) in [(3,3),(4,4),(5,5),(11,3),(10,4),(9,5),(7,7),(8,7),(8,8),(7,8),
                      (2,7),(2,8),(2,9),(12,7),(12,8),(12,9),(4,7),(4,8),(4,9),
                      (10,7),(10,8),(10,9),(6,3),(6,4),(8,3),(8,4)] { carve(x,y); }
        Self::new(w, h, tiles)
    }

    fn preset_medium_grid() -> Self {
        let w = 25usize; let h = 25usize;
        let mut tiles = vec![1u8; w * h];
        let mut carve = |x: usize, y: usize| tiles[y * w + x] = 0;
        for x in 1..24 { carve(x, 1); carve(x, 7); carve(x, 13); carve(x, 19); carve(x, 23); }
        for y in 1..24 { carve(1, y); carve(7, y); carve(13, y); carve(19, y); carve(23, y); }
        for (x,y) in [(3,3),(3,4),(3,5),(4,3),(5,3),(21,3),(21,4),(21,5),(20,5),(19,5),
                      (9,9),(9,10),(9,11),(10,9),(11,9),(8,11),(7,11),(15,9),(15,10),(15,11),
                      (16,9),(17,9),(14,11),(13,11),(3,21),(3,22),(4,21),(5,21),(21,21),(21,22),
                      (20,21),(19,21)] { carve(x,y); }
        Self::new(w, h, tiles)
    }

    fn preset_large_maze() -> Self {
        // Simple generated-like pattern: checker corridors with many dead ends
        let w = 40usize; let h = 40usize;
        let mut tiles = vec![1u8; w * h];
        let mut carve = |x: usize, y: usize| tiles[y * w + x] = 0;
        // carve a grid of corridors every 2 cells
        for y in (1..h-1).step_by(2) {
            for x in 1..w-1 { carve(x, y); }
        }
        for x in (1..w-1).step_by(2) {
            for y in 1..h-1 { carve(x, y); }
        }
        // add some extra openings
        for (x,y) in [(5,5),(6,5),(7,5),(8,5),(9,5),(15,15),(16,15),(17,15),(18,15),
                      (25,25),(26,25),(27,25),(35,35),(36,35),(10,10),(11,10),(12,10),
                      (30,30),(31,30),(32,30)] { carve(x,y); }
        Self::new(w, h, tiles)
    }
}

// ---------- Config ----------
const FOV_DEG: f32 = 70.0;
const MOVE_SPEED: f32 = 3.5;     // cells/sec
const MOUSE_SENSITIVITY: f32 = 0.3; // rad/pixel
const RENDER_SCALE: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.20; // radius in cells for collision
const CROSSHAIR_SIZE: f32 = 8.0;
const SHOOT_FLASH_TIME: f32 = 0.08; // seconds
const DAMAGE_FLASH_TIME: f32 = 0.12; // seconds

// ---------- Player ----------
#[derive(Clone, Copy)]
struct Player {
    pos: Vec2,
    dir: f32, // radians
}
impl Player {
    fn new(x: f32, y: f32, dir: f32) -> Self { Self { pos: vec2(x, y), dir } }
}

// ---------- Minimap ----------
fn draw_minimap(level: &Level, player: &Player, others: &[RemotePlayer]) {
    let map_scale = 4.0;
    let pad = 8.0;
    let w = level.w as f32 * map_scale;
    let h = level.h as f32 * map_scale;

    draw_rectangle(pad - 2.0, pad - 2.0, w + 4.0, h + 4.0, Color::from_rgba(0,0,0,160));

    // ✅ PATH (t==0) = WHITE, WALL (t==1) = DARKGREEN
    for y in 0..level.h {
        for x in 0..level.w {
            let t = level.tiles[y * level.w + x];
            let c = if t == 1 { DARKGREEN } else { WHITE };
            draw_rectangle(
                pad + x as f32 * map_scale,
                pad + y as f32 * map_scale,
                map_scale, map_scale, c
            );
        }
    }

    // player
    let px = pad + player.pos.x * map_scale;
    let py = pad + player.pos.y * map_scale;
    draw_circle(px, py, 2.0, YELLOW);

    // facing line
    let p2 = vec2(px, py) + vec2(player.dir.cos(), player.dir.sin()) * 8.0;
    draw_line(px, py, p2.x, p2.y, 1.0, ORANGE);

    // other players + facing arrows
    for rp in others.iter() {
        let ox = pad + rp.pos.x * map_scale;
        let oy = pad + rp.pos.y * map_scale;
        draw_circle(ox, oy, 2.0, RED);
        // facing arrow
        let ax = rp.angle.cos() as f32;
        let ay = rp.angle.sin() as f32;
        let tip = vec2(ox, oy) + vec2(ax, ay) * (6.0);
        draw_line(ox, oy, tip.x, tip.y, 1.5, ORANGE);
    }
}

// ---------- Collision helpers ----------
fn solid_at(level: &Level, p: Vec2) -> bool {
    let xi = p.x.floor() as i32;
    let yi = p.y.floor() as i32;
    level.at(xi, yi) == 1 // 1 = wall
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
        if solid_at(level, *c) { return true; }
    }
    false
}

// ---------- Movement with robust collision ----------
fn move_player(level: &Level, player: &mut Player, input: &InputState, dt: f32) {
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
        if len > max_step { step *= max_step / len; }

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

// ---------- Raycasting (DDA) ----------
fn draw_world(level: &Level, player: &Player, others: &[RemotePlayer], tracer_time: f32) {
    let (sw, sh) = (screen_width() * RENDER_SCALE, screen_height() * RENDER_SCALE);
    let fov = FOV_DEG.to_radians();
    let half_fov = fov * 0.5;
    let num_cols = sw as i32;

    // Sky & floor
    draw_rectangle(0.0, 0.0, sw, sh * 0.5, Color::from_rgba(30, 30, 50, 255));
    draw_rectangle(0.0, sh * 0.5, sw, sh * 0.5, Color::from_rgba(25, 35, 25, 255));

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
            if ray_dir.x.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir.x).abs() },
            if ray_dir.y.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir.y).abs() },
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

            if level.at(map_x, map_y) == 1 {
                hit = true;
                break;
            }
        }
        if !hit { continue; }

        // Perp distance to avoid fisheye
        let perp_dist = if side == 0 {
            (map_x as f32 - player.pos.x + (1 - step_x) as f32 / 2.0) / ray_dir.x
        } else {
            (map_y as f32 - player.pos.y + (1 - step_y) as f32 / 2.0) / ray_dir.y
        }.abs().max(0.0001);

        let line_h = (sh / perp_dist).min(sh);
        let y0 = (sh * 0.5 - line_h * 0.5).max(0.0);
        let y1 = (y0 + line_h).min(sh);

        // Gray walls with side-based shading
        let base = if side == 0 {
            Color::from_rgba(190, 190, 200, 255)
        } else {
            Color::from_rgba(120, 120, 140, 255)
        };

        draw_line(colf, y0, colf, y1, 1.0, base);

        // store depth
        zbuffer[col as usize] = perp_dist;
    }

    // Simple billboard rendering for other players (body + head), with occlusion
    for rp in others.iter() {
        let to = rp.pos - player.pos;
        let dir = vec2(player.dir.cos(), player.dir.sin());
        let right = vec2(-dir.y, dir.x);
        let depth = to.dot(dir);
        if depth <= 0.05 { continue; }
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
            // body
            draw_rectangle(x0, y0, (x1 - x0).max(1.0), (y1 - y0).max(1.0), Color::from_rgba(210, 80, 80, 255));
            // head: small circle at upper third
            let head_y = y0 + sprite_h * 0.25;
            let head_r = (sprite_w * 0.35).max(2.0);
            draw_circle(screen_x, head_y, head_r, Color::from_rgba(240, 200, 180, 255));
            // facing arrow on the body (project a small arrow along rp.angle)
            let ah = (sh / perp) * 0.08; // arrow length in screen space
            let ax = rp.angle.cos();
            let ay = rp.angle.sin();
            // approximate screen offset: map lateral displacement along player's right and up screen
            let arrow_x = screen_x;
            let arrow_y = y0 + sprite_h * 0.6;
            draw_line(arrow_x, arrow_y, arrow_x + ax as f32 * ah, arrow_y - ay as f32 * ah, 2.0, YELLOW);
            // name tag above
            let name_y = (y0 - 12.0).max(0.0);
            let tw = measure_text(&rp.name, None, 14, 1.0);
            draw_text(&rp.name, (screen_x - tw.width * 0.5).max(0.0), name_y, 14.0, WHITE);
        }
    }

    // Bullet tracer: draw a bright center-column line for a brief time
    if tracer_time > 0.0 {
        // recompute center column wall extents (cam_x=0)
        let cam_x = 0.0;
        let ray_dir = vec2(player.dir.cos(), player.dir.sin())
            + vec2(-player.dir.sin(), player.dir.cos()) * cam_x;

        let mut map_x = player.pos.x.floor() as i32;
        let mut map_y = player.pos.y.floor() as i32;
        let delta_dist = vec2(
            if ray_dir.x.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir.x).abs() },
            if ray_dir.y.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir.y).abs() },
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
        let mut side = 0;
        for _ in 0..1024 {
            if side_dist.x < side_dist.y { side_dist.x += delta_dist.x; map_x += step_x; side = 0; }
            else { side_dist.y += delta_dist.y; map_y += step_y; side = 1; }
            if level.at(map_x, map_y) == 1 { break; }
        }
        let perp_dist = if side == 0 {
            (map_x as f32 - player.pos.x + (1 - step_x) as f32 / 2.0) / ray_dir.x
        } else {
            (map_y as f32 - player.pos.y + (1 - step_y) as f32 / 2.0) / ray_dir.y
        }.abs().max(0.0001);
        let line_h = (sh / perp_dist).min(sh);
        let y0 = (sh * 0.5 - line_h * 0.5).max(0.0);
        let y1 = (y0 + line_h).min(sh);
        let cx = sw * 0.5;
        draw_line(cx, y0, cx, y1, 2.0, WHITE);
    }
}

// ---------- Input ----------
#[derive(Default)]
struct InputState { forward: f32, strafe: f32, rot: f32 }
fn gather_input(mouse_captured: bool) -> InputState {
    let mut s = InputState::default();
    if is_key_down(KeyCode::W) { s.forward += 1.0; }
    if is_key_down(KeyCode::S) { s.forward -= 1.0; }
    if is_key_down(KeyCode::D) { s.strafe  += 1.0; }
    if is_key_down(KeyCode::A) { s.strafe  -= 1.0; }

    // Mouse rotation (only when captured)
    if mouse_captured {
        let mouse_delta = mouse_delta_position();
        s.rot = -mouse_delta.x * MOUSE_SENSITIVITY;
    }

    s
}

// ---------- HUD ----------
fn draw_hud(level_id: u8, rtt_ms: Option<u64>, username: &str, player_count: usize, health: Option<u8>, score: Option<u32>) {
    let fps = macroquad::time::get_fps();
    let ping_txt = match rtt_ms { Some(v) => format!("{} ms", v), None => "--".to_string() };
    let hp_txt = health.map(|h| h.to_string()).unwrap_or("--".to_string());
    let sc_txt = score.map(|s| s.to_string()).unwrap_or("--".to_string());
    let txt = format!(
        "FPS: {fps}   Ping: {ping_txt}   Players: {player_count}   HP: {hp_txt}   Score: {sc_txt}\nUser: {username}   Level: {level_id}  (press 1/2/3)\nWASD move, Mouse look"
    );
    draw_text(&txt, 10.0, screen_height() - 40.0, 20.0, WHITE);
    // Crosshair
    let cx = screen_width() * 0.5;
    let cy = screen_height() * 0.5;
    draw_line(cx - CROSSHAIR_SIZE, cy, cx + CROSSHAIR_SIZE, cy, 1.0, WHITE);
    draw_line(cx, cy - CROSSHAIR_SIZE, cx, cy + CROSSHAIR_SIZE, 1.0, WHITE);
}

// ---------- Remote players ----------
#[derive(Clone, Debug)]
struct RemotePlayer { pos: Vec2, angle: f32, name: String }

// ---------- Protocol adapter ----------
fn level_from_maze_level(wire: &protocol::MazeLevel) -> Level {
    let w = wire.width as usize;
    let h = wire.height as usize;
    let mut tiles = vec![1u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let c = &wire.cells[y * w + x];
            // Consider any wall flag as a wall; otherwise floor
            let is_wall = c.wall_north || c.wall_south || c.wall_east || c.wall_west;
            tiles[y * w + x] = if is_wall { 1 } else { 0 };
        }
    }
    Level::new(w, h, tiles)
}

// ---------- Main ----------
#[macroquad::main("Maze Wars — Client (Single Player Prototype)")]
async fn main() {
    let mut level_id: u8 = 1;
    let mut level = Level::preset(level_id);
    let mut player = Player::new(2.5, 2.5, 0.0);
    let mut mouse_captured = false;
    show_mouse(true);

    // --- Simple UI for IP + username ---
    enum AppState { Connect, Playing }
    let mut app_state = AppState::Connect;
    let mut server_addr = String::from("127.0.0.1:34254");
    let mut username = String::from("player");
    let mut input_focus = 0; // 0=server,1=username
    let mut net: Option<network::NetClient> = None;

    // Tracks whether we replaced local level with server-provided level
    let mut accepted_level = false;
    // Player id assigned by server after Accept
    let mut my_player_id: Option<u64> = None;
    // Storage for other players
    let mut others: Vec<RemotePlayer> = Vec::new();
    // Reconciliation target for our own position
    let mut self_target_pos: Vec2 = player.pos;
    // Ping/latency state
    #[derive(Clone, Copy)]
    struct PingInfo { last_nonce: u64, last_send: f64, rtt_ms: u64 }
    let mut ping_state: Option<PingInfo> = None;
    let mut ping_timer: f32 = 0.0;
    let mut shoot_flash: f32 = 0.0;
    let mut dmg_flash: f32 = 0.0;
    let mut my_health: Option<u8> = None;
    let mut my_score: Option<u32> = None;

    loop {
        if is_key_pressed(KeyCode::Key1) { level_id = 1; level = Level::preset(1); player = Player::new(2.5, 2.5, 0.0); }
        if is_key_pressed(KeyCode::Key2) { level_id = 2; level = Level::preset(2); player = Player::new(2.5, 2.5, 0.0); }
        if is_key_pressed(KeyCode::Key3) { level_id = 3; level = Level::preset(3); player = Player::new(2.5, 2.5, 0.0); }

        let dt = macroquad::time::get_frame_time();

        // Toggle mouse capture: Left Click to capture, Esc to release
        if !mouse_captured && is_mouse_button_pressed(MouseButton::Left) {
            set_cursor_grab(true);
            show_mouse(false);
            mouse_captured = true;
        }
        if mouse_captured && is_key_pressed(KeyCode::Escape) {
            set_cursor_grab(false);
            show_mouse(true);
            mouse_captured = false;
        }

        clear_background(BLACK);

        match app_state {
            AppState::Connect => {
                // Render a simple input form
                let title = "Connect to Server";
                let tw = measure_text(title, None, 32, 1.0);
                draw_text(title, (screen_width()-tw.width)*0.5, 120.0, 32.0, WHITE);

                let label1 = "Server (IP:PORT):";
                draw_text(label1, 200.0, 200.0, 24.0, GRAY);
                let label2 = "Username:";
                draw_text(label2, 200.0, 260.0, 24.0, GRAY);

                // Input boxes
                let bx = 380.0; let bw = screen_width()-bx-200.0; let bh = 32.0;
                let by1 = 175.0; let by2 = 235.0;
                draw_rectangle_lines(bx-4.0, by1-24.0, bw+8.0, bh+8.0, 2.0, if input_focus==0 { YELLOW } else { DARKGRAY });
                draw_rectangle_lines(bx-4.0, by2-24.0, bw+8.0, bh+8.0, 2.0, if input_focus==1 { YELLOW } else { DARKGRAY });
                draw_text(&server_addr, bx, by1, 28.0, WHITE);
                draw_text(&username, bx, by2, 28.0, WHITE);

                let hint = "Tab switch, Enter connect";
                draw_text(hint, bx, by2+40.0, 20.0, GRAY);

                // Handle input
                if is_key_pressed(KeyCode::Tab) { input_focus = 1 - input_focus; }
                if let Some(c) = get_char_pressed() {
                    if input_focus==0 { server_addr.push(c); } else { username.push(c); }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    if input_focus==0 { server_addr.pop(); } else { username.pop(); }
                }
                if is_key_pressed(KeyCode::Enter) {
                    if let Ok(n) = network::NetClient::start(server_addr.clone(), username.clone()) {
                        net = Some(n);
                        app_state = AppState::Playing;
                    }
                }
            }
            AppState::Playing => {
                let tracer = if shoot_flash > 0.0 { shoot_flash } else { 0.0 };
                draw_world(&level, &player, &others, tracer);
            }
        }

        if let AppState::Playing = app_state {
            let input = gather_input(mouse_captured);
            move_player(&level, &mut player, &input, dt);
            // Reconcile smoothly toward server target
            let delta = self_target_pos - player.pos;
            let dist = delta.length();
            if dist > 0.001 {
                // snap if too far, otherwise lerp
                if dist > 1.0 { player.pos = self_target_pos; }
                else { player.pos += delta * (10.0 * dt).min(1.0); }
            }
        }

        // Networking integration
        if let (AppState::Playing, Some(net)) = (&app_state, &net) {
            // Receive messages
            while let Ok(msg) = net.rx_incoming.try_recv() {
                match msg {
                    protocol::ServerToClient::Accept(acc) => {
                        level = level_from_maze_level(&acc.level);
                        level_id = acc.level.level_id as u8;
                        accepted_level = true;
                        my_player_id = Some(acc.player_id);
                    }
                    protocol::ServerToClient::Snapshot(snap) => {
                        // Build other players list (excluding self if known)
                        others.clear();
                        let mut updated_self = false;
                        for p in snap.players.iter() {
                            if let Some(myid) = my_player_id {
                                if p.player_id == myid {
                                    // Update target only; lerp toward it in update step
                                    self_target_pos = vec2(p.x, p.y);
                                    // Track my health/score for HUD; trigger damage flash if health dropped
                                    if let Some(prev) = my_health { if p.health < prev { dmg_flash = DAMAGE_FLASH_TIME; } }
                                    my_health = Some(p.health);
                                    my_score = Some(p.score);
                                    updated_self = true;
                                    continue;
                                }
                            }
                            others.push(RemotePlayer { pos: vec2(p.x, p.y), angle: p.angle, name: p.username.clone() });
                        }
                        // If we just connected and were inside a wall locally, this ensures we snap to a valid spawn
                        let _ = updated_self;
                    }
                    protocol::ServerToClient::Pong(p) => {
                        if let Some(pi) = &mut ping_state {
                            if pi.last_nonce == p.nonce {
                                let now = macroquad::time::get_time();
                                let rtt = ((now - pi.last_send) * 1000.0).round() as u64;
                                pi.rtt_ms = rtt;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Send input update every frame
            let action = protocol::Action::Move;
            let input_msg = protocol::ClientToServer::Input(protocol::InputUpdate {
                player_id: my_player_id.unwrap_or(0),
                x: player.pos.x,
                y: player.pos.y,
                angle: player.dir,
                action,
            });
            let _ = net.tx_outgoing.send(input_msg);

            // Shooting input (left mouse)
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(pid) = my_player_id {
                    let shoot = protocol::ClientToServer::Shoot(protocol::ShootEvent {
                        player_id: pid,
                        origin_x: player.pos.x,
                        origin_y: player.pos.y,
                        angle: player.dir,
                    });
                    let _ = net.tx_outgoing.send(shoot);
                    shoot_flash = SHOOT_FLASH_TIME;
                }
            }

            // Periodic ping
            ping_timer += dt;
            if ping_timer > 1.0 {
                ping_timer = 0.0;
                let nonce = (macroquad::time::get_time() * 1_000_000.0) as u64;
                ping_state = Some(PingInfo { last_nonce: nonce, last_send: macroquad::time::get_time(), rtt_ms: ping_state.map(|p| p.rtt_ms).unwrap_or(0) });
                let _ = net.tx_outgoing.send(protocol::ClientToServer::Ping(protocol::Ping { nonce }));
            }
        }

        if let AppState::Playing = app_state {
            draw_minimap(&level, &player, &others);
            let count = others.len() + 1;
            draw_hud(level_id, ping_state.map(|p| p.rtt_ms), &username, count, my_health, my_score);
            // Muzzle flash overlay
            if shoot_flash > 0.0 {
                shoot_flash -= dt;
                let alpha = ((shoot_flash / SHOOT_FLASH_TIME) * 180.0) as u8;
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(255, 255, 200, alpha));
            }
            // Damage flash overlay
            if dmg_flash > 0.0 {
                dmg_flash -= dt;
                let alpha = ((dmg_flash / DAMAGE_FLASH_TIME) * 180.0) as u8;
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(255, 40, 40, alpha));
            }
        }

        // Hint to (re)capture the mouse
        if let AppState::Playing = app_state { if !mouse_captured {
            let hint = "Click to capture mouse (Esc to release)";
            let tw = measure_text(hint, None, 24, 1.0);
            draw_text(
                hint,
                (screen_width() - tw.width) * 0.5,
                screen_height() * 0.5,
                24.0,
                YELLOW,
            );
        } }

        next_frame().await;
    }
}
