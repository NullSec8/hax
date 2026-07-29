use std::path::PathBuf;
use std::fs;
use crate::theme::EditorTheme;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    FileExplorer,
    Search,
    CommandPalette,
    SaveAs,
    ConfirmQuit,
    Rename,
}

pub struct App {
    pub mode: Mode,
    pub buffer: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cursor_byte: usize,
    pub offset_y: usize,
    pub offset_x: usize,
    pub filename: Option<PathBuf>,
    pub modified: bool,
    pub theme_index: usize,
    pub themes: Vec<EditorTheme>,
    pub file_tree: Vec<PathBuf>,
    pub file_tree_selection: usize,
    pub file_tree_offset: usize,
    pub search_query: String,
    pub search_results: Vec<(PathBuf, usize, String)>,
    pub search_selection: usize,
    pub search_offset: usize,
    pub command_input: String,
    pub command_items: Vec<String>,
    pub command_selection: usize,
    pub command_offset: usize,
    pub saveas_input: String,
    pub rename_input: String,
    pub show_sidebar: bool,
    pub clipboard: String,
    pub keybindings: crate::config::KeyBindings,
    pub status_message: String,
    pub quit: bool,
    // editor area for mouse targeting
    pub editor_x: u16,
    pub editor_y: u16,
    pub line_num_w: u16,
    pub visible_lines: usize,
    pub visible_cols: usize,
    pub sidebar_height: usize,
    pub overlay_list_height: usize,
}

impl App {
    pub fn new() -> Self {
        let themes = crate::theme::get_themes();

        let mut app = App {
            mode: Mode::Normal,
            buffer: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            cursor_byte: 0,
            offset_y: 0,
            offset_x: 0,
            filename: None,
            modified: false,
            theme_index: 0,
            themes,
            file_tree: Vec::new(),
            file_tree_selection: 0,
            file_tree_offset: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selection: 0,
            search_offset: 0,
            command_input: String::new(),
            command_items: vec![
                "save".into(), "quit".into(), "open".into(), "new".into(),
                "theme monokai".into(), "theme dracula".into(), "theme nord".into(),
                "theme onedark".into(), "theme solarizeddark".into(), "theme gruvbox".into(),
                "toggle sidebar".into(), "close".into(),
            ],
            command_selection: 0,
            command_offset: 0,
            saveas_input: String::new(),
            rename_input: String::new(),
            show_sidebar: true,
            clipboard: String::new(),
            keybindings: crate::config::KeyBindings::load(),
            status_message: String::new(),
            quit: false,
            editor_x: 0,
            editor_y: 0,
            line_num_w: 4,
            visible_lines: 0,
            visible_cols: 0,
            sidebar_height: 0,
            overlay_list_height: 0,
        };
        if let Some(ref name) = app.keybindings.theme {
            if let Some(idx) = app.themes.iter().position(|t| t.name.to_lowercase() == name.to_lowercase()) {
                app.theme_index = idx;
            }
        }
        app.refresh_file_tree();
        app
    }

    pub fn current_theme(&self) -> &EditorTheme {
        &self.themes[self.theme_index]
    }

    pub fn recalc_cursor_byte(&mut self) {
        if self.cursor_y < self.buffer.len() {
            let line = &self.buffer[self.cursor_y];
            let cc = line.chars().count();
            if self.cursor_x > cc {
                self.cursor_x = cc;
            }
            self.cursor_byte = line.chars().take(self.cursor_x).map(|c| c.len_utf8()).sum();
        } else {
            self.cursor_byte = 0;
        }
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if path.is_dir() {
            return;
        }
        if self.modified {
            self.status_message = "Save changes before opening another file".into();
            return;
        }
        match fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                self.buffer = if lines.is_empty() { vec![String::new()] } else { lines };
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.cursor_byte = 0;
                self.offset_y = 0;
                self.offset_x = 0;
                self.modified = false;
                let disp = path.display().to_string();
                self.filename = Some(path);
                self.status_message = format!("Opened: {}", disp);
            }
            Err(e) => {
                self.status_message = format!("Error opening file: {}", e);
            }
        }
    }

    pub fn save_file(&mut self) {
        if self.filename.is_none() {
            self.mode = Mode::SaveAs;
            self.saveas_input.clear();
            return;
        }
        self.flush_file();
    }

    pub fn flush_file(&mut self) {
        let path = self.filename.as_ref().unwrap().clone();
        let joined = self.buffer.join("\n");
        let content = if joined.is_empty() { joined } else { joined + "\n" };
        match fs::write(&path, &content) {
            Ok(_) => {
                self.modified = false;
                self.status_message = format!("Saved: {}", path.display());
            }
            Err(e) => {
                self.status_message = format!("Error saving: {}", e);
            }
        }
    }

    pub fn new_file(&mut self) {
        if self.modified {
            self.status_message = "Save changes first".into();
            return;
        }
        self.buffer = vec![String::new()];
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.cursor_byte = 0;
        self.offset_y = 0;
        self.offset_x = 0;
        self.modified = false;
        self.filename = None;
        self.status_message = "New file".into();
    }

    pub fn refresh_file_tree(&mut self) {
        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                paths.push(entry.path());
            }
        }
        paths.sort();
        self.file_tree = paths;
        if !self.file_tree.is_empty() && self.file_tree_selection >= self.file_tree.len() {
            self.file_tree_selection = self.file_tree.len() - 1;
        }
    }

    pub fn do_rename(&mut self, new_name: String) {
        let Some(old_path) = self.file_tree.get(self.file_tree_selection).cloned() else {
            return;
        };
        let parent = old_path.parent().unwrap_or(&std::path::Path::new("."));
        let new_path = parent.join(&new_name);
        if new_path.exists() && new_path != old_path {
            self.status_message = format!("Target exists: {}", new_name);
            return;
        }
        match fs::rename(&old_path, &new_path) {
            Ok(_) => {
                if self.filename.as_deref() == Some(&old_path) {
                    self.filename = Some(new_path);
                }
                self.status_message = format!("Renamed: {}", new_name);
            }
            Err(e) => {
                self.status_message = format!("Rename error: {}", e);
            }
        }
    }

    pub fn do_search(&mut self) {
        self.search_results.clear();
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.clone();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() || path.extension().map(|e| e == "git").unwrap_or(false) {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(&query) {
                            self.search_results.push((path.clone(), i + 1, line.to_string()));
                        }
                    }
                }
            }
        }
        self.search_results.truncate(500);
        self.search_selection = 0;
        self.search_offset = 0;
    }

    pub fn get_filtered_commands(&self) -> Vec<(usize, &str)> {
        if self.command_input.is_empty() {
            return self.command_items.iter().enumerate().map(|(i, s)| (i, s.as_str())).collect();
        }
        let lower = self.command_input.to_lowercase();
        self.command_items.iter().enumerate()
            .filter(|(_, s)| s.contains(&lower))
            .map(|(i, s)| (i, s.as_str()))
            .collect()
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.cursor_y >= self.buffer.len() {
            return;
        }
        let line = &mut self.buffer[self.cursor_y];
        let char_count = line.chars().count();
        if self.cursor_x <= char_count {
            line.insert(self.cursor_byte, ch);
            self.cursor_x += 1;
            self.cursor_byte += ch.len_utf8();
            self.modified = true;
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_x > 0 && self.cursor_y < self.buffer.len() {
            let line = &mut self.buffer[self.cursor_y];
            let prev_byte = line[..self.cursor_byte].chars().next_back().map(|c| self.cursor_byte - c.len_utf8()).unwrap_or(0);
            line.remove(prev_byte);
            self.cursor_x -= 1;
            self.cursor_byte = prev_byte;
            self.modified = true;
        } else if self.cursor_x == 0 && self.cursor_y > 0 && self.cursor_y < self.buffer.len() {
            let removed = self.buffer.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.buffer[self.cursor_y].chars().count();
            self.buffer[self.cursor_y].push_str(&removed);
            self.recalc_cursor_byte();
            self.modified = true;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor_y >= self.buffer.len() {
            return;
        }
        let line_len = self.buffer[self.cursor_y].chars().count();
        if self.cursor_x < line_len {
            self.buffer[self.cursor_y].remove(self.cursor_byte);
            self.modified = true;
        } else if self.cursor_y + 1 < self.buffer.len() {
            let next = self.buffer.remove(self.cursor_y + 1);
            self.buffer[self.cursor_y].push_str(&next);
            self.modified = true;
        }
    }

    pub fn new_line(&mut self) {
        if self.cursor_y >= self.buffer.len() {
            return;
        }
        let line = &mut self.buffer[self.cursor_y];
        let rest = line[self.cursor_byte..].to_string();
        line.truncate(self.cursor_byte);
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.cursor_byte = 0;
        self.buffer.insert(self.cursor_y, rest);
        self.modified = true;
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..4 {
            self.insert_char(' ');
        }
    }

    pub fn yank_line(&mut self) {
        if self.cursor_y < self.buffer.len() {
            self.clipboard = self.buffer[self.cursor_y].clone() + "\n";
            self.status_message = "Yanked line".into();
        }
    }

    pub fn cut_line(&mut self) {
        if self.cursor_y < self.buffer.len() {
            self.clipboard = self.buffer.remove(self.cursor_y) + "\n";
            if self.buffer.is_empty() {
                self.buffer.push(String::new());
            }
            if self.cursor_y >= self.buffer.len() {
                self.cursor_y = self.buffer.len() - 1;
            }
            self.cursor_x = self.cursor_x.min(self.buffer[self.cursor_y].chars().count());
            self.recalc_cursor_byte();
            self.modified = true;
            self.status_message = "Cut line".into();
        }
    }

    pub fn paste_clipboard(&mut self) {
        let data = self.clipboard.clone();
        if data.is_empty() || self.cursor_y >= self.buffer.len() {
            return;
        }
        let pasted: Vec<&str> = data.split('\n').collect();
        if pasted.is_empty() { return; }

        let right = self.buffer[self.cursor_y][self.cursor_byte..].to_string();
        self.buffer[self.cursor_y].truncate(self.cursor_byte);
        self.buffer[self.cursor_y].push_str(pasted[0]);

        let n = pasted.len();
        if n > 1 {
            let mut last = pasted[n - 1].to_string();
            last.push_str(&right);
            for i in 1..n - 1 {
                self.buffer.insert(self.cursor_y + i, pasted[i].to_string());
            }
            self.buffer.insert(self.cursor_y + n - 1, last);
            self.cursor_y = self.cursor_y + n - 1;
            self.cursor_x = pasted[n - 1].chars().count();
        } else {
            self.buffer[self.cursor_y].push_str(&right);
            self.cursor_x += pasted[0].chars().count();
        }
        self.cursor_byte = self.buffer[self.cursor_y]
            .chars().take(self.cursor_x).map(|c| c.len_utf8()).sum();
        self.modified = true;
    }

    pub fn execute_command(&mut self, cmd: &str) {
        match cmd {
            "save" => { self.save_file(); if self.mode == Mode::SaveAs { return; } }
            "quit" => {
                if self.modified {
                    self.mode = Mode::ConfirmQuit;
                    return;
                } else {
                    self.quit = true;
                }
            }
            "open" => {
                self.mode = Mode::FileExplorer;
                self.show_sidebar = true;
                self.refresh_file_tree();
                return;
            }
            "new" => self.new_file(),
            "close" => self.new_file(),
            "toggle sidebar" => self.show_sidebar = !self.show_sidebar,
            _ => {
                if cmd.starts_with("theme ") {
                    let name = &cmd[6..];
                    if let Some(idx) = self.themes.iter().position(|t| t.name.to_lowercase() == name.to_lowercase()) {
                        self.theme_index = idx;
                        self.status_message = format!("Theme: {}", self.themes[idx].name);
                        crate::config::KeyBindings::save_theme(self.themes[idx].name);
                    }
                }
            }
        }
        self.mode = Mode::Normal;
        self.command_input.clear();
    }
}

