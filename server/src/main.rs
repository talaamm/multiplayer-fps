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
                    Cell::Exit => 'E',
                };
                print!("{ch}");
            }
        }
        println!();
    }
}

fn main_single_player() {
    let mut lvl = 1;

    // choose a level
    let mut maze = Maze::load_level(lvl);

    // Test multiplayer support
    maze.test_multiplayer_support();

    // pick first spawn or fallback
    let (sx, sy) = maze.spawn_points(1).get(0).copied().unwrap_or((0, 0));
    let mut p = Player::new(sx, sy);

    println!("Controls: W/A/S/D to move, Q to quit. Reach 'E' to win!\n");

    loop {
        // clear screen (simple)
        print!("\x1B[2J\x1B[H"); // ANSI clear + home
        render_with_player(&maze, p.x, p.y);

        if p.at_exit(&maze) {
            lvl += 1;
            if lvl > 3 {
                println!("\nYou reached the EXIT! 🎉");
                break;
            }
            maze = Maze::load_level(lvl);
            // Get a new spawn point for the new level
            let (sx, sy) = maze.spawn_points(1).get(0).copied().unwrap_or((1, 1));
            p.x = sx;
            p.y = sy;
            println!(
                "\nLevel {} loaded! Starting at position ({}, {})",
                lvl, sx, sy
            );
        }

        println!("\nPos: ({}, {}). Move [W/A/S/D], Quit [Q]: ", p.x, p.y);

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
    println!("Server listening on {}", socket.local_addr()?);

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
            let mut last_level_check = std::time::Instant::now();
            let level_check_cooldown = std::time::Duration::from_secs(2); // Only check every 2 seconds

            loop {
                ticker.tick().await;

                // Check for exit completion and handle level progression (with cooldown)
                let level_advanced = {
                    let now = std::time::Instant::now();
                    if now.duration_since(last_level_check) >= level_check_cooldown {
                        last_level_check = now;
                        let mut st = state_for_broadcast.lock();
                        if let Some(completed_level) = st.check_exits() {
                            println!(
                                "🎯 Level {} completed! Advancing to next level...",
                                completed_level
                            );
                            /*If the server prints level_id = 3 but the client still sees 2, the bug is in the protocol serialization/deserialization.
                            If the server prints level_id = 2 when it should be 3, the bug is in the server's level advancement logic. */
                            st.advance_level(&tx_out_broadcast); // <-- pass the sender here
                            println!(
                                "SERVER DEBUG: Broadcasting level_id = {}",
                                st.wire_level.level_id
                            );
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                // If level advanced, send new level data to all clients
                if level_advanced {
                    println!("🚀 Sending new level data to all clients");
                    let new_level_data = {
                        let st = state_for_broadcast.lock();
                        st.wire_level.clone()
                    };

                    let level_msg = protocol::ServerToClient::Accept(protocol::JoinAccept {
                        player_id: 0, // Special ID for level change
                        level: new_level_data,
                    });

                    let addrs: Vec<std::net::SocketAddr> = {
                        let st = state_for_broadcast.lock();
                        st.addr_by_player.values().copied().collect()
                    };
                    println!("📤 Broadcasting level change to {} clients", addrs.len());
                    for addr in addrs {
                        println!("SERVER DEBUG: Sending message to {}: {:?}", addr, level_msg);
                        let _ = tx_out_broadcast.send((addr, level_msg.clone()));
                    }
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
                        })
                        .collect::<Vec<_>>();

                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    protocol::ServerToClient::Snapshot(protocol::Snapshot {
                        server_time_ms: now_ms,
                        players,
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
                // Validate movement using your maze (no walking through walls)
                let mut st = state.lock();
                st.try_apply_input(input.player_id, input.x, input.y, input.angle);
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
    last_seen: std::time::Instant,
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
            last_seen: std::time::Instant::now(),
        };

        self.players.insert(player_id, info.clone());
        self.addr_by_player.insert(player_id, addr);
        self.player_by_addr.insert(addr, player_id);
        (player_id, info)
    }

    /// Validates a world-space move against your maze.
    /// For now we treat 1 world unit == 1 maze tile; we accept the move if the target tile is walkable.
    fn try_apply_input(&mut self, player_id: u64, new_x: f32, new_y: f32, new_angle: f32) {
        if let Some(p) = self.players.get_mut(&player_id) {
            let gx = if new_x >= 0.0 {
                new_x.floor() as usize
            } else {
                usize::MAX
            }; // outside -> reject
            let gy = if new_y >= 0.0 {
                new_y.floor() as usize
            } else {
                usize::MAX
            };
            if gx != usize::MAX && gy != usize::MAX && self.logic_maze.is_walkable(gx, gy) {
                p.pos_x = new_x;
                p.pos_y = new_y;
                p.angle = new_angle;
                p.last_seen = std::time::Instant::now();
            } else {
                // Reject invalid move; optional: snap back or ignore silently
                // (Here we just ignore; client will be corrected by snapshots.)
            }
        }
    }

    /// Checks if any player has reached the exit and handles level progression
    fn check_exits(&mut self) -> Option<u32> {
        for (player_id, player) in self.players.iter() {
            let gx = player.pos_x.floor() as usize;
            let gy = player.pos_y.floor() as usize;

            if gx < self.logic_maze.width && gy < self.logic_maze.height {
                let cell = &self.logic_maze.grid[gy][gx];
                if matches!(cell, Cell::Exit) {
                    println!(
                        "🎯 Player {} reached exit at ({}, {}) - Level {} completed!",
                        player_id, gx, gy, self.logic_maze.level_id
                    );
                    return Some(self.logic_maze.level_id);
                }
            }
        }
        None
    }

    /// Advances to the next level and resets all players
    // fn advance_level(&mut self) {
    //     let current_level = self.logic_maze.level_id;
    //     let next_level = if current_level < 3 { current_level + 1 } else { 1 };

    //     println!("🚀 Advancing from level {} to level {}", current_level, next_level);

    //     // Load new maze
    //     self.logic_maze = Maze::load_level(next_level as u8);

    //     // Update wire level
    //     self.wire_level = maze_to_protocol(next_level, &self.logic_maze);

    //     // Reset spawn cursor
    //     self.spawn_cursor = 0;

    //     // Respawn all players at new spawn points
    //     let player_count = self.players.len();
    //     println!("🔄 Respawning {} players for new level", player_count);

    //     // Collect spawn points first to avoid borrowing conflicts
    //     let mut spawn_points = Vec::new();
    //     for _ in 0..player_count {
    //         spawn_points.push(self.next_spawn());
    //     }

    //     // Now update player positions
    //     for (i, (player_id, player)) in self.players.iter_mut().enumerate() {
    //         let (sx, sy) = spawn_points[i];
    //         player.pos_x = sx;
    //         player.pos_y = sy;
    //         player.angle = 0.0;
    //         player.health = 100; // Reset health
    //         println!("📍 Respawned player {} at ({}, {})", player_id, sx, sy);
    //     }
    // }
    fn advance_level(
        &mut self,
        tx_out: &tokio::sync::mpsc::UnboundedSender<(
            std::net::SocketAddr,
            protocol::ServerToClient,
        )>,
    ) {
        // Advance level (wrap after 3)
        let next_level = match self.wire_level.level_id {
            1 => 2,
            2 => 3,
            _ => 1,
        };
        println!(
            "Advancing from level {} to {}",
            self.wire_level.level_id, next_level
        );
        self.logic_maze = Maze::load_level(next_level as u8);
        self.wire_level = maze_to_protocol(next_level, &self.logic_maze);
        self.spawns = self.logic_maze.spawn_points(128);
        self.spawn_cursor = 0;

        // Reset all player positions
        for (_pid, info) in self.players.iter_mut() {
            let (sx, sy) = self.spawns[self.spawn_cursor % self.spawns.len()];
            self.spawn_cursor += 1;
            info.pos_x = sx as f32 + 0.5;
            info.pos_y = sy as f32 + 0.5;
            info.angle = 0.0;
        }

        // Broadcast new maze to all clients
        // for (addr, _info) in self.addr_to_id.iter() {
        //     let accept_msg = protocol::ServerToClient::Accept(protocol::Accept {
        //         level: self.wire_level.clone(),
        //         player_id: 0, // 0 signals level change
        //     });
        //     let buf = protocol::encode_server(&accept_msg);
        //     let _ = self.socket.send_to(&buf, addr);
        // }
        for addr in self.addr_by_player.values() {
            let accept_msg = protocol::ServerToClient::Accept(protocol::JoinAccept {
                level: self.wire_level.clone(),
                player_id: 0, // 0 signals level change
            });
            let _ = tx_out.send((*addr, accept_msg));
        }
    }
}

/// Convert your logical maze to the protocol's wire format.
/// Simplest mapping: Cell::Wall => all edges = true; Path/Exit => all edges = false.
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
    }
}

// Helper function to encode server messages
fn encode_server(msg: &protocol::ServerToClient) -> Result<Vec<u8>, protocol::ProtocolError> {
    protocol::encode_server(msg)
}
