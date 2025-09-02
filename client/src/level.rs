use macroquad::prelude::*;
use protocol;

// ---------- Server-provided Level definition ----------
#[derive(Clone, Debug)]
pub struct Level {
    pub w: usize,
    pub h: usize,
    pub tiles: Vec<u8>, // 0 = floor/path, 1 = wall, 2 = spawn point, 3 = cover
    pub name: String,
    // pub description: String,
}

impl Level {
    pub fn new(w: usize, h: usize, tiles: Vec<u8>, name: String) -> Self {
        Self {
            w,
            h,
            tiles,
            name,
            // description,
        }
    }

    // Safe cell access: out-of-bounds are treated as walls
    pub fn at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 {
            return 1;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h {
            return 1;
        }
        self.tiles[y * self.w + x]
    }

    // Check if position is walkable (path, spawn point, or cover)
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        let tile = self.at(x, y);
        tile == 0 || tile == 2 || tile == 3 // path, spawn point, or cover
    }

    // Check if position is spawn point
    // pub fn is_spawn_point(&self, x: i32, y: i32) -> bool {
    //     self.at(x, y) == 2
    // }

    // // Check if position is cover
    // pub fn is_cover(&self, x: i32, y: i32) -> bool {
    //     self.at(x, y) == 3
    // }
}

// ---------- Protocol adapter ----------
pub fn level_from_maze_level(wire: &protocol::MazeLevel) -> Level {
    let w = wire.width as usize;
    let h = wire.height as usize;
    let mut tiles = vec![1u8; w * h];

    // Convert walls and paths
    for y in 0..h {
        for x in 0..w {
            let c = &wire.cells[y * w + x];
            // Consider any wall flag as a wall; otherwise floor
            let is_wall = c.wall_north || c.wall_south || c.wall_east || c.wall_west;
            tiles[y * w + x] = if is_wall { 1 } else { 0 };
        }
    }

    Level::new(w, h, tiles, wire.name.clone())
}

// Find a safe spawn position in the level
pub fn find_safe_spawn(level: &Level) -> Vec2 {
    // Try common safe spawn positions first
    let safe_positions = [
        vec2(1.5, 1.5), // Top-left corner
        vec2(2.5, 1.5), // Top-left + 1
        vec2(1.5, 2.5), // Top-left + 1 down
        vec2(2.5, 2.5), // Top-left + 1 diagonal
    ];

    for &pos in &safe_positions {
        let x = pos.x.floor() as i32;
        let y = pos.y.floor() as i32;
        if level.is_walkable(x, y) {
            return pos;
        }
    }

    // If no safe position found, search for the first walkable tile
    for y in 0..level.h {
        for x in 0..level.w {
            if level.tiles[y * level.w + x] == 0 {
                return vec2(x as f32 + 0.5, y as f32 + 0.5);
            }
        }
    }
    // Fallback to a safe default
    vec2(1.5, 1.5)
}
