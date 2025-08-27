# Maze Wars Client - Graphics & Rendering

This is the client implementation for Person 3's role as Graphics & Rendering Lead in the Maze Wars project.

## Features Implemented

### ✅ Graphics & Rendering
- **First-person 3D rendering** using raycasting (DDA algorithm)
- **Maze wall rendering** with proper depth and shading
- **Player billboard rendering** for other players with occlusion
- **Mini-map overlay** showing player position and other players
- **FPS counter** displayed on screen (target: >50 FPS)
- **Sky and floor rendering** for immersive environment

### ✅ User Interface
- **Connection screen** for IP address and username input
- **HUD display** with FPS, ping, player count, and level info
- **Mouse capture system** (click to capture, Esc to release)
- **Smooth player movement** with collision detection

### ✅ Networking Integration
- **UDP client** connecting to server
- **Real-time player synchronization** 
- **Server-provided maze levels** (no local level switching)
- **Ping/latency measurement**

## Removed Features (as requested)

### ❌ Shooting System
- Removed all shooting mechanics
- No crosshair display
- No bullet tracers
- No damage/health system

### ❌ Level Changing
- Removed local level presets (1/2/3 keys)
- Client now uses server-provided maze levels only
- No client-side level switching

## How to Run

1. **Start the server first:**
   ```bash
   cd server
   cargo run
   ```

2. **Run the client:**
   ```bash
   cd client
   cargo run
   ```

3. **Connect to server:**
   - Enter server IP (default: `127.0.0.1:34254`)
   - Enter your username
   - Press Enter to connect

4. **Game controls:**
   - **WASD**: Move
   - **Mouse**: Look around
   - **Left Click**: Capture mouse
   - **Esc**: Release mouse

## Technical Details

### Rendering Engine
- Uses **macroquad** for cross-platform graphics
- **Raycasting** for 3D wall rendering
- **Z-buffer** for proper occlusion
- **Billboard sprites** for other players

### Performance
- Target: **>50 FPS**
- Optimized raycasting with early termination
- Efficient collision detection
- Smooth interpolation for network updates

### Maze Integration
- Converts server maze format to client tiles
- Supports all 3 difficulty levels from server
- Fallback level if server doesn't provide one

## Architecture

```
Client (Person 3)
├── Graphics & Rendering
│   ├── Raycasting engine
│   ├── Player billboards
│   ├── Mini-map
│   └── HUD
├── Input handling
├── Network integration
└── UI (connection screen)
```

## Dependencies

- `macroquad = "0.4"` - Graphics engine
- `protocol = { path = "../protocol" }` - Shared protocol

## Role Responsibilities

As **Person 3 - Graphics & Rendering Lead**, this implementation focuses on:

1. **Visual presentation** of the game world
2. **Performance optimization** for smooth gameplay
3. **User interface** elements
4. **Integration** with server-provided game data

The client now works seamlessly with the server's maze levels and multiplayer functionality while providing an immersive first-person experience.
