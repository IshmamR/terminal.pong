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
    layout::{Alignment, Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use tui_big_text::{BigText, PixelSize};

mod game;
mod helpers;
use crate::{
    game::{Game, GameType, PLAYER_NAME_CHAR_LEN},
    helpers::{centered_rect, centered_rect_with_percentage},
};

#[derive(Debug)]
struct MainMenu {
    options: Vec<&'static str>,
    selected: usize,
}

#[derive(Debug)]
enum AppScreen {
    MainMenu,
    PlayerNameInput { current: usize, max: usize },
    Game,
}

struct App {
    exit: bool,
    main_menu: MainMenu,
    current_game: Option<Game>,
    screen: AppScreen,
    name_input: String,
    player_names: [String; 2],
}

const MAIN_MENU_OPTIONS: [&str; 5] = [
    "Play vs. AI",
    "Play with Friend",
    "I like to watch",
    "Settings",
    "Exit",
];

impl App {
    fn new() -> Self {
        let main_menu = MainMenu {
            options: MAIN_MENU_OPTIONS.to_vec(),
            selected: 0,
        };

        Self {
            exit: false,
            main_menu: main_menu,
            current_game: None,
            screen: AppScreen::MainMenu,
            name_input: String::new(),
            player_names: [String::new(), String::new()],
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
                    let game_area = centered_rect(130, 28, size.width, size.height);
                    match self.current_game.as_mut() {
                        Some(game) => game.set_area(game_area),
                        None => {}
                    }
                    last_size = 1;
                }

                match self.screen {
                    AppScreen::MainMenu => {
                        self.handle_events()?;
                        let _ = terminal.draw(|frame| self.draw(frame));
                    }
                    AppScreen::PlayerNameInput { current, max } => {
                        self.handle_player_name_input_events(current, max)?;
                        let _ = terminal.draw(|frame| self.draw_player_name_input(frame, current));
                    }
                    AppScreen::Game => match self.current_game.as_mut() {
                        Some(game) => {
                            let continue_game = game.game_loop()?;
                            if !continue_game {
                                self.current_game = None;
                                self.screen = AppScreen::MainMenu;
                            } else {
                                let _ = terminal.draw(|frame| game.draw(frame));
                            }
                        }
                        None => {
                            self.screen = AppScreen::MainMenu;
                        }
                    },
                }
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
            .constraints(vec![
                Constraint::Length(12),
                Constraint::Length(13),
                Constraint::Max(5),
            ])
            .flex(Flex::Center)
            .split(frame.area());

        let big_text = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .style(Style::new().blue())
            .lines(vec![
                "".into(),
                "terminal".cyan().into(),
                "PONG".white().into(),
                "~~~~~".light_green().into(),
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
            .constraints(vec![Constraint::Percentage(90)])
            .flex(Flex::Center)
            .split(options_block_layout[0]);

        let inner_options_layout = options_layout[0].inner(Margin::new(1, 0));
        let rows_stored = inner_options_layout.height.clamp(5, 15) as usize;

        let option_constraints = vec![Constraint::Max(1); rows_stored];
        let option_areas = Layout::vertical(option_constraints)
            .flex(Flex::Center)
            .split(inner_options_layout);

        let empty_line = Paragraph::new("")
            .style(Style::default())
            .alignment(Alignment::Center);

        frame.render_widget(empty_line.clone(), option_areas[0]);
        for (i, &option) in self.main_menu.options.iter().enumerate() {
            let mut option_widget = Paragraph::new(option)
                .style(Style::default().fg(Color::Green).bold())
                .alignment(Alignment::Center);

            if i == self.main_menu.selected {
                option_widget = option_widget.style(
                    Style::default()
                        .bg(Color::Reset)
                        .fg(Color::White)
                        .bold()
                        .italic(),
                );
            }

            frame.render_widget(option_widget, option_areas[(i + 1) * 2]);
        }
        frame.render_widget(empty_line, option_areas[0]);
    }

    fn draw_player_name_input(&mut self, frame: &mut Frame, current: usize) {
        let area = frame.area();
        let popup_area = centered_rect_with_percentage(60, 20, area.width, area.height);
        let label = if current == 0 {
            "Enter Player 1 name (max 16 chars):"
        } else {
            "Enter Player 2 name (max 16 chars):"
        };
        let name = &self.name_input;
        let input = format!("{}\n> {}", label, name);
        let popup = Paragraph::new(input)
            .block(
                Block::default()
                    .title("Player Names")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        frame.render_widget(popup, popup_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Non-blocking event polling with short timeout
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit(),
                        KeyCode::Up => {
                            if self.main_menu.selected > 0 {
                                self.main_menu.selected -= 1;
                            } else {
                                self.main_menu.selected = 4;
                            }
                        }
                        KeyCode::Down => {
                            if self.main_menu.selected < 4 {
                                self.main_menu.selected += 1;
                            } else {
                                self.main_menu.selected = 0;
                            }
                        }
                        KeyCode::Enter => {
                            match self.main_menu.selected {
                                0 => {
                                    // Play vs. AI
                                    self.name_input.clear();
                                    self.player_names = [String::new(), String::new()];
                                    self.screen = AppScreen::PlayerNameInput { current: 0, max: 0 };
                                }
                                1 => {
                                    // Play with Friend
                                    self.name_input.clear();
                                    self.player_names = [String::new(), String::new()];
                                    self.screen = AppScreen::PlayerNameInput { current: 0, max: 1 };
                                }
                                2 => {
                                    // I like to watch
                                    let game = Game::new(
                                        ["Forg", "Car"],
                                        Rect::default(),
                                        GameType::ScreenSaver,
                                        1.2.into(),
                                    );
                                    self.current_game = Some(game);
                                    self.screen = AppScreen::Game;
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

    fn handle_player_name_input_events(&mut self, current: usize, max: usize) -> io::Result<()> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Enter => {
                            let default_names = ["Player 1", "Player 2"];
                            let name = if self.name_input.trim().is_empty() {
                                default_names[current]
                            } else {
                                self.name_input.trim()
                            };
                            self.player_names[current] = name.to_string();
                            self.name_input.clear();
                            if current < max {
                                self.screen = AppScreen::PlayerNameInput {
                                    current: current + 1,
                                    max,
                                };
                            } else {
                                if max == 0 {
                                    // vs AI
                                    let game = Game::new(
                                        [self.player_names[0].as_str(), "Computer"],
                                        Rect::default(),
                                        GameType::AgainstAi,
                                        Some(0.8),
                                    );
                                    self.current_game = Some(game);
                                } else {
                                    // with friend
                                    let game = Game::new(
                                        [
                                            self.player_names[0].as_str(),
                                            self.player_names[1].as_str(),
                                        ],
                                        Rect::default(),
                                        GameType::WithFriend,
                                        None,
                                    );
                                    self.current_game = Some(game);
                                }
                                self.screen = AppScreen::Game;
                            }
                        }
                        KeyCode::Esc => {
                            self.screen = AppScreen::MainMenu;
                        }
                        KeyCode::Backspace => {
                            self.name_input.pop();
                        }
                        KeyCode::Char(c) => {
                            if self.name_input.len() < PLAYER_NAME_CHAR_LEN && c.is_ascii_graphic()
                            {
                                self.name_input.push(c);
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
            //     app.current_game.get_player(0).score,
            //     app.current_game.get_player(1).score
            // );
            if let Some(game) = app.current_game.as_ref() {
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
