//ofc it's themes
use ratatui::style::Color;

pub struct Theme {
    // General
    pub bg: Color,
    pub fg: Color,

    // Topbar
    pub topbar_bg: Color,
    pub topbar_fg: Color,

    // Sidebar
    pub sidebar_bg: Color,
    pub sidebar_fg: Color,
    pub sidebar_sel_bg: Color,
    pub sidebar_sel_fg: Color,

    // File list
    pub dir_fg: Color,
    pub file_fg: Color,
    pub broken_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,

    // Preview
    pub preview_label: Color,
    pub preview_value: Color,

    // Status bar
    pub status_bg: Color,
    pub status_fg: Color,

    // Borders
    pub border: Color,
    pub border_focused: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::White,
            topbar_bg: Color::Rgb(30, 30, 40),
            topbar_fg: Color::Rgb(150, 180, 255),
            sidebar_bg: Color::Rgb(20, 20, 28),
            sidebar_fg: Color::Gray,
            sidebar_sel_bg: Color::Rgb(50, 50, 80),
            sidebar_sel_fg: Color::White,
            dir_fg: Color::Rgb(100, 160, 255),
            file_fg: Color::White,
            broken_fg: Color::Red,
            selected_bg: Color::Rgb(50, 50, 80),
            selected_fg: Color::White,
            preview_label: Color::Rgb(120, 120, 160),
            preview_value: Color::White,
            status_bg: Color::Rgb(30, 30, 40),
            status_fg: Color::Gray,
            border: Color::Rgb(60, 60, 80),
            border_focused: Color::Rgb(100, 100, 180),
        }
    }
    pub fn gruvbox() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::Rgb(235, 219, 178),
            topbar_bg: Color::Rgb(40, 40, 40),
            topbar_fg: Color::Rgb(250, 189, 47),
            sidebar_bg: Color::Rgb(40, 40, 40),
            sidebar_fg: Color::Rgb(168, 153, 132),
            sidebar_sel_bg: Color::Rgb(80, 73, 69),
            sidebar_sel_fg: Color::Rgb(235, 219, 178),
            dir_fg: Color::Rgb(131, 165, 152),
            file_fg: Color::Rgb(235, 219, 178),
            broken_fg: Color::Rgb(251, 73, 52),
            selected_bg: Color::Rgb(80, 73, 69),
            selected_fg: Color::Rgb(235, 219, 178),
            preview_label: Color::Rgb(146, 131, 116),
            preview_value: Color::Rgb(235, 219, 178),
            status_bg: Color::Rgb(40, 40, 40),
            status_fg: Color::Rgb(168, 153, 132),
            border: Color::Rgb(80, 73, 69),
            border_focused: Color::Rgb(250, 189, 47),
        }
    }
    pub fn catpuccin_mocha() -> Self {
        Self {
            // Base
            bg: Color::Rgb(30, 30, 46),    // Base
            fg: Color::Rgb(205, 214, 244), // Text

            // Topbar
            topbar_bg: Color::Rgb(24, 24, 37),    // Mantle
            topbar_fg: Color::Rgb(180, 190, 254), // Lavender

            // Sidebar
            sidebar_bg: Color::Rgb(24, 24, 37),        // Mantle
            sidebar_fg: Color::Rgb(166, 173, 200),     // Subtext0
            sidebar_sel_bg: Color::Rgb(69, 71, 90),    // Surface2
            sidebar_sel_fg: Color::Rgb(245, 224, 220), // Rosewater

            // Files and directories
            dir_fg: Color::Rgb(137, 180, 250),    // Blue
            file_fg: Color::Rgb(205, 214, 244),   // Text
            broken_fg: Color::Rgb(243, 139, 168), // Red

            // Selection
            selected_bg: Color::Rgb(69, 71, 90),    // Surface2
            selected_fg: Color::Rgb(245, 224, 220), // Rosewater

            // Preview
            preview_label: Color::Rgb(203, 166, 247), // Mauve
            preview_value: Color::Rgb(205, 214, 244), // Text

            // Status bar
            status_bg: Color::Rgb(24, 24, 37),    // Mantle
            status_fg: Color::Rgb(166, 173, 200), // Subtext0

            // Borders
            border: Color::Rgb(69, 71, 90),            // Surface2
            border_focused: Color::Rgb(203, 166, 247), // Mauve
        }
    }
    pub fn nord() -> Self {
        Self {
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),

            topbar_bg: Color::Rgb(59, 66, 82),
            topbar_fg: Color::Rgb(136, 192, 208),

            sidebar_bg: Color::Rgb(46, 52, 64),
            sidebar_fg: Color::Rgb(180, 190, 204),
            sidebar_sel_bg: Color::Rgb(67, 76, 94),
            sidebar_sel_fg: Color::Rgb(236, 239, 244),

            dir_fg: Color::Rgb(129, 161, 193),
            file_fg: Color::Rgb(216, 222, 233),
            broken_fg: Color::Rgb(191, 97, 106),

            selected_bg: Color::Rgb(67, 76, 94),
            selected_fg: Color::Rgb(236, 239, 244),

            preview_label: Color::Rgb(136, 192, 208),
            preview_value: Color::Rgb(216, 222, 233),

            status_bg: Color::Rgb(59, 66, 82),
            status_fg: Color::Rgb(180, 190, 204),

            border: Color::Rgb(76, 86, 106),
            border_focused: Color::Rgb(136, 192, 208),
        }
    }
    pub fn neon1() -> Self {
        Self {
            bg: Color::Black,
            fg: Color::Rgb(140, 0, 255),

            topbar_bg: Color::Rgb(69, 6, 147),
            topbar_fg: Color::Rgb(255, 196, 0),

            sidebar_bg: Color::Black,
            sidebar_fg: Color::Rgb(140, 0, 255),
            sidebar_sel_bg: Color::Rgb(67, 76, 94),
            sidebar_sel_fg: Color::Rgb(236, 239, 244),

            dir_fg: Color::Rgb(140, 0, 255),
            file_fg: Color::Rgb(216, 222, 233),
            broken_fg: Color::Rgb(191, 97, 106),

            selected_bg: Color::Rgb(255, 196, 0),
            selected_fg: Color::Rgb(69, 6, 147),

            preview_label: Color::Rgb(255, 196, 0),
            preview_value: Color::Rgb(140, 0, 255),

            status_bg: Color::Rgb(69, 6, 147),
            status_fg: Color::Rgb(255, 63, 127),

            border: Color::Rgb(255, 196, 0),
            border_focused: Color::Rgb(255, 63, 127),
        }
    }
    pub fn by_index(i: usize) -> Self {
        match i {
            1 => Theme::default(),
            2 => Theme::gruvbox(),
            3 => Theme::catpuccin_mocha(),
            4 => Theme::nord(),
            5 => Theme::neon1(),
            _ => Theme::default(),
        }
    }
}
