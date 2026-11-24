use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::style::SetStyle;
use crossterm::terminal;
use human_bytes::human_bytes;
use ratatui::symbols::line::DOUBLE_VERTICAL_RIGHT;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::{default, fs, string};

mod file_ops;
mod input;
mod theme;
mod ui;

//CONFIG - persisted to disk

#[derive(Serialize, Deserialize, Default)]
struct Config {
    pinned: Vec<String>,
    show_hidden: bool,
}

#[derive(Clone)]
enum ClipboardAction {
    Copy,
    Cut,
}

impl Config {
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("blin").join("config.json"))
    }
    fn load() -> Self {
        if let Some(path) = Self::path() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        if let Some(path) = Self::path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(data) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, data);
            }
        }
    }
}

//FILE ENTRY what to store per item in the list
struct FileEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    is_broken: bool,
    size: Option<u64>,
    modified: Option<std::time::SystemTime>,
    created: Option<std::time::SystemTime>,
    accessed: Option<std::time::SystemTime>,
    owner: Option<String>,
}

fn load_entries(dir: &PathBuf, show_hidden: bool) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    if let Ok(read) = fs::read_dir(dir) {
        for entry in read.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !show_hidden && name.starts_with('.') {
                continue;
            }

            let path = entry.path();
            let meta = entry.metadata();

            match meta {
                Ok(m) => {
                    entries.push(FileEntry {
                        name,
                        path,
                        is_dir: m.is_dir(),
                        is_broken: false,
                        size: Some(m.len()),
                        modified: m.modified().ok(),
                        created: m.created().ok(),
                        accessed: m.accessed().ok(),
                        owner: get_owner(&entry),
                    });
                }
                Err(e) => {
                    let is_broken = e.kind() == std::io::ErrorKind::NotFound;
                    entries.push(FileEntry {
                        name,
                        path,
                        is_dir: false,
                        is_broken: true,
                        size: None,
                        modified: None,
                        created: None,
                        accessed: None,
                        owner: None,
                    });
                    let _ = is_broken; //suppress warning, used in redndering
                }
            }
        }
    }
    //dirs first then files, both sorted alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    entries
}

fn get_owner(_entry: &fs::DirEntry) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = _entry.metadata() {
            return Some(format!("uid:{}", meta.uid()));
        }
    }
    None
}

//FOCUS - which panel is active
#[derive(PartialEq)]
pub enum Focus {
    Sidebar,
    FileList,
}

//APP STATE
pub struct App {
    current_dir: PathBuf,
    entries: Vec<FileEntry>,
    list_state: ListState,
    sidebar_state: ListState,
    focus: Focus,
    config: Config,
    drives: Vec<PathBuf>, //C:\ D:\ on windows and mount points on unix
    jump_buf: String,     //buffer for numbr + enter jumps
    show_hidden: bool,
    theme: theme::Theme,
    mode: Mode,
    selected_items: Vec<usize>,
    clipboard: Vec<PathBuf>,
    clipboard_action: Option<ClipboardAction>,
    message: Option<String>,
    filter: String,
}

impl App {
    fn new() -> io::Result<Self> {
        let config = Config::load();
        let show_hidden = config.show_hidden;
        let current_dir = std::env::current_dir()?;
        let entries = load_entries(&current_dir, show_hidden);
        let drives = get_drives();

        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }

        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        Ok(App {
            current_dir,
            entries,
            list_state,
            sidebar_state,
            focus: Focus::FileList,
            config,
            drives,
            jump_buf: String::new(),
            show_hidden,
            theme: theme::Theme::by_index(1),
            mode: Mode::Normal,
            selected_items: Vec::new(),
            clipboard: Vec::new(),
            clipboard_action: None,
            message: None,
            filter: String::new(),
        })
    }

    fn compress_selected(&mut self) {
        let targets = self.get_targets();
        if targets.is_empty() {
            return;
        }

        // one item: name it after the item. multiple: archive.zip
        let base = if targets.len() == 1 {
            let stem = targets[0]
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "archive".to_string());
            format!("{}.zip", stem)
        } else {
            "archive.zip".to_string()
        };

        let dest = file_ops::unique_dest(&self.current_dir, std::ffi::OsStr::new(&base));

        match file_ops::create_zip(&targets, &dest) {
            Ok(_) => {
                let name = dest
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                self.message = Some(format!("created '{}'", name));
            }
            Err(e) => {
                // don't leave a broken half-written zip behind
                let _ = fs::remove_file(&dest);
                self.message = Some(format!("zip failed: {}", e));
            }
        }

        self.selected_items.clear();
        self.reload();
    }

    fn extract_selected(&mut self) {
        let entry = match self.selected_entry() {
            Some(e) => e,
            None => return,
        };

        let is_zip = entry
            .path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase() == "zip")
            .unwrap_or(false);

        if entry.is_dir || !is_zip {
            self.message = Some("not a .zip file".to_string());
            return;
        }

        let zip_path = entry.path.clone();

        // folder named after the zip, e.g. photos.zip -> photos/
        let stem = zip_path
            .file_stem()
            .map(|s| s.to_os_string())
            .unwrap_or_else(|| std::ffi::OsString::from("extracted"));
        let dest = file_ops::unique_dest(&self.current_dir, &stem);

        match file_ops::extract_zip(&zip_path, &dest) {
            Ok(_) => {
                let name = dest
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                self.message = Some(format!("extracted to '{}/'", name));
            }
            Err(e) => {
                // clean up the half-extracted folder
                let _ = fs::remove_dir_all(&dest);
                self.message = Some(format!("unzip failed: {}", e));
            }
        }

        self.reload();
    }

    fn reload(&mut self) {
        self.entries = load_entries(&self.current_dir, self.show_hidden);
        //keeping selection in bounds
        if !self.filter.is_empty() {
            let needle = self.filter.to_lowercase();
            self.entries
                .retain(|e| e.name.to_lowercase().contains(&needle));
        }

        let sel = self.list_state.selected().unwrap_or(0);
        if self.entries.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state
                .select(Some(sel.min(self.entries.len() - 1)));
        }
    }

    fn navigate_into(&mut self, path: PathBuf) {
        self.current_dir = path;
        self.reload();
        self.list_state.select(if self.entries.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    fn go_up(&mut self) {
        if self.current_dir.pop() {
            self.reload();
            self.list_state.select(if self.entries.is_empty() {
                None
            } else {
                Some(0)
            });
        }
    }

    // fn selected_entry(&self) -> Option<&FileEntry>{
    //    self.list_state.selected().and_then(|i| self.entries.get(i))
    // }

    fn move_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let next = self
            .list_state
            .selected()
            .map(|i| (i + 1).min(self.entries.len() - 1))
            .unwrap_or(0);
        self.list_state.select(Some(next));
    }

    fn move_up(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let prev = self
            .list_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.list_state.select(Some(prev));
    }

    fn jump_to_top(&mut self) {
        if !self.entries.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn jump_to_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.list_state.select(Some(self.entries.len() - 1));
        }
    }

    fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.config.show_hidden = self.show_hidden;
        self.config.save();
        self.reload();
    }

    fn pin_selected(&mut self) {
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir {
                let path_str = entry.path.to_string_lossy().to_string();
                if !self.config.pinned.contains(&path_str) {
                    self.config.pinned.push(path_str);
                    self.config.save();
                }
            }
        }
    }

    fn unpin_selected_sidebar(&mut self) {
        let sel = match self.sidebar_state.selected() {
            Some(s) => s,
            None => return,
        };

        // layout: index 0 is the "Pinned" header, then the pinned folders
        let pinned_start = 1;
        let pinned_end = pinned_start + self.config.pinned.len();

        // only works if the cursor is on a pinned folder (not a header or a drive)
        if sel >= pinned_start && sel < pinned_end {
            let removed = self.config.pinned.remove(sel - pinned_start);
            self.config.save();

            // the list got shorter, so keep the cursor in bounds
            let new_len = self.sidebar_items().len();
            self.sidebar_state
                .select(Some(sel.min(new_len.saturating_sub(1))));

            let name = PathBuf::from(&removed)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(removed);
            self.message = Some(format!("unpinned '{}'", name));
        }
    }

    fn commit_jump(&mut self) {
        if let Ok(n) = self.jump_buf.parse::<usize>() {
            if n > 0 && n <= self.entries.len() {
                self.list_state.select(Some(n - 1));
            }
        }
        self.jump_buf.clear();
    }

    //sidebar items, pinned folders, drives etc
    fn sidebar_items(&self) -> Vec<String> {
        let mut items = Vec::new();
        items.push("-Pinned-".to_string());
        for p in &self.config.pinned {
            let label = PathBuf::from(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.clone());
            items.push(format!(" 📌 {}", label));
        }
        items.push("-This PC-".to_string());
        for d in &self.drives {
            items.push(format!(" 💾 {}", d.display()));
        }
        items
    }

    fn activate_sidebar_item(&mut self) {
        let sel = match self.sidebar_state.selected() {
            Some(s) => s,
            None => return,
        };
        let items = self.sidebar_items();
        let label = match items.get(sel) {
            Some(l) => l.clone(),
            None => return,
        };

        //skip headers
        if label.starts_with("-") {
            return;
        }

        //pinned or drive?
        let pinned_start = 1;
        let pinned_end = pinned_start + self.config.pinned.len();
        let drives_start = pinned_end + 1;

        if sel >= pinned_start && sel < pinned_end {
            let path = PathBuf::from(&self.config.pinned[sel - pinned_start]);
            self.navigate_into(path);
        } else if sel >= drives_start {
            let drive = self.drives[sel - drives_start].clone();
            self.navigate_into(drive);
        }
        self.focus = Focus::FileList;
    }

    fn select_all(&mut self) {
        self.selected_items = (0..self.entries.len()).collect();
    }
    fn clear_selection(&mut self) {
        self.selected_items.clear();
    }

    fn copy_selected(&mut self) {
        self.clipboard = self.get_targets();
        self.clipboard_action = Some(ClipboardAction::Copy);
        self.message = Some(format!("copied {} item(s)", self.clipboard.len()));
    }

    fn cut_selected(&mut self) {
        self.clipboard = self.get_targets();
        self.clipboard_action = Some(ClipboardAction::Cut);
        self.message = Some(format!("cut {} item(s)", self.clipboard.len()));
    }

    //if items are multi-selected use those, otherwise use the highlighted one
    fn get_targets(&self) -> Vec<PathBuf> {
        if !self.selected_items.is_empty() {
            self.selected_items
                .iter()
                .filter_map(|&i| self.entries.get(i))
                .map(|e| e.path.clone())
                .collect()
        } else {
            self.selected_entry()
                .map(|e| vec![e.path.clone()])
                .unwrap_or_default()
        }
    }

    fn paste(&mut self) {
        let action = match &self.clipboard_action {
            Some(a) => a.clone(),
            None => return, // nothing copied or cut yet
        };

        for src in self.clipboard.clone() {
            let name = match src.file_name() {
                Some(n) => n.to_owned(),
                None => continue,
            };
            let dest = file_ops::unique_dest(&self.current_dir, &name);

            let result = if src.is_dir() {
                file_ops::copy_dir_recursive(&src, &dest)
            } else {
                fs::copy(&src, &dest).map(|_| ())
            };
            //for cut, delete the original only if the copysucceeded
            if result.is_ok() {
                if let ClipboardAction::Cut = action {
                    if src.is_dir() {
                        let _ = fs::remove_dir_all(&src);
                    } else {
                        let _ = fs::remove_file(&src);
                    }
                }
            }
        }
        //after a cut-paste the originals are gone, so clear the clipboard
        if let ClipboardAction::Cut = action {
            self.clipboard.clear();
            self.clipboard_action = None;
        }

        self.selected_items.clear();
        self.reload();
    }

    fn confirm_delete(&mut self) {
        let targets = self.get_targets();
        let count = targets.len();

        match trash::delete_all(&targets) {
            Ok(_) => self.message = Some(format!("moved {} item(s) to recycle bin", count)),
            Err(e) => self.message = Some(format!("delete failed: {}", e)),
        }

        self.selected_items.clear();
        self.mode = Mode::Normal;
        self.reload();
    }

    // 1. Helper to get the currently selected entry cleanly
    fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.list_state.selected().unwrap_or(0))
    }

    // 2. A unified, flexible function that can open the selected entry with ANY command string
    fn open_with(&mut self, command_line: &str) {
        let parts: Vec<&str> = command_line.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        if let Some(entry) = self.selected_entry() {
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("cmd")
                    .arg("/C")
                    .args(&parts)
                    .arg(&entry.path)
                    .spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new(parts[0])
                    .args(&parts[1..])
                    .arg(&entry.path)
                    .spawn();
            }
        }
    }

    fn change_theme(&mut self) {
        if let Mode::ChangeTheme(input) = self.mode.clone() {
            self.theme = theme::Theme::by_index(input);
        }
        self.mode = Mode::Normal;
    }

    // 3. Triggered by your TUI input mode (e.g., typing a command)
    fn execute_open_with(&mut self) {
        if let Mode::OpenWith(input) = self.mode.clone() {
            self.open_with(&input);
        }
        self.mode = Mode::Normal;
    }

    fn create_item(&mut self, name: &str, is_folder: bool) {
        let name = name.trim();
        if name.is_empty() {
            self.mode = Mode::Normal;
            return;
        }

        let path = self.current_dir.join(name);

        if path.exists() {
            self.message = Some(format!("'{}' already exists", name));
            self.mode = Mode::Normal;
            return;
        }

        let result = if is_folder {
            fs::create_dir_all(&path)
        } else {
            fs::File::create(&path).map(|_| ())
        };

        match result {
            Ok(_) => self.message = Some(format!("created '{}'", name)),
            Err(e) => self.message = Some(format!("couldn't create: {}", e)),
        }

        self.mode = Mode::Normal;
        self.reload();
    }

    fn rename_item(&mut self, new_name: &str) {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            self.mode = Mode::Normal;
            return;
        }

        let old_path = match self.selected_entry() {
            Some(e) => e.path.clone(),
            None => {
                self.mode = Mode::Normal;
                return;
            }
        };

        let new_path = self.current_dir.join(new_name);

        if new_path == old_path {
            self.mode = Mode::Normal;
            return;
        }

        let old_name = old_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let case_only_change = cfg!(windows) && old_name.to_lowercase() == new_name.to_lowercase();
        if new_path.exists() && !case_only_change {
            self.message = Some(format!("'{}' already exists", new_name));
            self.mode = Mode::Normal;
            return;
        }

        match fs::rename(&old_path, &new_path) {
            Ok(_) => self.message = Some(format!("renamed to '{}'", new_name)),
            Err(e) => self.message = Some(format!("rename failed: {}", e)),
        }

        self.mode = Mode::Normal;
        self.reload();
    }
}

fn get_drives() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        //check A-Z for existing drives
        (b'A'..=b'Z')
            .map(|c| PathBuf::from(format!("{}:\\", c as char)))
            .filter(|p| p.exists())
            .collect()
    }
    #[cfg(not(windows))]
    {
        //common unix mount points
        let candidates = ["/", "/home", "/mnt", "/media", "/tmp"];
        candidates
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect()
    }
}

fn run(app: &mut App) -> io::Result<()> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|f| ui::render(f, app))?;
        if let Event::Key(key) = event::read()? {
            //i think this will help the double key registration issue
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            app.message = None;
            //global quit
            if key.code == KeyCode::Char('q')
                && app.jump_buf.is_empty()
                && matches!(app.mode, Mode::Normal)
            {
                break;
            }

            //tab switches focus
            if key.code == KeyCode::Tab {
                app.focus = match app.focus {
                    Focus::FileList => Focus::Sidebar,
                    Focus::Sidebar => Focus::FileList,
                };
                continue;
            }

            match app.focus {
                Focus::FileList => input::handle_filelist_key(app, key.code, key.modifiers),
                Focus::Sidebar => input::handle_sidebar_key(app, key.code),
            }
        }
    }

    ratatui::restore();
    Ok(())
}

#[derive(Clone)]
pub enum Mode {
    Normal,
    OpenWith(String),
    ChangeTheme(usize),
    ConfirmDelete,
    NewFile(String),
    NewFolder(String),
    Command(String),
    Search(String),
}

fn main() -> io::Result<()> {
    let mut app = App::new()?;
    run(&mut app)
}
