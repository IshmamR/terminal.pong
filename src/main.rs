use std::{
    io::{self},
    time::Duration,
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

mod game;
mod helpers;
use crate::{
    game::{Game, GameType},
    helpers::{centered_rect, centered_rect_with_percentage},
};

#[derive(Debug)]
pub struct App {
    exit: bool,
    game_state: Game,
}

impl App {
    pub fn new() -> Self {
        let game_state = Game::new(
            ["Promethewz", "Computer"],
            Rect::default(),
            GameType::ScreenSaver,
        );

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

                self.game_state.update_game_state();

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
        self.game_state.set_area(block_area);

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
        let game_area = game_state.get_area();
        let inner_area = Rect::new(
            game_area.x + 1,
            game_area.y + 1,
            game_area.width - 2,
            game_area.height - 2,
        );

        // Player 1 bar (left side)
        let player1 = self.game_state.get_player(0);
        let bar_1_area = Rect::new(
            inner_area.x,
            inner_area.y + player1.bar_position,
            3,
            bar_height,
        );
        let bar_1 = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan).bg(Color::Blue));
        frame.render_widget(bar_1, bar_1_area);

        // Player 2 bar (right side)
        let player2 = self.game_state.get_player(1);
        let bar_2_area = Rect::new(
            inner_area.x + inner_area.width - 3,
            inner_area.y + player2.bar_position,
            3,
            bar_height,
        );
        let bar_2 = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan).bg(Color::Blue));
        frame.render_widget(bar_2, bar_2_area);

        // Ball
        let ball = self.game_state.get_ball();
        let ball_area = Rect::new(
            inner_area.x + ball.position[0],
            inner_area.y + ball.position[1],
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
            KeyCode::Char(' ') => self.game_state.power_move(0),
            KeyCode::Up => self.game_state.move_player(0, 1),
            KeyCode::Down => self.game_state.move_player(0, -1),
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollUp => self.game_state.move_player(0, 1),
            MouseEventKind::ScrollDown => self.game_state.move_player(0, -1),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn get_block_title(&self, app_name: &'static str) -> String {
        let player1 = self.game_state.get_player(0);
        let mut player_text = player1.name.iter().collect::<String>();
        player_text += &format!("({})", player1.score);

        let player2 = self.game_state.get_player(1);
        let mut computer_text = player2.name.iter().collect::<String>();
        computer_text += &format!("({})", player2.score);

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
