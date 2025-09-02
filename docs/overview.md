### What you should be able to explain (by task)

Based on your role (Server & Networking Lead), explain these files and how they work together:

- `server/src/main.rs`: UDP server entrypoint, async tasks (sender, broadcaster), receive loop, player/bullet management, level changes.
- `server/src/game/logic.rs`: Authoritative game world: `Cell`, `Maze`, level generation, spawn points, and helpers used by the server.
- `protocol/src/lib.rs`: Wire protocol shared by client and server. Message enums, structs, and JSON encode/decode.
- `client/src/network.rs`: Client UDP thread: join, send inputs, receive snapshots and events.
- `client/src/main.rs`: How protocol messages are consumed in the app loop.

Removed during minimization
- Single-player demo and terminal rendering.
- Legacy test `tests/integration_udp.rs`.
- Protocol variants/types: `LevelList`, `LevelInfo`, `RespawnEvent`.
- Unused server helpers and temporary variables.

### System summary

- Transport: UDP (`tokio::net::UdpSocket` on server; `std::net::UdpSocket` on client thread).
- Serialization: JSON via `serde` (`encode_*`, `decode_*` in `protocol`).
- Concurrency:
  - Server uses Tokio tasks: one send task, one broadcast tick task, and the main recv loop; shared `ServerState` with `parking_lot::Mutex`.
  - Client uses a dedicated networking thread with MPSC channels.
- Game loop on server:
  - On tick: bullets update, hit/death events; broadcast snapshots to all clients at 20 Hz.
  - On input: validate movement vs maze, spawn bullets on shoot, rate-limit firing, manage ammo.
- Scalability: stateless UDP per packet; address<->player_id maps; supports 10+ players.

### Why this architecture

- UDP for low latency; periodic snapshots for resilience to loss.
- Shared `protocol` crate guarantees client/server agreement; JSON for easy debugging.
- Server authoritative logic prevents cheating; client remains thin. 