use rand::random;
use std::{
    io::{self},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind},
    ExecutableCommand,
};
#[allow(unused_imports)]
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
    DefaultTerminal, Frame,
};

mod helpers;
mod types;
use crate::{
    helpers::{
        centered_rect, centered_rect_with_percentage, generate_direction, predict_ball_y_at_paddle,
        string_to_char_array,
    },
    types::{Ball, ComputerAI, GameState, Player},
};

#[derive(Debug)]
pub struct App {
    exit: bool,
    game_state: GameState,
}

impl App {
    pub fn new() -> Self {
        let player1 = Player {
            name: string_to_char_array("Promethewz"),
            bar_position: 0,
        };
        let player2 = Player {
            name: string_to_char_array("Computer"),
            bar_position: 0,
        };

        let difficulty: f32 = 0.5;
        let computer_ai = ComputerAI {
            reaction_delay: 0.2 + (1.0 - difficulty) * 0.5, // 0.2-0.7 seconds
            last_ball_direction: 0,
            reaction_timer: 0.0,
            prediction_error: 2.0 + (1.0 - difficulty) * 3.0, // 2-5 units error
            max_speed: 0.8 + difficulty * 0.7,                // 0.8-1.5 speed
            current_speed: 0.0,
            target_position: 0.0,
            // difficulty,
            fatigue: 0.0,
            last_update: Instant::now(),
        };

        let game_state = GameState {
            players: [player1, player2],
            ball: Ball {
                position: [70, 14],
                velocity: [3, 1],
                powered: false,
            },
            last_update: Instant::now(),
            game_area: Rect::default(),
            points: [0, 0],
            // is_paused: true,
            computer_ai,
        };

        Self {
            exit: false,
            game_state,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        let mut stdout = io::stdout();

        stdout.execute(event::EnableMouseCapture)?;

        while !self.exit {
            let min_width = 150;
            let min_height = 30;

            let size = terminal.size()?;
            if size.width < min_width || size.height < min_height {
                terminal.draw(|frame| self.show_terminal_resize_warning(frame))?;
            } else {
                terminal.draw(|frame| self.draw(frame))?;

                if self.game_state.last_update.elapsed() >= Duration::from_millis(33) {
                    self.update_game_state();
                    self.game_state.last_update = Instant::now();
                }

                self.handle_events()?;
            }
        }

        stdout.lock().execute(event::DisableMouseCapture)?;
        Ok(())
    }

    fn show_terminal_resize_warning(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let popup_area = centered_rect_with_percentage(60, 20, area.width, area.height);
        let popup = Paragraph::new("Terminal too small!\nPlease resize.")
            .block(Block::default().title("Warning").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(popup, popup_area);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let block_area = centered_rect(140, 28, area.width, area.height);
        self.game_state.game_area = block_area;

        let title = self.get_block_title("Pongerz ☘️");

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::Green))
            .title_alignment(Alignment::Center);
        frame.render_widget(block, block_area);

        self.draw_game_elements(frame);

        let controls_area = Rect::new(
            block_area.x + 1,
            block_area.y + block_area.height - 1,
            block_area.width - 2,
            1,
        );
        let controls_text = " Controls: ↑/↓ arrows / mouse wheel | Q - Quit ";
        let controls = Paragraph::new(controls_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(controls, controls_area);
    }

    fn draw_game_elements(&self, frame: &mut Frame) {
        let game_state = &self.game_state;
        let bar_height = 5;
        let inner_area = Rect::new(
            game_state.game_area.x + 1,
            game_state.game_area.y + 1,
            game_state.game_area.width - 2,
            game_state.game_area.height - 2,
        );

        // Player 1 bar (left side)
        let bar_1_area = Rect::new(
            inner_area.x,
            inner_area.y + game_state.players[0].bar_position,
            3,
            bar_height,
        );
        let bar_1 = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan).bg(Color::Blue));
        frame.render_widget(bar_1, bar_1_area);

        // Player 2 bar (right side)
        let bar_2_area = Rect::new(
            inner_area.x + inner_area.width - 3,
            inner_area.y + game_state.players[1].bar_position,
            3,
            bar_height,
        );
        let bar_2 = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan).bg(Color::Blue));
        frame.render_widget(bar_2, bar_2_area);

        // Ball
        let ball_area = Rect::new(
            inner_area.x + game_state.ball.position[0],
            inner_area.y + game_state.ball.position[1],
            2,
            2,
        );
        // let ball = Paragraph::new("●").style(Style::default().fg(Color::Red));
        // let ball = Paragraph::new("⬢").style(Style::default().fg(Color::Red));
        let ball = Paragraph::new("██").style(Style::default().fg(Color::Red));
        frame.render_widget(ball, ball_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Non-blocking event polling with short timeout
        if event::poll(Duration::from_millis(10))? {
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

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => self.power_move(),
            KeyCode::Up => self.move_up(0),
            KeyCode::Down => self.move_down(0),
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollUp => self.move_up(0),
            MouseEventKind::ScrollDown => self.move_down(0),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_up(&mut self, player_index: usize) {
        let players = &mut self.game_state.players;
        if players[player_index].bar_position > 0 {
            players[player_index].bar_position -= 1;
        }
    }

    fn move_down(&mut self, player_index: usize) {
        let players = &mut self.game_state.players;
        let inner_height = 26;
        let bar_height = 5;
        if players[player_index].bar_position + bar_height < inner_height {
            players[player_index].bar_position += 1;
        }
    }

    fn power_move(&mut self) {
        let ball = &mut self.game_state.ball;
        let player = &self.game_state.players[0];

        if ball.velocity[0] < 0
            && ball.position[1] >= player.bar_position
            && ball.position[1] < player.bar_position + 5
            && ball.position[0] > 3
            && ball.position[0] < 6
        {
            ball.velocity[0] = 6;
            ball.powered = true;
        }
    }

    fn update_game_state(&mut self) {
        self.update_ball_position();
        self.update_computer_player();
    }

    fn update_ball_position(&mut self) {
        let inner_width = 138;
        let inner_height = 26;

        let players = &self.game_state.players;
        let ball = &mut self.game_state.ball;

        let new_x = ball.position[0].saturating_add_signed(ball.velocity[0]);
        let new_y = ball.position[1].saturating_add_signed(ball.velocity[1]);

        // collision with top and bottom walls
        if new_y == 0 || new_y >= inner_height - 1 {
            ball.velocity[1] = -ball.velocity[1];
        } else {
            ball.position[1] = new_y;
        }

        // ball collision with Player 1's bar (left side)
        if new_x <= 3 && ball.velocity[0] < 0 {
            if ball.position[1] >= players[0].bar_position
                && ball.position[1] < players[0].bar_position + 5
            {
                ball.velocity[0] = -ball.velocity[0];
                ball.position[0] = 5;
                return;
            }
        }

        // ball collision with Player 2's bar (right side)
        if new_x >= inner_width - 4 && ball.velocity[0] > 0 {
            if ball.position[1] >= players[1].bar_position
                && ball.position[1] < players[1].bar_position + 5
            {
                ball.velocity[0] = -3;
                // ball.velocity[0] = -ball.velocity[0];
                ball.position[0] = inner_width - 5;
                ball.powered = false;
                return;
            }
        }

        // ball went off screen (reset)
        if new_x == 0 || new_x >= inner_width {
            // Ball exited the screen: left or right
            if new_x == 0 {
                // ball exited on the left → player missed → computer scores
                self.game_state.points[1] += 1;
            } else {
                // ball exited on the right → computer missed → player scores
                self.game_state.points[0] += 1;
            }

            // reset ball to center
            ball.position = [inner_width / 2, inner_height / 2];

            let direction = generate_direction();

            ball.velocity[0] = direction * 3;
            ball.powered = false;
        } else {
            ball.position[0] = new_x;
        }
    }

    fn update_computer_player(&mut self) {
        let computer = &mut self.game_state.players[1];
        let ball = &self.game_state.ball;
        let ai = &mut self.game_state.computer_ai;

        const INNER_HEIGHT: f32 = 26.0;
        const BAR_HEIGHT: f32 = 5.0;
        const PADDLE_X: f32 = 138.0 - 5.0;

        let dt = ai.last_update.elapsed().as_secs_f32();
        ai.last_update = Instant::now();

        // increases fatigue over time
        ai.fatigue = (ai.fatigue + dt * 0.01).min(0.3);

        let paddle_center = computer.bar_position as f32 + BAR_HEIGHT / 2.0;
        let ball_direction_x = ball.velocity[0].signum() as i16;

        // has ball direction changed
        if ball_direction_x != ai.last_ball_direction && ball_direction_x != 0 {
            ai.last_ball_direction = ball_direction_x;
            ai.reaction_timer = ai.reaction_delay + ai.fatigue * 0.5;
        }

        ai.reaction_timer = (ai.reaction_timer - dt).max(0.0);

        let is_ball_coming = ball.velocity[0] > 0;

        if !is_ball_coming || ai.reaction_timer > 0.0 {
            // slowly drift towards the center
            let center_y = INNER_HEIGHT / 2.0;
            ai.target_position = paddle_center + (center_y - paddle_center) * 0.1;
        } else {
            // "predict" ball position with wall bounces
            let mut predicted_y = predict_ball_y_at_paddle(
                ball.position[1] as f32,
                ball.velocity[1] as f32,
                ball.position[0] as f32,
                ball.velocity[0] as f32,
                PADDLE_X,
                INNER_HEIGHT,
            );

            // sprinkle some prediction errors -,-
            let error_magnitude = ai.prediction_error * (1.0 + ai.fatigue);
            let prediction_error = (random::<f32>() - 0.5) * error_magnitude;
            predicted_y += prediction_error;

            // make big oopsies occasionally
            if random::<f32>() < 0.05 + ai.fatigue * 0.1 {
                predicted_y += (random::<f32>() - 0.5) * 8.0;
            }

            // add a level of randomness
            predicted_y = predicted_y.clamp(0.0, INNER_HEIGHT - BAR_HEIGHT);

            if random::<f32>() < 0.1 {
                predicted_y += (random::<f32>() - 0.5) * 2.0;
            }

            ai.target_position = predicted_y;
        }

        // Smooth movement with acceleration
        let distance_to_target = ai.target_position - paddle_center;
        let desired_speed = distance_to_target.abs().min(ai.max_speed);

        let acceleration = 2.0;
        if distance_to_target.abs() > 0.5 {
            ai.current_speed = (ai.current_speed + acceleration * dt).min(desired_speed);
        } else {
            ai.current_speed = (ai.current_speed - acceleration * dt * 2.0).max(0.0);
        }

        // Apply movement with jitter and behavioral quirks
        let jitter = (random::<f32>() - 0.5) * 0.1 * (1.0 + ai.fatigue);
        let movement = distance_to_target.signum() * ai.current_speed + jitter;

        let final_movement = if random::<f32>() < 0.02 + ai.fatigue * 0.05 {
            movement * 0.3 // Hesitation
        } else if random::<f32>() < 0.03 {
            movement * 1.4 // Overshoot
        } else {
            movement
        };

        // Apply clamped movement
        let new_pos = (paddle_center + final_movement)
            .clamp(BAR_HEIGHT / 2.0, INNER_HEIGHT - BAR_HEIGHT / 2.0);

        computer.bar_position = (new_pos - BAR_HEIGHT / 2.0) as u16;
    }

    fn get_block_title(&self, app_name: &'static str) -> String {
        let mut player_text = self.game_state.players[0].name.iter().collect::<String>();
        player_text += &format!("({})", self.game_state.points[0]);

        let mut computer_text = self.game_state.players[1].name.iter().collect::<String>();
        computer_text += &format!("({})", self.game_state.points[1]);

        let padding_left: u16 = 32;
        let padding_right: u16 = 32;

        let total_padding = padding_left + padding_right + app_name.len() as u16;
        let available_space = 140 - total_padding;

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

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();

    app_result
}
