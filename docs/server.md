### Server networking and flow

Key file: `server/src/main.rs`

- Initialization
  - Binds UDP socket at `SERVER_BIND` or `0.0.0.0:34254`.
  - Loads level and builds wire format via `maze_to_protocol`.
  - Initializes `ServerState` inside `parking_lot::Mutex`.
- Outgoing send task
  - An unbounded MPSC channel `(SocketAddr, ServerToClient)` feeds a Tokio task that encodes via `encode_server` and `send_to`.
- Broadcast/simulation task
  - Ticks at 20Hz; advances bullets at 60Hz granularly; compiles a `Snapshot` with players and bullets and sends to all known client addresses.
- Receive loop
  - `recv_from` reads packets; decoded with `protocol::decode_client` and matched:
    - `Join`: registers player, assigns spawn, replies with `Accept{player_id, level}`.
    - `Input`: movement and actions handled by `ServerState::handle_input`.
    - `SelectLevel`: calls `change_level` to load a new maze and respawn all players; broadcasts new `Accept` (level payload) to all.
    - `Leave`: removes mappings, notifies others via `PlayerLeft`.
    - `Ping`: replies `Pong` with same nonce.
    - Errors: replies `Error { message }`.

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