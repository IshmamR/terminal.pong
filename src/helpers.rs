use ratatui::layout::Rect;

use crate::types::PLAYER_NAME_SIZE;

pub fn generate_direction() -> i16 {
    let random_number: i16 = rand::random_range(0..=1);
    if random_number == 0 {
        1
    } else {
        -1
    }
}

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

pub fn predict_ball_y_at_paddle(
    ball_y: f32,
    ball_vy: f32,
    ball_x: f32,
    ball_vx: f32,
    paddle_x: f32,
    height: f32,
) -> f32 {
    if ball_vx <= 0.0 {
        return ball_y;
    }

    let time_to_paddle = (paddle_x - ball_x) / ball_vx;
    let mut predicted_y = ball_y + ball_vy * time_to_paddle;

    while predicted_y < 0.0 || predicted_y > height {
        if predicted_y < 0.0 {
            predicted_y = -predicted_y;
        } else {
            predicted_y = 2.0 * height - predicted_y;
        }
    }

    predicted_y
}
