use std::{default, fs, string};
use std::io::{self, Write};
use std::path::PathBuf;
use crossterm::style::SetStyle;
use crossterm::terminal;
use human_bytes::human_bytes;
use ratatui::symbols::line::DOUBLE_VERTICAL_RIGHT;
use std::time::UNIX_EPOCH;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
   layout::{Constraint, Direction, Layout}, 
   style::{Color, Modifier, Style},
   text::{Line, Span},
   widgets::{Block, Borders, List, ListItem, ListState, Paragraph}, 
   Frame,
};
use serde::{Deserialize, Serialize};

//THEME - swap this struct for a different skin later
struct Theme{
   //general
   bg: Color,
   fg: Color,
   //topbar
   topbar_bg: Color,
   topbar_fg: Color,
   //sidebar
   sidebar_bg: Color,
   sidebar_fg: Color,
   sidebar_sel_bg: Color,
   sidebar_sel_fg: Color,
   //file list
   dir_fg: Color,
   file_fg: Color,
   broken_fg: Color,
   selected_bg: Color,
   selected_fg: Color,
   //preview
   preview_label: Color,
   preview_value: Color,
   //status bar
   status_bg: Color,
   status_fg: Color,
   //borders
   border: Color,
   border_focused: Color,
}

impl Theme {
   fn default() -> Self {
      Theme {
         bg:             Color::Reset,
         fg:             Color::White,
         topbar_bg:      Color::Rgb(30, 30, 40),
         topbar_fg:      Color::Rgb(150, 180, 255),
         sidebar_bg:     Color::Rgb(20, 20, 28),
         sidebar_fg:     Color::Gray,
         sidebar_sel_bg: Color::Rgb(50, 50, 80),
         sidebar_sel_fg: Color::White,
         dir_fg:         Color::Rgb(100, 160, 255),
         file_fg:        Color::White,
         broken_fg:      Color::Red,
         selected_bg:    Color::Rgb(50, 50, 80),
         selected_fg:    Color::White,
         preview_label:  Color::Rgb(120, 120, 160),
         preview_value:  Color::White,
         status_bg:      Color::Rgb(30, 30, 40),
         status_fg:      Color::Gray,
         border:         Color::Rgb(60, 60, 80),
         border_focused: Color::Rgb(100, 100, 180),
      }
   }
}

//CONFIG - persisted to disk

#[derive(Serialize, Deserialize, Default)]
 struct Config{
   pinned: Vec<String>,
   show_hidden: bool,
 }

 impl Config{
   fn path() -> Option<PathBuf>{
      dirs::config_dir().map(|d| d.join("blin").join("config.json"))
   }
   fn load() -> Self{
      if let Some(path) = Self::path(){
         if let Ok(data) = fs:: read_to_string(&path){
            if let Ok(cfg) = serde_json::from_str(&data){
               return cfg;
            }
         }
      }
      Self::default()
   }

   fn save(&self){
      if let Some(path) = Self::path(){
         if let Some(parent) = path.parent(){
            let _ = fs::create_dir_all(parent);
         }
         if let Ok(data) = serde_json::to_string_pretty(self){
            let _ = fs::write(path, data);
         }
      }
   }
 }



 //FILE ENTRY what to store per item in the list
 struct FileEntry{
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

 fn load_entries(dir: &PathBuf, show_hidden: bool) -> Vec<FileEntry>{
   let mut entries = Vec::new();
   if let Ok(read) = fs::read_dir(dir){
      for entry in read.flatten(){
         let name = entry.file_name().to_string_lossy().into_owned();
         if !show_hidden && name.starts_with('.'){
            continue;
         }

         let path = entry.path();
         let meta = entry.metadata();

         match meta{
            Ok(m) => {
               entries.push(FileEntry{
                  name,
                  path,
                  is_dir: m.is_dir(),
                  is_broken: false,
                  size: Some(m.len()),
                  modified: m.modified().ok(),
                  created:m.created().ok(),
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
   entries.sort_by(|a, b|{
      match(a.is_dir, b.is_dir){
         (true, false) => std::cmp::Ordering::Less,
         (false, true) => std::cmp::Ordering::Greater,
         _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
      }
   });
   entries 
 }

 
fn get_owner(_entry: &fs::DirEntry) -> Option<String>{
   #[cfg(unix)]
   {
      use std::os::unix::fs::MetadataExt;
      if let Ok(meta) = _entry.metadata(){
         return Some(format!("uid:{}", meta.uid()));
      }
   }
   None
}


fn format_time(t: Option<std::time::SystemTime>) -> String{
   match t {
      None => "-".to_string(),
      Some(t) => {
         match t.duration_since(UNIX_EPOCH){
            Ok(d) => {
               let secs = d.as_secs();
               //could swap to chrono, its prettier
               let mins = secs / 60 % 60;
               let hours = secs / 3600 % 24;
               let days = secs / 86400;
               let year = 1970 + days / 365;
               let month = (days % 365) / 30 + 1;
               let day = (days % 365) % 30 + 1;
               format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hours, mins)
            }
            Err(_) => "-".to_string(),
         }
      }
   }
}

//FOCUS - which panel is active
#[derive(PartialEq)]
enum Focus{
   Sidebar,
   FileList,
}


//APP STATE
struct App{
   current_dir: PathBuf,
   entries: Vec<FileEntry>,
   list_state: ListState,
   sidebar_state: ListState,
   focus: Focus,
   config: Config,
   drives: Vec<PathBuf>, //C:\ D:\ on windows and mount points on unix
   jump_buf: String, //buffer for numbr + enter jumps
   show_hidden: bool,
   theme: Theme,
}


impl  App{
   fn new() -> io::Result<Self>{
      let config = Config::load();
      let show_hidden = config.show_hidden;
      let current_dir = std::env::current_dir()?;
      let entries = load_entries(&current_dir, show_hidden);
      let drives = get_drives();
      
      let mut list_state = ListState::default();
      if !entries.is_empty(){
         list_state.select(Some(0));
      } 

      let mut sidebar_state = ListState::default();
      sidebar_state.select(Some(0));

      Ok(App{
         current_dir,
         entries,
         list_state,
         sidebar_state,
         focus: Focus::FileList,
         config,
         drives,
         jump_buf: String::new(),
         show_hidden, 
         theme: Theme::default(),
      })
   }

   fn reload(&mut self){
      self.entries = load_entries(&self.current_dir, self.show_hidden);
      //keeping selection in bounds
      let sel = self.list_state.selected().unwrap_or(0);
      if self.entries.is_empty(){
         self.list_state.select(None);
      }
      else{
         self.list_state.select(if self.entries.is_empty() {None} else{Some(0)});
      }
   }

   fn navigate_into(&mut self, path: PathBuf){
      self.current_dir = path;
      self.reload();
      self.list_state.select(if self.entries.is_empty() {None} else {Some(0)});
   }

   fn go_up(&mut self){
      if self.current_dir.pop(){
         self.reload();
         self.list_state.select(if self.entries.is_empty() {None} else {Some(0)});
      }
   }

   fn selected_entry(&self) -> Option<&FileEntry>{
      self.list_state.selected().and_then(|i| self.entries.get(i))
   }

   fn move_down(&mut self){
      if self.entries.is_empty() {return; }
      let next = self.list_state.selected()
        .map(|i| (i + 1).min(self.entries.len()-1)).unwrap_or(0);
      self.list_state.select(Some(next));
   }

   fn move_up(&mut self){
      if self.entries.is_empty() {return; }
      let prev = self.list_state.selected()
        .map(|i| i.saturating_sub(1)).unwrap_or(0);
      self.list_state.select(Some(prev));
   }   

   fn jump_to_top(&mut self){
      if !self.entries.is_empty(){
         self.list_state.select(Some(0));
      }
   }

   fn jump_to_bottom(&mut self){
      if !self.entries.is_empty(){
         self.list_state.select(Some(self.entries.len()-1));
      }
   }

   fn toggle_hidden(&mut self){
      self.show_hidden = !self.show_hidden;
      self.config.show_hidden = self.show_hidden;
      self.config.save();
      self.reload();
   }

   fn pin_selected(&mut self){
      if let Some(entry) = self.selected_entry(){
         if entry.is_dir{
            let path_str = entry.path.to_string_lossy().to_string();
            if !self.config.pinned.contains(&path_str){
               self.config.pinned.push(path_str);
               self.config.save();
            }
         }
      }
   }

   fn commit_jump(&mut self){
      if let Ok(n) = self.jump_buf.parse::<usize>(){
         if n > 0 && n <= self.entries.len(){
            self.list_state.select(Some(n-1));
         }
      }
      self.jump_buf.clear();
   }

   //sidebar items, pinned folders, drives etc
   fn sidebar_items(&self) -> Vec<String>{
      let mut items = Vec::new();
      items.push("-Pinned-".to_string());
      for p in &self.config.pinned{
         let label = PathBuf::from(p)
         .file_name()
         .map(|n| n.to_string_lossy().to_string())
         .unwrap_or_else(|| p.clone());
         items.push(format!(" 📌 {}", label));
      }
      items.push("-This PC-".to_string());
      for d in &self.drives{
         items.push(format!(" 💾 {}", d.display()));
      }
      items
   }

   fn activate_sidebar_item(&mut self){
      let sel = match self.sidebar_state.selected() { Some(s) => s, None => return};
      let items = self.sidebar_items();
      let label = match items.get(sel) {Some(l) => l.clone(), None => return};

      //skip headers
      if label.starts_with("-") {return;}

      //pinned or drive?
      let pinned_start = 1;
      let pinned_end = pinned_start + self.config.pinned.len();
      let drives_start = pinned_end + 1;

      if sel >= pinned_start && sel < pinned_end{
         let path = PathBuf::from(&self.config.pinned[sel - pinned_start]);
         self.navigate_into(path);
      }
      else if sel >= drives_start{
         let drive = self.drives[sel - drives_start].clone();
         self.navigate_into(drive);
      }
      self.focus = Focus::FileList;
   }
}

fn get_drives() -> Vec<PathBuf>{
   #[cfg(windows)]{
      //check A-Z for existing drives
      (b'A'..=b'Z')
        .map(|c| PathBuf::from(format!("{}:\\", c as char)))
        .filter(|p| p.exists())
        .collect()
   }
   #[cfg(not(windows))]{
      //common unix mount points
      let candidates = ["/", "/home", "/mnt", "/media", "/tmp"];
      candidates.iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
   }
}

//RENDERING
fn render(f: & mut Frame, app: &mut App){
   let t = &app.theme;
   let area = f.area();

   //outer layout: topbar/body/statusbar
   let outer = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(1), //top bar
      Constraint::Min(0),  //body
      Constraint::Length(1),  //status bar
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

fn render_topbar(f: &mut Frame, app: &App, area: ratatui::layout::Rect){
   let t = &app.theme;
   let bar = Paragraph::new(Line::from(vec![
      Span::styled("blin", Style::default().fg(t.topbar_fg).add_modifier(Modifier::BOLD)),
      Span::styled(
         format!(" - {}", app.current_dir.display()),
         Style::default().fg(Color::DarkGray),
      ),

   ]))
   .style(Style::default().bg(t.topbar_bg));
   f.render_widget(bar, area);
}

fn render_sidebar(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::Sidebar;
    let border_color = if focused { t.border_focused } else { t.border };
 
    let items: Vec<ListItem> = app.sidebar_items().into_iter().map(|label| {
        let is_header = label.starts_with("──");
        let style = if is_header {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(t.sidebar_fg)
        };
        ListItem::new(label).style(style)
    }).collect();
 
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

fn render_file_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::FileList;
    let border_color = if focused { t.border_focused } else { t.border };
 
    let items: Vec<ListItem> = app.entries.iter().enumerate().map(|(i, entry)| {
        let num = format!("{:>3}. ", i + 1);
 
        if entry.is_broken {
            let line = Line::from(vec![
                Span::styled(num, Style::default().fg(Color::DarkGray)),
                Span::styled("!!! ", Style::default().fg(t.broken_fg)),
                Span::styled(&entry.name, Style::default().fg(t.broken_fg)),
                Span::styled("  [BROKEN]", Style::default().fg(Color::DarkGray)),
            ]);
            return ListItem::new(line);
        }
 
        if entry.is_dir {
            let line = Line::from(vec![
                Span::styled(num, Style::default().fg(Color::DarkGray)),
                Span::styled("📁 ", Style::default()),
                Span::styled(format!("{}/", entry.name), Style::default().fg(t.dir_fg).add_modifier(Modifier::BOLD)),
            ]);
            ListItem::new(line)
        } else {
            let size_str = entry.size
                .map(|s| human_bytes(s as f64).replace("666", "between 665 and 667"))
                .unwrap_or_else(|| "—".to_string());
 
            let line = Line::from(vec![
                Span::styled(num, Style::default().fg(Color::DarkGray)),
                Span::styled("📄 ", Style::default()),
                Span::styled(&entry.name, Style::default().fg(t.file_fg)),
                Span::styled(format!("  {}", size_str), Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(line)
        }
    }).collect();
 
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

fn render_preview(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let t = &app.theme;
 
    let content: Vec<Line> = match app.selected_entry() {
        None => vec![Line::from(Span::styled("nothing selected", Style::default().fg(Color::DarkGray)))],
        Some(entry) => {
            let mut lines = Vec::new();
 
            let label = |s: &str| Span::styled(format!("{:<12}", s), Style::default().fg(t.preview_label));
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
                entry.name.rsplit('.').next()
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
            lines.push(Line::from(vec![label("modified"), value(format_time(entry.modified))]));
            lines.push(Line::from(vec![label("created"),  value(format_time(entry.created))]));
            lines.push(Line::from(vec![label("accessed"), value(format_time(entry.accessed))]));
 
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
 
fn render_statusbar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let t = &app.theme;
 
    let hidden_hint = if app.show_hidden { "  [h: hidden on]" } else { "  [h: hidden off]" };
    let keyhints = "  j/k: move  g/G: top/bot  enter: open  backspace: up  tab: sidebar  ctrl+p: pin  q: quit";
 
    let bar = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{}{}", hidden_hint, keyhints),
            Style::default().fg(t.status_fg),
        ),
    ]))
    .style(Style::default().bg(t.status_bg));
 
    f.render_widget(bar, area);
}


fn run(app: &mut App) -> io::Result<()>{
   let mut terminal = ratatui::init();

   loop{
      terminal.draw(|f| render(f, app))?;
      if let Event::Key(key) = event::read()?{

         //i think this will help the double key registration issue
         if key.kind != crossterm::event::KeyEventKind::Press {
            continue;
         } 
         //global quit
         if key.code == KeyCode::Char('q') && app.jump_buf.is_empty(){
            break;
         }

         //tab switches focus
         if key.code == KeyCode::Tab{
            app.focus = match app.focus{
               Focus::FileList => Focus::Sidebar,
               Focus::Sidebar => Focus::FileList,
            };
            continue;
         }

         match app.focus{
            Focus::FileList => handle_filelist_key(app, key.code, key.modifiers),
            Focus::Sidebar => handle_sidebar_key(app, key.code),
         }
      }
   }

   ratatui::restore();
   Ok(())
}


fn handle_filelist_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        // navigation
        KeyCode::Char('j') | KeyCode::Down  => { app.jump_buf.clear(); app.move_down(); }
        KeyCode::Char('k') | KeyCode::Up    => { app.jump_buf.clear(); app.move_up(); }
        KeyCode::Char('g')                  => { app.jump_buf.clear(); app.jump_to_top(); }
        KeyCode::Char('G')                  => { app.jump_buf.clear(); app.jump_to_bottom(); }
 
        // number buffer for jump-to
        KeyCode::Char(c) if c.is_ascii_digit() => { app.jump_buf.push(c); }
        KeyCode::Enter if !app.jump_buf.is_empty() => { app.commit_jump(); }
 
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
        KeyCode::Backspace | KeyCode::Char('h') => { app.jump_buf.clear(); app.go_up(); }
 
        // toggle hidden
        KeyCode::Char('H') => { app.jump_buf.clear(); app.toggle_hidden(); }
 
        // pin with ctrl+p
        KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => { app.pin_selected(); }
 
        // escape clears jump buffer
        KeyCode::Esc => { app.jump_buf.clear(); }
 
        _ => {}
    }
}
 
fn handle_sidebar_key(app: &mut App, code: KeyCode) {
    let items_len = app.sidebar_items().len();
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            let next = app.sidebar_state.selected()
                .map(|i| (i + 1).min(items_len.saturating_sub(1)))
                .unwrap_or(0);
            app.sidebar_state.select(Some(next));
            //skip headers
            let items = app.sidebar_items();
            if let Some(label) = items.get(app.sidebar_state.selected().unwrap_or(0)) {
               if label.starts_with("──") {
                  let skip = (app.sidebar_state.selected().unwrap_or(0) + 1).min(items_len.saturating_sub(1));
                  app.sidebar_state.select(Some(skip));
               }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev = app.sidebar_state.selected()
                .map(|i| i.saturating_sub(1))
                .unwrap_or(0);
            app.sidebar_state.select(Some(prev));
            //skip headers
            let items = app.sidebar_items();
            if let Some(label) = items.get(app.sidebar_state.selected().unwrap_or(0)) {
               if label.starts_with("──") {
                  let skip = (app.sidebar_state.selected().unwrap_or(0) + 1).min(items_len.saturating_sub(1));
                  app.sidebar_state.select(Some(skip));
               }
            }
        }
        KeyCode::Enter => { app.activate_sidebar_item(); }
        KeyCode::Esc   => { app.focus = Focus::FileList; }
        _ => {}
    }
}

fn main() -> io::Result<()>{
   let mut app = App::new()?;
   run(&mut app)
}