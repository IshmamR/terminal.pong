use std::time::Instant;

use ratatui::layout::Rect;

pub const PLAYER_NAME_SIZE: usize = 16;

#[derive(Debug, Default)]
pub struct Player {
    pub(crate) name: [char; PLAYER_NAME_SIZE],
    pub(crate) bar_position: u16,
}

#[derive(Debug, Default)]
pub struct Ball {
    pub(crate) position: [u16; 2],
    pub(crate) velocity: [i16; 2],
    pub(crate) powered: bool,
}

#[derive(Debug)]
pub struct ComputerAI {
    pub(crate) reaction_delay: f32, // Time before reacting to ball direction change
    pub(crate) last_ball_direction: i16, // Track ball direction changes
    pub(crate) reaction_timer: f32, // Current reaction delay timer
    pub(crate) prediction_error: f32, // How far off the prediction can be
    pub(crate) max_speed: f32,      // Maximum movement speed
    pub(crate) current_speed: f32,  // Current movement speed (with acceleration)
    pub(crate) target_position: f32, // Where the AI wants to move
    // pub(crate) difficulty: f32,     // 0.0 to 1.0, affects all parameters
    pub(crate) fatigue: f32,        // Increases over time, affects performance
    pub(crate) last_update: Instant, // For delta time calculations
}

#[derive(Debug)]
pub struct GameState {
    pub(crate) players: [Player; 2],
    pub(crate) points: [i64; 2],
    pub(crate) ball: Ball,
    pub(crate) last_update: Instant,
    pub(crate) game_area: Rect,
    // pub(crate) is_paused: bool,
    pub(crate) computer_ai: ComputerAI,
}
