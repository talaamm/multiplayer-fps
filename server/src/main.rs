use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use parking_lot::Mutex;
use protocol::{
    Action, ClientToServer, MazeCell, MazeLevel, PlayerState, Pong, ServerToClient, decode_client,
    encode_server,
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::interval;

/*
PlayerInfo: Tracks each player's state.
ServerState: Holds all players, their addresses, and the maze.
generate_dummy_maze: Creates a simple maze for the game.
main: Sets up the server, handles networking, and processes client messages.
Sender task: Sends messages to clients.
Snapshot broadcast task: Periodically sends the game state to all clients.
Receive loop: Handles all incoming client requests (join, input, shoot, leave, ping). */

/// Stores information about a player currently connected to the server.
/// Used for tracking position, health, score, and last activity.
#[derive(Debug, Clone)]
struct PlayerInfo {
    username: String,
    pos_x: f32,
    pos_y: f32,
    angle: f32,
    health: u8,
    score: u32,
    last_seen: Instant,
}

/// The main server state, containing all players, their addresses, and the maze level.
/// This is wrapped in a Mutex for safe concurrent access.
#[derive(Debug)]
struct ServerState {
    level: MazeLevel,
    players: HashMap<u64, PlayerInfo>, // player_id -> PlayerInfo
    addr_by_player: HashMap<u64, SocketAddr>, // player_id -> address
    player_by_addr: HashMap<SocketAddr, u64>, // address -> player_id
    next_player_id: u64,               // Used to assign unique player IDs
}

impl ServerState {
    /// Creates a new server state with the given maze level.
    fn new(level: MazeLevel) -> Self {
        Self {
            level,
            players: HashMap::new(),
            addr_by_player: HashMap::new(),
            player_by_addr: HashMap::new(),
            next_player_id: 1,
        }
    }

    /// Registers a new player, assigns a unique ID, and stores their info.
    /// Returns the player ID and their info.
    fn register_player(&mut self, addr: SocketAddr, username: String) -> (u64, PlayerInfo) {
        let player_id = self.next_player_id;
        self.next_player_id += 1;
        let spawn = (1.5f32, 1.5f32); // Default spawn position
        let info = PlayerInfo {
            username,
            pos_x: spawn.0,
            pos_y: spawn.1,
            angle: 0.0,
            health: 100,
            score: 0,
            last_seen: Instant::now(),
        };
        self.players.insert(player_id, info.clone());
        self.addr_by_player.insert(player_id, addr);
        self.player_by_addr.insert(addr, player_id);
        (player_id, info)
    }

    /// Removes a player by their address, cleaning up all mappings.
    fn remove_player_by_addr(&mut self, addr: &SocketAddr) -> Option<u64> {
        if let Some(pid) = self.player_by_addr.remove(addr) {
            self.addr_by_player.remove(&pid);
            self.players.remove(&pid);
            Some(pid)
        } else {
            None
        }
    }
}

/// Generates a dummy maze for testing or default gameplay.
/// The maze is a grid with borders and some internal walls.
fn generate_dummy_maze(level_id: u32, width: u32, height: u32) -> MazeLevel {
    let mut cells = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let border = x == 0 || y == 0 || x == width - 1 || y == height - 1;
            cells.push(MazeCell {
                wall_north: border || (y % 2 == 0 && x % 3 == 0),
                wall_south: border,
                wall_east: border,
                wall_west: border,
            });
        }
    }
    MazeLevel {
        level_id,
        width,
        height,
        cells,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Bind UDP socket to address (default: 0.0.0.0:34254)
    let bind_addr = std::env::var("SERVER_BIND").unwrap_or_else(|_| "0.0.0.0:34254".to_string());
    let broadcast_hz: u64 = 20; // How many times per second to broadcast snapshots
    let socket = Arc::new(UdpSocket::bind(&bind_addr).await?);
    println!("Server listening on {}", socket.local_addr()?);

    // Initialize maze and server state
    let level = generate_dummy_maze(1, 32, 32);
    let state = Arc::new(Mutex::new(ServerState::new(level.clone())));

    // Channel for outgoing messages to clients
    let (tx_out, mut rx_out) = mpsc::unbounded_channel::<(SocketAddr, ServerToClient)>();

    // Sender task: sends messages from rx_out to clients via UDP
    let socket_send = Arc::clone(&socket);
    tokio::spawn(async move {
        while let Some((addr, msg)) = rx_out.recv().await {
            if let Ok(bytes) = encode_server(&msg) {
                let _ = socket_send.send_to(&bytes, addr).await;
            }
        }
    });

    // Snapshot broadcast task: periodically sends the current game state to all clients
    let state_for_broadcast = Arc::clone(&state);
    let tx_out_broadcast = tx_out.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(1000 / broadcast_hz));
        loop {
            ticker.tick().await;
            // Build snapshot of all players
            let snapshot = {
                let st = state_for_broadcast.lock();
                let players = st
                    .players
                    .iter()
                    .map(|(pid, info)| PlayerState {
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
                ServerToClient::Snapshot(protocol::Snapshot {
                    server_time_ms: now_ms,
                    players,
                })
            };
            // Send snapshot to all connected clients
            let addrs: Vec<SocketAddr> = {
                let st = state_for_broadcast.lock();
                st.addr_by_player.values().copied().collect()
            };
            for addr in addrs {
                let _ = tx_out_broadcast.send((addr, snapshot.clone()));
            }
        }
    });

    // Main receive loop: handles incoming client messages
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];
        match decode_client(data) {
            Ok(ClientToServer::Join(join)) => {
                // Handle new player joining
                let (pid, info, accept_msg) = {
                    let mut st = state.lock();
                    let (pid, info) = st.register_player(addr, join.username);
                    let accept = ServerToClient::Accept(protocol::JoinAccept {
                        player_id: pid,
                        level: st.level.clone(),
                    });
                    (pid, info, accept)
                };
                let _ = tx_out.send((addr, accept_msg));
                println!("Player {} joined as {} from {}", pid, info.username, addr);
            }
            Ok(ClientToServer::Input(input)) => {
                // Handle player movement/input
                let mut st = state.lock();
                if let Some(info) = st.players.get_mut(&input.player_id) {
                    info.pos_x = input.x;
                    info.pos_y = input.y;
                    info.angle = input.angle;
                    info.last_seen = Instant::now();
                    if let Some(addr) = st.addr_by_player.get(&input.player_id) {
                        // Could send an acknowledgement here if needed
                        let _ = addr;
                    }
                }
            }
            Ok(ClientToServer::Shoot(ev)) => {
                // Handle shooting event (currently a no-op)
                let st = state.lock();
                if st.players.contains_key(&ev.player_id) {
                    // Could broadcast shoot event to other players here
                }
            }
            Ok(ClientToServer::Leave(leave)) => {
                // Handle player leaving
                let mut st = state.lock();
                if let Some(addr) = st.addr_by_player.get(&leave.player_id).copied() {
                    st.addr_by_player.remove(&leave.player_id);
                    st.players.remove(&leave.player_id);
                    st.player_by_addr.remove(&addr);
                    // Inform all clients that this player left
                    let msg = ServerToClient::PlayerLeft(protocol::LeaveNotice {
                        player_id: leave.player_id,
                    });
                    for dest in st.addr_by_player.values().copied().collect::<Vec<_>>() {
                        let _ = tx_out.send((dest, msg.clone()));
                    }
                }
            }
            Ok(ClientToServer::Ping(p)) => {
                // Respond to ping for latency measurement
                let _ = tx_out.send((addr, ServerToClient::Pong(Pong { nonce: p.nonce })));
            }
            Err(err) => {
                // Handle protocol errors (bad requests)
                let _ = tx_out.send((
                    addr,
                    ServerToClient::Error {
                        message: format!("bad request: {}", err),
                    },
                ));
            }
        }
    }
}
