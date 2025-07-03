use std::{
    io::{self},
    thread::sleep,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    ExecutableCommand,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use tui_big_text::{BigText, PixelSize};

mod game;
mod helpers;
use crate::{
    game::{Game, GameType},
    helpers::{centered_rect, centered_rect_with_percentage},
};

#[derive(Debug)]
pub struct App {
    exit: bool,
    selected_option: usize,
    game_state: Option<Game>,
}

impl App {
    pub fn new() -> Self {
        let _game_state = Game::new(
            ["Promethewz", "Computer"],
            Rect::default(),
            GameType::ScreenSaver,
            Some(0.8),
        );

        Self {
            exit: false,
            selected_option: 0,
            // game_state: Some(game_state),
            game_state: None,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        let mut last_size: u8 = 0; // 0 -> too small | 1 -> normal

        while !self.exit {
            let min_width = 130;
            let min_height = 28;

            let size = terminal.size()?;
            if size.width < min_width || size.height < min_height {
                if last_size == 1 {
                    sleep(Duration::from_millis(100));
                    last_size = 0;
                }
                self.handle_events()?;
                terminal.draw(|frame| self.show_terminal_resize_warning(frame))?;
            } else {
                if last_size == 0 {
                    sleep(Duration::from_millis(100));
                    let terminal_area = centered_rect(130, 28, size.width, size.height);

                    // self.game_state.set_area(terminal_area);
                    match self.game_state.as_mut() {
                        Some(game) => game.set_area(terminal_area),
                        None => {}
                    }

                    last_size = 1;
                }

                // self.game_state.game_loop(&mut self.exit)?;
                // terminal.draw(|frame| self.game_state.draw(frame))?;
                match self.game_state.as_mut() {
                    Some(game) => {
                        game.game_loop(&mut self.exit)?;
                        terminal.draw(|frame| game.draw(frame))?
                    }
                    None => {
                        self.handle_events()?;
                        terminal.draw(|frame| self.draw(frame))?
                    }
                };
            }
        }

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
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(45)])
            .flex(Flex::Start)
            .split(frame.area());

        let big_text = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .style(Style::new().blue())
            .lines(vec![
                "".into(),
                "Terminal".red().into(),
                "PONG".white().into(),
                "~~~~~".into(),
            ])
            .alignment(Alignment::Center)
            .build();
        frame.render_widget(big_text, vertical_layout[0]);

        let options_block_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30)])
            .flex(Flex::Center)
            .split(vertical_layout[1]);
        frame.render_widget(
            Block::default()
                .title("")
                .style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
            options_block_layout[0],
        );

        let options_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(60)])
            .flex(Flex::Center)
            .split(options_block_layout[0]);

        let option_areas = Layout::vertical([Constraint::Max(1); 15])
            .flex(Flex::Center)
            .split(options_layout[0]);

        let options = [
            "Play vs. AI",
            "Play with Fren",
            "I like to watch",
            "Settings",
            "Exit",
        ];

        for (i, &option) in options.iter().enumerate() {
            let mut option_widget = Paragraph::new(option)
                .style(Style::default().fg(Color::Green).bold())
                .alignment(Alignment::Center);

            if i == self.selected_option {
                option_widget =
                    option_widget.style(Style::default().bg(Color::Blue).fg(Color::White).bold());
            }

            frame.render_widget(option_widget, option_areas[i * 3]);
            // frame.render_widget(empty_line_widget.clone(), option_areas[(i * 3) + 1]);
            // frame.render_widget(empty_line_widget.clone(), option_areas[(i * 3) + 2]);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Non-blocking event polling with short timeout
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Up => {
                            if self.selected_option > 0 {
                                self.selected_option -= 1;
                            } else {
                                self.selected_option = 4;
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_option < 4 {
                                self.selected_option += 1;
                            } else {
                                self.selected_option = 0;
                            }
                        }
                        KeyCode::Enter => {
                            match self.selected_option {
                                0 => {
                                    // Play vs. AI
                                    // let game = Game::new(
                                    //     ["Promethewz", "Computer"],
                                    //     Rect::default(),
                                    //     GameType::SinglePlayer,
                                    //     Some(0.8),
                                    // );
                                    // self.game_state = Some(game);
                                }
                                1 => {
                                    // Play with Friend
                                    // let game = Game::new(
                                    //     ["Player 1", "Player 2"],
                                    //     Rect::default(),
                                    //     GameType::Multiplayer,
                                    //     None,
                                    // );
                                    // self.game_state = Some(game);
                                }
                                2 => {
                                    // I like to watch
                                    let game = Game::new(
                                        ["Spectator", "Spectator"],
                                        Rect::default(),
                                        GameType::ScreenSaver,
                                        None,
                                    );
                                    self.game_state = Some(game);
                                }
                                3 => {}
                                4 => {
                                    self.exit();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();

    let mut stdout = io::stdout();
    stdout.execute(event::EnableMouseCapture)?;

    let app_result = app.run(terminal);

    stdout.lock().execute(event::DisableMouseCapture)?;

    ratatui::restore();

    match &app_result {
        Ok(()) => {
            println!("Thanks for playing terminal.pong! 🏓");
            // println!(
            //     "Final Score: {} - {}",
            //     app.game_state.get_player(0).score,
            //     app.game_state.get_player(1).score
            // );
            if let Some(game) = app.game_state.as_ref() {
                // Display final scores
                println!(
                    "Final Score: {} - {}",
                    game.get_player(0).score,
                    game.get_player(1).score
                );
            }
        }
        Err(e) => {
            eprintln!("Game ended with error: {}", e);
        }
    }

    app_result
}
