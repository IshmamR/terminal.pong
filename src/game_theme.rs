use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameTheme {
    Dark,
    Light,
}

pub struct ThemeColors {
    pub background: Color,
    pub border: Color,
    pub text: Color,
    pub accent: Color,
    pub player_bar: Color,
    pub player_bar_power: Color,
    pub ball: Color,
}

impl GameTheme {
    pub fn colors(&self) -> ThemeColors {
        match self {
            GameTheme::Dark => ThemeColors {
                background: Color::Rgb(0, 43, 54),
                border: Color::Yellow,
                text: Color::Yellow,
                accent: Color::Magenta,
                player_bar: Color::Cyan,
                player_bar_power: Color::Yellow,
                ball: Color::LightRed,
            },
            GameTheme::Light => ThemeColors {
                // Solarized-inspired light palette: high contrast, bluish background
                background: Color::Rgb(238, 232, 213), // solarized base3
                border: Color::Rgb(38, 139, 210),      // solarized blue
                text: Color::Rgb(44, 62, 80),          // dark blue-gray for text
                accent: Color::Rgb(42, 161, 152),      // solarized cyan
                player_bar: Color::Rgb(38, 139, 210),  // solarized blue
                player_bar_power: Color::Rgb(181, 137, 0), // solarized yellow
                ball: Color::Rgb(220, 50, 47),         // solarized red
            },
        }
    }
}
