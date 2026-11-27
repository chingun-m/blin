use crate::FileEntry;

use ratatui::style::Color;

pub fn icon_for(entry: &FileEntry) -> (&'static str, Color) {
    if entry.is_broken {
        return ("\u{f071}", Color::Red);
    }

    if entry.is_dir {
        return ("\u{f07b}", Color::Rgb(229, 180, 84));
    }

    let ext = entry.name.rsplit('.').next().unwrap_or("");

    match ext.to_ascii_lowercase().as_str() {
        "py" => ("\u{e73c}", Color::Rgb(255, 212, 59)),
        "rs" => ("\u{e7a8}", Color::Rgb(222, 100, 50)),
        "java" => ("\u{e738}", Color::Rgb(231, 111, 81)),
        "c" | "h" => ("\u{e61e}", Color::Rgb(89, 196, 245)),
        "cpp" | "hpp" => ("\u{e61d}", Color::Rgb(100, 149, 237)),
        "js" | "jsx" => ("\u{e74e}", Color::Yellow),
        "ts" | "tsx" => ("\u{e628}", Color::Blue),
        "html" => ("\u{e736}", Color::Rgb(228, 100, 45)),
        "css" => ("\u{e749}", Color::Rgb(86, 156, 214)),
        "json" => ("\u{e60b}", Color::Rgb(235, 203, 139)),
        "md" => ("\u{e73e}", Color::White),
        "zip" => ("\u{f1c6}", Color::Rgb(200, 160, 100)),
        "png" | "jpg" | "jpeg" | "gif" | "webp" => ("\u{f1c5}", Color::Magenta),
        "txt" => ("\u{f0f6}", Color::Gray),
        _ => ("\u{f15b}", Color::DarkGray),
    }
}
