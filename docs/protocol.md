### Protocol

Key file: `protocol/src/lib.rs`

- Serialization: JSON via Serde (`encode_client/server`, `decode_client/server`).
- Actions: `Action::{None, Move, Shoot, Ping, SelectLevel}`.
- Core messages:
  - Client → Server (`ClientToServer`): `Join`, `Input`, `Leave`, `Ping`, `SelectLevel`.
  - Server → Client (`ServerToClient`): `Accept`, `Snapshot`, `PlayerLeft`, `Pong`, `Hit`, `Death`, `Respawn`, `LevelList`, `Error`.
- State payloads:
  - `PlayerState`: id, name, pos (x,y), `angle`, `health`, `score`, `ammo`, `kills`, `deaths`.
  - `Bullet`: id, shooter, pos, angle, speed, damage, lifetime.
  - `MazeLevel`: `level_id`, dimensions, `cells: Vec<MazeCell>`, `name`, `description`.

### Typical flows

- Join
  - Client sends `Join{username}` → Server registers and replies `Accept{player_id, level}`.
- Movement/Shooting
  - Client sends `Input{player_id, x, y, angle, action}` at some rate.
  - Server validates and updates, possibly spawns bullets; server broadcasts `Snapshot` periodically.
- Level change
  - Client sends `SelectLevel{level_id}` → Server loads, respawns, broadcasts `Accept{level}` to all.
- Ping
  - Client sends `Ping{nonce}` → Server replies `Pong{nonce}`.

### Why JSON

- Human-readable and easy to debug (store/replay packets from logs).
- Cross-language friendly; future clients can be written in any language.
- Overhead acceptable for gameplay state at current rates (20Hz snapshots, compact structs). 