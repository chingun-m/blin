use crate::{App, Focus, Mode};
use human_bytes::human_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::time::UNIX_EPOCH;

//RENDERING
pub fn render(f: &mut Frame, app: &mut App) {
    let t = &app.theme;
    let area = f.area();

    //outer layout: topbar/body/statusbar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), //top bar
            Constraint::Min(0),    //body
            Constraint::Length(1), //status bar
        ])
        .split(area);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ])
        .split(outer[1]);

    render_topbar(f, app, outer[0]);
    render_sidebar(f, app, body[0]);
    render_file_list(f, app, body[1]);
    render_preview(f, app, body[2]);
    render_statusbar(f, app, outer[2]);
}

pub fn render_topbar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let t = &app.theme;
    let bar = Paragraph::new(Line::from(vec![
        Span::styled(
            "blin",
            Style::default()
                .fg(t.topbar_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" - {}", app.current_dir.display()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .style(Style::default().bg(t.topbar_bg));
    f.render_widget(bar, area);
}

pub fn render_sidebar(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::Sidebar;
    let border_color = if focused { t.border_focused } else { t.border };

    let items: Vec<ListItem> = app
        .sidebar_items()
        .into_iter()
        .map(|label| {
            let is_header = label.starts_with("──");
            let style = if is_header {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC)
            } else {
                Style::default().fg(t.sidebar_fg)
            };
            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(" sidebar ")
                .title_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .bg(t.sidebar_sel_bg)
                .fg(t.sidebar_sel_fg)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut app.sidebar_state);
}

pub fn render_file_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::FileList;
    let border_color = if focused { t.border_focused } else { t.border };

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = app.selected_items.contains(&i);
            let num_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let num = format!("{:>3}. ", i + 1);

            let display_name = if is_selected {
                format!("✓ {}", entry.name)
            } else {
                entry.name.clone()
            };

            if entry.is_broken {
                let line = Line::from(vec![
                    Span::styled(num, num_style),
                    Span::styled("!!! ", Style::default().fg(t.broken_fg)),
                    Span::styled(display_name, Style::default().fg(t.broken_fg)),
                    Span::styled("  [BROKEN]", Style::default().fg(Color::DarkGray)),
                ]);
                return ListItem::new(line);
            }

            if entry.is_dir {
                let line = Line::from(vec![
                    Span::styled(num, num_style),
                    Span::styled("📁 ", Style::default()),
                    Span::styled(
                        format!("{}/", display_name),
                        Style::default().fg(t.dir_fg).add_modifier(Modifier::BOLD),
                    ),
                ]);
                ListItem::new(line)
            } else {
                let size_str = entry
                    .size
                    .map(|s| human_bytes(s as f64).replace("666", "between 665 and 667"))
                    .unwrap_or_else(|| "—".to_string());

                let line = Line::from(vec![
                    Span::styled(num, num_style),
                    Span::styled("📄 ", Style::default()),
                    Span::styled(display_name, Style::default().fg(t.file_fg)),
                    Span::styled(
                        format!("  {}", size_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);
                ListItem::new(line)
            }
        })
        .collect();

    let title = if !app.jump_buf.is_empty() {
        format!(" files  [jump: {}] ", app.jump_buf)
    } else {
        format!(" files  ({}) ", app.entries.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .bg(t.selected_bg)
                .fg(t.selected_fg)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut app.list_state);
}

pub fn render_preview(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let t = &app.theme;

    let content: Vec<Line> = match app.selected_entry() {
        None => vec![Line::from(Span::styled(
            "nothing selected",
            Style::default().fg(Color::DarkGray),
        ))],
        Some(entry) => {
            let mut lines = Vec::new();

            let label =
                |s: &str| Span::styled(format!("{:<12}", s), Style::default().fg(t.preview_label));
            let value = |s: String| Span::styled(s, Style::default().fg(t.preview_value));

            // name
            lines.push(Line::from(vec![label("name"), value(entry.name.clone())]));
            lines.push(Line::from(""));

            // type
            let kind = if entry.is_broken {
                "broken link".to_string()
            } else if entry.is_dir {
                "directory".to_string()
            } else {
                entry
                    .name
                    .rsplit('.')
                    .next()
                    .map(|ext| format!(".{} file", ext))
                    .unwrap_or_else(|| "file".to_string())
            };
            lines.push(Line::from(vec![label("type"), value(kind)]));

            // size
            if let Some(size) = entry.size {
                let size_str = human_bytes(size as f64).replace("666", "between 665 and 667");
                lines.push(Line::from(vec![label("size"), value(size_str)]));
            }

            lines.push(Line::from(""));

            // dates
            lines.push(Line::from(vec![
                label("modified"),
                value(format_time(entry.modified)),
            ]));
            lines.push(Line::from(vec![
                label("created"),
                value(format_time(entry.created)),
            ]));
            lines.push(Line::from(vec![
                label("accessed"),
                value(format_time(entry.accessed)),
            ]));

            // owner
            if let Some(owner) = &entry.owner {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![label("owner"), value(owner.clone())]));
            }

            lines
        }
    };

    let preview = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.border))
                .title(" preview ")
                .title_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(preview, area);
}

pub fn render_statusbar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let t = &app.theme;

    let content = if let Some(msg) = &app.message {
        Line::from(Span::styled(
            format!(" {}", msg),
            Style::default().fg(Color::Yellow),
        ))
    } else {
        match &app.mode {
            Mode::OpenWith(input) => Line::from(vec![
                Span::styled(" open with: ", Style::default().fg(t.preview_label)),
                Span::styled(input.clone(), Style::default().fg(Color::White)),
                Span::styled("", Style::default().fg(Color::White)),
            ]),
            Mode::ChangeTheme(input) => Line::from(vec![
                Span::styled(" change theme: ", Style::default().fg(t.preview_label)),
                Span::styled(input.to_string(), Style::default().fg(Color::White)),
                Span::styled("", Style::default().fg(Color::White)),
            ]),
            Mode::Command(input) => Line::from(vec![
                Span::styled("cmd: ", Style::default().fg(t.preview_label)),
                Span::styled(input.clone(), Style::default().fg(Color::White)),
                Span::styled("", Style::default().fg(Color::White)),
            ]),
            Mode::ConfirmDelete => {
                let count = app.get_targets().len();
                Line::from(Span::styled(
                    format!(" delete {} item(s)? (y/n)", count),
                    Style::default().fg(Color::Red),
                ))
            }
            Mode::Normal => {
                if let Some(msg) = &app.message {
                    Line::from(vec![Span::styled(
                        format!(" {}", msg),
                        Style::default().fg(Color::Yellow),
                    )])
                } else {
                    let hidden_hint = if app.show_hidden {
                        "  [H: hidden on]"
                    } else {
                        "  [H: hidden off]"
                    };
                    let keyhints = "  j/k: move  g/G: top/bot  ctrl+p: pin  o: open with d: delete ctrl+b: paste x: unpin q: quit";
                    Line::from(vec![Span::styled(
                        format!("{}{}", hidden_hint, keyhints),
                        Style::default().fg(t.status_fg),
                    )])
                }
            }
            Mode::NewFile(input) => Line::from(vec![
                Span::styled(" new file: ", Style::default().fg(t.preview_label)),
                Span::styled(input.clone(), Style::default().fg(Color::White)),
                Span::styled("█", Style::default().fg(Color::White)),
            ]),
            Mode::NewFolder(input) => Line::from(vec![
                Span::styled(" new folder: ", Style::default().fg(t.preview_label)),
                Span::styled(input.clone(), Style::default().fg(Color::White)),
                Span::styled("█", Style::default().fg(Color::White)),
            ]),
        }
    };

    let bar = Paragraph::new(content).style(Style::default().bg(t.status_bg));

    f.render_widget(bar, area);
}

fn format_time(t: Option<std::time::SystemTime>) -> String {
    match t {
        None => "-".to_string(),
        Some(t) => {
            match t.duration_since(UNIX_EPOCH) {
                Ok(d) => {
                    let secs = d.as_secs();
                    //could swap to chrono, its prettier
                    let mins = secs / 60 % 60;
                    let hours = secs / 3600 % 24;
                    let days = secs / 86400;
                    let year = 1970 + days / 365;
                    let month = (days % 365) / 30 + 1;
                    let day = (days % 365) % 30 + 1;
                    format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}",
                        year, month, day, hours, mins
                    )
                }
                Err(_) => "-".to_string(),
            }
        }
    }
}
