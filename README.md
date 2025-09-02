# Maze War FPS

A multiplayer first-person shooter game inspired by the classic Maze War, built in Rust with real-time combat mechanics.

## Features

### 🎯 FPS Combat
- **Shooting Mechanics**: Left-click to shoot laser projectiles
- **Health System**: Players have 100 HP, take 25 damage per hit
- **Ammo System**: 30 ammo capacity with 0.5-second cooldown between shots
- **Kill/Death Tracking**: Real-time statistics for kills and deaths
- **Respawn System**: Automatic respawn with full health and ammo

### 🗺️ Multiple Maps
Choose from 5 different combat-oriented maps:

1. **The Arena** (20x20) - Close-quarters combat arena with central cover
2. **The Corridors** (25x25) - Tactical corridor combat with room intersections
3. **The Complex** (30x30) - Complex maze with many paths for exploration and ambush
4. **Symmetry** (24x24) - Symmetrical deathmatch arena for balanced gameplay
5. **Open Field** (35x35) - Open field for long-range combat with scattered cover

### 🎮 Gameplay
- **Movement**: WASD keys for movement
- **Looking**: Mouse for camera control
- **Shooting**: Left mouse button
- **Mouse Capture**: Click to capture mouse, Esc to release
- **Level Selection**: Choose your preferred map before joining
- **Map Change**: Press F1 during gameplay to change maps mid-game

### 🌐 Multiplayer
- Real-time multiplayer with UDP networking
- Support for up to 15 players depending on map
- Ping/latency display
- Player name tags and health indicators

## How to Play

### Starting the Server
```bash
cargo run --bin server
```
The server will start on `0.0.0.0:34254` by default.

### Starting the Client
```bash
cargo run --bin client
```

### Game Flow
1. **Connect**: Enter server IP and your username
2. **Select Map**: Choose from the available maps using arrow keys
3. **Play**: Navigate the maze, shoot enemies, and try to get the highest score!

### Controls
- **W/A/S/D**: Move forward/left/backward/right
- **Mouse**: Look around
- **Left Click**: Shoot
- **Esc**: Release mouse capture
- **F1**: Toggle map change mode (during gameplay)
- **Tab**: Switch between input fields (connection screen) / Switch between maps and skins (map selection)
- **Up/Down**: Navigate through maps and skins
- **Enter**: Confirm selection

## Technical Details

### Architecture
- **Client**: Rust with Macroquad for graphics and networking
- **Server**: Rust with Tokio for async networking
- **Protocol**: Custom binary protocol over UDP
- **Graphics**: Raycasting engine for 3D-like maze rendering

### Networking
- UDP-based for low latency
- Client-side prediction with server reconciliation
- Bullet physics handled server-side
- Real-time player state synchronization

### Maps
- Procedurally generated with different layouts
- Spawn points strategically placed
- Cover objects for tactical gameplay
- No exits - pure deathmatch gameplay

## Development

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Single Player Mode (for testing)
```bash
cargo run --bin server -- --single-player
```

## Game Modes

This is currently a **Deathmatch** game where:
- Players spawn with 100 HP and 30 ammo
- Shooting enemies awards kills and points
- Dying respawns you with full health and ammo
- No time limit or score limit
- Pure skill-based combat

## Future Enhancements

Potential features for future versions:
- Team deathmatch modes
- Different weapons and power-ups
- Capture the flag objectives
- More complex maps with multiple levels
- Sound effects and music
- Particle effects for impacts
- Leaderboards and statistics

Enjoy the game! 🎮
