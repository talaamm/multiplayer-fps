use macroquad::prelude::*;
use crate::player::PlayerSkin;

// ---------- Level Selection UI ----------
pub fn draw_level_selection(levels: &[(u8, String, String, u8)], selected_level: &mut usize, selected_skin: &mut PlayerSkin, selection_mode: usize) {
    clear_background(BLACK);
    
    let title = "Select Your Map & Skin";
    let tw = measure_text(title, None, 32, 1.0);
    draw_text(title, (screen_width() - tw.width) * 0.5, 30.0, 32.0, WHITE);
    
    // Draw levels on the left side
    let levels_title = if selection_mode == 0 { "Maps: [SELECTING]" } else { "Maps:" };
    let levels_color = if selection_mode == 0 { YELLOW } else { WHITE };
    draw_text(levels_title, 50.0, 80.0, 24.0, levels_color);
    
    let start_y = 120.0;
    let item_height = 60.0;
    
    for (i, (level_id, name, description, max_players)) in levels.iter().enumerate() {
        let y = start_y + i as f32 * item_height;
        let is_selected = i == *selected_level;
        
        // Background
        let bg_color = if is_selected { Color::from_rgba(100, 100, 200, 255) } else { Color::from_rgba(50, 50, 50, 255) };
        draw_rectangle(50.0, y, screen_width() * 0.4, item_height - 5.0, bg_color);
        
        // Level info
        let level_txt = format!("Level {}: {}", level_id, name);
        draw_text(&level_txt, 70.0, y + 10.0, 18.0, WHITE);
        
        let desc_txt = format!("{} (Max {} players)", description, max_players);
        draw_text(&desc_txt, 70.0, y + 30.0, 12.0, GRAY);
        
        // Selection indicator
        if is_selected {
            draw_text(">", 30.0, y + 20.0, 18.0, YELLOW);
        }
    }
    
    // Draw skins on the right side
    let skins_title = if selection_mode == 1 { "Skins: [SELECTING]" } else { "Skins:" };
    let skins_color = if selection_mode == 1 { YELLOW } else { WHITE };
    draw_text(skins_title, screen_width() * 0.5 + 50.0, 80.0, 24.0, skins_color);
    
    let skins = [
        PlayerSkin::Soldier,
        PlayerSkin::Sniper,
        PlayerSkin::Heavy,
        PlayerSkin::Scout,
        PlayerSkin::Medic,
        PlayerSkin::Engineer,
    ];
    
    let skin_names = [
        "Soldier",
        "Sniper", 
        "Heavy",
        "Scout",
        "Medic",
        "Engineer",
    ];
    
    for (i, (skin, name)) in skins.iter().zip(skin_names.iter()).enumerate() {
        let y = start_y + i as f32 * item_height;
        let is_selected = *skin == *selected_skin;
        
        // Background
        let bg_color = if is_selected { Color::from_rgba(100, 100, 200, 255) } else { Color::from_rgba(50, 50, 50, 255) };
        draw_rectangle(screen_width() * 0.5 + 50.0, y, screen_width() * 0.4, item_height - 5.0, bg_color);
        
        // Skin preview (small colored rectangle)
        let preview_x = screen_width() * 0.5 + 70.0;
        let preview_y = y + 10.0;
        let preview_size = 20.0;
        draw_rectangle(preview_x, preview_y, preview_size, preview_size, skin.get_body_color());
        draw_rectangle(preview_x + 2.0, preview_y + 2.0, preview_size - 4.0, preview_size - 4.0, skin.get_armor_color());
        
        // Skin name
        draw_text(name, preview_x + preview_size + 10.0, y + 15.0, 18.0, WHITE);
        
        // Selection indicator
        if is_selected {
            draw_text(">", screen_width() * 0.5 + 30.0, y + 20.0, 18.0, YELLOW);
        }
    }
    
    let hint = "Tab to switch between maps/skins, Up/Down to select, Enter to confirm";
    let hint_tw = measure_text(hint, None, 16, 1.0);
    draw_text(hint, (screen_width() - hint_tw.width) * 0.5, screen_height() - 30.0, 16.0, GRAY);
}

// ---------- Connection Screen ----------
pub fn draw_connection_screen(server_addr: &str, username: &str, input_focus: usize) {
    // Render a simple input form
    let title = "Connect to Maze War FPS Server";
    let tw = measure_text(title, None, 32, 1.0);
    draw_text(title, (screen_width() - tw.width) * 0.5, 120.0, 32.0, WHITE);

    let label1 = "Server (IP:PORT):";
    draw_text(label1, 200.0, 200.0, 24.0, GRAY);
    let label2 = "Username:";
    draw_text(label2, 200.0, 260.0, 24.0, GRAY);

    // Input boxes
    let bx = 380.0;
    let bw = screen_width() - bx - 200.0;
    let bh = 32.0;
    let by1 = 175.0;
    let by2 = 235.0;
    draw_rectangle_lines(
        bx - 4.0,
        by1 - 24.0,
        bw + 8.0,
        bh + 8.0,
        2.0,
        if input_focus == 0 { YELLOW } else { DARKGRAY },
    );
    draw_rectangle_lines(
        bx - 4.0,
        by2 - 24.0,
        bw + 8.0,
        bh + 8.0,
        2.0,
        if input_focus == 1 { YELLOW } else { DARKGRAY },
    );
    draw_text(server_addr, bx, by1, 28.0, WHITE);
    draw_text(username, bx, by2, 28.0, WHITE);

    let hint = "Tab switch, Enter connect";
    draw_text(hint, bx, by2 + 40.0, 20.0, GRAY);
}
