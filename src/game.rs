use rand::random;
use std::time::{Duration, Instant};

use ratatui::{layout::Rect, style::Color};

use crate::helpers::string_to_char_array;

pub const PLAYER_NAME_SIZE: usize = 16;
const DEFAULT_BAR_LENGTH: u8 = 5;
const DEFAULT_BALL_VELOCITY_X: i8 = 3;
const DEFAULT_BALL_VELOCITY_Y: i8 = 1;
const DEFAULT_PADDLE_WIDTH: u16 = 3;
const STARTING_POWER_MOVES: u8 = 10;
const DEFAULT_DIFFICULTY: f32 = 0.5;

#[derive(Debug, Clone, Copy)]
struct ComputerAI {
    pub(crate) reaction_delay: f32, // Time before reacting to ball direction change
    pub(crate) last_ball_direction: i8, // Track ball direction changes
    pub(crate) reaction_timer: f32, // Current reaction delay timer
    pub(crate) prediction_error: f32, // How far off the prediction can be
    pub(crate) max_speed: f32,      // Maximum movement speed
    pub(crate) current_speed: f32,  // Current movement speed (with acceleration)
    pub(crate) target_position: f32, // Where the AI wants to move
    pub(crate) difficulty: f32,     // 0.0 to 1.0, affects all parameters
    pub(crate) fatigue: f32,        // Increases over time, affects performance
    pub(crate) last_update: Instant, // For delta time calculations
}

#[derive(Debug, Default)]
pub struct Player {
    pub(crate) name: [char; PLAYER_NAME_SIZE],
    pub(crate) score: u32,

    pub(crate) power_moves_left: u8,
    pub(crate) last_power_used_at: Option<Instant>,

    pub(crate) bar_position: u16,
    pub(crate) bar_length: u8,
    pub(crate) bar_color: Color,

    pub(crate) is_ready: bool,

    pub(crate) is_computer: bool,
    computer_ai: Option<ComputerAI>,
}

#[derive(Debug, Default)]
pub struct Ball {
    pub(crate) position: [u16; 2],
    pub(crate) velocity: [i8; 2],
    pub(crate) is_powered: bool,
    pub(crate) color: Color,
}

#[derive(PartialEq)]
pub enum GameType {
    AgainstAi,
    ScreenSaver,
}

#[derive(Debug)]
pub struct Game {
    players: [Player; 2],
    ball: Ball,
    game_area: Rect,
    started_at: Option<Instant>,
    last_update: Instant,
    is_paused: bool,
}

impl Game {
    pub fn new(player_names: [&str; 2], game_area: Rect, game_type: GameType) -> Self {
        let ai_player = ComputerAI {
            reaction_delay: 0.2 + (1.0 - DEFAULT_DIFFICULTY) * 0.5, // 0.2-0.7 seconds
            last_ball_direction: 0,
            reaction_timer: 0.0,
            prediction_error: 2.0 + (1.0 - DEFAULT_DIFFICULTY) * 3.0, // 2-5 units error
            max_speed: 0.8 + DEFAULT_DIFFICULTY * 0.7,                // 0.8-1.5 speed
            current_speed: 0.0,
            target_position: 0.0,
            difficulty: DEFAULT_DIFFICULTY,
            fatigue: 0.0,
            last_update: Instant::now(),
        };

        let player1 = Player {
            name: string_to_char_array(player_names[0]),
            bar_position: (game_area.height / 2).saturating_sub((DEFAULT_BAR_LENGTH / 2) as u16),
            bar_color: Color::Cyan,
            bar_length: DEFAULT_BAR_LENGTH,
            is_computer: false,
            computer_ai: if game_type == GameType::ScreenSaver {
                Some(ai_player.clone())
            } else {
                None
            },
            is_ready: false,
            power_moves_left: STARTING_POWER_MOVES,
            last_power_used_at: None,
            score: 0,
        };

        let player2 = Player {
            name: string_to_char_array(player_names[1]),
            bar_position: (game_area.height / 2).saturating_sub((DEFAULT_BAR_LENGTH / 2) as u16),
            bar_color: Color::Cyan,
            bar_length: DEFAULT_BAR_LENGTH,
            is_computer: false,
            computer_ai: if game_type == GameType::AgainstAi || game_type == GameType::ScreenSaver {
                Some(ai_player.clone())
            } else {
                None
            },
            is_ready: true,
            power_moves_left: STARTING_POWER_MOVES,
            last_power_used_at: None,
            score: 0,
        };

        Self {
            players: [player1, player2],
            ball: Ball {
                position: [70, 14],
                velocity: [DEFAULT_BALL_VELOCITY_X, DEFAULT_BALL_VELOCITY_Y],
                is_powered: false,
                color: Color::LightRed,
            },
            last_update: Instant::now(),
            game_area: game_area,
            is_paused: true,
            started_at: None,
        }
    }

    pub fn get_area(&self) -> Rect {
        self.game_area
    }

    pub fn set_area(&mut self, game_area: Rect) {
        self.game_area = game_area;
    }

    pub fn get_player(&self, index: usize) -> &Player {
        &self.players[index]
    }

    pub fn get_ball(&self) -> &Ball {
        &self.ball
    }

    pub fn ready_player(&mut self, index: usize) {
        self.players[index].is_ready = true;
    }

    pub fn move_player(&mut self, player_index: usize, direction: i8) {
        if direction == 0 {
            return;
        }

        let player = &mut self.players[player_index];

        if direction > 0 {
            // up
            if player.bar_position > 0 {
                player.bar_position -= 1;
            }
        } else {
            // down
            let inner_height = self.game_area.height - 2;
            if player.bar_position + (player.bar_length as u16) < inner_height {
                player.bar_position += 1;
            }
        }
    }

    /**
     * 1 -> ball collision with Player 1's bar
     * 2 -> ball collision with Player 2's bar
     * None -> no collision, ball position updated normally
     */
    fn update_ball_position(&mut self) -> Option<u8> {
        let inner_width = self.game_area.width - 2;
        let inner_height = self.game_area.height - 2;

        let players = &self.players;
        let ball = &mut self.ball;

        let new_x = ball.position[0].saturating_add_signed(ball.velocity[0] as i16);
        let new_y = ball.position[1].saturating_add_signed(ball.velocity[1] as i16);

        // collision with top and bottom walls
        if new_y == 0 || new_y >= inner_height - 1 {
            ball.velocity[1] = -ball.velocity[1];
            ball.position[1] = if new_y == 0 { 0 } else { inner_height - 1 };
        } else {
            ball.position[1] = new_y;
        }

        // ball collision with Player 1's bar (left side)
        if new_x <= 3 && ball.velocity[0] < 0 {
            if new_y >= players[0].bar_position && new_y < players[0].bar_position + 5 {
                ball.velocity[0] = -ball.velocity[0];
                ball.position[0] = 5;
                return Some(1);
            }
        }

        // ball collision with Player 2's bar (right side)
        if new_x >= inner_width - 4 && ball.velocity[0] > 0 {
            if new_y >= players[1].bar_position && new_y < players[1].bar_position + 5 {
                ball.velocity[0] = -DEFAULT_BALL_VELOCITY_X;
                // ball.velocity[0] = -ball.velocity[0];
                ball.position[0] = inner_width - 5;
                ball.is_powered = false;
                return Some(2);
            }
        }

        // ball went off screen (reset)
        if new_x <= 0 || new_x >= inner_width {
            // Ball exited the screen: left or right
            if new_x <= 0 {
                // ball exited on the left → player missed → computer scores
                self.players[1].score += 1;
            } else {
                // ball exited on the right → computer missed → player scores
                self.players[0].score += 1;
            }

            // reset ball to center
            ball.position = [inner_width / 2, inner_height / 2];

            let random_number: i16 = rand::random_range(0..=1);
            let direction = if random_number == 0 { 1 } else { -1 };

            ball.velocity[0] = direction * 3;
            ball.is_powered = false;
        } else {
            ball.position[0] = new_x;
        }

        None
    }

    fn update_computer_player(&mut self, player_index: usize) {
        let computer = &mut self.players[player_index];
        let ball = &self.ball;

        if computer.computer_ai.is_none() {
            return;
        }
        let ai = computer.computer_ai.as_mut().unwrap();

        let inner_height = self.game_area.height - 2;
        let paddle_x = if player_index == 0 {
            DEFAULT_PADDLE_WIDTH // Player 1's paddle is on the left
        } else {
            self.game_area.width - DEFAULT_PADDLE_WIDTH // Player 2's paddle is on the right
        };

        // accumulate delta time to add fatigue
        let dt = ai.last_update.elapsed().as_secs_f32();
        ai.last_update = Instant::now();
        // increase fatigue over time
        ai.fatigue = (ai.fatigue + dt * 0.009).min(0.3);

        // now, let's calculate the direction the ball is moving towards
        let ball_direction_x: i8 = ball.velocity[0].signum(); // one of these 3 -> { -1, 0, 1 }

        // if ball direction changed, add a reaction timer
        if ball_direction_x != ai.last_ball_direction && ball_direction_x != 0 {
            ai.last_ball_direction = ball_direction_x;
            ai.reaction_timer = ai.reaction_delay + ai.fatigue * 0.5;
        }

        ai.reaction_timer = (ai.reaction_timer - dt).max(0.0);

        // check if ball is coming towards computer paddle
        let is_ball_coming = if player_index == 0 {
            ball.velocity[0] < 0 // Player 1's paddle is on the left
        } else {
            ball.velocity[0] > 0 // Player 2's paddle is on the right
        };

        let paddle_center = computer.bar_position as f32 + computer.bar_length as f32 / 2.0;

        if !is_ball_coming || ai.reaction_timer > 0.0 {
            // neutral positioning
            // slowly drift towards the center

            let center_y = inner_height / 2;
            ai.target_position = paddle_center + (center_y as f32 - paddle_center) * 0.1;
        } else {
            // active/predictive positioning
            // "predict" ball position with wall bounces

            let time_to_paddle_x =
                (paddle_x as f32 - ball.position[0] as f32) / ball.velocity[0] as f32;
            let mut pred_y = ball.position[1] as f32 + ball.velocity[1] as f32 * time_to_paddle_x;

            // simulate top and bottom wall bounces
            while pred_y < 0.0 || pred_y > inner_height as f32 {
                if pred_y < 0.0 {
                    pred_y = -pred_y; // bounce off top wall
                } else {
                    pred_y = 2.0 * inner_height as f32 - pred_y; // bounce off bottom wall
                }
            }

            // sprinkle some prediction errors -,-
            let error_magnitude = ai.prediction_error * (1.0 + ai.fatigue);
            let prediction_error = (random::<f32>() - 0.5) * error_magnitude;
            pred_y += prediction_error;

            // make big oopsies occasionally (5% chance)
            if random::<f32>() < 0.05 + ai.fatigue * 0.1 {
                pred_y += (random::<f32>() - 0.5) * 8.0;
            }

            // clamp to fix
            pred_y = pred_y.clamp(0.0, (inner_height - computer.bar_length as u16) as f32);

            // add some final randomness (10% chance)
            if random::<f32>() < 0.1 {
                pred_y += (random::<f32>() - 0.5) * 2.0;
            }

            ai.target_position = pred_y;
        }

        // smooth movement with acceleration
        let distance_to_target = ai.target_position - paddle_center;
        let desired_speed = distance_to_target.abs().min(ai.max_speed);
        let acceleration = 2.0;
        if distance_to_target.abs() > 0.5 {
            ai.current_speed = (ai.current_speed + acceleration * dt).min(desired_speed);
        } else {
            ai.current_speed = (ai.current_speed - acceleration * dt * 2.0).max(0.0);
        }

        // add some jitter and behavioral quirks
        let jitter = (random::<f32>() - 0.5) * 0.1 * (1.0 + ai.fatigue);
        let movement = distance_to_target.signum() * ai.current_speed + jitter;

        let final_movement = if random::<f32>() < 0.02 + ai.fatigue * 0.05 {
            movement * 0.3 // hesitation (2% chance)
        } else if random::<f32>() < 0.03 {
            movement * 1.4 // overshoot (3% chance)
        } else {
            movement
        };

        // apply new position with clamping
        let new_pos = (paddle_center + final_movement).clamp(
            computer.bar_length as f32 / 2.0,
            inner_height as f32 - computer.bar_length as f32 / 2.0,
        );

        computer.bar_position = (new_pos - computer.bar_length as f32 / 2.0) as u16;
    }

    pub fn update_game_state(&mut self) {
        if self.last_update.elapsed() >= Duration::from_millis(33) {
            let _ = self.update_ball_position();
            self.update_computer_player(0);
            self.update_computer_player(1);

            self.last_update = Instant::now();
        }
    }

    pub fn power_move(&mut self, player_index: usize) {
        let player = &mut self.players[player_index];
        if player.power_moves_left <= 0 {
            return; // no power move left
        }

        let ball = &mut self.ball;

        if ball.velocity[0] < 0
            && ball.position[1] >= player.bar_position
            && ball.position[1] < player.bar_position + 5
            && ball.position[0] > 3
            && ball.position[0] < 6
        {
            ball.velocity[0] = 6;
            ball.is_powered = true;
            player.power_moves_left -= 1;
        }
    }
}
