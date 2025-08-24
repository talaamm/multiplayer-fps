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
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////      Maze struct      ///////////////////////////////////////
/// /////////////////////////////////////////////////////////////////////////////////////////////////
impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        // Start with all walls
        let grid = vec![vec![Cell::Wall; width]; height];
        Maze { width, height, grid }
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
        let mut maze = Maze::new(15, 15);
        
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
        
        maze.set_exit(13, 13);
        maze
    }

    fn level2() -> Self {
        let mut maze = Maze::new(25, 25);
        
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
        
        // Horizontal main corridors
        for x in 1..24 {
            maze.set_path(x, 1);   // Top
            maze.set_path(x, 7);   // Upper middle
            maze.set_path(x, 13);  // Center
            maze.set_path(x, 19);  // Lower middle
            maze.set_path(x, 23);  // Bottom
        }
        
        // Vertical main corridors
        for y in 1..24 {
            maze.set_path(1, y);   // Left
            maze.set_path(7, y);   // Left middle
            maze.set_path(13, y);  // Center
            maze.set_path(19, y);  // Right middle
            maze.set_path(23, y);  // Right
        }
        
        // Add branching paths with dead ends
        // Upper left area
        maze.set_path(3, 3); maze.set_path(3, 4); maze.set_path(3, 5);
        maze.set_path(4, 3); maze.set_path(5, 3); // Dead end branch
        
        // Upper right area
        maze.set_path(21, 3); maze.set_path(21, 4); maze.set_path(21, 5);
        maze.set_path(20, 5); maze.set_path(19, 5); // Dead end branch
        
        // Center area with multiple branches
        maze.set_path(9, 9); maze.set_path(9, 10); maze.set_path(9, 11);
        maze.set_path(10, 9); maze.set_path(11, 9); // Dead end
        maze.set_path(8, 11); maze.set_path(7, 11); // Dead end
        
        maze.set_path(15, 9); maze.set_path(15, 10); maze.set_path(15, 11);
        maze.set_path(16, 9); maze.set_path(17, 9); // Dead end
        maze.set_path(14, 11); maze.set_path(13, 11); // Dead end
        
        // Lower area
        maze.set_path(3, 21); maze.set_path(3, 22);
        maze.set_path(4, 21); maze.set_path(5, 21); // Dead end
        
        maze.set_path(21, 21); maze.set_path(21, 22);
        maze.set_path(20, 21); maze.set_path(19, 21); // Dead end
        
        maze.set_exit(23, 23);
        maze
    }

    fn level3() -> Self {
        let mut maze = Maze::new(40, 40);
        
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
        maze.set_exit(maze.width - 2, maze.height - 2);
        
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
        // Add some additional dead end paths for extra challenge
        let dead_end_positions = [
            (5, 5), (6, 5), (7, 5), (8, 5), (9, 5), // Long dead end
            (15, 15), (16, 15), (17, 15), (18, 15), // Another long dead end
            (25, 25), (26, 25), (27, 25), // Medium dead end
            (35, 35), (36, 35), // Short dead end
            (10, 10), (11, 10), (12, 10), // Center dead end
            (30, 30), (31, 30), (32, 30), // Lower right dead end
        ];
        
        for &(x, y) in &dead_end_positions {
            if x < maze.width && y < maze.height {
                maze.set_path(x, y);
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













