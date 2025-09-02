#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Path,
    SpawnPoint, // New: dedicated spawn points for FPS
    Cover,      // New: cover objects for tactical gameplay
}

#[derive(Debug)]
pub struct Maze {
    // This struct stores the maze size and the actual grid.
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Cell>>,
    pub level_id: u32,
    pub name: String,
    pub description: String,
    pub max_players: u8,
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////      Maze struct      ///////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////
impl Maze {
    pub fn new(
        width: usize,
        height: usize,
        level_id: u32,
        name: String,
        description: String,
        max_players: u8,
    ) -> Self {
        // Start with all walls
        let grid = vec![vec![Cell::Wall; width]; height];
        Maze {
            width,
            height,
            grid,
            level_id,
            name,
            description,
            max_players,
        }
    }

    pub fn set_path(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.grid[y][x] = Cell::Path;
            //Turns a cell into a Path (walkable).
        }
    }

    pub fn set_spawn_point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.grid[y][x] = Cell::SpawnPoint;
            // Marks a cell as a spawn point for FPS gameplay.
        }
    }

    pub fn set_cover(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.grid[y][x] = Cell::Cover;
            // Marks a cell as cover (walkable but provides protection).
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        matches!(self.grid[y][x], Cell::Path | Cell::SpawnPoint | Cell::Cover)
    }

    pub fn is_spawn_point(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        matches!(self.grid[y][x], Cell::SpawnPoint)
    }

    pub fn is_cover(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        matches!(self.grid[y][x], Cell::Cover)
    }

    ///////////////////////////////////////      levels            /////////////////////////////////////

    pub fn load_level(level: u8) -> Self {
        match level {
            1 => Maze::level1_arena(),
            2 => Maze::level2_corridors(),
            3 => Maze::level3_complex(),
            4 => Maze::level4_symmetrical(),
            5 => Maze::level5_open(),
            _ => Maze::level1_arena(),
        }
    }

    // Arena-style map - good for close combat
    fn level1_arena() -> Self {
        let mut maze = Maze::new(
            20,
            20,
            1,
            "The Arena".to_string(),
            "Close-quarters combat arena".to_string(),
            8,
        );

        // Create a central arena with surrounding paths
        // Outer ring
        for x in 1..19 {
            maze.set_path(x, 1);
            maze.set_path(x, 18);
        }
        for y in 1..19 {
            maze.set_path(1, y);
            maze.set_path(18, y);
        }

        // Inner arena area
        for x in 3..17 {
            for y in 3..17 {
                maze.set_path(x, y);
            }
        }

        // Add some cover in the center
        maze.set_cover(8, 8);
        maze.set_cover(9, 8);
        maze.set_cover(10, 8);
        maze.set_cover(11, 8);
        maze.set_cover(8, 9);
        maze.set_cover(11, 9);
        maze.set_cover(8, 10);
        maze.set_cover(11, 10);
        maze.set_cover(8, 11);
        maze.set_cover(9, 11);
        maze.set_cover(10, 11);
        maze.set_cover(11, 11);

        // Spawn points around the arena
        let spawn_points = [
            (2, 2),
            (17, 2),
            (2, 17),
            (17, 17), // Corners
            (10, 2),
            (10, 17),
            (2, 10),
            (17, 10), // Mid-sides
        ];

        for &(x, y) in &spawn_points {
            maze.set_spawn_point(x, y);
        }

        maze
    }

    // Corridor-based map - good for tactical gameplay
    fn level2_corridors() -> Self {
        let mut maze = Maze::new(
            25,
            25,
            2,
            "The Corridors".to_string(),
            "Tactical corridor combat".to_string(),
            10,
        );

        // Create a grid of corridors with rooms
        // Horizontal corridors
        for x in 1..24 {
            maze.set_path(x, 5);
            maze.set_path(x, 10);
            maze.set_path(x, 15);
            maze.set_path(x, 20);
        }

        // Vertical corridors
        for y in 1..24 {
            maze.set_path(5, y);
            maze.set_path(10, y);
            maze.set_path(15, y);
            maze.set_path(20, y);
        }

        // Create rooms at intersections
        for x in [4, 9, 14, 19] {
            for y in [4, 9, 14, 19] {
                maze.set_path(x, y);
                maze.set_path(x + 1, y);
                maze.set_path(x, y + 1);
                maze.set_path(x + 1, y + 1);
            }
        }

        // Add cover in rooms
        for x in [4, 14] {
            for y in [4, 14] {
                maze.set_cover(x + 1, y + 1);
            }
        }

        // Spawn points at room corners
        let spawn_points = [
            (3, 3),
            (21, 3),
            (3, 21),
            (21, 21), // Corner rooms
            (9, 3),
            (15, 3),
            (9, 21),
            (15, 21), // Side rooms
            (3, 9),
            (21, 9),
            (3, 15),
            (21, 15), // Side rooms
        ];

        for &(x, y) in &spawn_points {
            maze.set_spawn_point(x, y);
        }

        maze
    }

    // Compact zigzag maze - completely unique design
    fn level3_complex() -> Self {
        let mut maze = Maze::new(
            20,
            20,
            3,
            "The Zigzag".to_string(),
            "Compact zigzag maze with tight corridors".to_string(),
            12,
        );

        // Create a zigzag pattern that fills the maze densely
        // Start with outer walls
        for x in 0..20 {
            maze.set_path(x, 0);
            maze.set_path(x, 19);
        }
        for y in 0..20 {
            maze.set_path(0, y);
            maze.set_path(19, y);
        }

        // Create zigzag corridors
        for y in 2..18 {
            if y % 3 == 0 {
                // Horizontal corridor
                for x in 2..18 {
                    maze.set_path(x, y);
                }
            } else if y % 3 == 1 {
                // Vertical corridor on left side
                for x in 2..10 {
                    maze.set_path(x, y);
                }
            } else {
                // Vertical corridor on right side
                for x in 10..18 {
                    maze.set_path(x, y);
                }
            }
        }

        // Add connecting corridors
        for x in [5, 15] {
            for y in 1..19 {
                maze.set_path(x, y);
            }
        }

        // Add strategic cover points
        let cover_positions = [
            (3, 3),
            (16, 3),
            (3, 16),
            (16, 16), // Corner cover
            (10, 10), // Central cover
            (7, 7),
            (12, 7),
            (7, 12),
            (12, 12), // Mid-cover
        ];

        for &(cx, cy) in &cover_positions {
            if maze.is_walkable(cx, cy) {
                maze.set_cover(cx, cy);
            }
        }

        // Strategic spawn points
        let spawn_points = [
            (2, 2),
            (17, 2),
            (2, 17),
            (17, 17), // Corners
            (10, 2),
            (10, 17),
            (2, 10),
            (17, 10), // Mid-sides
            (5, 5),
            (14, 5),
            (5, 14),
            (14, 14), // Inner corners
        ];

        for &(sx, sy) in &spawn_points {
            if maze.is_walkable(sx, sy) {
                maze.set_spawn_point(sx, sy);
            }
        }

        maze
    }

    // Complex maze with multiple layers - good for tactical gameplay
    fn level4_symmetrical() -> Self {
        let mut maze = Maze::new(
            28,
            28,
            4,
            "The Labyrinth".to_string(),
            "Complex multi-layer maze".to_string(),
            10,
        );

        // Create a complex maze using recursive backtracking
        Maze::generate_recursive_maze(&mut maze, 1, 1);

        // Add some strategic open areas
        for x in 8..20 {
            for y in 8..20 {
                if maze.grid[y][x] == Cell::Wall {
                    maze.set_path(x, y);
                }
            }
        }

        // Add more cover throughout the maze
        for x in [3, 7, 11, 15, 19, 23] {
            for y in [3, 7, 11, 15, 19, 23] {
                if maze.is_walkable(x, y) {
                    maze.set_cover(x, y);
                }
            }
        }

        // Find spawn points in walkable areas
        let mut spawn_count = 0;
        for y in 0..maze.height {
            for x in 0..maze.width {
                if maze.is_walkable(x, y) && spawn_count < 10 {
                    maze.set_spawn_point(x, y);
                    spawn_count += 1;
                }
            }
        }

        maze
    }

    // Brutal death maze - extremely complex and challenging
    fn level5_open() -> Self {
        let mut maze = Maze::new(
            25,
            25,
            5,
            "The Brutal Death Maze".to_string(),
            "Brutal death maze - extremely complex and challenging".to_string(),
            15,
        );

        // Start with outer walls
        for x in 0..25 {
            maze.set_path(x, 0);
            maze.set_path(x, 24);
        }
        for y in 0..25 {
            maze.set_path(0, y);
            maze.set_path(24, y);
        }

        // Create a brutal death maze with extreme complexity
        // Layer 1: Create a complex interconnected network
        // Main horizontal corridors with gaps
        for y in [2, 6, 10, 14, 18, 22] {
            for x in 1..24 {
                if x % 4 != 0 {
                    // Create gaps every 4 cells
                    maze.set_path(x, y);
                }
            }
        }

        // Main vertical corridors with gaps
        for x in [2, 6, 10, 14, 18, 22] {
            for y in 1..24 {
                if y % 4 != 0 {
                    // Create gaps every 4 cells
                    maze.set_path(x, y);
                }
            }
        }

        // Layer 2: Add diagonal corridors for extreme confusion
        // Diagonal corridors from top-left to bottom-right
        for i in 0..20 {
            let x = 2 + i;
            let y = 2 + i;
            if x < 23 && y < 23 {
                maze.set_path(x, y);
            }
        }

        // Diagonal corridors from top-right to bottom-left
        for i in 0..20 {
            let x = 22 - i;
            let y = 2 + i;
            if x > 1 && y < 23 {
                maze.set_path(x, y);
            }
        }

        // Layer 3: Add deadly chokepoints and dead ends
        // Create deadly chokepoints at intersections
        for x in [4, 8, 12, 16, 20] {
            for y in [4, 8, 12, 16, 20] {
                // Create small dead ends around intersections
                if x > 1 && x < 23 && y > 1 && y < 23 {
                    // Add some dead ends
                    if (x + y) % 3 == 0 {
                        maze.set_path(x + 1, y);
                        maze.set_path(x - 1, y);
                    } else if (x + y) % 3 == 1 {
                        maze.set_path(x, y + 1);
                        maze.set_path(x, y - 1);
                    }
                }
            }
        }

        // Layer 4: Add extreme complexity with random patterns
        // Create additional confusing paths
        for i in 0..15 {
            let x = 3 + (i * 2) % 20;
            let y = 3 + (i * 3) % 20;
            if x < 22 && y < 22 {
                maze.set_path(x, y);
                maze.set_path(x + 1, y);
                maze.set_path(x, y + 1);
            }
        }

        // Layer 5: Add deadly spiral-like patterns in corners
        // Top-left deadly zone
        for i in 0..6 {
            maze.set_path(3 + i, 3);
            maze.set_path(3, 3 + i);
        }

        // Top-right deadly zone
        for i in 0..6 {
            maze.set_path(21 - i, 3);
            maze.set_path(21, 3 + i);
        }

        // Bottom-left deadly zone
        for i in 0..6 {
            maze.set_path(3 + i, 21);
            maze.set_path(3, 21 - i);
        }

        // Bottom-right deadly zone
        for i in 0..6 {
            maze.set_path(21 - i, 21);
            maze.set_path(21, 21 - i);
        }

        // Add extreme cover throughout the maze - much more than level 4
        for x in [3, 7, 11, 15, 19, 23] {
            for y in [3, 7, 11, 15, 19, 23] {
                if maze.is_walkable(x, y) {
                    maze.set_cover(x, y);
                }
            }
        }

        // Add additional strategic cover points in key areas
        let strategic_cover = [
            // Central death zone
            (12, 12),
            (13, 12),
            (14, 12),
            (11, 13),
            (15, 13),
            (11, 14),
            (15, 14),
            (12, 15),
            (13, 15),
            (14, 15),
            // Corner death zones
            (5, 5),
            (19, 5),
            (5, 19),
            (19, 19),
            (6, 6),
            (18, 6),
            (6, 18),
            (18, 18),
            // Mid-side death zones
            (12, 5),
            (12, 19),
            (5, 12),
            (19, 12),
            (11, 5),
            (13, 5),
            (11, 19),
            (13, 19),
            (5, 11),
            (5, 13),
            (19, 11),
            (19, 13),
            // Internal death zones
            (10, 10),
            (14, 10),
            (10, 14),
            (14, 14),
            (11, 11),
            (13, 11),
            (11, 13),
            (13, 13),
        ];

        for &(cx, cy) in &strategic_cover {
            if maze.is_walkable(cx, cy) {
                maze.set_cover(cx, cy);
            }
        }

        // Find spawn points in walkable areas - much more than level 4
        let mut spawn_count = 0;
        for y in 0..maze.height {
            for x in 0..maze.width {
                if maze.is_walkable(x, y) && spawn_count < 15 {
                    maze.set_spawn_point(x, y);
                    spawn_count += 1;
                }
            }
        }

        maze
    }

    // Bonus: Auto-maze generator algorithm using recursive backtracking
    fn generate_recursive_maze(maze: &mut Maze, x: usize, y: usize) {
        if x >= maze.width - 1 || y >= maze.height - 1 || x == 0 || y == 0 {
            return;
        }

        maze.set_path(x, y);

        // Directions: up, right, down, left
        let directions = [(0, -2), (2, 0), (0, 2), (-2, 0)];
        let _rng = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};

        // Shuffle directions using a simple hash-based approach
        let mut dirs = directions.to_vec();
        for i in 0..dirs.len() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            (x, y, i).hash(&mut hasher);
            let j = (hasher.finish() as usize) % dirs.len();
            dirs.swap(i, j);
        }

        for &(dx, dy) in &dirs {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;

            if nx > 0
                && nx < maze.width - 1
                && ny > 0
                && ny < maze.height - 1
                && maze.grid[ny][nx] == Cell::Wall
            {
                // Carve path to neighbor
                let wall_x = (x as i32 + dx / 2) as usize;
                let wall_y = (y as i32 + dy / 2) as usize;
                maze.set_path(wall_x, wall_y);
                Maze::generate_recursive_maze(maze, nx, ny);
            }
        }
    }

    pub fn spawn_points(&self, count: usize) -> Vec<(usize, usize)> {
        let mut pts = Vec::with_capacity(count);
        'outer: for y in 0..self.height {
            for x in 0..self.width {
                if self.grid[y][x] == Cell::SpawnPoint {
                    pts.push((x, y));
                    if pts.len() == count {
                        break 'outer;
                    }
                }
            }
        }

        // If not enough spawn points, add walkable areas
        if pts.len() < count {
            'outer2: for y in 0..self.height {
                for x in 0..self.width {
                    if self.is_walkable(x, y) && !pts.contains(&(x, y)) {
                        pts.push((x, y));
                        if pts.len() == count {
                            break 'outer2;
                        }
                    }
                }
            }
        }

        pts
    }

    // Ensure maze has enough walkable spaces for multiple players
    pub fn has_enough_spawns(&self, required_count: usize) -> bool {
        let mut walkable_count = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_walkable(x, y) {
                    walkable_count += 1;
                    if walkable_count >= required_count {
                        return true;
                    }
                }
            }
        }
        false
    }

    // Get total number of walkable cells in the maze
    pub fn total_walkable_cells(&self) -> usize {
        let mut count = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_walkable(x, y) {
                    count += 1;
                }
            }
        }
        count
    }
}