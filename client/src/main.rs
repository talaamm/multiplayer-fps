use macroquad::prelude::*;
use protocol;
mod network;
mod player;
mod level;
mod input;
mod rendering;
mod movement;
mod ui;

use player::{Player, RemotePlayer, PlayerSkin};
use level::{Level, level_from_maze_level, find_safe_spawn};
use input::gather_input;
use rendering::{Bullet, draw_world, draw_minimap, draw_hud, draw_crosshair, draw_screen_flash};
use movement::move_player;
use ui::{draw_level_selection, draw_connection_screen};

// ---------- Main ----------
#[macroquad::main("Maze War FPS — Client")]
async fn main() {
    let mut level: Option<Level> = None;
    let mut player = Player::new(1.5, 1.5, 0.0);
    let mut mouse_captured = false;
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut screen_flash_timer: f32 = 0.0; // Screen flash timer
    show_mouse(true);

    // --- Simple UI for IP + username ---
    enum AppState {
        Connect,
        LevelSelect,
        Playing,
    }
    let mut app_state = AppState::Connect;
    let mut server_addr = String::from("127.0.0.1:34254");
    let mut username = String::from("player");
    let mut input_focus = 0; // 0=server,1=username
    let mut net: Option<network::NetClient> = None;
    let mut selected_level = 0;
    let mut selected_skin = PlayerSkin::Soldier;
    let mut selection_mode = 0; // 0 = level selection, 1 = skin selection

    // Player id assigned by server after Accept
    let mut my_player_id: Option<u64> = None;
    // Storage for other players
    let mut others: Vec<RemotePlayer> = Vec::new();
    // Reconciliation target for our own position
    let mut self_target_pos = player.pos;
    // Ping/latency state
    #[derive(Clone, Copy)]
    struct PingInfo {
        last_nonce: u64,
        last_send: f64,
        rtt_ms: u64,
    }
    let mut ping_state: Option<PingInfo> = None;
    let mut ping_timer: f32 = 0.0;

    // Movement tracking
    let mut last_movement_time: f32 = 0.0;
    let mut has_moved_locally = false;

    // Available levels
    let available_levels = [
        (1, "The Arena".to_string(), "Close-quarters combat arena".to_string(), 8),
        (2, "The Corridors".to_string(), "Tactical corridor combat".to_string(), 10),
        (3, "The Zigzag".to_string(), "Compact zigzag maze with tight corridors".to_string(), 12),
        (4, "The Labyrinth".to_string(), "Complex multi-layer maze".to_string(), 10),
        (5, "The Brutal Death Maze".to_string(), "Brutal death maze - extremely complex and challenging".to_string(), 15),
    ];

    // Map change state
    let mut map_change_mode = false; // Whether we're in map change mode during gameplay

    loop {
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

        // Map change mode toggle (F1 key)
        if is_key_pressed(KeyCode::F1) {
            map_change_mode = !map_change_mode;
            if map_change_mode {
                // Enter map change mode - release mouse and show cursor
                set_cursor_grab(false);
                show_mouse(true);
                mouse_captured = false;
            } else {
                // Exit map change mode - capture mouse again
                set_cursor_grab(true);
                show_mouse(false);
                mouse_captured = true;
            }
        }

        clear_background(BLACK);

        match app_state {
            AppState::Connect => {
                draw_connection_screen(&server_addr, &username, input_focus);

                // Handle input
                while let Some(c) = get_char_pressed() {
                    if c == '\t' {
                        input_focus = 1 - input_focus;
                        continue;
                    }
                    if c.is_control() {
                        continue;
                    }
                    if input_focus == 0 {
                        server_addr.push(c);
                    } else {
                        username.push(c);
                    }
                }

                if is_key_pressed(KeyCode::Backspace) {
                    if input_focus == 0 {
                        server_addr.pop();
                    } else {
                        username.pop();
                    }
                }

                if is_key_pressed(KeyCode::Enter) {
                    server_addr.retain(|ch| !ch.is_control());
                    username.retain(|ch| !ch.is_control());
                    let addr = server_addr.trim();
                    let name = username.trim();

                    if input_focus == 0 && name.is_empty() {
                        input_focus = 1;
                    } else if !addr.is_empty() && !name.is_empty() {
                        if let Ok(n) = network::NetClient::start(addr.to_string(), name.to_string())
                        {
                            net = Some(n);
                            app_state = AppState::LevelSelect;
                        }
                    }
                }
            }
            AppState::LevelSelect => {
                draw_level_selection(&available_levels, &mut selected_level, &mut selected_skin, selection_mode);
                
                // Handle selection input
                if is_key_pressed(KeyCode::Tab) {
                    selection_mode = 1 - selection_mode; // Toggle between level and skin selection
                }
                
                if selection_mode == 0 {
                    // Level selection mode
                    if is_key_pressed(KeyCode::Up) {
                        selected_level = selected_level.saturating_sub(1);
                    }
                    if is_key_pressed(KeyCode::Down) {
                        selected_level = (selected_level + 1).min(available_levels.len() - 1);
                    }
                } else {
                    // Skin selection mode
                    let skins = [
                        PlayerSkin::Soldier,
                        PlayerSkin::Sniper,
                        PlayerSkin::Heavy,
                        PlayerSkin::Scout,
                        PlayerSkin::Medic,
                        PlayerSkin::Engineer,
                    ];
                    
                    if is_key_pressed(KeyCode::Up) {
                        let current_index = skins.iter().position(|&s| s == selected_skin).unwrap_or(0);
                        let new_index = current_index.saturating_sub(1);
                        selected_skin = skins[new_index];
                    }
                    if is_key_pressed(KeyCode::Down) {
                        let current_index = skins.iter().position(|&s| s == selected_skin).unwrap_or(0);
                        let new_index = (current_index + 1).min(skins.len() - 1);
                        selected_skin = skins[new_index];
                    }
                }
                
                if is_key_pressed(KeyCode::Enter) {
                    // Set the selected skin for the player
                    player.skin = selected_skin;
                    
                    // Send level selection to server
                    if let Some(ref net) = net {
                        let selected_level_id = available_levels[selected_level].0 as u32;
                        let selected_level_name = &available_levels[selected_level].1;
                        println!("🎯 CLIENT: Selecting level {}: '{}' with skin {:?}", selected_level_id, selected_level_name, selected_skin);
                        
                        let level_selection = protocol::ClientToServer::SelectLevel(protocol::LevelSelection {
                            player_id: my_player_id.unwrap_or(0),
                            level_id: selected_level_id,
                        });
                        let _ = net.tx_outgoing.send(level_selection);
                    }
                    app_state = AppState::Playing;
                }
            }
            AppState::Playing => {
                if let Some(ref level) = level {
                    draw_world(level, &player, &others, &bullets);
                }
            }
        }

        if let AppState::Playing = app_state {
            let input = gather_input(mouse_captured);

            // Trigger screen flash when shooting
            if input.shoot {
                screen_flash_timer = 0.1; // Start flash timer
            }

            // Update flash timer
            screen_flash_timer = (screen_flash_timer - dt).max(0.0);

            // Apply local movement first (only if we have a level and not in map change mode)
            if let Some(ref level) = level {
                if !map_change_mode {
                    move_player(level, &mut player, &input, dt);
                }
            }

            // Track if we've moved locally
            let is_moving = input.forward.abs() > 0.1 || input.strafe.abs() > 0.1;
            if is_moving {
                has_moved_locally = true;
                last_movement_time = 0.0;
            } else {
                last_movement_time += dt;
            }

            // Only reconcile if we haven't moved recently and we're very far out of sync
            if !is_moving && last_movement_time > 0.5 && !has_moved_locally {
                let delta = self_target_pos - player.pos;
                let dist = delta.length();
                if dist > 2.0 {
                    // Only snap if we're very far out of sync and haven't moved recently
                    player.pos = self_target_pos;
                }
            }

            // Handle map change mode input
            if map_change_mode {
                // Handle selection input for map change
                if is_key_pressed(KeyCode::Tab) {
                    selection_mode = 1 - selection_mode; // Toggle between level and skin selection
                }
                
                if selection_mode == 0 {
                    // Level selection mode
                    if is_key_pressed(KeyCode::Up) {
                        selected_level = selected_level.saturating_sub(1);
                    }
                    if is_key_pressed(KeyCode::Down) {
                        selected_level = (selected_level + 1).min(available_levels.len() - 1);
                    }
                } else {
                    // Skin selection mode
                    let skins = [
                        PlayerSkin::Soldier,
                        PlayerSkin::Sniper,
                        PlayerSkin::Heavy,
                        PlayerSkin::Scout,
                        PlayerSkin::Medic,
                        PlayerSkin::Engineer,
                    ];
                    
                    if is_key_pressed(KeyCode::Up) {
                        let current_index = skins.iter().position(|&s| s == selected_skin).unwrap_or(0);
                        let new_index = current_index.saturating_sub(1);
                        selected_skin = skins[new_index];
                    }
                    if is_key_pressed(KeyCode::Down) {
                        let current_index = skins.iter().position(|&s| s == selected_skin).unwrap_or(0);
                        let new_index = (current_index + 1).min(skins.len() - 1);
                        selected_skin = skins[new_index];
                    }
                }
                
                if is_key_pressed(KeyCode::Enter) {
                    // Set the selected skin for the player
                    player.skin = selected_skin;
                    
                    // Send level selection to server
                    if let Some(ref net) = net {
                        let selected_level_id = available_levels[selected_level].0 as u32;
                        let selected_level_name = &available_levels[selected_level].1;
                        println!("🎯 CLIENT: Changing to level {}: '{}' with skin {:?}", selected_level_id, selected_level_name, selected_skin);
                        
                        let level_selection = protocol::ClientToServer::SelectLevel(protocol::LevelSelection {
                            player_id: my_player_id.unwrap_or(0),
                            level_id: selected_level_id,
                        });
                        let _ = net.tx_outgoing.send(level_selection);
                    }
                    
                    // Exit map change mode
                    map_change_mode = false;
                    set_cursor_grab(true);
                    show_mouse(false);
                    mouse_captured = true;
                }
            }
        }

        // Networking integration
        if let (AppState::Playing, Some(net)) = (&app_state, &net) {
            // Receive messages
            while let Ok(msg) = net.rx_incoming.try_recv() {
                println!("CLIENT DEBUG: Received message: {:?}", msg);

                match msg {
                    protocol::ServerToClient::Accept(acc) => {
                        // Accept server level data
                        level = Some(level_from_maze_level(&acc.level));

                        println!("🎮 CLIENT: Level {} loaded: '{}' ({}x{})", 
                                acc.level.level_id, acc.level.name, acc.level.width, acc.level.height);

                        // Only set player ID if it's not a level change (player_id != 0)
                        if acc.player_id != 0 {
                            my_player_id = Some(acc.player_id);
                            // Assign skin based on player ID
                            player.skin = PlayerSkin::from_id(acc.player_id);
                            println!(
                                "⚠️ CLIENT: Initial join - You are now player {} with skin {:?}!",
                                acc.player_id, player.skin
                            );
                        }

                        // If this is a level change (player_id == 0), reset player position and state
                        if acc.player_id == 0 {
                            println!(
                                "🎯 CLIENT: Level changed to {}! Resetting player position...",
                                acc.level.level_id
                            );

                            if let Some(ref lvl) = level {
                                let spawn_pos = find_safe_spawn(lvl);
                                println!(
                                    "DEBUG: Spawn position for level {} is {:?}",
                                    acc.level.level_id, spawn_pos
                                );

                                player.pos = spawn_pos;
                                self_target_pos = player.pos;
                            }
                            // Reset movement and state
                            has_moved_locally = false;
                            last_movement_time = 0.0;
                            others.clear();
                            bullets.clear();
                        } else {
                            // Reset movement tracking when joining a new server
                            has_moved_locally = false;
                            last_movement_time = 0.0;
                        }
                    }
                    protocol::ServerToClient::Snapshot(snap) => {
                        // Build other players list (excluding self if known)
                        others.clear();
                        let mut updated_self = false;
                        for p in snap.players.iter() {
                            if let Some(myid) = my_player_id {
                                if p.player_id == myid {
                                    // Only update server target if we haven't moved locally recently
                                    if !has_moved_locally || last_movement_time > 1.0 {
                                        self_target_pos = vec2(p.x, p.y);
                                    }
                                    // Update player stats
                                    player.health = p.health;
                                    player.ammo = p.ammo;
                                    player.kills = p.kills;
                                    player.deaths = p.deaths;
                                    updated_self = true;
                                    continue;
                                }
                            }
                            others.push(RemotePlayer {
                                pos: vec2(p.x, p.y),
                                angle: p.angle,
                                name: p.username.clone(),
                                health: p.health,
                                ammo: p.ammo,
                                kills: p.kills,
                                deaths: p.deaths,
                                skin: PlayerSkin::from_id(p.player_id),
                            });
                        }
                        let _ = updated_self;

                        // Update bullets
                        bullets.clear();
                        for b in snap.bullets.iter() {
                            bullets.push(Bullet {
                                x: b.x,
                                y: b.y,
                                angle: b.angle,
                                lifetime: b.lifetime,
                            });
                        }
                    }
                    protocol::ServerToClient::Hit(hit_event) => {
                        println!("💥 Hit! Damage: {}", hit_event.damage);
                    }
                    protocol::ServerToClient::Death(death_event) => {
                        println!("💀 {} killed {} with {}", 
                                death_event.killer_id, death_event.victim_id, death_event.weapon);
                    }
                    protocol::ServerToClient::Respawn(respawn_event) => {
                        println!("🔄 Player {} respawned at ({}, {})", 
                                respawn_event.player_id, respawn_event.x, respawn_event.y);
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
                    _ => {
                        println!("CLIENT DEBUG: Received unknown message: {:?}", msg);
                    }
                }
            }

            // Send input update every frame
            let input = gather_input(mouse_captured);
            let action = if input.shoot { protocol::Action::Shoot } else { protocol::Action::Move };
            let input_msg = protocol::ClientToServer::Input(protocol::InputUpdate {
                player_id: my_player_id.unwrap_or(0),
                x: player.pos.x,
                y: player.pos.y,
                angle: player.dir,
                action,
            });
            let _ = net.tx_outgoing.send(input_msg);

            // Periodic ping
            ping_timer += dt;
            if ping_timer > 1.0 {
                ping_timer = 0.0;
                let nonce = (macroquad::time::get_time() * 1_000_000.0) as u64;
                ping_state = Some(PingInfo {
                    last_nonce: nonce,
                    last_send: macroquad::time::get_time(),
                    rtt_ms: ping_state.map(|p| p.rtt_ms).unwrap_or(0),
                });
                let _ = net
                    .tx_outgoing
                    .send(protocol::ClientToServer::Ping(protocol::Ping { nonce }));
            }
        }

        if let AppState::Playing = app_state {
            if let Some(ref level) = level {
                draw_minimap(level, &player, &others, &bullets);
                let count = others.len() + 1;
                draw_hud(
                    level,
                    ping_state.map(|p| p.rtt_ms),
                    &username,
                    count,
                    mouse_captured,
                    &player,
                    has_moved_locally,
                    map_change_mode,
                );
                
                // Draw crosshair when mouse is captured
                if mouse_captured {
                    draw_crosshair();
                }
                
                // Draw screen flash (on top of everything)
                draw_screen_flash(screen_flash_timer);

                // Draw map change UI if in map change mode
                if map_change_mode {
                    // Semi-transparent overlay
                    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 180));
                    
                    // Draw the level selection UI
                    draw_level_selection(&available_levels, &mut selected_level, &mut selected_skin, selection_mode);
                    
                    // Add map change specific instructions
                    let map_change_hint = "F1 to exit map change mode | Enter to confirm selection";
                    let hint_tw = measure_text(map_change_hint, None, 16, 1.0);
                    draw_text(map_change_hint, (screen_width() - hint_tw.width) * 0.5, screen_height() - 50.0, 16.0, YELLOW);
                }
            }
        }

        // Hint to (re)capture the mouse
        if let AppState::Playing = app_state {
            if !mouse_captured {
                let hint = "Click to capture mouse (Esc to release)";
                let tw = measure_text(hint, None, 24, 1.0);
                draw_text(
                    hint,
                    (screen_width() - tw.width) * 0.5,
                    screen_height() * 0.5,
                    24.0,
                    YELLOW,
                );
            }
        }

        next_frame().await;
    }
}
