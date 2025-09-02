### What you should be able to explain (by task)

Based on your role (Server & Networking Lead), you should be able to clearly explain these files and why/how they work together:

- `server/src/main.rs`: UDP server entrypoint, async runtime, receive loop, broadcast snapshot task, player/bullet management, level changes.
- `server/src/game/logic.rs`: Authoritative game world model: `Cell`, `Maze`, level generation, spawn points, player mechanics (movement, health, ammo), and helpers used by the server.
- `protocol/src/lib.rs`: Wire protocol shared by client and server. Message enums, structs, and JSON encode/decode helpers.
- `client/src/network.rs`: Minimal client UDP networking loop: join, send inputs, receive snapshots and events.
- `tests/integration_udp.rs`: Legacy example of UDP handshake style test (not aligned with current modules). Useful to explain differences.

Additionally, know high-level project files:
- Top-level `Cargo.toml`: workspace crates.
- `server/Cargo.toml`, `client/Cargo.toml`, `protocol/Cargo.toml`: crate deps (tokio, serde, etc.).

### System summary

- Transport: UDP (`tokio::net::UdpSocket` on server; `std::net::UdpSocket` on client thread).
- Serialization: JSON via `serde` (`encode_*`, `decode_*` in `protocol`).
- Concurrency:
  - Server uses Tokio tasks: one send task, one broadcast tick task, and the main recv loop; shared `ServerState` guarded by `parking_lot::Mutex`.
  - Client uses a dedicated networking thread with MPSC channels to app code.
- Game loop on server:
  - On tick: move bullets, detect hits, deaths, respawns; broadcast snapshots to all clients at 20 Hz.
  - On input: validate movement vs maze, spawn bullets on shoot, rate-limit firing, manage ammo.
- Scalability: stateless UDP per packet, server tracks address<->player_id maps; designed for 10+ players.

### Why this architecture

- UDP lowers latency and avoids head-of-line blocking; app-level reliability is implemented only where needed (periodic snapshots, idempotent events).
- Shared `protocol` crate ensures client/server agree on message shapes; JSON is human‑readable for debugging.
- Server is authoritative to prevent cheating and resolve conflicts; clients are thin, only send intents and render snapshots. 