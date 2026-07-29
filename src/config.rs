use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyModifiers};

pub struct KeyBindings {
    overrides: HashMap<String, String>,
    pub theme: Option<String>,
}

fn normalize(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

fn config_path() -> Option<std::path::PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME").ok().map(|h| std::path::PathBuf::from(h).join(".config"))
        })
        .map(|p| p.join("hax").join("config"))
}

impl KeyBindings {
    pub fn load() -> Self {
        let mut overrides = HashMap::new();
        let mut theme = None;
        if let Some(path) = config_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim();
                        if normalize(key) == "theme" {
                            theme = Some(value.to_string());
                        } else {
                            overrides.insert(normalize(key), normalize(value));
                        }
                    }
                }
            }
        }
        KeyBindings { overrides, theme }
    }

    pub fn save_theme(theme_name: &str) {
        let Some(path) = config_path() else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        // find or replace theme= line
        let mut found = false;
        for line in &mut lines {
            if let Some((key, _)) = line.split_once('=') {
                if normalize(key.trim()) == "theme" {
                    *line = format!("theme = {theme_name}");
                    found = true;
                    break;
                }
            }
        }
        if !found {
            lines.push(format!("theme = {theme_name}"));
        }
        let _ = std::fs::write(&path, lines.join("\n") + "\n");
    }

    pub fn lookup(&self, key: &str) -> Option<&str> {
        self.overrides.get(key).map(|s| s.as_str())
    }
}

pub fn key_to_string(code: &KeyCode, mods: KeyModifiers) -> String {
    let prefix = if mods == KeyModifiers::CONTROL {
        "ctrl"
    } else if mods == KeyModifiers::ALT {
        "alt"
    } else if mods == KeyModifiers::SHIFT {
        "shift"
    } else {
        ""
    };
    let raw = match code {
        KeyCode::Char(c) => format!("{prefix}{c}"),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Home => "home".into(),
        KeyCode::End => "end".into(),
        KeyCode::PageUp => "pageup".into(),
        KeyCode::PageDown => "pagedown".into(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Tab => "tab".into(),
        KeyCode::Backspace => "backspace".into(),
        KeyCode::Delete => "delete".into(),
        KeyCode::Esc => "escape".into(),
        _ => format!("{prefix}{code:?}"),
    };
    normalize(&raw)
}

pub fn exec_normal(app: &mut crate::app::App, action: &str) {
    match action {
        "quit" => {
            if app.modified {
                app.mode = crate::app::Mode::ConfirmQuit;
            } else {
                app.quit = true;
            }
        }
        "save" => app.save_file(),
        "open" => {
            app.mode = crate::app::Mode::FileExplorer;
            app.show_sidebar = true;
            app.refresh_file_tree();
        }
        "search" => {
            app.mode = crate::app::Mode::Search;
            app.search_query.clear();
            app.search_results.clear();
        }
        "command_palette" => {
            app.mode = crate::app::Mode::CommandPalette;
            app.command_input.clear();
            app.command_selection = 0;
        }
        "new_file" => app.new_file(),
        "toggle_sidebar" => { app.show_sidebar = !app.show_sidebar; }
        "yank" => app.yank_line(),
        "cut" => app.cut_line(),
        "paste" => app.paste_clipboard(),
        "cursor_left" => {
            if app.cursor_x > 0 {
                app.cursor_x -= 1;
                let line = app.get_line(app.cursor_y);
                app.cursor_byte = line[..app.cursor_byte].chars().next_back().map(|c| app.cursor_byte - c.len_utf8()).unwrap_or(0);
            }
        }
        "cursor_right" => {
            if app.cursor_y < app.line_count() {
                let cc = app.get_line(app.cursor_y).chars().count();
                if app.cursor_x < cc {
                    let line = app.get_line(app.cursor_y);
                    if let Some(c) = line[app.cursor_byte..].chars().next() {
                        app.cursor_x += 1;
                        app.cursor_byte += c.len_utf8();
                    }
                }
            }
        }
        "cursor_up" => {
            if app.cursor_y > 0 {
                app.cursor_y -= 1;
                app.recalc_cursor_byte();
            }
        }
        "cursor_down" => {
            if app.cursor_y + 1 < app.line_count() {
                app.cursor_y += 1;
                app.recalc_cursor_byte();
            }
        }
        "cursor_home" => {
            app.cursor_x = 0;
            app.cursor_byte = 0;
        }
        "cursor_end" => {
            if app.cursor_y < app.line_count() {
                app.cursor_x = app.get_line(app.cursor_y).chars().count();
                app.cursor_byte = app.get_line(app.cursor_y).len();
            }
        }
        "page_up" => {
            app.cursor_y = app.cursor_y.saturating_sub(10);
            app.recalc_cursor_byte();
        }
        "page_down" => {
            app.cursor_y = (app.cursor_y + 10).min(app.line_count().saturating_sub(1));
            app.recalc_cursor_byte();
        }
        "new_line" => app.new_line(),
        "insert_tab" => app.insert_tab(),
        "delete_forward" => app.delete_forward(),
        "delete_backward" => app.delete_char(),
        "escape" => {}  // no-op in normal mode
        _ => {}
    }
}

pub fn is_backspace(code: &KeyCode, mods: KeyModifiers) -> bool {
    *code == KeyCode::Backspace
        || (*code == KeyCode::Char('h') && mods == KeyModifiers::CONTROL)
        || (*code == KeyCode::Char('\x7f'))
}

pub fn exec_file_explorer(app: &mut crate::app::App, action: &str) -> bool {
    match action {
        "up" => {
            if app.file_tree_selection > 0 {
                app.file_tree_selection -= 1;
                if app.file_tree_selection < app.file_tree_offset {
                    app.file_tree_offset = app.file_tree_selection;
                }
            }
            true
        }
        "down" => {
            if app.file_tree_selection + 1 < app.file_tree.len() {
                app.file_tree_selection += 1;
                if app.file_tree_selection >= app.file_tree_offset + app.sidebar_height.max(1) {
                    app.file_tree_offset = app.file_tree_selection.saturating_sub(app.sidebar_height.max(1) - 1);
                }
            }
            true
        }
        "open" => {
            if let Some(path) = app.file_tree.get(app.file_tree_selection).cloned() {
                if path.is_dir() {
                    let _ = std::env::set_current_dir(&path);
                    app.refresh_file_tree();
                    app.file_tree_offset = 0;
                } else {
                    app.open_file(path);
                    app.mode = crate::app::Mode::Normal;
                }
            }
            true
        }
        "rename" => {
            if let Some(path) = app.file_tree.get(app.file_tree_selection) {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                app.rename_input = name;
                app.mode = crate::app::Mode::Rename;
            }
            true
        }
        "parent" => {
            let parent = std::env::current_dir().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
            if let Some(p) = parent {
                let _ = std::env::set_current_dir(&p);
                app.refresh_file_tree();
                app.file_tree_offset = 0;
            }
            true
        }
        "back" => {
            app.mode = crate::app::Mode::Normal;
            true
        }
        _ => false,
    }
}

pub fn exec_search(app: &mut crate::app::App, action: &str) -> bool {
    match action {
        "up" => {
            if app.search_selection > 0 {
                app.search_selection -= 1;
                if app.search_selection < app.search_offset {
                    app.search_offset = app.search_selection;
                }
            }
            true
        }
        "down" => {
            if app.search_selection + 1 < app.search_results.len() {
                app.search_selection += 1;
                if app.search_selection >= app.search_offset + app.overlay_list_height.max(1) {
                    app.search_offset = app.search_selection.saturating_sub(app.overlay_list_height.max(1) - 1);
                }
            }
            true
        }
        "open" => {
            if let Some((path, _, _)) = app.search_results.get(app.search_selection).cloned() {
                app.open_file(path);
                app.mode = crate::app::Mode::Normal;
            }
            true
        }
        "back" => {
            app.mode = crate::app::Mode::Normal;
            app.search_query.clear();
            app.search_results.clear();
            true
        }
        _ => false,
    }
}

pub fn exec_command(app: &mut crate::app::App, action: &str) -> bool {
    match action {
        "up" => {
            if app.command_selection > 0 {
                app.command_selection -= 1;
                if app.command_selection < app.command_offset {
                    app.command_offset = app.command_selection;
                }
            }
            true
        }
        "down" => {
            let filtered = app.get_filtered_commands();
            if app.command_selection + 1 < filtered.len() {
                app.command_selection += 1;
                if app.command_selection >= app.command_offset + app.overlay_list_height.max(1) {
                    app.command_offset = app.command_selection.saturating_sub(app.overlay_list_height.max(1) - 1);
                }
            }
            true
        }
        "execute" => {
            let filtered = app.get_filtered_commands();
            if let Some(&(idx, _)) = filtered.get(app.command_selection) {
                let cmd = app.command_items[idx].clone();
                app.execute_command(&cmd);
            }
            true
        }
        "back" => {
            app.mode = crate::app::Mode::Normal;
            app.command_input.clear();
            true
        }
        _ => false,
    }
}
