### Server networking and flow

Key file: `server/src/main.rs`

- Initialization
  - Binds UDP socket at `SERVER_BIND` or `0.0.0.0:34254`.
  - Loads level and builds wire format via `maze_to_protocol`.
  - Initializes `ServerState` inside `parking_lot::Mutex`.
- Outgoing send task
  - Unbounded MPSC `(SocketAddr, ServerToClient)` feeds a Tokio task that encodes via `protocol::encode_server` and `send_to`.
- Broadcast/simulation task
  - Ticks at 20Hz; advances bullets at 60Hz; compiles a `Snapshot` and sends to all known client addresses.
- Receive loop
  - `recv_from` → `protocol::decode_client` → match:
    - `Join` → `register_player` → reply `Accept{player_id, level}`.
    - `Input` → `handle_input` for movement/shooting.
    - `SelectLevel` → `change_level` (reload maze, respawn, broadcast new `Accept{level}` with `player_id==0`).
    - `Leave` → remove mappings and send `PlayerLeft`.
    - `Ping` → reply `Pong`.
    - Error → reply `Error { message }`.

Notes
- Single-player mode and terminal renderer were removed during minimization.
- Minor cleanup: inlined encode helper, removed an unused temporary in bullet collision loop.

### Authoritative state

`ServerState` holds:
- Maze logic (`Maze`) and wire level (`MazeLevel`).
- Players map, address maps, next ids.
- Bullets vector.
- Spawn ring buffer (`spawns`, `spawn_cursor`).

Critical methods:
- `register_player`: creates `PlayerInfo`, picks next spawn, maps addr<->id.
- `handle_input`: validates movement vs `Maze::is_walkable`, updates position/angle and fires bullets on `Action::Shoot` with cooldown and ammo.
- `update_bullets`: moves bullets, checks wall collisions, player hits, applies damage, produces hit/death events, respawns, removes expired bullets, and emits events.
- `change_level`: reloads maze, rebuilds `wire_level`, resets spawns/bullets, respawns all players, sends updated level to all clients.

### Why it works

- UDP + periodic snapshots make the system resilient to packet loss; clients will catch up next snapshot.
- Server-side validation ensures no walking through walls; bullets are simulated server-side only.
- Separation of concerns: networking tasks are small and focused; game rules remain in `ServerState` and `logic.rs`.

### Scalability & reliability notes

- 20Hz snapshots keep bandwidth moderate; events (Hit/Death) are also sent to all to reduce reliance on only snapshots.
- Address maps allow multi-client support; data structures are O(1) for lookup.
- `parking_lot::Mutex` reduces lock contention; long work kept outside critical sections where possible. 