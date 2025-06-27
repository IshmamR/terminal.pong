use ratatui::layout::Rect;

use crate::game::PLAYER_NAME_SIZE;

pub fn centered_rect_with_percentage(percent_x: u16, percent_y: u16, cols: u16, rows: u16) -> Rect {
    let width = cols * percent_x / 100;
    let height = std::cmp::min(std::cmp::max(rows * percent_y / 100, 5), rows);
    Rect::new((cols - width) / 2, (rows - height) / 2, width, height)
}

pub fn centered_rect(width: u16, height: u16, cols: u16, rows: u16) -> Rect {
    let x = (cols - width) / 2;
    let y = (rows - height) / 2;
    Rect::new(x, y, width, height)
}

pub fn string_to_char_array(s: &str) -> [char; PLAYER_NAME_SIZE] {
    let mut chars = s.chars().collect::<Vec<char>>(); // Collect the string into a vector of chars
    chars.resize(PLAYER_NAME_SIZE, ' '); // Pad with spaces if shorter than 16
    let mut array = [' '; PLAYER_NAME_SIZE]; // Initialize an empty array
    array.copy_from_slice(&chars[0..PLAYER_NAME_SIZE]); // Copy the first 16 characters
    array
}
