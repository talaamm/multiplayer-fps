#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Path,
    Exit,
}

#[derive(Debug)]
pub struct Maze {
    // This struct stores the maze size and the actual grid.
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Cell>>,
    pub level_id: u32,
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////      Maze struct      ///////////////////////////////////////
/// /////////////////////////////////////////////////////////////////////////////////////////////////
impl Maze {
    pub fn new(width: usize, height: usize, level_id: u32) -> Self {
        // Start with all walls
        let grid = vec![vec![Cell::Wall; width]; height];
        Maze { width, height, grid, level_id }
    }

    pub fn set_path(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.grid[y][x] = Cell::Path;
            //Turns a cell into a Path (walkable).
        }
    }

    pub fn set_exit(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.grid[y][x] = Cell::Exit;
            // Marks a cell as the Exit.
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height { return false; }
        !matches!(self.grid[y][x], Cell::Wall)
    }

    ///////////////////////////////////////      levels            /////////////////////////////////////

    pub fn load_level(level: u8) -> Self {
        match level {
            1 => Maze::level1(),
            2 => Maze::level2(),
            3 => Maze::level3(),
            _ => Maze::level1(),
        }
    }

    fn level1() -> Self {
        let mut maze = Maze::new(15, 15, 1);
        
        // Create a very simple, open maze with almost no walls
        // Just a basic L-shaped path from start to exit with some extra paths
        // Enhanced to support 10+ players comfortably
        
        // Main path - simple L-shape from (1,1) to (13,13)
        let main_path = vec![
            // Start and go right
            (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1), (9, 1), (10, 1), (11, 1), (12, 1), (13, 1),
            // Go down to exit
            (13, 2), (13, 3), (13, 4), (13, 5), (13, 6), (13, 7), (13, 8), (13, 9), (13, 10), (13, 11), (13, 12), (13, 13)
        ];
        
        for &(x, y) in &main_path {
            maze.set_path(x, y);
        }
        
        // Make the maze more interesting with additional paths
        // Add horizontal paths for variety and more spawn points
        for x in 3..12 {
            maze.set_path(x, 5);  // Middle horizontal path
            maze.set_path(x, 9);  // Lower horizontal path
        }
        
        // Add vertical paths for variety and more spawn points
        for y in 3..12 {
            maze.set_path(5, y);  // Left vertical path
            maze.set_path(9, y);  // Right vertical path
        }
        
        // Add some diagonal-like paths for extra exploration and spawn points
        maze.set_path(3, 3); maze.set_path(4, 4); maze.set_path(5, 5);
        maze.set_path(11, 3); maze.set_path(10, 4); maze.set_path(9, 5);
        
        // Add a small loop path for more spawn options
        maze.set_path(7, 7); maze.set_path(8, 7); maze.set_path(8, 8); maze.set_path(7, 8);
        
        // Add some branching paths for more spawn points
        maze.set_path(2, 7); maze.set_path(2, 8); maze.set_path(2, 9);
        maze.set_path(12, 7); maze.set_path(12, 8); maze.set_path(12, 9);
        
        // Add additional open areas for more players
        maze.set_path(4, 7); maze.set_path(4, 8); maze.set_path(4, 9);
        maze.set_path(10, 7); maze.set_path(10, 8); maze.set_path(10, 9);
        maze.set_path(6, 3); maze.set_path(6, 4);
        maze.set_path(8, 3); maze.set_path(8, 4);
        
        // Add more dead ends and complex paths
        maze.set_path(1, 3); maze.set_path(1, 4); maze.set_path(1, 5); // Left dead end
        maze.set_path(1, 11); maze.set_path(1, 12); // Another left dead end
        
        maze.set_path(13, 3); maze.set_path(13, 4); maze.set_path(13, 5); // Right dead end
        maze.set_path(13, 11); maze.set_path(13, 12); // Another right dead end
        
        // Add some isolated areas that are harder to reach
        maze.set_path(3, 7); maze.set_path(3, 8); maze.set_path(3, 9);
        maze.set_path(11, 7); maze.set_path(11, 8); maze.set_path(11, 9);
        
        // Add some cross paths for more complexity
        maze.set_path(5, 7); maze.set_path(5, 8); maze.set_path(5, 9);
        maze.set_path(9, 7); maze.set_path(9, 8); maze.set_path(9, 9);
        
        // Ensure there's a clear path from start to exit
        maze.set_path(1, 1);  // Start point
        maze.set_exit(13, 13);
        
        // Create a guaranteed path from start to exit
        maze.create_guaranteed_path(1, 1, 13, 13);
        
        maze
    }

    fn level2() -> Self {
        let mut maze = Maze::new(25, 25, 2);
        
        // More complex maze with multiple paths and dead ends
        // Use a modified depth-first search approach
        
        // Start with all walls
        for y in 0..maze.height {
            for x in 0..maze.width {
                maze.grid[y][x] = Cell::Wall;
            }
        }
        
        // Create main corridors with branching paths
        let _corridor_width = 1;
        
        // Create a more complex maze structure instead of straight corridors
        // Start with some key connection points but make them winding
        
        // Top area - winding path instead of straight line
        maze.set_path(1, 1); maze.set_path(2, 1); maze.set_path(3, 1);
        maze.set_path(5, 1); maze.set_path(7, 1); maze.set_path(9, 1);
        maze.set_path(11, 1); maze.set_path(13, 1); maze.set_path(15, 1);
        maze.set_path(17, 1); maze.set_path(19, 1); maze.set_path(21, 1);
        maze.set_path(23, 1);
        
        // Center area - create a complex hub instead of straight lines
        maze.set_path(1, 13); maze.set_path(3, 13); maze.set_path(5, 13);
        maze.set_path(7, 13); maze.set_path(9, 13); maze.set_path(11, 13);
        maze.set_path(13, 13); maze.set_path(15, 13); maze.set_path(17, 13);
        maze.set_path(19, 13); maze.set_path(21, 13); maze.set_path(23, 13);
        
        // Bottom area - winding path
        maze.set_path(1, 23); maze.set_path(3, 23); maze.set_path(5, 23);
        maze.set_path(7, 23); maze.set_path(9, 23); maze.set_path(11, 23);
        maze.set_path(13, 23); maze.set_path(15, 23); maze.set_path(17, 23);
        maze.set_path(19, 23); maze.set_path(21, 23); maze.set_path(23, 23);
        
        // Left side - winding path
        maze.set_path(1, 1); maze.set_path(1, 3); maze.set_path(1, 5);
        maze.set_path(1, 7); maze.set_path(1, 9); maze.set_path(1, 11);
        maze.set_path(1, 13); maze.set_path(1, 15); maze.set_path(1, 17);
        maze.set_path(1, 19); maze.set_path(1, 21); maze.set_path(1, 23);
        
        // Right side - winding path
        maze.set_path(23, 1); maze.set_path(23, 3); maze.set_path(23, 5);
        maze.set_path(23, 7); maze.set_path(23, 9); maze.set_path(23, 11);
        maze.set_path(23, 13); maze.set_path(23, 15); maze.set_path(23, 17);
        maze.set_path(23, 19); maze.set_path(23, 21); maze.set_path(23, 23);
        
        // Add complex internal connections and dead ends
        // Upper left area - complex maze-like structure
        maze.set_path(3, 3); maze.set_path(3, 4); maze.set_path(3, 5);
        maze.set_path(4, 3); maze.set_path(5, 3); maze.set_path(6, 3);
        maze.set_path(4, 5); maze.set_path(5, 5); maze.set_path(6, 5);
        maze.set_path(7, 4); maze.set_path(8, 4); // Dead end branch
        
        // Upper right area - complex maze-like structure
        maze.set_path(21, 3); maze.set_path(21, 4); maze.set_path(21, 5);
        maze.set_path(20, 3); maze.set_path(19, 3); maze.set_path(18, 3);
        maze.set_path(20, 5); maze.set_path(19, 5); maze.set_path(18, 5);
        maze.set_path(17, 4); maze.set_path(16, 4); // Dead end branch
        
        // Center area with complex branching and loops
        maze.set_path(9, 9); maze.set_path(9, 10); maze.set_path(9, 11);
        maze.set_path(10, 9); maze.set_path(11, 9); maze.set_path(12, 9);
        maze.set_path(8, 11); maze.set_path(7, 11); maze.set_path(6, 11);
        maze.set_path(10, 11); maze.set_path(11, 11); maze.set_path(12, 11);
        
        maze.set_path(15, 9); maze.set_path(15, 10); maze.set_path(15, 11);
        maze.set_path(16, 9); maze.set_path(17, 9); maze.set_path(18, 9);
        maze.set_path(14, 11); maze.set_path(13, 11); maze.set_path(12, 11);
        maze.set_path(16, 11); maze.set_path(17, 11); maze.set_path(18, 11);
        
        // Lower area - complex maze-like structure
        maze.set_path(3, 21); maze.set_path(3, 22);
        maze.set_path(4, 21); maze.set_path(5, 21); maze.set_path(6, 21);
        maze.set_path(4, 22); maze.set_path(5, 22); maze.set_path(6, 22);
        maze.set_path(7, 21); maze.set_path(8, 21); // Dead end branch
        
        maze.set_path(21, 21); maze.set_path(21, 22);
        maze.set_path(20, 21); maze.set_path(19, 21); maze.set_path(18, 21);
        maze.set_path(20, 22); maze.set_path(19, 22); maze.set_path(18, 22);
        maze.set_path(17, 21); maze.set_path(16, 21); // Dead end branch
        
        // Add more complex dead ends and isolated areas
        maze.set_path(5, 5); maze.set_path(6, 5); maze.set_path(7, 5); // Long horizontal dead end
        maze.set_path(17, 5); maze.set_path(18, 5); maze.set_path(19, 5); // Another long horizontal dead end
        
        maze.set_path(5, 17); maze.set_path(6, 17); maze.set_path(7, 17); // Lower horizontal dead end
        maze.set_path(17, 17); maze.set_path(18, 17); maze.set_path(19, 17); // Another lower horizontal dead end
        
        // Add vertical dead ends
        maze.set_path(5, 5); maze.set_path(5, 6); maze.set_path(5, 7); // Left vertical dead end
        maze.set_path(19, 5); maze.set_path(19, 6); maze.set_path(19, 7); // Right vertical dead end
        
        maze.set_path(5, 17); maze.set_path(5, 18); maze.set_path(5, 19); // Lower left vertical dead end
        maze.set_path(19, 17); maze.set_path(19, 18); maze.set_path(19, 19); // Lower right vertical dead end
        
        // Add some isolated rooms and complex internal paths
        maze.set_path(11, 5); maze.set_path(12, 5); maze.set_path(13, 5);
        maze.set_path(11, 6); maze.set_path(12, 6); maze.set_path(13, 6);
        maze.set_path(11, 7); maze.set_path(12, 7); maze.set_path(13, 7);
        
        maze.set_path(11, 17); maze.set_path(12, 17); maze.set_path(13, 17);
        maze.set_path(11, 18); maze.set_path(12, 18); maze.set_path(13, 18);
        maze.set_path(11, 19); maze.set_path(12, 19); maze.set_path(13, 19);
        
        // Add more complex internal connections and dead ends
        maze.set_path(7, 7); maze.set_path(8, 7); maze.set_path(9, 7);
        maze.set_path(7, 8); maze.set_path(8, 8); maze.set_path(9, 8);
        maze.set_path(7, 9); maze.set_path(8, 9); maze.set_path(9, 9);
        
        maze.set_path(15, 7); maze.set_path(16, 7); maze.set_path(17, 7);
        maze.set_path(15, 8); maze.set_path(16, 8); maze.set_path(17, 8);
        maze.set_path(15, 9); maze.set_path(16, 9); maze.set_path(17, 9);
        
        // Add some diagonal-like paths for complexity
        maze.set_path(5, 7); maze.set_path(6, 8); maze.set_path(7, 9);
        maze.set_path(19, 7); maze.set_path(18, 8); maze.set_path(17, 9);
        
        // Add some isolated dead ends in the middle
        maze.set_path(10, 15); maze.set_path(11, 15); maze.set_path(12, 15);
        maze.set_path(10, 16); maze.set_path(11, 16); maze.set_path(12, 16);
        maze.set_path(10, 17); maze.set_path(11, 17); maze.set_path(12, 17);
        
        // Ensure there's a clear path from start to exit
        maze.set_path(1, 1);  // Start point
        maze.set_exit(23, 23);
        
        // Create a guaranteed path from start to exit
        maze.create_guaranteed_path(1, 1, 23, 23);
        
        maze
    }

    fn level3() -> Self {
        let mut maze = Maze::new(40, 40, 3);
        
        // Complex maze with many dead ends and challenging navigation
        // Use a recursive backtracking algorithm for proper maze generation
        
        // Start with all walls
        for y in 0..maze.height {
            for x in 0..maze.width {
                maze.grid[y][x] = Cell::Wall;
            }
        }
        
        // Generate maze using recursive backtracking
        Maze::generate_recursive_maze(&mut maze, 1, 1);
        
        // Ensure start and exit are accessible
        maze.set_path(1, 1);
        let exit_x = maze.width - 3;
        let exit_y = maze.height - 3;
        maze.set_exit(exit_x, exit_y);
        
        // Create a guaranteed path from start to exit
        maze.create_guaranteed_path(1, 1, exit_x, exit_y);
        
        // Add some additional complexity with extra dead ends
        Maze::add_extra_dead_ends(&mut maze);
        
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
            
            if nx > 0 && nx < maze.width - 1 && ny > 0 && ny < maze.height - 1 
               && maze.grid[ny][nx] == Cell::Wall {
                // Carve path to neighbor
                let wall_x = (x as i32 + dx / 2) as usize;
                let wall_y = (y as i32 + dy / 2) as usize;
                maze.set_path(wall_x, wall_y);
                Maze::generate_recursive_maze(maze, nx, ny);
            }
        }
    }
    
    fn add_extra_dead_ends(maze: &mut Maze) {
        // Add many more dead end paths for extra challenge and complexity
        let dead_end_positions = [
            // Long horizontal dead ends
            (5, 5), (6, 5), (7, 5), (8, 5), (9, 5), (10, 5), (11, 5),
            (15, 15), (16, 15), (17, 15), (18, 15), (19, 15), (20, 15),
            (25, 25), (26, 25), (27, 25), (28, 25), (29, 25),
            (35, 35), (36, 35), (37, 35), (38, 35),
            
            // Long vertical dead ends
            (5, 5), (5, 6), (5, 7), (5, 8), (5, 9), (5, 10),
            (15, 15), (15, 16), (15, 17), (15, 18), (15, 19),
            (25, 25), (25, 26), (25, 27), (25, 28), (25, 29),
            (35, 35), (35, 36), (35, 37), (35, 38),
            
            // Diagonal dead ends
            (10, 10), (11, 11), (12, 12), (13, 13), (14, 14),
            (30, 30), (31, 31), (32, 32), (33, 33), (34, 34),
            
            // Isolated rooms and areas
            (8, 8), (8, 9), (8, 10), (9, 8), (9, 9), (9, 10), (10, 8), (10, 9), (10, 10),
            (18, 18), (18, 19), (18, 20), (19, 18), (19, 19), (19, 20), (20, 18), (20, 19), (20, 20),
            (28, 28), (28, 29), (28, 30), (29, 28), (29, 29), (29, 30), (30, 28), (30, 29), (30, 30),
            
            // Additional scattered dead ends
            (3, 3), (3, 4), (3, 5), (4, 3), (4, 4), (4, 5),
            (13, 3), (13, 4), (13, 5), (14, 3), (14, 4), (14, 5),
            (23, 3), (23, 4), (23, 5), (24, 3), (24, 4), (24, 5),
            (33, 3), (33, 4), (33, 5), (34, 3), (34, 4), (34, 5),
            
            // Lower area dead ends
            (3, 33), (3, 34), (3, 35), (4, 33), (4, 34), (4, 35),
            (13, 33), (13, 34), (13, 35), (14, 33), (14, 34), (14, 35),
            (23, 33), (23, 34), (23, 35), (24, 33), (24, 34), (24, 35),
            (33, 33), (33, 34), (33, 35), (34, 33), (34, 34), (34, 35),
        ];
        
        for &(x, y) in &dead_end_positions {
            if x < maze.width && y < maze.height {
                maze.set_path(x, y);
            }
        }
    }
    
    /// Creates a guaranteed path from start to exit with twists and turns
    fn create_guaranteed_path(&mut self, start_x: usize, start_y: usize, exit_x: usize, exit_y: usize) {
        // Create a winding path with some randomness to make it more interesting
        // This ensures there's always a way to win but not in a straight line
        
        let mut x = start_x;
        let mut y = start_y;
        
        // Create a zigzag path with some randomness
        let mut direction: usize = 0; // 0: right, 1: down, 2: left, 3: up
        
        while x != exit_x || y != exit_y {
            let old_x = x;
            let old_y = y;
            
            match direction {
                0 => { // Right
                    if x < exit_x && x < self.width - 1 {
                        x += 1;
                    } else {
                        direction = 1; // Switch to down
                        continue;
                    }
                }
                1 => { // Down
                    if y < exit_y && y < self.height - 1 {
                        y += 1;
                    } else {
                        direction = 2; // Switch to left
                        continue;
                    }
                }
                2 => { // Left
                    if x > exit_x && x > 0 {
                        x -= 1;
                    } else {
                        direction = 3; // Switch to up
                        continue;
                    }
                }
                3 => { // Up
                    if y > exit_y && y > 0 {
                        y -= 1;
                    } else {
                        direction = 0; // Switch to right
                        continue;
                    }
                }
                _ => unreachable!(), // This should never happen with % 4
            }
            
            // Set the path if it's a wall
            if self.grid[y][x] == Cell::Wall {
                self.set_path(x, y);
            }
            
            // Occasionally change direction to create more winding paths
            if (x + y) % 7 == 0 {
                direction = (direction + 1) % 4;
            }
            
            // If we're stuck, force progress toward exit
            if x == old_x && y == old_y {
                if x < exit_x { x += 1; }
                if y < exit_y { y += 1; }
                if x > exit_x { x -= 1; }
                if y > exit_y { y -= 1; }
                
                if self.grid[y][x] == Cell::Wall {
                    self.set_path(x, y);
                }
            }
        }
    }

    pub fn spawn_points(&self, count: usize) -> Vec<(usize, usize)> {
        let mut pts = Vec::with_capacity(count);
        'outer: for y in 0..self.height {
            for x in 0..self.width {
                if self.grid[y][x] == Cell::Path {
                    pts.push((x, y));
                    if pts.len() == count { break 'outer; }
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
                if self.grid[y][x] == Cell::Path {
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
                if self.grid[y][x] == Cell::Path {
                    count += 1;
                }
            }
        }
        count
    }

    // Test function to verify maze can handle multiple players
    pub fn test_multiplayer_support(&self) {
        let total_walkable = self.total_walkable_cells();
        let can_handle_10 = self.has_enough_spawns(10);
        let spawn_points_10 = self.spawn_points(10);
        
        println!("=== Multiplayer Support Test ===");
        println!("Total walkable cells: {}", total_walkable);
        println!("Can handle 10 players: {}", can_handle_10);
        println!("Available spawn points for 10 players: {}", spawn_points_10.len());
        println!("First 5 spawn points: {:?}", &spawn_points_10[..spawn_points_10.len().min(5)]);
        println!("================================");
    }
}
/////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////      Maze struct      ///////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////      Payer Logic    ///////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Player {
   pub x: usize,
   pub y: usize,
}

impl Player {
    pub fn new(x: usize, y: usize) -> Self {
        Player { x, y }
    }

    // Player movement mechanics with collision detection
    pub fn move_up(&mut self, maze: &Maze) {
        if self.y > 0 && maze.is_walkable(self.x, self.y - 1) {
            self.y -= 1;
            // Moves the player up
        }
    }

    pub fn move_down(&mut self, maze: &Maze) {
        if self.y + 1 < maze.height && maze.is_walkable(self.x, self.y + 1) {
            self.y += 1;
            // Moves the player down
        }
    }

    pub fn move_left(&mut self, maze: &Maze) {
        if self.x > 0 && maze.is_walkable(self.x - 1, self.y) {
            self.x -= 1;
            // Moves the player left
        }
    }

    pub fn move_right(&mut self, maze: &Maze) {
        if self.x + 1 < maze.width && maze.is_walkable(self.x + 1, self.y) {
            self.x += 1;    
            // Moves the player right
        }
    }

    ///////////////////////////////////////      Victorias    ///////////////////////////////////////\

    pub fn at_exit(&self, maze: &Maze) -> bool {
        maze.grid[self.y][self.x] == Cell::Exit
    }
}
/////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////      Payer Logic    ///////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////













