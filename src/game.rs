use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use rand::random;
use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::helpers::{centered_rect, string_to_char_array};

pub const PLAYER_NAME_CHAR_LEN: usize = 16;
const DEFAULT_BAR_LENGTH: u8 = 5;
const DEFAULT_BALL_VELOCITY_X: i8 = 3;
const DEFAULT_BALL_VELOCITY_Y: i8 = 1;
const DEFAULT_PADDLE_WIDTH: u16 = 3;
const STARTING_POWER_MOVES: u8 = 10;
const DEFAULT_DIFFICULTY: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
struct ComputerAI {
    reaction_delay: f32,     // Time before reacting to ball direction change
    last_ball_direction: i8, // Track ball direction changes
    reaction_timer: f32,     // Current reaction delay timer
    prediction_error: f32,   // How far off the prediction can be
    max_speed: f32,          // Maximum movement speed
    current_speed: f32,      // Current movement speed (with acceleration)
    target_position: f32,    // Where the AI wants to move
    // difficulty: f32,         // 0.0 to 1.0, affects all parameters
    fatigue: f32,         // Increases over time, affects performance
    last_update: Instant, // For delta time calculations
}

#[derive(Debug, Default)]
pub struct Player {
    pub name: [char; PLAYER_NAME_CHAR_LEN],
    pub score: u32,

    pub power_moves_left: u8,
    pub last_power_used_at: Option<Instant>,

    pub bar_position: u16,
    pub bar_length: u8,
    pub bar_color: Color,

    pub is_computer: bool,
    computer_ai: Option<ComputerAI>,
}

#[derive(Debug, Default)]
pub struct Ball {
    pub position: [u16; 2],
    pub velocity: [i8; 2],
    pub is_powered: bool,
    pub color: Color,
}

#[derive(Debug, PartialEq)]
pub enum GameType {
    AgainstAi,
    ScreenSaver,
    WithFriend,
}

#[derive(Debug)]
pub struct Game {
    game_type: GameType,
    players: [Player; 2],
    ball: Ball,
    game_area: Rect,
    // started_at: Option<Instant>,
    last_update: Instant,
    is_paused: bool,
    scored_keep_display: bool,
    difficulty: f32,
    should_exit: bool,
}

impl Game {
    pub fn new(
        player_names: [&str; 2],
        game_area: Rect,
        game_type: GameType,
        difficulty: Option<f32>,
    ) -> Self {
        let final_difficulty = difficulty.unwrap_or(DEFAULT_DIFFICULTY).clamp(0.0, 1.0);
        let ai_player = ComputerAI {
            reaction_delay: 0.2 + (1.0 - final_difficulty) * 0.5, // 0.2-0.7 seconds
            last_ball_direction: 0,
            reaction_timer: 0.0,
            prediction_error: 2.0 + (1.0 - final_difficulty) * 3.0, // 2-5 units error
            max_speed: 0.8 + final_difficulty * 0.7,                // 0.8-1.5 speed
            current_speed: 0.0,
            target_position: 0.0,
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
            power_moves_left: STARTING_POWER_MOVES,
            last_power_used_at: None,
            score: 0,
        };

        Self {
            game_type,
            players: [player1, player2],
            ball: Ball {
                position: [
                    game_area.width.saturating_sub(4) / 2,
                    game_area.height.saturating_sub(4) / 2,
                ],
                velocity: [DEFAULT_BALL_VELOCITY_X, DEFAULT_BALL_VELOCITY_Y],
                is_powered: false,
                color: Color::LightRed,
            },
            last_update: Instant::now(),
            game_area: game_area,
            is_paused: false,
            // started_at: None,
            scored_keep_display: false,
            difficulty: final_difficulty,
            should_exit: false,
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

    fn move_player(&mut self, player_index: usize, direction: i8) {
        if direction == 0 {
            return;
        }

        let player = &mut self.players[player_index];

        if player.is_computer {
            return;
        }

        let step: u16 = 1;

        if direction > 0 {
            // up
            if player.bar_position > 0 {
                player.bar_position -= step;
            }
        } else {
            // down
            let inner_height = self.game_area.height.saturating_sub(2);
            if player.bar_position + (player.bar_length as u16) < inner_height {
                player.bar_position += step;
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Char('p') => self.toggle_pause(),
            // player 1
            KeyCode::Char('/') => self.power_move(0),
            KeyCode::Up => self.move_player(0, 1),
            KeyCode::Down => self.move_player(0, -1),
            // player 2
            KeyCode::Char(' ') => self.power_move(1),
            KeyCode::Char('w') => self.move_player(1, 1),
            KeyCode::Char('s') => self.move_player(1, -1),
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollUp => self.move_player(0, 1),
            MouseEventKind::ScrollDown => self.move_player(0, -1),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Process all pending events for better responsiveness
        while event::poll(Duration::from_millis(5))? {
            match event::read()? {
                Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
        }
        Ok(())
    }

    // key events while paused (pause/options popup)
    fn handle_pause_events(&mut self) -> io::Result<()> {
        while event::poll(Duration::from_millis(5))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('p') => self.is_paused = false, // Resume
                        KeyCode::Enter => self.is_paused = false,     // Resume
                        KeyCode::Esc => self.should_exit = true,
                        KeyCode::Char('d') => {} // No-op, just for UI
                        KeyCode::Left => {
                            self.difficulty = (self.difficulty - 0.05).clamp(0.0, 1.0);
                        }
                        KeyCode::Right => {
                            self.difficulty = (self.difficulty + 0.05).clamp(0.0, 1.0);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /**
     * 1 -> ball collision with Player 1's bar
     * 2 -> ball collision with Player 2's bar
     * None -> no collision, ball position updated normally
     */
    fn update_ball_position(&mut self) -> Option<u8> {
        let inner_width = self.game_area.width.saturating_sub(3);
        let inner_height = self.game_area.height.saturating_sub(2);

        let players = &self.players;
        let ball = &mut self.ball;

        let new_x = ball.position[0].saturating_add_signed(ball.velocity[0] as i16);
        let new_y = ball.position[1].saturating_add_signed(ball.velocity[1] as i16);

        // collision with top and bottom walls
        if new_y == 0 || new_y >= inner_height {
            ball.velocity[1] = -ball.velocity[1];
            ball.position[1] = if new_y == 0 { 0 } else { inner_height - 1 };
        } else {
            ball.position[1] = new_y;
        }

        if !self.scored_keep_display {
            // ball collision with Player 1's bar (left side)
            if new_x <= DEFAULT_PADDLE_WIDTH && ball.velocity[0] < 0 {
                if new_y >= players[0].bar_position
                    && new_y < players[0].bar_position + players[0].bar_length as u16
                {
                    ball.velocity[0] = -ball.velocity[0];
                    ball.position[0] = DEFAULT_PADDLE_WIDTH;
                    return Some(1);
                }
            }

            // ball collision with Player 2's bar (right side)
            if new_x >= inner_width - DEFAULT_PADDLE_WIDTH - 1 && ball.velocity[0] > 0 {
                if new_y >= players[1].bar_position
                    && new_y < players[1].bar_position + players[1].bar_length as u16
                {
                    ball.velocity[0] = -DEFAULT_BALL_VELOCITY_X;
                    ball.position[0] = inner_width - DEFAULT_PADDLE_WIDTH - 1;
                    ball.is_powered = false;
                    return Some(2);
                }
            }
        }

        // ball went off screen (reset)
        if new_x < DEFAULT_PADDLE_WIDTH || new_x > inner_width - DEFAULT_PADDLE_WIDTH - 1 {
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
                ball.position = [
                    inner_width / 2,
                    rand::random_range(1..inner_height.saturating_sub(1)),
                ];

                let random_number: i16 = rand::random_range(0..=1);
                let direction = if random_number == 0 { 1 } else { -1 };

                ball.velocity[0] = direction * DEFAULT_BALL_VELOCITY_X;
                ball.is_powered = false;

                self.scored_keep_display = false;

                return None;
            } else {
                // keep drawing
                self.scored_keep_display = true;
                ball.position[0] = new_x;
            }
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

        let inner_height = self.game_area.height;
        let paddle_x = if player_index == 0 {
            DEFAULT_PADDLE_WIDTH // Player 1's paddle is on the left
        } else {
            self.game_area.width - DEFAULT_PADDLE_WIDTH // Player 2's paddle is on the right
        };

        // accumulate delta time to add fatigue
        let dt = ai.last_update.elapsed().as_secs_f32();
        ai.last_update = Instant::now();
        // increase fatigue over time
        if self.game_type == GameType::ScreenSaver {
            ai.fatigue = (ai.fatigue + dt * 0.001).min(0.05); // much less fatigue for AI vs AI
        } else {
            ai.fatigue = (ai.fatigue + dt * 0.009).min(0.3);
        }

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
            let (error_magnitude, oops_chance, random_chance) = match self.game_type {
                GameType::ScreenSaver => (ai.prediction_error * 0.3, 0.01, 0.02), // much less error
                _ => (
                    ai.prediction_error * (1.0 + ai.fatigue),
                    0.05 + ai.fatigue * 0.1,
                    0.1,
                ),
            };
            let prediction_error = (random::<f32>() - 0.5) * error_magnitude;
            pred_y += prediction_error;

            // make big oopsies occasionally
            if random::<f32>() < oops_chance {
                pred_y += (random::<f32>() - 0.5) * 3.0;
            }

            // add some final randomness
            if random::<f32>() < random_chance {
                pred_y += (random::<f32>() - 0.5) * 1.0;
            }

            // clamp to fix
            pred_y = pred_y.clamp(0.0, (inner_height - computer.bar_length as u16) as f32);

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
        let jitter = match self.game_type {
            GameType::ScreenSaver => (random::<f32>() - 0.5) * 0.02 * (1.0 + ai.fatigue), // much less jitter
            GameType::AgainstAi => (random::<f32>() - 0.5) * 0.1 * (1.0 + ai.fatigue),
            _ => 0.0,
        };
        let movement = distance_to_target.signum() * ai.current_speed + jitter;

        let final_movement = match self.game_type {
            GameType::ScreenSaver => movement, // no hesitation/overshoot in screensaver
            GameType::AgainstAi => {
                if random::<f32>() < 0.01 + ai.fatigue * 0.02 {
                    movement * 0.3 // less hesitation
                } else if random::<f32>() < 0.015 {
                    movement * 1.2 // less overshoot
                } else {
                    movement
                }
            }
            _ => movement,
        };

        // apply new position with clamping
        let new_pos = (paddle_center + final_movement).clamp(
            computer.bar_length as f32 / 2.0,
            (inner_height - computer.bar_length as u16) as f32,
        );

        computer.bar_position = (new_pos - computer.bar_length as f32 / 2.0) as u16;
    }

    fn power_move(&mut self, player_index: usize) {
        let player = &mut self.players[player_index];

        if player.power_moves_left <= 0 {
            return; // no power move left
        }

        let ball = &mut self.ball;

        let is_ball_approaching = if player_index == 0 {
            ball.velocity[0] < 0
        } else {
            ball.velocity[0] > 0
        };

        let within_bar = ball.position[1] >= player.bar_position
            && ball.position[1] < player.bar_position + player.bar_length as u16;

        let min_range = 4.0;
        let max_range = 12.0;
        let allowed_range = (max_range - min_range) * (1.0 - self.difficulty) + min_range;
        let allowed_range = allowed_range.round() as u16;

        let within_x = if player_index == 0 {
            ball.position[0] > 1 && ball.position[0] < 1 + allowed_range
        } else {
            let right_edge = self.game_area.width.saturating_sub(1);
            ball.position[0] > right_edge.saturating_sub(allowed_range)
                && ball.position[0] < right_edge.saturating_sub(1)
        };

        if is_ball_approaching && within_bar && within_x {
            // power move: send ball flying in the correct direction
            ball.velocity[0] = if player_index == 0 { 6 } else { -6 };
            ball.is_powered = true;
            player.power_moves_left -= 1;
            player.bar_color = Color::Yellow; // Highlight bar on power move
            player.last_power_used_at = Some(Instant::now());
        }
    }

    fn draw_core_elements(&self, frame: &mut Frame) {
        let game_area = self.get_area();
        let inner_area = Rect::new(
            game_area.x + 1,
            game_area.y + 1,
            game_area.width - 1,
            game_area.height - 1,
        );

        // Player 1 bar (left side)
        let player1 = self.get_player(0);
        let bar_1_area = Rect::new(
            inner_area.x,
            inner_area.y + player1.bar_position,
            3,
            player1.bar_length as u16,
        );
        let bar_1_color = if let Some(last) = player1.last_power_used_at {
            if last.elapsed() < Duration::from_millis(200) {
                Color::Yellow
            } else {
                Color::Cyan
            }
        } else {
            Color::Cyan
        };
        let bar_1 = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan).bg(bar_1_color));
        frame.render_widget(bar_1, bar_1_area);

        // Player 2 bar (right side)
        let player2 = self.get_player(1);
        let bar_2_area = Rect::new(
            inner_area.x + inner_area.width - 4,
            inner_area.y + player2.bar_position,
            3,
            player2.bar_length as u16,
        );
        let bar_2_color = if let Some(last) = player2.last_power_used_at {
            if last.elapsed() < Duration::from_millis(200) {
                Color::Yellow
            } else {
                Color::Cyan
            }
        } else {
            Color::Cyan
        };
        let bar_2 = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan).bg(bar_2_color));
        frame.render_widget(bar_2, bar_2_area);

        // Ball
        let ball = self.get_ball();
        let ball_area = Rect::new(
            inner_area.x + ball.position[0],
            inner_area.y + ball.position[1],
            2,
            2,
        );
        let ball = Paragraph::new("██").style(Style::default().fg(ball.color));
        frame.render_widget(ball, ball_area);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(130)])
            .flex(Flex::Center)
            .split(area);

        // Layout: [spacer] [[game block] [spacer] [controls block]] [spacer]
        // Use more space for the top and bottom spacers to center the game block vertically
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(28), // game block
                Constraint::Length(3),  // controls block
            ])
            .flex(Flex::Center)
            .split(main_layout[0]);

        // Center the game block horizontally in its layout area
        let game_area = layout[0];
        // let block_area = centered_rect(130, 28, game_area.width, game_area.height);
        self.set_area(game_area);

        let title = self.get_block_title("terminal.pong");
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::Green))
            .title_alignment(Alignment::Center);
        frame.render_widget(block, game_area);

        self.draw_core_elements(frame);

        let controls_text = " Player 1: ↑/↓ or mouse wheel, '/'=Power    |    Player 2: W/S, Space=Power    |    P=Pause    |    Esc=Quit ";
        let controls = Paragraph::new(controls_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(controls, layout[1]);

        if self.is_paused {
            // draw pause/options popup if paused
            let popup_width = 48;
            let popup_height = 10;
            let popup_area = centered_rect(popup_width, popup_height, area.width, area.height);
            let popup_block = Block::default()
                .title("Paused - Options")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .style(Style::default().fg(Color::Magenta))
                .title_alignment(Alignment::Center);
            frame.render_widget(popup_block, popup_area);

            // options and instructions
            let diff_label = match self.difficulty {
                d if d < 0.33 => "Easy",
                d if d < 0.66 => "Normal",
                _ => "Hard",
            };
            let options_text = format!(
                "\n  Difficulty: {} ({:.2})\n  [←/→] Adjust  [P] Resume  [Esc] Quit\n",
                diff_label, self.difficulty
            );
            let options = Paragraph::new(options_text)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);
            let options_area = Rect::new(
                popup_area.x + 2,
                popup_area.y + 2,
                popup_area.width - 4,
                popup_area.height - 4,
            );
            frame.render_widget(options, options_area);
        }
    }

    pub fn game_loop(&mut self) -> io::Result<bool> {
        // If paused, only handle pause menu events
        if self.is_paused {
            self.handle_pause_events()?;
            if self.should_exit {
                return Ok(false);
            }
            return Ok(true);
        }

        // FPS/speed is relative to difficulty
        let min_fps = 15.0;
        let max_fps = 40.0;
        let fps = min_fps + (max_fps - min_fps) * self.difficulty;
        let each_frame = (1000.0 / fps).round() as u64;

        if self.last_update.elapsed() >= Duration::from_millis(each_frame) {
            self.handle_events()?;
            if self.should_exit {
                return Ok(false);
            }
            if self.game_type == GameType::AgainstAi {
                self.update_computer_player(1);
            } else {
                if rand::random() {
                    self.update_computer_player(0);
                    self.update_computer_player(1);
                } else {
                    self.update_computer_player(1);
                    self.update_computer_player(0);
                }
            }
            let _ = self.update_ball_position();

            self.last_update = Instant::now();
        }

        Ok(true)
    }

    fn get_block_title(&self, app_name: &'static str) -> String {
        let player1 = self.get_player(0);
        let mut player_text = player1.name.iter().collect::<String>();
        player_text += &format!("({})", player1.score);

        let player2 = self.get_player(1);
        let mut computer_text = player2.name.iter().collect::<String>();
        computer_text += &format!("({})", player2.score);

        let padding_left: u16 = 32;
        let padding_right: u16 = 32;

        let total_padding = padding_left + padding_right + app_name.len() as u16;
        let available_space = 130 - total_padding;

        format!(
            " {} {} {} {} {} ",
            player_text.trim_end(),
            "─".repeat((available_space / 2) as usize),
            app_name.trim_end(),
            "─".repeat((available_space / 2) as usize),
            computer_text.trim_start(),
        )
    }
}
