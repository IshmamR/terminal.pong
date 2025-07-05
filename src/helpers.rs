use ratatui::layout::Rect;

use crate::game::PLAYER_NAME_CHAR_LEN;

pub fn centered_rect_with_percentage(percent_x: u16, percent_y: u16, cols: u16, rows: u16) -> Rect {
    let width = cols * percent_x / 100;
    let height = std::cmp::min(std::cmp::max(rows * percent_y / 100, 5), rows);
    Rect::new((cols - width) / 2, (rows - height) / 2, width, height)
}

pub fn centered_rect(width: u16, height: u16, cols: u16, rows: u16) -> Rect {
    // Ensure we don't try to create a rect larger than available space
    let actual_width = std::cmp::min(width, cols);
    let actual_height = std::cmp::min(height, rows);
    
    // Safely calculate center position, avoiding underflow
    let x = if cols >= actual_width {
        (cols - actual_width) / 2
    } else {
        0
    };
    let y = if rows >= actual_height {
        (rows - actual_height) / 2
    } else {
        0
    };
    
    Rect::new(x, y, actual_width, actual_height)
}

pub fn string_to_char_array(s: &str) -> [char; PLAYER_NAME_CHAR_LEN] {
    let mut chars = s.chars().collect::<Vec<char>>(); // Collect the string into a vector of chars
    chars.resize(PLAYER_NAME_CHAR_LEN, ' '); // Pad with spaces if shorter than 16
    let mut array = [' '; PLAYER_NAME_CHAR_LEN]; // Initialize an empty array
    array.copy_from_slice(&chars[0..PLAYER_NAME_CHAR_LEN]); // Copy the first 16 characters
    array
}
