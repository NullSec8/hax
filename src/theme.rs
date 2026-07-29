use ratatui::style::Color;

pub struct EditorTheme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub cursor_line: Color,
    pub selection: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub status_mode: Color,
    pub line_numbers: Color,
    pub line_numbers_active: Color,
    pub sidebar_bg: Color,
    pub sidebar_fg: Color,
    pub sidebar_selected: Color,
}

pub fn get_themes() -> Vec<EditorTheme> {
    vec![monokai(), dracula(), nord(), one_dark(), solarized_dark(), gruvbox()]
}

fn monokai() -> EditorTheme {
    EditorTheme {
        name: "Monokai",
        bg: Color::Rgb(39, 40, 34),
        fg: Color::Rgb(248, 248, 242),
        cursor_line: Color::Rgb(54, 55, 48),
        selection: Color::Rgb(73, 72, 62),
        status_bg: Color::Rgb(33, 34, 28),
        status_fg: Color::Rgb(166, 166, 156),
        status_mode: Color::Rgb(249, 38, 114),
        line_numbers: Color::Rgb(117, 113, 94),
        line_numbers_active: Color::Rgb(248, 248, 242),
        sidebar_bg: Color::Rgb(33, 34, 28),
        sidebar_fg: Color::Rgb(166, 166, 156),
        sidebar_selected: Color::Rgb(54, 55, 48),
    }
}

fn dracula() -> EditorTheme {
    EditorTheme {
        name: "Dracula",
        bg: Color::Rgb(40, 42, 54),
        fg: Color::Rgb(248, 248, 242),
        cursor_line: Color::Rgb(68, 71, 90),
        selection: Color::Rgb(68, 71, 90),
        status_bg: Color::Rgb(33, 34, 44),
        status_fg: Color::Rgb(139, 143, 167),
        status_mode: Color::Rgb(255, 85, 85),
        line_numbers: Color::Rgb(98, 102, 127),
        line_numbers_active: Color::Rgb(248, 248, 242),
        sidebar_bg: Color::Rgb(33, 34, 44),
        sidebar_fg: Color::Rgb(139, 143, 167),
        sidebar_selected: Color::Rgb(68, 71, 90),
    }
}

fn nord() -> EditorTheme {
    EditorTheme {
        name: "Nord",
        bg: Color::Rgb(46, 52, 64),
        fg: Color::Rgb(216, 222, 233),
        cursor_line: Color::Rgb(59, 66, 82),
        selection: Color::Rgb(59, 66, 82),
        status_bg: Color::Rgb(41, 46, 57),
        status_fg: Color::Rgb(148, 158, 178),
        status_mode: Color::Rgb(136, 192, 208),
        line_numbers: Color::Rgb(76, 86, 106),
        line_numbers_active: Color::Rgb(216, 222, 233),
        sidebar_bg: Color::Rgb(41, 46, 57),
        sidebar_fg: Color::Rgb(148, 158, 178),
        sidebar_selected: Color::Rgb(59, 66, 82),
    }
}

fn one_dark() -> EditorTheme {
    EditorTheme {
        name: "OneDark",
        bg: Color::Rgb(40, 44, 52),
        fg: Color::Rgb(171, 178, 191),
        cursor_line: Color::Rgb(44, 48, 57),
        selection: Color::Rgb(62, 68, 81),
        status_bg: Color::Rgb(33, 37, 43),
        status_fg: Color::Rgb(127, 135, 150),
        status_mode: Color::Rgb(224, 108, 117),
        line_numbers: Color::Rgb(76, 82, 99),
        line_numbers_active: Color::Rgb(171, 178, 191),
        sidebar_bg: Color::Rgb(33, 37, 43),
        sidebar_fg: Color::Rgb(127, 135, 150),
        sidebar_selected: Color::Rgb(54, 58, 69),
    }
}

fn solarized_dark() -> EditorTheme {
    EditorTheme {
        name: "SolarizedDark",
        bg: Color::Rgb(0, 43, 54),
        fg: Color::Rgb(147, 161, 161),
        cursor_line: Color::Rgb(7, 54, 66),
        selection: Color::Rgb(0, 53, 66),
        status_bg: Color::Rgb(7, 54, 66),
        status_fg: Color::Rgb(88, 110, 117),
        status_mode: Color::Rgb(220, 50, 47),
        line_numbers: Color::Rgb(88, 110, 117),
        line_numbers_active: Color::Rgb(147, 161, 161),
        sidebar_bg: Color::Rgb(7, 54, 66),
        sidebar_fg: Color::Rgb(88, 110, 117),
        sidebar_selected: Color::Rgb(0, 53, 66),
    }
}

fn gruvbox() -> EditorTheme {
    EditorTheme {
        name: "Gruvbox",
        bg: Color::Rgb(40, 40, 40),
        fg: Color::Rgb(235, 219, 178),
        cursor_line: Color::Rgb(50, 50, 50),
        selection: Color::Rgb(60, 56, 54),
        status_bg: Color::Rgb(28, 28, 28),
        status_fg: Color::Rgb(146, 131, 116),
        status_mode: Color::Rgb(251, 73, 52),
        line_numbers: Color::Rgb(102, 92, 84),
        line_numbers_active: Color::Rgb(235, 219, 178),
        sidebar_bg: Color::Rgb(28, 28, 28),
        sidebar_fg: Color::Rgb(146, 131, 116),
        sidebar_selected: Color::Rgb(50, 50, 50),
    }
}
