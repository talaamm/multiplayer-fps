mod game { pub mod logic; }
use game::logic::{Maze, Player, Cell};
use std::io::{self, Read};

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

fn main() {
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
            println!("\nLevel {} loaded! Starting at position ({}, {})", lvl, sx, sy);
        }

        println!("\nPos: ({}, {}). Move [W/A/S/D], Quit [Q]: ", p.x, p.y);

        // non-blocking single-char read (simple blocking read fallback)
        let mut buf = [0u8; 1];
        if io::stdin().read_exact(&mut buf).is_err() { break; }
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
// Multiplayer server implementation (default entry)
#[tokio::main]
async fn main_multiplayer() -> anyhow::Result<()> {
    // ---- Networking setup ----
    let bind_addr = std::env::var("SERVER_BIND").unwrap_or_else(|_| "0.0.0.0:34254".to_string());
    let broadcast_hz: u64 = 20;
    let socket = std::sync::Arc::new(tokio::net::UdpSocket::bind(&bind_addr).await?);
    println!("Server listening on {}", socket.local_addr()?);

    // ---- Load your maze + make wire level ----
    let logic_maze = Maze::load_level(2);   // choose 1/2/3
    // logic_maze.print_ascii(); // debug if you want
    let wire_level = maze_to_protocol(2, &logic_maze);

    let state = std::sync::Arc::new(parking_lot::Mutex::new(ServerState::new(logic_maze, wire_level.clone())));

    // ---- Outbound channel + sender task ----
    let (tx_out, mut rx_out) = tokio::sync::mpsc::unbounded_channel::<(std::net::SocketAddr, protocol::ServerToClient)>();
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
            let mut ticker = tokio::time::interval(std::time::Duration::from_millis(1000 / broadcast_hz));
            loop {
                ticker.tick().await;

                let snapshot = {
                    let st = state_for_broadcast.lock();
                    let players = st.players.iter().map(|(pid, info)| protocol::PlayerState {
                        player_id: *pid,
                        username: info.username.clone(),
                        x: info.pos_x,
                        y: info.pos_y,
                        angle: info.angle,
                        health: info.health,
                        score: info.score,
                    }).collect::<Vec<_>>();

                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap()
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

            Ok(protocol::ClientToServer::Shoot(_ev)) => {
                // Shooting not implemented on the server yet (Amro/Tala scope)
                // Intentionally no-op to keep the server compiling and running
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
                let _ = tx_out.send((addr, protocol::ServerToClient::Pong(protocol::Pong { nonce: p.nonce })));
            }

            Err(err) => {
                let _ = tx_out.send((addr, protocol::ServerToClient::Error {
                    message: format!("bad request: {}", err),
                }));
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

    players: std::collections::HashMap<u64, PlayerInfo>,        // player_id -> PlayerInfo
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
    fn register_player(&mut self, addr: std::net::SocketAddr, username: String) -> (u64, PlayerInfo) {
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

    fn remove_player_by_addr(&mut self, addr: &std::net::SocketAddr) -> Option<u64> {
        if let Some(pid) = self.player_by_addr.remove(addr) {
            self.addr_by_player.remove(&pid);
            self.players.remove(&pid);
            Some(pid)
        } else {
            None
        }
    }

    /// Validates a world-space move against your maze.
    /// For now we treat 1 world unit == 1 maze tile; we accept the move if the target tile is walkable.
    fn try_apply_input(&mut self, player_id: u64, new_x: f32, new_y: f32, new_angle: f32) {
        if let Some(p) = self.players.get_mut(&player_id) {
            let gx = if new_x >= 0.0 { new_x.floor() as usize } else { usize::MAX }; // outside -> reject
            let gy = if new_y >= 0.0 { new_y.floor() as usize } else { usize::MAX };
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
                wall_east:  is_wall,
                wall_west:  is_wall,
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

// // src/main.rs

// use std::collections::HashMap;
// use std::net::SocketAddr;
// use std::sync::Arc;
// use std::time::{Duration, Instant};

// use anyhow::Result;
// use parking_lot::Mutex;
// use tokio::net::UdpSocket;
// use tokio::sync::mpsc;
// use tokio::time::interval;

// mod game { pub mod logic; }
// use game::logic::{Maze as LogicMaze, Cell};

// use protocol::{
//     ClientToServer, MazeCell, MazeLevel, PlayerState, Pong, ServerToClient,
//     decode_client, encode_server,
// };

// /// Server-side info for a connected player.
// #[derive(Debug, Clone)]
// struct PlayerInfo {
//     username: String,
//     pos_x: f32,
//     pos_y: f32,
//     angle: f32,
//     health: u8,
//     score: u32,
//     last_seen: Instant,
// }

// /// Shared server state.
// #[derive(Debug)]
// struct ServerState {
//     // Your gameplay logic (authoritative)
//     logic_maze: LogicMaze,
//     // The serialized form we send to clients on accept
//     wire_level: MazeLevel,

//     players: HashMap<u64, PlayerInfo>,        // player_id -> PlayerInfo
//     addr_by_player: HashMap<u64, SocketAddr>, // player_id -> address
//     player_by_addr: HashMap<SocketAddr, u64>, // address -> player_id
//     next_player_id: u64,

//     // Precomputed spawn points from your maze logic
//     spawns: Vec<(usize, usize)>,
//     spawn_cursor: usize,
// }

// impl ServerState {
//     fn new(logic_maze: LogicMaze, wire_level: MazeLevel) -> Self {
//         // Grab plenty of spawns; if fewer, we'll reuse cyclically.
//         let spawns = {
//             let mut s = logic_maze.spawn_points(128);
//             if s.is_empty() {
//                 // fall back to a safe-ish default
//                 s.push((1, 1));
//             }
//             s
//         };

//         Self {
//             logic_maze,
//             wire_level,
//             players: HashMap::new(),
//             addr_by_player: HashMap::new(),
//             player_by_addr: HashMap::new(),
//             next_player_id: 1,
//             spawns,
//             spawn_cursor: 0,
//         }
//     }

//     fn next_spawn(&mut self) -> (f32, f32) {
//         let (x, y) = self.spawns[self.spawn_cursor % self.spawns.len()];
//         self.spawn_cursor = (self.spawn_cursor + 1) % self.spawns.len();
//         // center of the tile
//         (x as f32 + 0.5, y as f32 + 0.5)
//         // NOTE: If your client uses a different scale, adjust here.
//     }

//     /// Registers a new player and returns (player_id, info).
//     fn register_player(&mut self, addr: SocketAddr, username: String) -> (u64, PlayerInfo) {
//         let player_id = self.next_player_id;
//         self.next_player_id += 1;

//         let (sx, sy) = self.next_spawn();
//         let info = PlayerInfo {
//             username,
//             pos_x: sx,
//             pos_y: sy,
//             angle: 0.0,
//             health: 100,
//             score: 0,
//             last_seen: Instant::now(),
//         };

//         self.players.insert(player_id, info.clone());
//         self.addr_by_player.insert(player_id, addr);
//         self.player_by_addr.insert(addr, player_id);
//         (player_id, info)
//     }

//     fn remove_player_by_addr(&mut self, addr: &SocketAddr) -> Option<u64> {
//         if let Some(pid) = self.player_by_addr.remove(addr) {
//             self.addr_by_player.remove(&pid);
//             self.players.remove(&pid);
//             Some(pid)
//         } else {
//             None
//         }
//     }

//     /// Validates a world-space move against your maze.
//     /// For now we treat 1 world unit == 1 maze tile; we accept the move if the target tile is walkable.
//     fn try_apply_input(&mut self, player_id: u64, new_x: f32, new_y: f32, new_angle: f32) {
//         if let Some(p) = self.players.get_mut(&player_id) {
//             let gx = if new_x >= 0.0 { new_x.floor() as usize } else { usize::MAX }; // outside -> reject
//             let gy = if new_y >= 0.0 { new_y.floor() as usize } else { usize::MAX };
//             if gx != usize::MAX && gy != usize::MAX && self.logic_maze.is_walkable(gx, gy) {
//                 p.pos_x = new_x;
//                 p.pos_y = new_y;
//                 p.angle = new_angle;
//                 p.last_seen = Instant::now();
//             } else {
//                 // Reject invalid move; optional: snap back or ignore silently
//                 // (Here we just ignore; client will be corrected by snapshots.)
//             }
//         }
//     }
// }

// /// Convert your logical maze to the protocol's wire format.
// /// Simplest mapping: Cell::Wall => all edges = true; Path/Exit => all edges = false.
// fn maze_to_protocol(level_id: u32, m: &LogicMaze) -> MazeLevel {
//     let mut cells = Vec::with_capacity(m.width * m.height);
//     for y in 0..m.height {
//         for x in 0..m.width {
//             let is_wall = matches!(m.grid[y][x], Cell::Wall);
//             cells.push(MazeCell {
//                 wall_north: is_wall,
//                 wall_south: is_wall,
//                 wall_east:  is_wall,
//                 wall_west:  is_wall,
//             });
//         }
//     }
//     MazeLevel {
//         level_id,
//         width: m.width as u32,
//         height: m.height as u32,
//         cells,
//     }
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     // ---- Networking setup ----
//     let bind_addr = std::env::var("SERVER_BIND").unwrap_or_else(|_| "0.0.0.0:34254".to_string());
//     let broadcast_hz: u64 = 20;
//     let socket = Arc::new(UdpSocket::bind(&bind_addr).await?);
//     println!("Server listening on {}", socket.local_addr()?);

//     // ---- Load your maze + make wire level ----
//     let logic_maze = LogicMaze::load_level(2);   // choose 1/2/3
//     // logic_maze.print_ascii(); // debug if you want
//     let wire_level = maze_to_protocol(2, &logic_maze);

//     let state = Arc::new(Mutex::new(ServerState::new(logic_maze, wire_level.clone())));

//     // ---- Outbound channel + sender task ----
//     let (tx_out, mut rx_out) = mpsc::unbounded_channel::<(SocketAddr, ServerToClient)>();
//     {
//         let socket_send = Arc::clone(&socket);
//         tokio::spawn(async move {
//             while let Some((addr, msg)) = rx_out.recv().await {
//                 if let Ok(bytes) = encode_server(&msg) {
//                     let _ = socket_send.send_to(&bytes, addr).await;
//                 }
//             }
//         });
//     }

//     // ---- Snapshot broadcast task ----
//     {
//         let state_for_broadcast = Arc::clone(&state);
//         let tx_out_broadcast = tx_out.clone();
//         tokio::spawn(async move {
//             let mut ticker = interval(Duration::from_millis(1000 / broadcast_hz));
//             loop {
//                 ticker.tick().await;

//                 let snapshot = {
//                     let st = state_for_broadcast.lock();
//                     let players = st.players.iter().map(|(pid, info)| PlayerState {
//                         player_id: *pid,
//                         username: info.username.clone(),
//                         x: info.pos_x,
//                         y: info.pos_y,
//                         angle: info.angle,
//                         health: info.health,
//                         score: info.score,
//                     }).collect::<Vec<_>>();

//                     let now_ms = std::time::SystemTime::now()
//                         .duration_since(std::time::UNIX_EPOCH).unwrap()
//                         .as_millis() as u64;

//                     ServerToClient::Snapshot(protocol::Snapshot {
//                         server_time_ms: now_ms,
//                         players,
//                     })
//                 };

//                 let addrs: Vec<SocketAddr> = {
//                     let st = state_for_broadcast.lock();
//                     st.addr_by_player.values().copied().collect()
//                 };
//                 for addr in addrs {
//                     let _ = tx_out_broadcast.send((addr, snapshot.clone()));
//                 }
//             }
//         });
//     }

//     // ---- Main receive loop ----
//     let mut buf = vec![0u8; 64 * 1024];
//     loop {
//         let (len, addr) = socket.recv_from(&mut buf).await?;
//         let data = &buf[..len];
//         match decode_client(data) {
//             Ok(ClientToServer::Join(join)) => {
//                 // Register and send Accept with your maze
//                 let (pid, info, accept_msg) = {
//                     let mut st = state.lock();
//                     let (pid, info) = st.register_player(addr, join.username);
//                     let accept = ServerToClient::Accept(protocol::JoinAccept {
//                         player_id: pid,
//                         level: st.wire_level.clone(),
//                     });
//                     (pid, info, accept)
//                 };
//                 let _ = tx_out.send((addr, accept_msg));
//                 println!("Player {} joined as {} from {}", pid, info.username, addr);
//             }

//             Ok(ClientToServer::Input(input)) => {
//                 // Validate movement using your maze (no walking through walls)
//                 let mut st = state.lock();
//                 st.try_apply_input(input.player_id, input.x, input.y, input.angle);
//             }

//             Ok(ClientToServer::Shoot(ev)) => {
//                 // No-op for now, could broadcast effects.
//                 let st = state.lock();
//                 if st.players.contains_key(&ev.player_id) {
//                     // TODO: handle shooting
//                 }
//             }

//             Ok(ClientToServer::Leave(leave)) => {
//                 // Remove and inform others
//                 let mut st = state.lock();
//                 let mut st = state.lock();
//                 if let Some(addr) = st.addr_by_player.get(&leave.player_id).copied() {
//                     st.addr_by_player.remove(&leave.player_id);
//                     st.players.remove(&leave.player_id);
//                     st.player_by_addr.remove(&addr);

//                     let msg = ServerToClient::PlayerLeft(protocol::LeaveNotice {
//                         player_id: leave.player_id,
//                     });
//                     for dest in st.addr_by_player.values().copied().collect::<Vec<_>>() {
//                         let _ = tx_out.send((dest, msg.clone()));
//                     }
//                 }
//             }

//             Ok(ClientToServer::Ping(p)) => {
//                 let _ = tx_out.send((addr, ServerToClient::Pong(Pong { nonce: p.nonce })));
//             }

//             Err(err) => {
//                 let _ = tx_out.send((addr, ServerToClient::Error {
//                     message: format!("bad request: {}", err),
//                 }));
//             }
//         }
//     }
// }

