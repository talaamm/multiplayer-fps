### Game logic

Key file: `server/src/game/logic.rs`

- Cells and walkability
  - `Cell::{Wall, Path, SpawnPoint, Cover}`; only Path/SpawnPoint/Cover are walkable (`is_walkable`).
- Maze
  - `Maze` stores dimensions, `grid`, metadata (`level_id`, `name`, `description`, `max_players`).
  - Constructors and mutators: `new`, `set_path`, `set_spawn_point`, `set_cover`.
  - Queries: `is_walkable`, `is_spawn_point`, `is_cover`, `spawn_points(count)`, `has_enough_spawns`, `total_walkable_cells`.
  - Level loading: `load_level(n)` builds one of 5 predefined maps with different gameplay characteristics; includes a recursive backtracking generator.
  - Diagnostics: `test_multiplayer_support` prints stats for 10+ player readiness.
- Player (single-player demo)
  - `Player` struct with movement (`move_*`), combat (`take_damage`, `heal`, `add_ammo`, `shoot`, `respawn`).

### How server uses it

- Movement validation in `ServerState::handle_input` relies on `Maze::is_walkable` grid-space check for authoritative collision.
- Spawns: server precomputes `spawns` via `spawn_points(128)` and cycles through with `spawn_cursor`; `next_spawn` returns tile centers.
- Level changes: `change_level` reloads maze, resets spawns, clears bullets, respawns everyone, and sends updated wire level.

### Why these choices

- Explicit cell types let us model FPS mechanics (cover vs walls) without heavy geometry; server collision remains O(1).
- Prebuilt levels plus generator enable quick iteration and variety; spawn distribution supports 10–15 players.
- Server-authoritative checks prevent wall hacks; tile-center spawn reduces immediate spawn collisions. 