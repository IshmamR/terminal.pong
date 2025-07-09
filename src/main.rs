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
mod game_theme;
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
    Settings,
}

use crate::game_theme::GameTheme;

struct App {
    exit: bool,
    main_menu: MainMenu,
    current_game: Option<Game>,
    screen: AppScreen,
    name_input: String,
    player_names: [String; 2],
    // Settings
    default_difficulty_vs_ai: f32,
    default_difficulty_with_friend: f32,
    default_difficulty_screensaver: f32,
    selected_theme: GameTheme,
    settings_selected: usize, // 0: vs AI, 1: with friend, 2: screensaver, 3: theme, 4: back
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
            default_difficulty_vs_ai: 0.8,
            default_difficulty_with_friend: 1.0,
            default_difficulty_screensaver: 1.2,
            selected_theme: GameTheme::Monokai,
            settings_selected: 0,
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
                    AppScreen::Settings => {
                        self.handle_settings_events()?;
                        let _ = terminal.draw(|frame| self.draw_settings(frame));
                    }
                }
            }
        }

        Ok(())
    }

    fn show_terminal_resize_warning(&mut self, frame: &mut Frame) {
        let colors = self.selected_theme.colors();
        let area = frame.area();
        let popup_area = centered_rect_with_percentage(60, 20, area.width, area.height);
        let popup = Paragraph::new("Terminal too small!\nPlease resize.")
            .block(
                Block::default()
                    .title("Warning")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().fg(colors.ball))
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
                                    let mut game = Game::new(
                                        ["Forg", "Car"],
                                        Rect::default(),
                                        GameType::ScreenSaver,
                                        Some(self.default_difficulty_screensaver),
                                    );
                                    game.set_theme(self.selected_theme);
                                    self.current_game = Some(game);
                                    self.screen = AppScreen::Game;
                                }
                                3 => {
                                    // Settings
                                    self.settings_selected = 0;
                                    self.screen = AppScreen::Settings;
                                }
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
                                    let mut game = Game::new(
                                        [self.player_names[0].as_str(), "Computer"],
                                        Rect::default(),
                                        GameType::AgainstAi,
                                        Some(self.default_difficulty_vs_ai),
                                    );
                                    game.set_theme(self.selected_theme);
                                    self.current_game = Some(game);
                                } else {
                                    // with friend
                                    let mut game = Game::new(
                                        [
                                            self.player_names[0].as_str(),
                                            self.player_names[1].as_str(),
                                        ],
                                        Rect::default(),
                                        GameType::WithFriend,
                                        Some(self.default_difficulty_with_friend),
                                    );
                                    game.set_theme(self.selected_theme);
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

    // --- Settings Screen ---
    fn draw_settings(&mut self, frame: &mut Frame) {
        let colors = self.selected_theme.colors();
        let area = frame.area();
        let theme_names = [
            "Monokai",
            "Solarized",
            "Dracula",
            "Gruvbox Dark",
            "Nord",
            "One Dark",
            "High Contrast",
        ];
        let theme_idx = self.selected_theme as usize;
        let settings = [
            format!(
                "Default Difficulty (vs AI): {:.2}",
                self.default_difficulty_vs_ai
            ),
            format!(
                "Default Difficulty (with Friend): {:.2}",
                self.default_difficulty_with_friend
            ),
            format!(
                "Default Difficulty (Screensaver): {:.2}",
                self.default_difficulty_screensaver
            ),
            format!("Theme: {}", theme_names[theme_idx]),
            "Back".to_string(),
        ];

        let mut styled_lines = Vec::new();
        for (i, s) in settings.iter().enumerate() {
            if i == self.settings_selected {
                styled_lines.push(
                    Paragraph::new(format!("> {} <", s))
                        .style(Style::default().fg(Color::White).bold())
                        .alignment(Alignment::Center),
                );
            } else {
                styled_lines.push(
                    Paragraph::new(format!("  {}  ", s))
                        .style(Style::default().fg(colors.text))
                        .alignment(Alignment::Center),
                );
            }
        }

        let [settings_area] = Layout::horizontal([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .areas(area);
        let [settings_block_area, preview_area] =
            Layout::vertical([Constraint::Length(12), Constraint::Length(3)])
                .flex(Flex::Center)
                .areas(settings_area);
        let settings_block = Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(colors.accent));
        frame.render_widget(settings_block, settings_block_area);

        let line_height = 2;
        let total_height = styled_lines.len() * line_height;
        let start_y = settings_block_area.y
            + (settings_block_area
                .height
                .saturating_sub(total_height as u16)
                / 2);
        for (i, para) in styled_lines.into_iter().enumerate() {
            let y = start_y + (i as u16) * line_height as u16;
            let line_area = Rect {
                x: settings_block_area.x + 2,
                y,
                width: settings_block_area.width.saturating_sub(4),
                height: 1,
            };
            frame.render_widget(para, line_area);
        }

        let preview_colors = [
            ("Player Bar", colors.player_bar),
            ("Power Bar", colors.player_bar_power),
            ("Ball", colors.ball),
            ("Text", colors.text),
            ("Accent", colors.accent),
            ("Border", colors.border),
            ("Background", colors.background),
        ];
        let color_bar_width = preview_area.width.saturating_sub(4);
        let color_block_width = if preview_colors.len() > 0 {
            color_bar_width / preview_colors.len() as u16
        } else {
            1
        };
        for (i, (_, color)) in preview_colors.iter().enumerate() {
            let x = preview_area.x + 2 + (i as u16) * color_block_width;
            let width = if i == preview_colors.len() - 1 {
                color_bar_width - (color_block_width * (preview_colors.len() as u16 - 1))
            } else {
                color_block_width
            };
            let color_rect = Rect {
                x,
                y: preview_area.y + 1,
                width: width.max(1),
                height: 1,
            };
            let color_block = Paragraph::new("")
                .style(Style::default().bg(*color))
                .alignment(Alignment::Center);
            frame.render_widget(color_block, color_rect);
        }

        let label_area = Rect {
            x: preview_area.x + 2,
            y: preview_area.y + 2,
            width: color_bar_width,
            height: 1,
        };
        let label_text = preview_colors
            .iter()
            .map(|(label, _)| format!("{:^width$}", label, width = color_block_width as usize))
            .collect::<Vec<_>>()
            .join("");
        let label_para = Paragraph::new(label_text)
            .style(Style::default().fg(colors.text))
            .alignment(Alignment::Center);
        frame.render_widget(label_para, label_area);
    }

    fn handle_settings_events(&mut self) -> io::Result<()> {
        use crate::game_theme::GameTheme;
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Up => {
                            if self.settings_selected > 0 {
                                self.settings_selected -= 1;
                            } else {
                                self.settings_selected = 4;
                            }
                        }
                        KeyCode::Down => {
                            if self.settings_selected < 4 {
                                self.settings_selected += 1;
                            } else {
                                self.settings_selected = 0;
                            }
                        }
                        KeyCode::Left => match self.settings_selected {
                            0 => {
                                self.default_difficulty_vs_ai =
                                    (self.default_difficulty_vs_ai - 0.1).clamp(0.0, 2.0)
                            }
                            1 => {
                                self.default_difficulty_with_friend =
                                    (self.default_difficulty_with_friend - 0.1).clamp(0.0, 2.0)
                            }
                            2 => {
                                self.default_difficulty_screensaver =
                                    (self.default_difficulty_screensaver - 0.1).clamp(0.0, 2.0)
                            }
                            3 => {
                                let idx = self.selected_theme as usize;
                                let new_idx = if idx == 0 { 6 } else { idx - 1 };
                                self.selected_theme = match new_idx {
                                    0 => GameTheme::Monokai,
                                    1 => GameTheme::Solarized,
                                    2 => GameTheme::Dracula,
                                    3 => GameTheme::GruvboxDark,
                                    4 => GameTheme::Nord,
                                    5 => GameTheme::OneDark,
                                    6 => GameTheme::HighContrast,
                                    _ => GameTheme::Monokai,
                                };
                            }
                            _ => {}
                        },
                        KeyCode::Right => match self.settings_selected {
                            0 => {
                                self.default_difficulty_vs_ai =
                                    (self.default_difficulty_vs_ai + 0.1).clamp(0.0, 2.0)
                            }
                            1 => {
                                self.default_difficulty_with_friend =
                                    (self.default_difficulty_with_friend + 0.1).clamp(0.0, 2.0)
                            }
                            2 => {
                                self.default_difficulty_screensaver =
                                    (self.default_difficulty_screensaver + 0.1).clamp(0.0, 2.0)
                            }
                            3 => {
                                let idx = self.selected_theme as usize;
                                let new_idx = if idx == 6 { 0 } else { idx + 1 };
                                self.selected_theme = match new_idx {
                                    0 => GameTheme::Monokai,
                                    1 => GameTheme::Solarized,
                                    2 => GameTheme::Dracula,
                                    3 => GameTheme::GruvboxDark,
                                    4 => GameTheme::Nord,
                                    5 => GameTheme::OneDark,
                                    6 => GameTheme::HighContrast,
                                    _ => GameTheme::Monokai,
                                };
                            }
                            _ => {}
                        },
                        KeyCode::Enter => {
                            if self.settings_selected == 4 {
                                self.screen = AppScreen::MainMenu;
                            }
                        }
                        KeyCode::Esc => {
                            self.screen = AppScreen::MainMenu;
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
