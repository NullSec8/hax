use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, BorderType, Clear},
};
use crate::app::{App, Mode};

fn char_width(c: char) -> usize {
    let cp = c as u32;
    if cp < 0x20 || (0x7F..0xA0).contains(&cp) { return 0; }
    if cp < 0x1100 { return 1; }
    if cp <= 0x115F || cp == 0x2329 || cp == 0x232A { return 2; }
    if (0x2E80..=0x303E).contains(&cp)
        || (0x3040..=0x309F).contains(&cp)
        || (0x30A0..=0x30FF).contains(&cp)
        || (0x3105..=0x312F).contains(&cp)
        || (0x3130..=0x318E).contains(&cp)
        || (0x3190..=0x31E3).contains(&cp)
        || (0x31F0..=0x321E).contains(&cp)
        || (0x3220..=0x3247).contains(&cp)
        || (0x3250..=0x4DBF).contains(&cp)
        || (0x4E00..=0xA4CF).contains(&cp)
        || (0xA960..=0xA97C).contains(&cp)
        || (0xAC00..=0xD7A3).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0xFE10..=0xFE19).contains(&cp)
        || (0xFE30..=0xFE6F).contains(&cp)
        || (0xFF01..=0xFF60).contains(&cp)
        || (0xFFE0..=0xFFE6).contains(&cp)
        || cp >= 0x20000
    {
        2
    } else {
        1
    }
}

fn string_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    if area.width < 40 || area.height < 10 {
        let text = Text::from("Terminal too small");
        frame.render_widget(Paragraph::new(text).centered(), area);
        return;
    }

    let main_layout = if app.show_sidebar {
        Layout::horizontal([Constraint::Length(24), Constraint::Min(1)])
    } else {
        Layout::horizontal([Constraint::Min(1)])
    };

    let chunks = main_layout.split(area);
    let editor_area = if app.show_sidebar { chunks[1] } else { chunks[0] };

    let line_num_w = app.line_count().to_string().len().max(3) as u16;
    app.editor_x = editor_area.x + line_num_w + 1;
    app.editor_y = editor_area.y;
    app.line_num_w = line_num_w;
    let vert = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);
    let vert_chunks = vert.split(editor_area);
    app.visible_lines = vert_chunks[0].height as usize;
    app.visible_cols = (editor_area.width as usize).saturating_sub(line_num_w as usize + 1);

    if app.show_sidebar {
        app.sidebar_height = chunks[0].height as usize;
    }

    app.overlay_list_height = match app.mode {
        Mode::Search => ((area.height as usize * 60) / 100).saturating_sub(6),
        Mode::CommandPalette => ((area.height as usize * 40) / 100).saturating_sub(6),
        _ => app.overlay_list_height,
    };

    let theme = app.current_theme();

    if app.show_sidebar {
        draw_sidebar(frame, chunks[0], app, theme);
    }

    draw_editor(frame, vert_chunks[0], app, theme);
    draw_status_bar(frame, vert_chunks[1], app, theme);

    match &app.mode {
        Mode::Search => draw_search_overlay(frame, area, app, theme),
        Mode::CommandPalette => draw_command_overlay(frame, area, app, theme),
        Mode::SaveAs => draw_saveas_overlay(frame, area, app, theme),
        Mode::ConfirmQuit => draw_confirm_quit_overlay(frame, area, theme),
        Mode::Rename => draw_rename_overlay(frame, area, app, theme),
        _ => {}
    }
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App, theme: &crate::theme::EditorTheme) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.line_numbers))
        .style(Style::default().bg(theme.sidebar_bg).fg(theme.sidebar_fg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.file_tree.is_empty() {
        return;
    }

    let max_visible = inner.height as usize;

    let items = app.file_tree.iter().enumerate()
        .skip(app.file_tree_offset)
        .take(max_visible)
        .map(|(_, path)| {
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let display = if path.is_dir() {
                format!("  {}/", name)
            } else {
                format!("  {}", name)
            };
            let style = if path.is_dir() {
                Style::default().fg(theme.status_mode)
            } else {
                Style::default().fg(theme.sidebar_fg)
            };
            ListItem::new(display).style(style)
        });

    let selected = app.file_tree_selection.saturating_sub(app.file_tree_offset);
    let list = List::new(items)
        .highlight_style(Style::default().bg(theme.sidebar_selected).fg(theme.fg).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, inner, &mut ratatui::widgets::ListState::default().with_selected(Some(selected)));
}

fn draw_editor(frame: &mut Frame, area: Rect, app: &App, theme: &crate::theme::EditorTheme) {
    let block = Block::default().style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let max_lines = inner.height as usize;
    let line_num_width = app.line_count().to_string().len().max(3);

    for i in 0..max_lines {
        let buf_idx = app.offset_y + i;
        if buf_idx >= app.line_count() {
            break;
        }

        let y = inner.top() + i as u16;
        let is_cursor_line = buf_idx == app.cursor_y;

        if is_cursor_line {
            let rect = Rect::new(inner.x, y, inner.width, 1);
            frame.render_widget(Clear, rect);
        }

        let line_num = format!("{:>width$} ", buf_idx + 1, width = line_num_width);
        let num_style = if is_cursor_line {
            Style::default().fg(theme.line_numbers_active).bg(theme.cursor_line)
        } else {
            Style::default().fg(theme.line_numbers)
        };
        frame.render_widget(
            Paragraph::new(Text::from(Line::from(Span::styled(line_num, num_style)))),
            Rect::new(inner.x, y, line_num_width as u16 + 1, 1),
        );

        let line = app.get_line(buf_idx);
        let visible_start_byte: usize = line.chars().take(app.offset_x).map(|c| c.len_utf8()).sum();
        let visible = if visible_start_byte < line.len() {
            &line[visible_start_byte..]
        } else {
            ""
        };

        let content_style = if is_cursor_line {
            Style::default().bg(theme.cursor_line).fg(theme.fg)
        } else {
            Style::default().fg(theme.fg)
        };

        let max_w = inner.width.saturating_sub(line_num_width as u16 + 1) as usize;
        let display: String = visible.chars()
            .scan(0, |w, c| {
                let cw = char_width(c);
                if *w + cw > max_w { None } else { *w += cw; Some(c) }
            })
            .collect();

        let spans = vec![Span::styled(display, content_style)];
        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(inner.x + line_num_width as u16 + 1, y, max_w as u16, 1),
        );
    }

    if matches!(app.mode, Mode::Normal) {
        let vis_x = app.get_line(app.cursor_y)
            .chars().skip(app.offset_x)
            .take(app.cursor_x.saturating_sub(app.offset_x))
            .map(char_width)
            .sum::<usize>();
        let vis_y = app.cursor_y.saturating_sub(app.offset_y);
        if vis_y < max_lines {
            let cx = inner.x + line_num_width as u16 + 1 + vis_x as u16;
            let cy = inner.top() + vis_y as u16;
            if cx < inner.x + inner.width {
                frame.set_cursor_position((cx, cy));
            }
        }
    }
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &crate::theme::EditorTheme) {
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::FileExplorer => "FILE",
        Mode::Search => "SEARCH",
        Mode::CommandPalette => "CMD",
        Mode::SaveAs => "SAVEAS",
        Mode::ConfirmQuit => "QUIT?",
        Mode::Rename => "RENAME",
    };

    let filename = app.filename.as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "[No Name]".into());

    let modified = if app.modified { " +" } else { "" };
    let cursor = format!("{}:{}", app.cursor_y + 1, app.cursor_x + 1);
    let total = app.line_count();
    let lang = app.filename.as_ref()
        .and_then(|p| p.extension())
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "text".into());
    let theme_name = theme.name;

    let mode_style = Style::default().bg(theme.status_mode).fg(theme.bg).add_modifier(Modifier::BOLD);
    let rest_style = Style::default().bg(theme.status_bg).fg(theme.status_fg);
    let bg_style = Style::default().bg(theme.status_bg);

    let left_spans: Vec<Span> = vec![
        Span::styled(mode_str, mode_style),
        Span::styled(format!(" {} ", filename), rest_style),
        Span::styled(lang, Style::default().bg(theme.status_bg).fg(theme.status_mode)),
        Span::styled(modified, Style::default().bg(theme.status_bg).fg(theme.status_mode)),
    ];

    let right_spans: Vec<Span> = vec![
        Span::styled(format!(" {} ", cursor), Style::default().bg(theme.status_bg).fg(theme.status_fg)),
        Span::styled("|", Style::default().bg(theme.status_bg).fg(theme.line_numbers)),
        Span::styled(format!(" {} ", total), Style::default().bg(theme.status_bg).fg(theme.status_fg)),
        Span::styled("|", Style::default().bg(theme.status_bg).fg(theme.line_numbers)),
        Span::styled(format!(" {} ", theme_name), Style::default().bg(theme.status_bg).fg(theme.status_fg)),
    ];

    let mut spans = left_spans;
    let left_width: usize = spans.iter().map(|s| string_width(s.content.as_ref())).sum();
    let right_width: usize = right_spans.iter().map(|s| string_width(s.content.as_ref())).sum();
    let padding = (area.width as usize).saturating_sub(left_width + right_width);
    spans.push(Span::styled(" ".repeat(padding), rest_style));
    spans.extend(right_spans);

    let msg = if !app.status_message.is_empty() {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&app.status_message, Style::default().fg(theme.status_mode)),
        ])
    } else {
        Line::from(spans)
    };

    frame.render_widget(Paragraph::new(msg).style(bg_style), area);
}

fn draw_search_overlay(frame: &mut Frame, area: Rect, app: &App, theme: &crate::theme::EditorTheme) {
    let overlay = centered_rect(60, 60, area);
    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.status_mode))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, overlay);
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let input = Paragraph::new(app.search_query.as_str())
        .style(Style::default().fg(theme.fg))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.line_numbers)));
    let input_area = Rect::new(inner.x, inner.y, inner.width, 3);
    frame.render_widget(input, input_area);

    let list_height = inner.height.saturating_sub(4);
    let results = app.search_results.iter()
        .skip(app.search_offset)
        .take(list_height as usize)
        .map(|(path, line, text)| {
            let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            ListItem::new(format!("{}:{}: {}", name, line, text))
        });

    let selected = app.search_selection.saturating_sub(app.search_offset);
    let list = List::new(results)
        .highlight_style(Style::default().bg(theme.selection).fg(theme.fg))
        .highlight_symbol("> ");
    let list_area = Rect::new(inner.x, inner.y + 3, inner.width, list_height);
    frame.render_stateful_widget(list, list_area, &mut ratatui::widgets::ListState::default().with_selected(Some(selected)));

    let q_len = app.search_query.len() as u16;
    let cx = (inner.x + 1 + q_len).min(inner.x + inner.width - 1);
    frame.set_cursor_position((cx, inner.y + 1));
}

fn draw_command_overlay(frame: &mut Frame, area: Rect, app: &App, theme: &crate::theme::EditorTheme) {
    let overlay = centered_rect(60, 40, area);
    let block = Block::default()
        .title(" Command Palette ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.status_mode))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, overlay);
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let input = Paragraph::new(app.command_input.as_str())
        .style(Style::default().fg(theme.fg))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.line_numbers)));
    let input_area = Rect::new(inner.x, inner.y, inner.width, 3);
    frame.render_widget(input, input_area);

    let list_height = inner.height.saturating_sub(4);
    let items = app.get_filtered_commands().into_iter()
        .skip(app.command_offset)
        .take(list_height as usize)
        .map(|(_, name)| ListItem::new(name));

    let selected = app.command_selection.saturating_sub(app.command_offset);
    let list = List::new(items)
        .highlight_style(Style::default().bg(theme.selection).fg(theme.fg))
        .highlight_symbol("> ");
    let list_area = Rect::new(inner.x, inner.y + 3, inner.width, list_height);
    frame.render_stateful_widget(list, list_area, &mut ratatui::widgets::ListState::default().with_selected(Some(selected)));

    let q_len = app.command_input.len() as u16;
    let cx = (inner.x + 1 + q_len).min(inner.x + inner.width - 1);
    frame.set_cursor_position((cx, inner.y + 1));
}

fn draw_confirm_quit_overlay(frame: &mut Frame, area: Rect, theme: &crate::theme::EditorTheme) {
    let overlay = centered_rect(40, 20, area);
    let block = Block::default()
        .title(" Unsaved Changes ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.status_mode))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, overlay);
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let text = Paragraph::new("Quit without saving?\n\n  y / Enter  –  Yes\n  n / Esc    –  No")
        .style(Style::default().fg(theme.fg))
        .alignment(Alignment::Center);
    frame.render_widget(text, inner);

    frame.set_cursor_position((inner.x + 1, inner.y + 1));
}

fn draw_saveas_overlay(frame: &mut Frame, area: Rect, app: &App, theme: &crate::theme::EditorTheme) {
    let overlay = centered_rect(50, 20, area);
    let block = Block::default()
        .title(" Save As ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.status_mode))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, overlay);
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let input = Paragraph::new(app.saveas_input.as_str())
        .style(Style::default().fg(theme.fg))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.line_numbers)));
    frame.render_widget(input, inner);

    let q_len = app.saveas_input.len() as u16;
    let cx = (inner.x + 1 + q_len).min(inner.x + inner.width - 1);
    frame.set_cursor_position((cx, inner.y + 1));
}

fn draw_rename_overlay(frame: &mut Frame, area: Rect, app: &App, theme: &crate::theme::EditorTheme) {
    let overlay = centered_rect(50, 20, area);
    let block = Block::default()
        .title(" Rename ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.status_mode))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(Clear, overlay);
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let input = Paragraph::new(app.rename_input.as_str())
        .style(Style::default().fg(theme.fg))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.line_numbers)));
    frame.render_widget(input, inner);

    let q_len = app.rename_input.len() as u16;
    let cx = (inner.x + 1 + q_len).min(inner.x + inner.width - 1);
    frame.set_cursor_position((cx, inner.y + 1));
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Length((r.height * (100 - percent_y)) / 200),
        Constraint::Length((r.height * percent_y) / 100),
        Constraint::Length((r.height * (100 - percent_y)) / 200),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Length((r.width * (100 - percent_x)) / 200),
        Constraint::Length((r.width * percent_x) / 100),
        Constraint::Length((r.width * (100 - percent_x)) / 200),
    ])
    .split(popup_layout[1])[1]
}
