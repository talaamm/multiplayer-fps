use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Action {
	None = 0,
	Move = 1,
	Shoot = 2,
	Ping = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
	pub player_id: u64,
	pub username: String,
	pub x: f32,
	pub y: f32,
	pub angle: f32,
	pub health: u8,
	pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeCell {
	pub wall_north: bool,
	pub wall_south: bool,
	pub wall_east: bool,
	pub wall_west: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeLevel {
	pub level_id: u32,
	pub width: u32,
	pub height: u32,
	pub cells: Vec<MazeCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
	pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinAccept {
	pub player_id: u64,
	pub level: MazeLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputUpdate {
	pub player_id: u64,
	pub x: f32,
	pub y: f32,
	pub angle: f32,
	pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShootEvent {
	pub player_id: u64,
	pub origin_x: f32,
	pub origin_y: f32,
	pub angle: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveNotice {
	pub player_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
	pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
	pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
	pub server_time_ms: u64,
	pub players: Vec<PlayerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientToServer {
	Join(JoinRequest),
	Input(InputUpdate),
	Shoot(ShootEvent),
	Leave(LeaveNotice),
	Ping(Ping),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerToClient {
	Accept(JoinAccept),
	Snapshot(Snapshot),
	PlayerLeft(LeaveNotice),
	Pong(Pong),
	Error { message: String },
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(thiserror::Error, Debug)]
pub enum ProtocolError {
	#[error("serialization error: {0}")]
	Serialization(#[from] serde_json::Error),
}

pub fn encode_client(msg: &ClientToServer) -> Result<Vec<u8>> {
	Ok(serde_json::to_vec(msg)?)
}

pub fn decode_client(bytes: &[u8]) -> Result<ClientToServer> {
	Ok(serde_json::from_slice(bytes)?)
}

pub fn encode_server(msg: &ServerToClient) -> Result<Vec<u8>> {
	Ok(serde_json::to_vec(msg)?)
}

pub fn decode_server(bytes: &[u8]) -> Result<ServerToClient> {
	Ok(serde_json::from_slice(bytes)?)
}
