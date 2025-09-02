### Protocol

Key file: `protocol/src/lib.rs`

- Serialization: JSON via Serde (`encode_client/server`, `decode_client/server`).
- Actions: `Action::{None, Move, Shoot, Ping, SelectLevel}`.
- Core messages:
  - Client → Server (`ClientToServer`): `Join`, `Input`, `Leave`, `Ping`, `SelectLevel`.
  - Server → Client (`ServerToClient`): `Accept`, `Snapshot`, `PlayerLeft`, `Pong`, `Hit`, `Death`, `Error`.
- State payloads:
  - `PlayerState`: id, name, pos (x,y), `angle`, `health`, `score`, `ammo`, `kills`, `deaths`.
  - `Bullet`: id, shooter, pos, angle, speed, damage, lifetime.
  - `MazeLevel`: `level_id`, dimensions, `cells: Vec<MazeCell>`, `name`, `description`.

Removed during minimization
- `LevelList` and `LevelInfo` (not used by client/server flows).
- `RespawnEvent` and `ServerToClient::Respawn` (respawns are handled implicitly by snapshots and level reloads).

### Typical flows

- Join: Client sends `Join{username}` → Server replies `Accept{player_id, level}`.
- Movement/Shooting: Client sends `Input{...}`; Server validates, simulates, and periodically sends `Snapshot`.
- Level change: Client sends `SelectLevel{level_id}` → Server loads, respawns everyone, and sends `Accept{level}` to all (with `player_id==0`).
- Ping: Client sends `Ping{nonce}` → Server replies `Pong{nonce}`.

### Why JSON

- Human-readable and easy to debug (store/replay packets from logs).
- Cross-language friendly; future clients can be written in any language.
- Overhead acceptable for gameplay state at current rates (20Hz snapshots, compact structs). 