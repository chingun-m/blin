use crate::theme;
use crate::{App, Focus, Mode};
use crossterm::event::{KeyCode, KeyModifiers};

pub fn handle_filelist_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    if let Mode::NewFile(ref mut input) = app.mode {
        match code {
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Enter => {
                let name = input.clone();
                app.create_item(&name, false);
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        }
        return;
    }
    if let Mode::NewFolder(ref mut input) = app.mode {
        match code {
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Enter => {
                let name = input.clone();
                app.create_item(&name, true);
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        }
        return;
    }

    if let Mode::ConfirmDelete = app.mode {
        match code {
            KeyCode::Char('y') => {
                app.confirm_delete();
            }
            _ => {
                app.mode = Mode::Normal;
                app.message = Some("delete cancelled".to_string());
            }
        }
        return;
    }

    if let Mode::OpenWith(ref mut input) = app.mode {
        match code {
            KeyCode::Char('c') | KeyCode::Char('x')
                if modifiers.contains(KeyModifiers::CONTROL) =>
            {
                app.message = Some("unable to copy/cut here".to_string());
            }
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Enter => {
                app.execute_open_with();
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        }
        return;
    }

    if let Mode::RunWith(ref mut input) = app.mode {
        match code {
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Enter => {
                app.execute_run_with();
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        }
        return;
    }

    if let Mode::ChangeTheme(ref mut input) = app.mode {
        match code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    *input = digit as usize;
                }
            }
            KeyCode::Backspace => {
                *input = 0;
            }
            KeyCode::Enter => {
                app.change_theme();
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    if let Mode::Command(ref mut input) = app.mode {
        match code {
            KeyCode::Char(c) => {
                input.push(c);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Enter => {
                let raw = input.clone();
                let trimmed = raw.trim();
                let (cmd, arg) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
                let arg = arg.trim();

                match cmd {
                    "newfile" | "nfil" if !arg.is_empty() => {
                        app.create_item(arg, false);
                    }
                    "newfolder" | "nfol" if !arg.is_empty() => {
                        app.create_item(arg, true);
                    }
                    "changetheme" | "ct" if !arg.is_empty() => {
                        if let Ok(idx) = arg.parse::<usize>() {
                            app.theme = theme::Theme::by_index(idx);
                        } else {
                            //if the top fails, it changes the theme to default one.
                            app.theme = theme::Theme::default();
                        }
                    }
                    "selectall" | "sela" => {
                        app.select_all();
                    }
                    "openwith" | "ow" if !arg.is_empty() => {
                        app.open_with(arg);
                    }
                    "z" | "zip" => {
                        app.compress_selected();
                    }
                    "uz" | "unzip" => {
                        app.extract_selected();
                    }
                    "rn" | "rename" if !arg.is_empty() => {
                        app.rename_item(arg);
                    }
                    "r" | "run" if !arg.is_empty() => {
                        app.run_with(arg);
                    }

                    _ => {}
                }
                app.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                app.mode = Mode::Normal;
            }
            _ => {}
        }
        return;
    }
    if let Mode::Search(ref mut input) = app.mode {
        match code {
            KeyCode::Char(c) => {
                input.push(c);
                app.filter = input.clone();
                app.selected_items.clear();
                app.reload();
                app.list_state.select(if app.entries.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
            KeyCode::Backspace => {
                input.pop();
                app.filter = input.clone();
                app.selected_items.clear();
                app.reload();
                app.list_state.select(if app.entries.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
            KeyCode::Enter => {
                app.mode = Mode::Normal;
            } // keep the filter, go back to navigating
            KeyCode::Esc => {
                app.filter.clear();
                app.mode = Mode::Normal;
                app.selected_items.clear();
                app.reload();
            }
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Char('/') => {
            app.mode = Mode::Search(app.filter.clone());
        }
        KeyCode::Char('C') => {
            app.mode = Mode::Command(String::new());
        }

        KeyCode::Char('n') => {
            app.mode = Mode::NewFile(String::new());
        }
        KeyCode::Char('N') => {
            app.mode = Mode::NewFolder(String::new());
        }

        KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_all();
        }

        // navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.jump_buf.clear();
            app.move_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.jump_buf.clear();
            app.move_up();
        }
        KeyCode::Char('g') => {
            app.jump_buf.clear();
            app.jump_to_top();
        }
        KeyCode::Char('G') => {
            app.jump_buf.clear();
            app.jump_to_bottom();
        }
        KeyCode::Char('o') => {
            app.mode = Mode::OpenWith(String::new());
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.copy_selected();
        }
        KeyCode::Char('x') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.cut_selected();
        }

        KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.paste();
        }

        KeyCode::Char('d') => {
            if !app.get_targets().is_empty() {
                app.mode = Mode::ConfirmDelete;
            }
        }

        KeyCode::Char('t') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.mode = Mode::ChangeTheme(1);
        }

        // number buffer for jump-to
        KeyCode::Char(c) if c.is_ascii_digit() => {
            app.jump_buf.push(c);
        }
        KeyCode::Enter if !app.jump_buf.is_empty() => {
            app.commit_jump();
        }

        // enter directory
        KeyCode::Enter => {
            if let Some(entry) = app.selected_entry() {
                if entry.is_dir && !entry.is_broken {
                    let path = entry.path.clone();
                    app.navigate_into(path);
                }
            }
        }

        // go up
        KeyCode::Backspace | KeyCode::Char('h') => {
            app.jump_buf.clear();
            app.go_up();
        }

        // toggle hidden
        KeyCode::Char('H') => {
            app.jump_buf.clear();
            app.toggle_hidden();
        }

        // pin with ctrl+p
        KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.pin_selected();
        }

        // escape clears selection and jump buffer
        KeyCode::Esc => {
            app.clear_selection();
            app.jump_buf.clear();
            if !app.filter.is_empty() {
                app.filter.clear();
                app.reload();
            }
        }

        _ => {}
    }
}

pub fn handle_sidebar_key(app: &mut App, code: KeyCode) {
    let items_len = app.sidebar_items().len();
    match code {
        KeyCode::Char('x') => {
            app.unpin_selected_sidebar();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let next = app
                .sidebar_state
                .selected()
                .map(|i| (i + 1).min(items_len.saturating_sub(1)))
                .unwrap_or(0);
            app.sidebar_state.select(Some(next));
            //skip headers
            let items = app.sidebar_items();
            if let Some(label) = items.get(app.sidebar_state.selected().unwrap_or(0)) {
                if label.starts_with("──") {
                    let skip = (app.sidebar_state.selected().unwrap_or(0) + 1)
                        .min(items_len.saturating_sub(1));
                    app.sidebar_state.select(Some(skip));
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev = app
                .sidebar_state
                .selected()
                .map(|i| i.saturating_sub(1))
                .unwrap_or(0);
            app.sidebar_state.select(Some(prev));
            //skip headers
            let items = app.sidebar_items();
            if let Some(label) = items.get(app.sidebar_state.selected().unwrap_or(0)) {
                if label.starts_with("──") {
                    let skip = (app.sidebar_state.selected().unwrap_or(0) + 1)
                        .min(items_len.saturating_sub(1));
                    app.sidebar_state.select(Some(skip));
                }
            }
        }
        KeyCode::Enter => {
            app.activate_sidebar_item();
        }
        KeyCode::Esc => {
            app.focus = Focus::FileList;
        }
        _ => {}
    }
}
