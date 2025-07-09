use ratatui::style::Color;

/// Available game themes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameTheme {
    Monokai,
    Solarized,
    Dracula,
    GruvboxDark,
    Nord,
    OneDark,
    HighContrast,
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
            GameTheme::Monokai => ThemeColors {
                background: Color::Reset,
                border: Color::Rgb(249, 38, 114), // Monokai pink
                text: Color::Rgb(248, 248, 242),  // Monokai foreground
                accent: Color::Rgb(166, 226, 46), // Monokai green
                player_bar: Color::Rgb(102, 217, 239), // Monokai cyan
                player_bar_power: Color::Rgb(230, 219, 116), // Monokai yellow
                ball: Color::Rgb(255, 95, 135),   // Monokai light pink
            },
            GameTheme::Solarized => ThemeColors {
                background: Color::Reset,
                border: Color::Rgb(38, 139, 210), // Solarized blue
                text: Color::Rgb(101, 123, 131),  // Solarized base00
                accent: Color::Rgb(42, 161, 152), // Solarized cyan
                player_bar: Color::Rgb(133, 153, 0), // Solarized green
                player_bar_power: Color::Rgb(181, 137, 0), // Solarized yellow
                ball: Color::Rgb(220, 50, 47),    // Solarized red
            },
            GameTheme::Dracula => ThemeColors {
                background: Color::Reset,
                border: Color::Rgb(255, 121, 198), // Dracula pink
                text: Color::Rgb(248, 248, 242),   // Dracula foreground
                accent: Color::Rgb(189, 147, 249), // Dracula purple
                player_bar: Color::Rgb(80, 250, 123), // Dracula green
                player_bar_power: Color::Rgb(241, 250, 140), // Dracula yellow
                ball: Color::Rgb(255, 85, 85),     // Dracula red
            },
            GameTheme::GruvboxDark => ThemeColors {
                background: Color::Reset,
                border: Color::Rgb(250, 189, 47), // Gruvbox yellow
                text: Color::Rgb(235, 219, 178),  // Gruvbox fg
                accent: Color::Rgb(184, 187, 38), // Gruvbox green
                player_bar: Color::Rgb(131, 165, 152), // Gruvbox blue
                player_bar_power: Color::Rgb(254, 128, 25), // Gruvbox orange
                ball: Color::Rgb(251, 73, 52),    // Gruvbox red
            },
            GameTheme::Nord => ThemeColors {
                background: Color::Reset,
                border: Color::Rgb(136, 192, 208),    // Nord border
                text: Color::Rgb(216, 222, 233),      // Nord fg
                accent: Color::Rgb(143, 188, 187),    // Nord cyan
                player_bar: Color::Rgb(94, 129, 172), // Nord blue
                player_bar_power: Color::Rgb(235, 203, 139), // Nord yellow
                ball: Color::Rgb(191, 97, 106),       // Nord red
            },
            GameTheme::OneDark => ThemeColors {
                background: Color::Reset,
                border: Color::Rgb(198, 120, 221), // One Dark purple
                text: Color::Rgb(171, 178, 191),   // One Dark fg
                accent: Color::Rgb(97, 175, 239),  // One Dark blue
                player_bar: Color::Rgb(152, 195, 121), // One Dark green
                player_bar_power: Color::Rgb(229, 192, 123), // One Dark yellow
                ball: Color::Rgb(224, 108, 117),   // One Dark red
            },
            GameTheme::HighContrast => ThemeColors {
                background: Color::Black,                // true black for max contrast
                border: Color::White,                    // white border
                text: Color::White,                      // bright white text
                accent: Color::Yellow,                   // bright yellow accent
                player_bar: Color::Rgb(0, 255, 255),     // bright cyan (player bar)
                player_bar_power: Color::Rgb(0, 255, 0), // bright green (power bar)
                ball: Color::Rgb(255, 0, 0),             // bright red (ball)
                                                         // Extra accent colors can be added in the future if needed
            },
        }
    }
}
