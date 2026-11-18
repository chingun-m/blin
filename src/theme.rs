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

impl Default for Theme {
    fn default() -> Self {
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
    fn gruvbox() -> Self {
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
    fn by_index(i: usize) -> Self {
        match i % 2 {
            0 => Theme::default(),
            _ => Theme::gruvbox(),
        }
    }
    const COUNT: usize = 2;
}
