// Import the Macroquad game framework prelude for easy access to all game functions
use macroquad::prelude::*;
// Import our custom protocol module for network communication
use protocol;

// Import all our custom modules that handle different aspects of the game
mod network;      // Handles network communication with server
mod player;       // Player logic and state management
mod level;        // Level loading and management
mod input;        // Input handling and processing
mod rendering;    // Graphics and rendering functions
mod movement;     // Player movement physics and collision
mod ui;           // User interface elements

// Import specific types and functions from our modules
use player::{Player, RemotePlayer, PlayerSkin};
use level::{Level, level_from_maze_level, find_safe_spawn};
use input::gather_input;
use rendering::{Bullet, draw_world, draw_minimap, draw_hud, draw_crosshair, draw_screen_flash};
use movement::move_player;
use ui::{draw_level_selection, draw_connection_screen};

// ---------- Main Game Entry Point ----------
/// Main function that runs the entire game client
/// Uses Macroquad's async runtime for smooth 60 FPS gameplay
#[macroquad::main("Maze War FPS — Client")]
async fn main() {
    // Initialize core game state variables
    let mut level: Option<Level> = None;                    // Current loaded level (None until server sends it)
    let mut player = Player::new(1.5, 1.5, 0.0);           // Local player starting at position (1.5, 1.5) with 0° rotation
    let mut mouse_captured = false;                         // Whether mouse is captured for FPS controls
    let mut bullets: Vec<Bullet> = Vec::new();              // List of active bullets in the world
    let mut screen_flash_timer: f32 = 0.0;                  // Timer for screen flash effect when shooting
    show_mouse(true);                                        // Show mouse cursor initially

    // --- Application State Machine ---
    /// Enum defining the three main states of the application
    enum AppState {
        Connect,        // Initial connection screen
        LevelSelect,    // Level and skin selection screen
        Playing,        // Active gameplay state
    }
    let mut app_state = AppState::Connect;                   // Start in connection state
    
    // --- Connection and UI State Variables ---
    let mut server_addr = String::from("127.0.0.1:34254");  // Default server address (localhost)
    let mut username = String::from("player");               // Default username
    let mut input_focus = 0;                                 // 0=server address field, 1=username field
    let mut net: Option<network::NetClient> = None;          // Network client (None until connected)
    let mut selected_level = 0;                              // Currently selected level index
    let mut selected_skin = PlayerSkin::Soldier;             // Currently selected player skin
    let mut selection_mode = 0;                              // 0=level selection, 1=skin selection

    // --- Multiplayer State Variables ---
    let mut my_player_id: Option<u64> = None;               // Player ID assigned by server
    let mut others: Vec<RemotePlayer> = Vec::new();          // List of other players in the game
    let mut self_target_pos = player.pos;                    // Server's version of our position for reconciliation
    let mut ping_state: Option<PingInfo> = None;             // Ping/latency measurement state
    let mut ping_timer: f32 = 0.0;                          // Timer for sending periodic pings

    // --- Movement and Reconciliation Variables ---
    let mut last_movement_time: f32 = 0.0;                   // Time since last local movement
    let mut has_moved_locally = false;                       // Whether we've moved locally recently

    // --- Ping Information Structure ---
    /// Stores information about ping measurements for latency monitoring
    #[derive(Clone, Copy)]
    struct PingInfo {
        last_nonce: u64,                                     // Unique identifier for ping request
        last_send: f64,                                      // Timestamp when ping was sent
        rtt_ms: u64,                                         // Round-trip time in milliseconds
    }

    // --- Available Game Levels ---
    /// Array of available levels with ID, name, description, and complexity
    let available_levels = [
        (1, "The Arena".to_string(), "Close-quarters combat arena".to_string(), 8),
        (2, "The Corridors".to_string(), "Tactical corridor combat".to_string(), 10),
        (3, "The Zigzag".to_string(), "Compact zigzag maze with tight corridors".to_string(), 12),
        (4, "The Labyrinth".to_string(), "Complex multi-layer maze".to_string(), 10),
        (5, "The Brutal Death Maze".to_string(), "Brutal death maze - extremely complex and challenging".to_string(), 15),
    ];

    // --- Map Change State ---
    let mut map_change_mode = false;                         // Whether we're in map change mode during gameplay

    // --- Main Game Loop ---
    /// Infinite loop that runs the game at 60 FPS
    loop {
        let dt = macroquad::time::get_frame_time();           // Get time since last frame (delta time)

        // --- Mouse Capture Logic ---
        // Left Click to capture mouse for FPS controls, Escape to release
        if !mouse_captured && is_mouse_button_pressed(MouseButton::Left) {
            set_cursor_grab(true);                           // Capture mouse cursor
            show_mouse(false);                                // Hide mouse cursor
            mouse_captured = true;                            // Update capture state
        }
        if mouse_captured && is_key_pressed(KeyCode::Escape) {
            set_cursor_grab(false);                          // Release mouse cursor
            show_mouse(true);                                 // Show mouse cursor
            mouse_captured = false;                           // Update capture state
        }

        // --- Map Change Mode Toggle ---
        // F1 key toggles map change mode during gameplay
        if is_key_pressed(KeyCode::F1) {
            map_change_mode = !map_change_mode;               // Toggle map change mode
            if map_change_mode {
                // Enter map change mode - release mouse and show cursor for UI interaction
                set_cursor_grab(false);
                show_mouse(true);
                mouse_captured = false;
            } else {
                // Exit map change mode - capture mouse again for gameplay
                set_cursor_grab(true);
                show_mouse(false);
                mouse_captured = true;
            }
        }

        // Clear screen to black background
        clear_background(BLACK);

        // --- State Machine: Handle Different App States ---
        match app_state {
            // --- Connection State ---
            AppState::Connect => {
                // Draw the connection screen UI
                draw_connection_screen(&server_addr, &username, input_focus);

                // Handle text input for server address and username
                while let Some(c) = get_char_pressed() {
                    if c == '\t' {
                        input_focus = 1 - input_focus;        // Tab key switches between input fields
                        continue;
                    }
                    if c.is_control() {                       // Skip control characters
                        continue;
                    }
                    // Add character to appropriate input field
                    if input_focus == 0 {
                        server_addr.push(c);                   // Add to server address
                    } else {
                        username.push(c);                      // Add to username
                    }
                }

                // Handle backspace key
                if is_key_pressed(KeyCode::Backspace) {
                    if input_focus == 0 {
                        server_addr.pop();                     // Remove last character from server address
                    } else {
                        username.pop();                        // Remove last character from username
                    }
                }

                // Handle Enter key to connect
                if is_key_pressed(KeyCode::Enter) {
                    // Sanitize input by removing control characters
                    server_addr.retain(|ch| !ch.is_control());
                    username.retain(|ch| !ch.is_control());
                    let addr = server_addr.trim();             // Remove whitespace
                    let name = username.trim();

                    // Validate input and attempt connection
                    if input_focus == 0 && name.is_empty() {
                        input_focus = 1;                      // Switch to username field if empty
                    } else if !addr.is_empty() && !name.is_empty() {
                        // Try to start network client
                        if let Ok(n) = network::NetClient::start(addr.to_string(), name.to_string()) {
                            net = Some(n);                     // Store network client
                            app_state = AppState::LevelSelect; // Move to level selection
                        }
                    }
                }
            }
            
            // --- Level Selection State ---
            AppState::LevelSelect => {
                // Draw level selection UI
                draw_level_selection(&available_levels, &mut selected_level, &mut selected_skin, selection_mode);
                
                // Handle selection input
                if is_key_pressed(KeyCode::Tab) {
                    selection_mode = 1 - selection_mode;       // Toggle between level and skin selection
                }
                
                if selection_mode == 0 {
                    // Level selection mode - use Up/Down arrows
                    if is_key_pressed(KeyCode::Up) {
                        selected_level = selected_level.saturating_sub(1);  // Move up in level list
                    }
                    if is_key_pressed(KeyCode::Down) {
                        selected_level = (selected_level + 1).min(available_levels.len() - 1);  // Move down in level list
                    }
                } else {
                    // Skin selection mode - use Up/Down arrows
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
                        let new_index = current_index.saturating_sub(1);   // Move up in skin list
                        selected_skin = skins[new_index];
                    }
                    if is_key_pressed(KeyCode::Down) {
                        let current_index = skins.iter().position(|&s| s == selected_skin).unwrap_or(0);
                        let new_index = (current_index + 1).min(skins.len() - 1);  // Move down in skin list
                        selected_skin = skins[new_index];
                    }
                }
                
                // Handle Enter key to confirm selection
                if is_key_pressed(KeyCode::Enter) {
                    // Set the selected skin for the player
                    player.skin = selected_skin;
                    
                    // Send level selection to server
                    if let Some(ref net) = net {
                        let selected_level_id = available_levels[selected_level].0 as u32;
                        let selected_level_name = &available_levels[selected_level].1;
                        println!("🎯 CLIENT: Selecting level {}: '{}' with skin {:?}", selected_level_id, selected_level_name, selected_skin);
                        
                        // Create and send level selection message to server
                        let level_selection = protocol::ClientToServer::SelectLevel(protocol::LevelSelection {
                            player_id: my_player_id.unwrap_or(0),
                            level_id: selected_level_id,
                        });
                        let _ = net.tx_outgoing.send(level_selection);
                    }
                    app_state = AppState::Playing;            // Move to gameplay state
                }
            }
            
            // --- Playing State ---
            AppState::Playing => {
                // Draw the 3D game world if we have a level loaded
                if let Some(ref level) = level {
                    draw_world(level, &player, &others, &bullets);
                }
            }
        }

        // --- Gameplay Logic (Only when in Playing state) ---
        if let AppState::Playing = app_state {
            // Gather input from keyboard and mouse
            let input = gather_input(mouse_captured);

            // Trigger screen flash when shooting
            if input.shoot {
                screen_flash_timer = 0.1;                     // Start flash timer for 0.1 seconds
            }

            // Update flash timer (countdown)
            screen_flash_timer = (screen_flash_timer - dt).max(0.0);

            // Apply local movement first (only if we have a level and not in map change mode)
            if let Some(ref level) = level {
                if !map_change_mode {
                    move_player(level, &mut player, &input, dt);  // Update player position based on input
                }
            }

            // Track if we've moved locally for reconciliation
            let is_moving = input.forward.abs() > 0.1 || input.strafe.abs() > 0.1;  // Check if player is moving
            if is_moving {
                has_moved_locally = true;                     // Mark that we've moved locally
                last_movement_time = 0.0;                     // Reset movement timer
            } else {
                last_movement_time += dt;                     // Increment time since last movement
            }

            // --- Position Reconciliation Logic ---
            // Only reconcile if we haven't moved recently and we're very far out of sync
            if !is_moving && last_movement_time > 0.5 && !has_moved_locally {
                let delta = self_target_pos - player.pos;     // Calculate position difference
                let dist = delta.length();                     // Get distance to server position
                if dist > 2.0 {
                    // Only snap if we're very far out of sync and haven't moved recently
                    player.pos = self_target_pos;              // Snap to server position
                }
            }

            // --- Map Change Mode Input Handling ---
            if map_change_mode {
                // Handle selection input for map change (same logic as level selection)
                if is_key_pressed(KeyCode::Tab) {
                    selection_mode = 1 - selection_mode;       // Toggle between level and skin selection
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
                
                // Handle Enter key to confirm map change
                if is_key_pressed(KeyCode::Enter) {
                    // Set the selected skin for the player
                    player.skin = selected_skin;
                    
                    // Send level selection to server
                    if let Some(ref net) = net {
                        let selected_level_id = available_levels[selected_level].0 as u32;
                        let selected_level_name = &available_levels[selected_level].1;
                        println!("🎯 CLIENT: Changing to level {}: '{}' with skin {:?}", selected_level_id, selected_level_name, selected_skin);
                        
                        // Create and send level selection message to server
                        let level_selection = protocol::ClientToServer::SelectLevel(protocol::LevelSelection {
                            player_id: my_player_id.unwrap_or(0),
                            level_id: selected_level_id,
                        });
                        let _ = net.tx_outgoing.send(level_selection);
                    }
                    
                    // Exit map change mode and return to gameplay
                    map_change_mode = false;
                    set_cursor_grab(true);                     // Capture mouse again
                    show_mouse(false);                         // Hide mouse cursor
                    mouse_captured = true;                     // Update capture state
                }
            }
        }

        // --- Networking Integration (Only when playing and connected) ---
        if let (AppState::Playing, Some(net)) = (&app_state, &net) {
            // Receive and process incoming messages from server
            while let Ok(msg) = net.rx_incoming.try_recv() {
                println!("CLIENT DEBUG: Received message: {:?}", msg);

                // Handle different types of server messages
                match msg {
                    // --- Server Accept Message ---
                    protocol::ServerToClient::Accept(acc) => {
                        // Accept server level data and load the level
                        level = Some(level_from_maze_level(&acc.level));

                        println!("🎮 CLIENT: Level {} loaded: '{}' ({}x{})", 
                                acc.level.level_id, acc.level.name, acc.level.width, acc.level.height);

                        // Only set player ID if it's not a level change (player_id != 0)
                        if acc.player_id != 0 {
                            my_player_id = Some(acc.player_id);  // Store our player ID
                            // Assign skin based on player ID for consistency
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
                                let spawn_pos = find_safe_spawn(lvl);  // Find safe spawn point
                                println!(
                                    "DEBUG: Spawn position for level {} is {:?}",
                                    acc.level.level_id, spawn_pos
                                );

                                player.pos = spawn_pos;                // Set new spawn position
                                self_target_pos = player.pos;          // Update reconciliation target
                            }
                            // Reset movement and game state
                            has_moved_locally = false;
                            last_movement_time = 0.0;
                            others.clear();                           // Clear other players
                            bullets.clear();                          // Clear bullets
                        } else {
                            // Reset movement tracking when joining a new server
                            has_moved_locally = false;
                            last_movement_time = 0.0;
                        }
                    }
                    
                    // --- Server Snapshot Message ---
                    protocol::ServerToClient::Snapshot(snap) => {
                        // Build other players list (excluding self if known)
                        others.clear();                               // Clear previous player list
                        let mut updated_self = false;                 // Track if we updated our own data
                        for p in snap.players.iter() {
                            if let Some(myid) = my_player_id {
                                if p.player_id == myid {
                                    // This is us - update server target position for reconciliation
                                    // Only update server target if we haven't moved locally recently
                                    if !has_moved_locally || last_movement_time > 1.0 {
                                        self_target_pos = vec2(p.x, p.y);  // Update reconciliation target
                                    }
                                    // Update player stats from server
                                    player.health = p.health;
                                    player.ammo = p.ammo;
                                    player.kills = p.kills;
                                    player.deaths = p.deaths;
                                    updated_self = true;
                                    continue;                           // Skip adding to others list
                                }
                            }
                            // Add other players to the list
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
                        let _ = updated_self;                         // Suppress unused variable warning

                        // Update bullets from server snapshot
                        bullets.clear();                               // Clear previous bullets
                        for b in snap.bullets.iter() {
                            bullets.push(Bullet {
                                x: b.x,
                                y: b.y,
                                angle: b.angle,
                                lifetime: b.lifetime,
                            });
                        }
                    }
                    
                    // --- Game Event Messages ---
                    protocol::ServerToClient::Hit(hit_event) => {
                        println!("💥 Hit! Damage: {}", hit_event.damage);  // Log hit event
                    }
                    protocol::ServerToClient::Death(death_event) => {
                        println!("💀 {} killed {} with {}", 
                                death_event.killer_id, death_event.victim_id, death_event.weapon);  // Log death event
                    }
                    protocol::ServerToClient::Respawn(respawn_event) => {
                        println!("🔄 Player {} respawned at ({}, {})", 
                                respawn_event.player_id, respawn_event.x, respawn_event.y);  // Log respawn event
                    }
                    
                    // --- Ping Response ---
                    protocol::ServerToClient::Pong(p) => {
                        if let Some(pi) = &mut ping_state {
                            if pi.last_nonce == p.nonce {              // Verify this is our ping response
                                let now = macroquad::time::get_time();
                                let rtt = ((now - pi.last_send) * 1000.0).round() as u64;  // Calculate round-trip time
                                pi.rtt_ms = rtt;                       // Store RTT in milliseconds
                            }
                        }
                    }
                    
                    // --- Unknown Message Handling ---
                    _ => {
                        println!("CLIENT DEBUG: Received unknown message: {:?}", msg);
                    }
                }
            }

            // --- Send Input Update to Server ---
            // Send input update every frame for smooth movement
            let input = gather_input(mouse_captured);
            let action = if input.shoot { protocol::Action::Shoot } else { protocol::Action::Move };  // Determine action type
            let input_msg = protocol::ClientToServer::Input(protocol::InputUpdate {
                player_id: my_player_id.unwrap_or(0),         // Our player ID
                x: player.pos.x,                              // Current X position
                y: player.pos.y,                              // Current Y position
                angle: player.dir,                            // Current rotation angle
                action,                                       // Current action (move or shoot)
            });
            let _ = net.tx_outgoing.send(input_msg);          // Send input to server

            // --- Periodic Ping System ---
            // Send ping every second to measure latency
            ping_timer += dt;
            if ping_timer > 1.0 {
                ping_timer = 0.0;                             // Reset ping timer
                let nonce = (macroquad::time::get_time() * 1_000_000.0) as u64;  // Generate unique ping ID
                ping_state = Some(PingInfo {
                    last_nonce: nonce,                        // Store ping ID
                    last_send: macroquad::time::get_time(),   // Store send timestamp
                    rtt_ms: ping_state.map(|p| p.rtt_ms).unwrap_or(0),  // Keep previous RTT
                });
                // Send ping message to server
                let _ = net
                    .tx_outgoing
                    .send(protocol::ClientToServer::Ping(protocol::Ping { nonce }));
            }
        }

        // --- Rendering and UI (Only when playing) ---
        if let AppState::Playing = app_state {
            if let Some(ref level) = level {
                // Draw game UI elements
                draw_minimap(level, &player, &others, &bullets);  // Draw minimap in corner
                let count = others.len() + 1;                     // Total player count (including self)
                draw_hud(
                    level,
                    ping_state.map(|p| p.rtt_ms),              // Display ping/latency
                    &username,                                  // Display username
                    count,                                      // Display player count
                    mouse_captured,                             // Show capture status
                    &player,                                    // Display player stats
                    has_moved_locally,                          // Show movement status
                    map_change_mode,                            // Show map change mode status
                );
                
                // Draw crosshair when mouse is captured (FPS mode)
                if mouse_captured {
                    draw_crosshair();
                }
                
                // Draw screen flash effect (on top of everything)
                draw_screen_flash(screen_flash_timer);

                // --- Map Change UI Overlay ---
                if map_change_mode {
                    // Semi-transparent black overlay
                    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 180));
                    
                    // Draw the level selection UI on top
                    draw_level_selection(&available_levels, &mut selected_level, &mut selected_skin, selection_mode);
                    
                    // Add map change specific instructions
                    let map_change_hint = "F1 to exit map change mode | Enter to confirm selection";
                    let hint_tw = measure_text(map_change_hint, None, 16, 1.0);  // Measure text width
                    draw_text(map_change_hint, (screen_width() - hint_tw.width) * 0.5, screen_height() - 50.0, 16.0, YELLOW);
                }
            }
        }

        // --- Mouse Capture Hint ---
        // Show hint to recapture mouse when not captured
        if let AppState::Playing = app_state {
            if !mouse_captured {
                let hint = "Click to capture mouse (Esc to release)";
                let tw = measure_text(hint, None, 24, 1.0);   // Measure text width for centering
                draw_text(
                    hint,
                    (screen_width() - tw.width) * 0.5,        // Center horizontally
                    screen_height() * 0.5,                     // Center vertically
                    24.0,                                     // Font size
                    YELLOW,                                    // Text color
                );
            }
        }

        // Wait for next frame to maintain 60 FPS
        next_frame().await;
    }
}
