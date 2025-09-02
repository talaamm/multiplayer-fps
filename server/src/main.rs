mod game {
    pub mod logic;
}
use game::logic::{Cell, Maze, Player};
use std::io::Read;

fn render_with_player(maze: &Maze, px: usize, py: usize) {
    for y in 0..maze.height {
        for x in 0..maze.width {
            if x == px && y == py {
                print!("P");
            } else {
                let ch = match maze.grid[y][x] {
                    Cell::Wall => '#',
                    Cell::Path => ' ',
                    Cell::SpawnPoint => 'S',
                    Cell::Cover => 'C',
                };
                print!("{ch}");
            }
        }
        println!();
    }
}

fn main_single_player() {
    let lvl = 1;

    // choose a level
    let maze = Maze::load_level(lvl);

    // Test multiplayer support
    maze.test_multiplayer_support();

    // pick first spawn or fallback
    let (sx, sy) = maze.spawn_points(1).get(0).copied().unwrap_or((0, 0));
    let mut p = Player::new(sx, sy);

    println!("Controls: W/A/S/D to move, Q to quit. FPS Deathmatch Mode!\n");

    loop {
        // clear screen (simple)
        print!("\x1B[2J\x1B[H"); // ANSI clear + home
        render_with_player(&maze, p.x, p.y);

        println!("\nPos: ({}, {}). Health: {}, Ammo: {}, Kills: {}, Deaths: {}. Move [W/A/S/D], Quit [Q]: ", 
                 p.x, p.y, p.health, p.ammo, p.kills, p.deaths);

        // non-blocking single-char read (simple blocking read fallback)
        let mut buf = [0u8; 1];
        if std::io::stdin().read_exact(&mut buf).is_err() {
            break;
        }
        let c = (buf[0] as char).to_ascii_lowercase();

        match c {
            'w' => p.move_up(&maze),
            's' => p.move_down(&maze),
            'a' => p.move_left(&maze),
            'd' => p.move_right(&maze),
            'q' => break,
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check if user wants single-player mode
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--single-player" {
        main_single_player();
        return Ok(());
    }

    // Run multiplayer server by default
    main_multiplayer().await
}

// Multiplayer server implementation
async fn main_multiplayer() -> anyhow::Result<()> {
    // ---- Networking setup ----
    let bind_addr = std::env::var("SERVER_BIND").unwrap_or_else(|_| "0.0.0.0:34254".to_string());
    let broadcast_hz: u64 = 20;
    let socket = std::sync::Arc::new(tokio::net::UdpSocket::bind(&bind_addr).await?);
    println!("Maze War FPS Server listening on {}", socket.local_addr()?);

    // ---- Load your maze + make wire level ----
    let logic_maze = Maze::load_level(1); // Start at level 1
    // logic_maze.print_ascii(); // debug if you want
    let wire_level = maze_to_protocol(1, &logic_maze);

    let state = std::sync::Arc::new(parking_lot::Mutex::new(ServerState::new(
        logic_maze,
        wire_level.clone(),
    )));

    // ---- Outbound channel + sender task ----
    let (tx_out, mut rx_out) =
        tokio::sync::mpsc::unbounded_channel::<(std::net::SocketAddr, protocol::ServerToClient)>();
    {
        let socket_send = std::sync::Arc::clone(&socket);
        tokio::spawn(async move {
            while let Some((addr, msg)) = rx_out.recv().await {
                if let Ok(bytes) = encode_server(&msg) {
                    let _ = socket_send.send_to(&bytes, addr).await;
                }
            }
        });
    }

    // ---- Snapshot broadcast task ----
    {
        let state_for_broadcast = std::sync::Arc::clone(&state);
        let tx_out_broadcast = tx_out.clone();
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_millis(1000 / broadcast_hz));
            let mut bullet_timer = 0.0;
            let bullet_update_rate = 1.0 / 60.0; // 60 FPS for bullets

            loop {
                ticker.tick().await;
                bullet_timer += 1.0 / broadcast_hz as f32;

                // Update bullets
                if bullet_timer >= bullet_update_rate {
                    bullet_timer = 0.0;
                    let mut st = state_for_broadcast.lock();
                    st.update_bullets(&tx_out_broadcast);
                }

                let snapshot = {
                    let st = state_for_broadcast.lock();
                    let players = st
                        .players
                        .iter()
                        .map(|(pid, info)| protocol::PlayerState {
                            player_id: *pid,
                            username: info.username.clone(),
                            x: info.pos_x,
                            y: info.pos_y,
                            angle: info.angle,
                            health: info.health,
                            score: info.score,
                            ammo: info.ammo,
                            kills: info.kills,
                            deaths: info.deaths,
                        })
                        .collect::<Vec<_>>();

                    let bullets = st
                        .bullets
                        .iter()
                        .map(|bullet| protocol::Bullet {
                            bullet_id: bullet.bullet_id,
                            shooter_id: bullet.shooter_id,
                            x: bullet.x,
                            y: bullet.y,
                            angle: bullet.angle,
                            speed: bullet.speed,
                            damage: bullet.damage,
                            lifetime: bullet.lifetime,
                        })
                        .collect::<Vec<_>>();

                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    protocol::ServerToClient::Snapshot(protocol::Snapshot {
                        server_time_ms: now_ms,
                        players,
                        bullets,
                    })
                };

                let addrs: Vec<std::net::SocketAddr> = {
                    let st = state_for_broadcast.lock();
                    st.addr_by_player.values().copied().collect()
                };
                for addr in addrs {
                    let _ = tx_out_broadcast.send((addr, snapshot.clone()));
                }
            }
        });
    }

    // ---- Main receive loop ----
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];
        match protocol::decode_client(data) {
            Ok(protocol::ClientToServer::Join(join)) => {
                // Register and send Accept with your maze
                let (pid, info, accept_msg) = {
                    let mut st = state.lock();
                    let (pid, info) = st.register_player(addr, join.username);
                    let accept = protocol::ServerToClient::Accept(protocol::JoinAccept {
                        player_id: pid,
                        level: st.wire_level.clone(),
                    });
                    (pid, info, accept)
                };
                let _ = tx_out.send((addr, accept_msg));
                println!("Player {} joined as {} from {}", pid, info.username, addr);
            }

            Ok(protocol::ClientToServer::Input(input)) => {
                // Handle movement and shooting
                let mut st = state.lock();
                st.handle_input(input, &tx_out);
            }

            Ok(protocol::ClientToServer::SelectLevel(selection)) => {
                // Handle level selection
                let mut st = state.lock();
                st.change_level(selection.level_id, &tx_out);
            }

            Ok(protocol::ClientToServer::Leave(leave)) => {
                // Remove and inform others
                let mut st = state.lock();
                if let Some(addr) = st.addr_by_player.get(&leave.player_id).copied() {
                    st.addr_by_player.remove(&leave.player_id);
                    st.players.remove(&leave.player_id);
                    st.player_by_addr.remove(&addr);

                    let msg = protocol::ServerToClient::PlayerLeft(protocol::LeaveNotice {
                        player_id: leave.player_id,
                    });
                    for dest in st.addr_by_player.values().copied().collect::<Vec<_>>() {
                        let _ = tx_out.send((dest, msg.clone()));
                    }
                }
            }

            Ok(protocol::ClientToServer::Ping(p)) => {
                let _ = tx_out.send((
                    addr,
                    protocol::ServerToClient::Pong(protocol::Pong { nonce: p.nonce }),
                ));
            }

            Err(err) => {
                let _ = tx_out.send((
                    addr,
                    protocol::ServerToClient::Error {
                        message: format!("bad request: {}", err),
                    },
                ));
            }
        }
    }
}

// Server-side info for a connected player.
#[derive(Debug, Clone)]
struct PlayerInfo {
    username: String,
    pos_x: f32,
    pos_y: f32,
    angle: f32,
    health: u8,
    score: u32,
    ammo: u8,
    kills: u32,
    deaths: u32,
    last_seen: std::time::Instant,
    last_shot_time: f64,
}

// Bullet information
#[derive(Debug, Clone)]
struct BulletInfo {
    bullet_id: u64,
    shooter_id: u64,
    x: f32,
    y: f32,
    angle: f32,
    speed: f32,
    damage: u8,
    lifetime: f32,
    max_lifetime: f32,
}

// Shared server state.
#[derive(Debug)]
struct ServerState {
    // Your gameplay logic (authoritative)
    logic_maze: Maze,
    // The serialized form we send to clients on accept
    wire_level: protocol::MazeLevel,

    players: std::collections::HashMap<u64, PlayerInfo>, // player_id -> PlayerInfo
    addr_by_player: std::collections::HashMap<u64, std::net::SocketAddr>, // player_id -> address
    player_by_addr: std::collections::HashMap<std::net::SocketAddr, u64>, // address -> player_id
    next_player_id: u64,
    next_bullet_id: u64,

    // Bullets in the world
    bullets: Vec<BulletInfo>,

    // Precomputed spawn points from your maze logic
    spawns: Vec<(usize, usize)>,
    spawn_cursor: usize,
}

impl ServerState {
    fn new(logic_maze: Maze, wire_level: protocol::MazeLevel) -> Self {
        // Grab plenty of spawns; if fewer, we'll reuse cyclically.
        let spawns = {
            let mut s = logic_maze.spawn_points(128);
            if s.is_empty() {
                // fall back to a safe-ish default
                s.push((1, 1));
            }
            s
        };

        Self {
            logic_maze,
            wire_level,
            players: std::collections::HashMap::new(),
            addr_by_player: std::collections::HashMap::new(),
            player_by_addr: std::collections::HashMap::new(),
            next_player_id: 1,
            next_bullet_id: 1,
            bullets: Vec::new(),
            spawns,
            spawn_cursor: 0,
        }
    }

    fn next_spawn(&mut self) -> (f32, f32) {
        let (x, y) = self.spawns[self.spawn_cursor % self.spawns.len()];
        self.spawn_cursor = (self.spawn_cursor + 1) % self.spawns.len();
        // center of the tile
        (x as f32 + 0.5, y as f32 + 0.5)
        // NOTE: If your client uses a different scale, adjust here.
    }

    /// Registers a new player and returns (player_id, info).
    fn register_player(
        &mut self,
        addr: std::net::SocketAddr,
        username: String,
    ) -> (u64, PlayerInfo) {
        let player_id = self.next_player_id;
        self.next_player_id += 1;

        let (sx, sy) = self.next_spawn();
        let info = PlayerInfo {
            username,
            pos_x: sx,
            pos_y: sy,
            angle: 0.0,
            health: 100,
            score: 0,
            ammo: 30,
            kills: 0,
            deaths: 0,
            last_seen: std::time::Instant::now(),
            last_shot_time: 0.0,
        };

        self.players.insert(player_id, info.clone());
        self.addr_by_player.insert(player_id, addr);
        self.player_by_addr.insert(addr, player_id);
        (player_id, info)
    }

    /// Handles player input including movement and shooting
    fn handle_input(
        &mut self,
        input: protocol::InputUpdate,
        tx_out: &tokio::sync::mpsc::UnboundedSender<(
            std::net::SocketAddr,
            protocol::ServerToClient,
        )>,
    ) {
        if let Some(p) = self.players.get_mut(&input.player_id) {
            // Handle movement
            let gx = if input.x >= 0.0 {
                input.x.floor() as usize
            } else {
                usize::MAX
            };
            let gy = if input.y >= 0.0 {
                input.y.floor() as usize
            } else {
                usize::MAX
            };
            if gx != usize::MAX && gy != usize::MAX && self.logic_maze.is_walkable(gx, gy) {
                p.pos_x = input.x;
                p.pos_y = input.y;
                p.angle = input.angle;
                p.last_seen = std::time::Instant::now();
            }

            // Handle shooting
            if input.action == protocol::Action::Shoot {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                
                if p.ammo > 0 && (current_time - p.last_shot_time) > 0.5 {
                    p.ammo -= 1;
                    p.last_shot_time = current_time;
                    
                    // Create bullet
                    let bullet = BulletInfo {
                        bullet_id: self.next_bullet_id,
                        shooter_id: input.player_id,
                        x: input.x,
                        y: input.y,
                        angle: input.angle,
                        speed: 25.0, // Increased bullet speed for better gameplay
                        damage: 25,
                        lifetime: 0.0,
                        max_lifetime: 3.0, // 3 seconds max
                    };
                    self.next_bullet_id += 1;
                    self.bullets.push(bullet);
                }
            }
        }
    }

    /// Updates bullet positions and handles collisions
    fn update_bullets(
        &mut self,
        tx_out: &tokio::sync::mpsc::UnboundedSender<(
            std::net::SocketAddr,
            protocol::ServerToClient,
        )>,
    ) {
        let dt = 1.0 / 60.0; // 60 FPS
        let mut bullets_to_remove = Vec::new();
        let mut hit_events = Vec::new();
        let mut death_events = Vec::new();
        let mut respawn_events = Vec::new();

        for (i, bullet) in self.bullets.iter_mut().enumerate() {
            // Update bullet position
            bullet.x += bullet.angle.cos() * bullet.speed * dt;
            bullet.y += bullet.angle.sin() * bullet.speed * dt;
            bullet.lifetime += dt;

            // Check if bullet hit a wall
            let gx = bullet.x.floor() as usize;
            let gy = bullet.y.floor() as usize;
            if gx >= self.logic_maze.width || gy >= self.logic_maze.height || 
               !self.logic_maze.is_walkable(gx, gy) {
                bullets_to_remove.push(i);
                continue;
            }

            // Check if bullet hit a player
            let mut hit_player = false;
            for (player_id, player) in self.players.iter_mut() {
                if *player_id == bullet.shooter_id {
                    continue; // Can't hit yourself
                }

                let dx = bullet.x - player.pos_x;
                let dy = bullet.y - player.pos_y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < 0.5 { // Hit radius
                    // Player hit!
                    let was_alive = player.health > 0;
                    if player.health > bullet.damage {
                        player.health -= bullet.damage;
                    } else {
                        player.health = 0;
                        player.deaths += 1;
                    }

                    hit_events.push(protocol::HitEvent {
                        shooter_id: bullet.shooter_id,
                        victim_id: *player_id,
                        damage: bullet.damage,
                        bullet_id: bullet.bullet_id,
                    });

                    if was_alive && player.health == 0 {
                        death_events.push(protocol::DeathEvent {
                            victim_id: *player_id,
                            killer_id: bullet.shooter_id,
                            weapon: "Laser".to_string(),
                        });

                        // Prepare respawn event
                        respawn_events.push((*player_id, *player_id));
                    }

                    hit_player = true;
                    bullets_to_remove.push(i);
                    break;
                }
            }

            // Remove bullet if lifetime expired
            if bullet.lifetime >= bullet.max_lifetime {
                bullets_to_remove.push(i);
            }
        }

        // Award kills to shooters (separate loop to avoid borrowing issues)
        for event in &death_events {
            if let Some(shooter) = self.players.get_mut(&event.killer_id) {
                shooter.kills += 1;
                shooter.score += 100;
            }
        }

        // Handle respawns (separate loop to avoid borrowing issues)
        let mut respawn_positions = Vec::new();
        for _ in 0..respawn_events.len() {
            respawn_positions.push(self.next_spawn());
        }
        
        for ((player_id, _), (sx, sy)) in respawn_events.iter().zip(respawn_positions.iter()) {
            if let Some(player) = self.players.get_mut(player_id) {
                player.pos_x = *sx;
                player.pos_y = *sy;
                player.health = 100;
                player.ammo = 30;
                player.angle = 0.0;
            }
        }

        // Remove bullets (in reverse order to maintain indices)
        bullets_to_remove.sort_by(|a, b| b.cmp(a));
        for &index in &bullets_to_remove {
            if index < self.bullets.len() {
                self.bullets.remove(index);
            }
        }

        // Send hit and death events to all clients
        let addrs: Vec<std::net::SocketAddr> = self.addr_by_player.values().copied().collect();
        for event in hit_events {
            for addr in &addrs {
                let _ = tx_out.send((*addr, protocol::ServerToClient::Hit(event.clone())));
            }
        }
        for event in death_events {
            for addr in &addrs {
                let _ = tx_out.send((*addr, protocol::ServerToClient::Death(event.clone())));
            }
        }
    }

    /// Changes the level and respawns all players
    fn change_level(
        &mut self,
        level_id: u32,
        _tx_out: &tokio::sync::mpsc::UnboundedSender<(
            std::net::SocketAddr,
            protocol::ServerToClient,
        )>,
    ) {
        println!("🎯 SERVER: Changing to level {} ({} -> {})", level_id, self.logic_maze.name, self.logic_maze.level_id);
        
        // Load new maze
        self.logic_maze = Maze::load_level(level_id as u8);
        self.wire_level = maze_to_protocol(level_id, &self.logic_maze);
        
        println!("✅ SERVER: Loaded level {}: '{}' ({}x{})", level_id, self.logic_maze.name, self.logic_maze.width, self.logic_maze.height);
        
        // Update spawn points
        self.spawns = self.logic_maze.spawn_points(128);
        self.spawn_cursor = 0;
        
        // Clear bullets
        self.bullets.clear();
        
        // Collect spawn positions first to avoid borrowing issues
        let mut spawn_positions = Vec::new();
        for _ in 0..self.players.len() {
            spawn_positions.push(self.next_spawn());
        }
        
        // Respawn all players
        for ((player_id, player), (sx, sy)) in self.players.iter_mut().zip(spawn_positions.iter()) {
            player.pos_x = *sx;
            player.pos_y = *sy;
            player.health = 100;
            player.ammo = 30;
            player.angle = 0.0;
        }
        
        // Send new level to all clients
        let level_msg = protocol::ServerToClient::Accept(protocol::JoinAccept {
            player_id: 0, // Special ID for level change
            level: self.wire_level.clone(),
        });
        
        for addr in self.addr_by_player.values() {
            let _ = _tx_out.send((*addr, level_msg.clone()));
        }
        
        println!("📤 SERVER: Sent level change to {} clients", self.addr_by_player.len());
    }
}

/// Convert your logical maze to the protocol's wire format.
/// Simplest mapping: Cell::Wall => all edges = true; Path/SpawnPoint/Cover => all edges = false.
fn maze_to_protocol(level_id: u32, m: &Maze) -> protocol::MazeLevel {
    let mut cells = Vec::with_capacity(m.width * m.height);
    for y in 0..m.height {
        for x in 0..m.width {
            let is_wall = matches!(m.grid[y][x], Cell::Wall);
            cells.push(protocol::MazeCell {
                wall_north: is_wall,
                wall_south: is_wall,
                wall_east: is_wall,
                wall_west: is_wall,
            });
        }
    }
    protocol::MazeLevel {
        level_id,
        width: m.width as u32,
        height: m.height as u32,
        cells,
        name: m.name.clone(),
        description: m.description.clone(),
    }
}

// Helper function to encode server messages
fn encode_server(msg: &protocol::ServerToClient) -> Result<Vec<u8>, protocol::ProtocolError> {
    protocol::encode_server(msg)
}
