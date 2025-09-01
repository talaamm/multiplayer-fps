use macroquad::prelude::*;

// ---------- Player Skins ----------
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerSkin {
    Soldier,    // Green military uniform
    Sniper,     // Dark blue/black tactical gear
    Heavy,      // Red/brown heavy armor
    Scout,      // Light blue/yellow fast gear
    Medic,      // White/red medical uniform
    Engineer,   // Orange/yellow construction gear
}

impl PlayerSkin {
    pub fn get_body_color(&self) -> Color {
        match self {
            PlayerSkin::Soldier => Color::from_rgba(34, 139, 34, 255),    // Forest green
            PlayerSkin::Sniper => Color::from_rgba(25, 25, 112, 255),     // Midnight blue
            PlayerSkin::Heavy => Color::from_rgba(139, 69, 19, 255),      // Saddle brown
            PlayerSkin::Scout => Color::from_rgba(30, 144, 255, 255),     // Dodger blue
            PlayerSkin::Medic => Color::from_rgba(220, 20, 60, 255),      // Crimson
            PlayerSkin::Engineer => Color::from_rgba(255, 140, 0, 255),   // Dark orange
        }
    }
    
    pub fn get_head_color(&self) -> Color {
        match self {
            PlayerSkin::Soldier => Color::from_rgba(210, 180, 140, 255),  // Tan
            PlayerSkin::Sniper => Color::from_rgba(105, 105, 105, 255),   // Dim gray
            PlayerSkin::Heavy => Color::from_rgba(160, 82, 45, 255),      // Sienna
            PlayerSkin::Scout => Color::from_rgba(255, 215, 0, 255),      // Gold
            PlayerSkin::Medic => Color::from_rgba(255, 255, 255, 255),    // White
            PlayerSkin::Engineer => Color::from_rgba(255, 255, 0, 255),   // Yellow
        }
    }
    
    pub fn get_helmet_color(&self) -> Color {
        match self {
            PlayerSkin::Soldier => Color::from_rgba(85, 107, 47, 255),    // Dark olive green
            PlayerSkin::Sniper => Color::from_rgba(47, 79, 79, 255),      // Dark slate gray
            PlayerSkin::Heavy => Color::from_rgba(128, 0, 0, 255),        // Maroon
            PlayerSkin::Scout => Color::from_rgba(0, 100, 0, 255),        // Dark green
            PlayerSkin::Medic => Color::from_rgba(178, 34, 34, 255),      // Fire brick
            PlayerSkin::Engineer => Color::from_rgba(255, 69, 0, 255),    // Orange red
        }
    }
    
    pub fn get_armor_color(&self) -> Color {
        match self {
            PlayerSkin::Soldier => Color::from_rgba(0, 100, 0, 255),      // Dark green
            PlayerSkin::Sniper => Color::from_rgba(25, 25, 25, 255),      // Very dark gray
            PlayerSkin::Heavy => Color::from_rgba(101, 67, 33, 255),      // Dark brown
            PlayerSkin::Scout => Color::from_rgba(0, 191, 255, 255),      // Deep sky blue
            PlayerSkin::Medic => Color::from_rgba(255, 0, 0, 255),        // Red
            PlayerSkin::Engineer => Color::from_rgba(255, 165, 0, 255),   // Orange
        }
    }
    
    pub fn from_id(id: u64) -> Self {
        match id % 6 {
            0 => PlayerSkin::Soldier,
            1 => PlayerSkin::Sniper,
            2 => PlayerSkin::Heavy,
            3 => PlayerSkin::Scout,
            4 => PlayerSkin::Medic,
            5 => PlayerSkin::Engineer,
            _ => PlayerSkin::Soldier,
        }
    }
}

// ---------- Player ----------
#[derive(Clone, Copy)]
pub struct Player {
    pub pos: Vec2,
    pub dir: f32, // radians
    pub health: u8,
    pub ammo: u8,
    pub kills: u32,
    pub deaths: u32,
    pub skin: PlayerSkin,
}

impl Player {
    pub fn new(x: f32, y: f32, dir: f32) -> Self {
        Self {
            pos: vec2(x, y),
            dir,
            health: 100,
            ammo: 30,
            kills: 0,
            deaths: 0,
            skin: PlayerSkin::Soldier, // Default skin
        }
    }
}

// ---------- Remote players ----------
#[derive(Clone, Debug)]
pub struct RemotePlayer {
    pub pos: Vec2,
    pub angle: f32,
    pub name: String,
    pub health: u8,
    pub ammo: u8,
    pub kills: u32,
    pub deaths: u32,
    pub skin: PlayerSkin,
}

// ---------- Player Rendering with Skins ----------
pub fn draw_player_with_skin(x: f32, y: f32, width: f32, height: f32, skin: PlayerSkin, angle: f32, screen_height: f32, depth: f32) {
    // Body (torso)
    let body_y = y + height * 0.4;
    let body_height = height * 0.6;
    draw_rectangle(
        x - width * 0.4,
        body_y,
        width * 0.8,
        body_height,
        skin.get_body_color(),
    );
    
    // Armor vest (chest piece)
    let vest_y = y + height * 0.45;
    let vest_height = height * 0.3;
    draw_rectangle(
        x - width * 0.3,
        vest_y,
        width * 0.6,
        vest_height,
        skin.get_armor_color(),
    );
    
    // Head
    let head_y = y + height * 0.15;
    let head_r = width * 0.25;
    draw_circle(x, head_y, head_r, skin.get_head_color());
    
    // Helmet
    let helmet_y = y + height * 0.1;
    let helmet_r = width * 0.28;
    draw_circle(x, helmet_y, helmet_r, skin.get_helmet_color());
    
    // Eyes (small white dots)
    let eye_y = head_y - head_r * 0.3;
    let eye_r = head_r * 0.15;
    draw_circle(x - head_r * 0.3, eye_y, eye_r, WHITE);
    draw_circle(x + head_r * 0.3, eye_y, eye_r, WHITE);
    
    // Arms
    let arm_width = width * 0.15;
    let arm_height = height * 0.4;
    let arm_y = y + height * 0.45;
    
    // Left arm
    draw_rectangle(
        x - width * 0.6,
        arm_y,
        arm_width,
        arm_height,
        skin.get_body_color(),
    );
    
    // Right arm
    draw_rectangle(
        x + width * 0.45,
        arm_y,
        arm_width,
        arm_height,
        skin.get_body_color(),
    );
    
    // Legs
    let leg_width = width * 0.2;
    let leg_height = height * 0.5;
    let leg_y = y + height * 0.9;
    
    // Left leg
    draw_rectangle(
        x - width * 0.35,
        leg_y,
        leg_width,
        leg_height,
        skin.get_body_color(),
    );
    
    // Right leg
    draw_rectangle(
        x + width * 0.15,
        leg_y,
        leg_width,
        leg_height,
        skin.get_body_color(),
    );
    
    // Weapon (gun)
    let weapon_length = width * 0.8;
    let weapon_width = width * 0.08;
    let weapon_y = y + height * 0.5;
    let weapon_x = x + width * 0.5;
    
    // Gun barrel
    draw_rectangle(
        weapon_x,
        weapon_y - weapon_width * 0.5,
        weapon_length,
        weapon_width,
        Color::from_rgba(64, 64, 64, 255), // Dark gray
    );
    
    // Gun handle
    draw_rectangle(
        weapon_x + weapon_length * 0.7,
        weapon_y + weapon_width * 0.5,
        weapon_width * 1.5,
        weapon_width * 2.0,
        Color::from_rgba(139, 69, 19, 255), // Brown
    );
    
    // Facing direction indicator (small arrow)
    let arrow_length = (screen_height / depth) * 0.1;
    let ax = angle.cos();
    let ay = angle.sin();
    let arrow_x = x;
    let arrow_y = y + height * 0.7;
    draw_line(
        arrow_x,
        arrow_y,
        arrow_x + ax as f32 * arrow_length,
        arrow_y - ay as f32 * arrow_length,
        2.0,
        YELLOW,
    );
}
