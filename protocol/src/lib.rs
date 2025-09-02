use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Represents possible player actions sent from client to server.
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Action {
    None = 0,      // No action
    Move = 1,      // Movement input
    Shoot = 2,     // Shooting action
    Ping = 3,      // Ping for latency measurement
    SelectLevel = 4, // Level selection
}

/// Describes the state of a player in the game.
/// Used in snapshots and for syncing player info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_id: u64,
    pub username: String,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub health: u8,
    pub score: u32,
    pub ammo: u8,
    pub kills: u32,
    pub deaths: u32,
}

/// Represents a bullet in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bullet {
    pub bullet_id: u64,
    pub shooter_id: u64,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub speed: f32,
    pub damage: u8,
    pub lifetime: f32,
}

/// Represents a single cell in the maze.
/// Each cell can have walls on any side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeCell {
    pub wall_north: bool,
    pub wall_south: bool,
    pub wall_east: bool,
    pub wall_west: bool,
}

/// Describes the entire maze level.
/// Contains all cells and maze dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeLevel {
    pub level_id: u32,
    pub width: u32,
    pub height: u32,
    pub cells: Vec<MazeCell>,
    pub name: String,
    pub description: String,
}

/// Available levels for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelList {
    pub levels: Vec<LevelInfo>,
}

/// Information about a specific level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelInfo {
    pub level_id: u32,
    pub name: String,
    pub description: String,
    pub max_players: u8,
    pub size: (u32, u32),
}

/// Sent by client to request joining the game.
/// Contains the desired username.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    pub username: String,
}

/// Sent by server to accept a join request.
/// Provides player ID and maze info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinAccept {
    pub player_id: u64,
    pub level: MazeLevel,
}

/// Sent by client to update movement or action.
/// Contains new position, angle, and action type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputUpdate {
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub action: Action,
}

/// Sent by client to notify server of leaving the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveNotice {
    pub player_id: u64,
}

/// Sent by client to measure latency.
/// Contains a nonce for matching ping/pong.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    pub nonce: u64,
}

/// Sent by server in response to a ping.
/// Echoes the nonce for latency calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
    pub nonce: u64,
}

/// Sent by server to all clients to synchronize game state.
/// Contains current time and all player states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub server_time_ms: u64,
    pub players: Vec<PlayerState>,
    pub bullets: Vec<Bullet>,
}

/// Sent when a player is hit by a bullet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitEvent {
    pub shooter_id: u64,
    pub victim_id: u64,
    pub damage: u8,
    pub bullet_id: u64,
}

/// Sent when a player dies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathEvent {
    pub victim_id: u64,
    pub killer_id: u64,
    pub weapon: String,
}

/// Sent when a player respawns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespawnEvent {
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
    pub health: u8,
    pub ammo: u8,
}

/// Sent by client to select a level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelSelection {
    pub player_id: u64,
    pub level_id: u32,
}

/// All possible messages sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientToServer {
    Join(JoinRequest),
    Input(InputUpdate),
    Leave(LeaveNotice),
    Ping(Ping),
    SelectLevel(LevelSelection),
}

/// All possible messages sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerToClient {
    Accept(JoinAccept),
    Snapshot(Snapshot),
    PlayerLeft(LeaveNotice),
    Pong(Pong),
    Hit(HitEvent),
    Death(DeathEvent),
    Respawn(RespawnEvent),
    LevelList(LevelList),
    Error { message: String },
}

/// Protocol error type for serialization/deserialization.
#[derive(thiserror::Error, Debug)]
pub enum ProtocolError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type used for protocol functions.
pub type Result<T> = std::result::Result<T, ProtocolError>;

/// Serializes a client-to-server message to bytes.
pub fn encode_client(msg: &ClientToServer) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(msg)?)
}

/// Deserializes bytes into a client-to-server message.
pub fn decode_client(bytes: &[u8]) -> Result<ClientToServer> {
    Ok(serde_json::from_slice(bytes)?)
}

/// Serializes a server-to-client message to bytes.
pub fn encode_server(msg: &ServerToClient) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(msg)?)
}

/// Deserializes bytes into a server-to-client message.
pub fn decode_server(bytes: &[u8]) -> Result<ServerToClient> {
    Ok(serde_json::from_slice(bytes)?)
}
