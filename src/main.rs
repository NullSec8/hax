mod app;
mod config;
mod theme;
mod ui;

use app::{App, Mode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> io::Result<()> {
    // disable flow control so Ctrl+S / Ctrl+Q aren't intercepted
    let _ = std::process::Command::new("stty").args(["-ixon"]).status();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // open file from CLI arg: hax <filename>
    if let Some(path) = std::env::args().nth(1) {
        app.open_file(std::path::PathBuf::from(&path));
    }

    while !app.quit {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        handle_events(&mut app)?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    let _ = std::process::Command::new("stty").args(["ixon"]).status();
    terminal.show_cursor()?;
    Ok(())
}

fn handle_events(app: &mut App) -> io::Result<()> {
    // process all queued events each frame for instant response
    loop {
        if !event::poll(std::time::Duration::from_millis(5))? {
            break;
        }
        let evt = event::read()?;
        if let Event::Mouse(mouse) = evt {
            handle_mouse(app, mouse);
            continue;
        }
        if let Event::Key(key) = evt {
            let is_bksp = key.code == KeyCode::Backspace
                || (key.code == KeyCode::Char('h') && key.modifiers == KeyModifiers::CONTROL)
                || (key.code == KeyCode::Char('\x7f'));
            match app.mode {
                Mode::Normal => {
                    handle_normal_mode(app, key.code, key.modifiers);
                }
                Mode::FileExplorer => {
                    let key_str = config::key_to_string(&key.code, key.modifiers);
                    let action = app.keybindings.lookup(&key_str).map(|s| s.to_string());
                    if let Some(ref a) = action {
                        config::exec_file_explorer(app, a);
                    } else if is_bksp || key.code == KeyCode::Esc {
                        app.mode = Mode::Normal;
                    } else {
                        handle_file_explorer(app, key.code);
                    }
                }
                Mode::Search => {
                    let key_str = config::key_to_string(&key.code, key.modifiers);
                    let action = app.keybindings.lookup(&key_str).map(|s| s.to_string());
                    if let Some(ref a) = action {
                        config::exec_search(app, a);
                    } else if key.code == KeyCode::Esc {
                        app.mode = Mode::Normal;
                        app.search_query.clear();
                        app.search_results.clear();
                    } else if is_bksp {
                        app.search_query.pop();
                        app.do_search();
                    } else {
                        handle_search_mode(app, key.code);
                    }
                }
                Mode::CommandPalette => {
                    let key_str = config::key_to_string(&key.code, key.modifiers);
                    let action = app.keybindings.lookup(&key_str).map(|s| s.to_string());
                    if let Some(ref a) = action {
                        config::exec_command(app, a);
                    } else if key.code == KeyCode::Esc {
                        app.mode = Mode::Normal;
                        app.command_input.clear();
                    } else if is_bksp {
                        app.command_input.pop();
                    } else {
                        handle_command_mode(app, key.code);
                    }
                }
                Mode::ConfirmQuit => {
                    match key.code {
                        KeyCode::Char('y' | 'Y') | KeyCode::Enter => app.quit = true,
                        KeyCode::Char('n' | 'N') | KeyCode::Esc => app.mode = Mode::Normal,
                        _ => {}
                    }
                }
                Mode::SaveAs => {
                    if key.code == KeyCode::Esc {
                        app.mode = Mode::Normal;
                        app.saveas_input.clear();
                    } else if is_bksp {
                        app.saveas_input.pop();
                    } else if key.code == KeyCode::Enter {
                        let name = app.saveas_input.trim().to_string();
                        if !name.is_empty() {
                            app.filename = Some(std::path::PathBuf::from(&name));
                            app.flush_file();
                        }
                        app.mode = Mode::Normal;
                    } else if let KeyCode::Char(c) = key.code {
                        app.saveas_input.push(c);
                    }
                }
                Mode::Rename => {
                    if key.code == KeyCode::Esc {
                        app.mode = Mode::FileExplorer;
                        app.rename_input.clear();
                    } else if is_bksp {
                        app.rename_input.pop();
                    } else if key.code == KeyCode::Enter {
                        let name = app.rename_input.trim().to_string();
                        if !name.is_empty() {
                            app.do_rename(name);
                        }
                        app.refresh_file_tree();
                        app.mode = Mode::FileExplorer;
                    } else if let KeyCode::Char(c) = key.code {
                        app.rename_input.push(c);
                    }
                }
            }
        }
    }
    Ok(())
}


fn handle_normal_mode(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    app.status_message.clear();
    let key_str = config::key_to_string(&code, mods);
    let action = app.keybindings.lookup(&key_str).map(|s| s.to_string());
    if let Some(ref a) = action {
        config::exec_normal(app, a);
    } else if code == KeyCode::Delete {
        app.delete_forward();
    } else if config::is_backspace(&code, mods) {
        app.delete_char();
    } else { match (code, mods) {
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
            if app.modified {
                app.mode = Mode::ConfirmQuit;
            } else {
                app.quit = true;
            }
        }
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => app.save_file(),
        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
            app.mode = Mode::FileExplorer;
            app.show_sidebar = true;
            app.refresh_file_tree();
        }
        (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
            app.mode = Mode::Search;
            app.search_query.clear();
            app.search_results.clear();
        }
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
            app.mode = Mode::CommandPalette;
            app.command_input.clear();
            app.command_selection = 0;
        }
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => app.new_file(),
        (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
            app.show_sidebar = !app.show_sidebar;
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.yank_line(),
        (KeyCode::Char('x'), KeyModifiers::CONTROL) => app.cut_line(),
        (KeyCode::Char('v'), KeyModifiers::CONTROL) => app.paste_clipboard(),
        (KeyCode::Left, _) if app.cursor_x > 0 => app.cursor_x -= 1,
        (KeyCode::Right, _) => {
            if app.cursor_y < app.buffer.len() {
                let cc = app.buffer[app.cursor_y].chars().count();
                if app.cursor_x < cc {
                    app.cursor_x += 1;
                }
            }
        }
        (KeyCode::Up, _) if app.cursor_y > 0 => {
            app.cursor_y -= 1;
            clamp_cursor_x(app);
        }
        (KeyCode::Down, _) => {
            if app.cursor_y + 1 < app.buffer.len() {
                app.cursor_y += 1;
                clamp_cursor_x(app);
            }
        }
        (KeyCode::Home, _) => app.cursor_x = 0,
        (KeyCode::End, _) => {
            if app.cursor_y < app.buffer.len() {
                app.cursor_x = app.buffer[app.cursor_y].chars().count();
            }
        }
        (KeyCode::PageUp, _) => {
            app.cursor_y = app.cursor_y.saturating_sub(10);
            clamp_cursor_x(app);
        }
        (KeyCode::PageDown, _) => {
            app.cursor_y = (app.cursor_y + 10).min(app.buffer.len().saturating_sub(1));
            clamp_cursor_x(app);
        }
        (KeyCode::Enter, _) => app.new_line(),
        (KeyCode::Tab, _) => app.insert_tab(),
        (KeyCode::Char(c), _) => app.insert_char(c),
        _ => {}
    } }

    let vis_lines = app.visible_lines.max(1);
    if app.cursor_y < app.offset_y {
        app.offset_y = app.cursor_y;
    }
    if app.cursor_y >= app.offset_y + vis_lines {
        app.offset_y = app.cursor_y.saturating_sub(vis_lines.saturating_sub(2));
    }
    let vis_cols = app.visible_cols.max(1);
    if app.cursor_x < app.offset_x {
        app.offset_x = app.cursor_x.saturating_sub(5);
    }
    if app.cursor_x > app.offset_x + vis_cols.saturating_sub(1) {
        app.offset_x = app.cursor_x.saturating_sub(vis_cols.saturating_sub(10));
    }
}

fn handle_file_explorer(app: &mut App, code: KeyCode) {
    let sidebar_vis = app.sidebar_height.max(1);
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.file_tree_selection > 0 {
                app.file_tree_selection -= 1;
                if app.file_tree_selection < app.file_tree_offset {
                    app.file_tree_offset = app.file_tree_selection;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.file_tree_selection + 1 < app.file_tree.len() {
                app.file_tree_selection += 1;
                if app.file_tree_selection >= app.file_tree_offset + sidebar_vis {
                    app.file_tree_offset = app.file_tree_selection.saturating_sub(sidebar_vis - 1);
                }
            }
        }
        KeyCode::Enter => {
            if let Some(path) = app.file_tree.get(app.file_tree_selection).cloned() {
                if path.is_dir() {
                    let _ = std::env::set_current_dir(&path);
                    app.refresh_file_tree();
                    app.file_tree_offset = 0;
                } else {
                    app.open_file(path);
                    app.mode = Mode::Normal;
                }
            }
        }
        KeyCode::Char('r') => {
            if let Some(path) = app.file_tree.get(app.file_tree_selection) {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                app.rename_input = name;
                app.mode = Mode::Rename;
            }
        }
        KeyCode::Char('h') => {
            let parent = std::env::current_dir().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
            if let Some(p) = parent {
                let _ = std::env::set_current_dir(&p);
                app.refresh_file_tree();
                app.file_tree_offset = 0;
            }
        }
        _ => {}
    }
}

fn handle_search_mode(app: &mut App, code: KeyCode) {
    let ov_vis = app.overlay_list_height.max(1);
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.search_selection > 0 {
                app.search_selection -= 1;
                if app.search_selection < app.search_offset {
                    app.search_offset = app.search_selection;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.search_selection + 1 < app.search_results.len() {
                app.search_selection += 1;
                if app.search_selection >= app.search_offset + ov_vis {
                    app.search_offset = app.search_selection.saturating_sub(ov_vis - 1);
                }
            }
        }
        KeyCode::Enter => {
            if let Some((path, _, _)) = app.search_results.get(app.search_selection).cloned() {
                app.open_file(path);
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.do_search();
        }
        _ => {}
    }
}

fn handle_command_mode(app: &mut App, code: KeyCode) {
    let ov_vis = app.overlay_list_height.max(1);
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.command_selection > 0 {
                app.command_selection -= 1;
                if app.command_selection < app.command_offset {
                    app.command_offset = app.command_selection;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let filtered = app.get_filtered_commands();
            if app.command_selection + 1 < filtered.len() {
                app.command_selection += 1;
                if app.command_selection >= app.command_offset + ov_vis {
                    app.command_offset = app.command_selection.saturating_sub(ov_vis - 1);
                }
            }
        }
        KeyCode::Enter => {
            let filtered = app.get_filtered_commands();
            if let Some(&(idx, _)) = filtered.get(app.command_selection) {
                let cmd = &app.command_items[idx].clone();
                app.execute_command(cmd);
            }
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
            let filtered_len = app.get_filtered_commands().len();
            app.command_selection = 0;
            app.command_offset = 0;
            if filtered_len == 0 {
                app.command_selection = 0;
            }
        }
        _ => {}
    }
}



fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if app.mode != Mode::Normal {
                return;
            }
            let col = mouse.column;
            let row = mouse.row;
            if col >= app.editor_x && row >= app.editor_y {
                let rel_y = (row - app.editor_y) as usize;
                if rel_y >= app.visible_lines {
                    return;
                }
                let buf_y = rel_y + app.offset_y;
                let buf_x = (col - app.editor_x) as usize + app.offset_x;
                if buf_y < app.buffer.len() {
                    app.cursor_y = buf_y;
                    let line_len = app.buffer[buf_y].chars().count();
                    app.cursor_x = buf_x.min(line_len);
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if app.offset_y > 0 {
                app.offset_y = app.offset_y.saturating_sub(3);
            }
            clamp_cursor_visible(app);
        }
        MouseEventKind::ScrollDown => {
            let max_offset = app.buffer.len().saturating_sub(app.visible_lines);
            app.offset_y = (app.offset_y + 3).min(max_offset);
            clamp_cursor_visible(app);
        }
        _ => {}
    }
}

fn clamp_cursor_visible(app: &mut App) {
    let vis_lines = app.visible_lines.max(1);
    app.cursor_y = app.cursor_y
        .max(app.offset_y)
        .min(app.offset_y + vis_lines - 1);
    if app.cursor_y < app.buffer.len() {
        let line_len = app.buffer[app.cursor_y].chars().count();
        app.cursor_x = app.cursor_x.min(line_len);
    }
}

fn clamp_cursor_x(app: &mut App) {
    if app.cursor_y < app.buffer.len() {
        let cc = app.buffer[app.cursor_y].chars().count();
        if app.cursor_x > cc {
            app.cursor_x = cc;
        }
    }
}
