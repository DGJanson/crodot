// Component for handling multiline text input

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

#[derive(Default, Clone)]
pub struct TextArea {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn reset(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_text(&mut self, text: &str) {
        if text.is_empty() {
            self.lines = vec![String::new()];
        } else {
            self.lines = text.lines().map(|s| s.to_string()).collect();
            if self.lines.is_empty() {
                self.lines.push(String::new());
            }
        }
        self.clamp_cursor();
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ensure we always have at least one line
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        match (key.code, key.modifiers) {
            (KeyCode::Left, KeyModifiers::CONTROL) => {
                self.move_word_left();
            }
            (KeyCode::Right, KeyModifiers::CONTROL) => {
                self.move_word_right();
            }
            (KeyCode::Left, _) => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.lines[self.cursor_row].chars().count();
                }
            }
            (KeyCode::Right, _) => {
                let current_line_len = self.lines[self.cursor_row].chars().count();
                if self.cursor_col < current_line_len {
                    self.cursor_col += 1;
                } else if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                }
            }
            (KeyCode::Up, _) => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.clamp_cursor();
                }
            }
            (KeyCode::Down, _) => {
                if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.clamp_cursor();
                }
            }
            (KeyCode::Backspace, _) => {
                if self.cursor_col > 0 {
                    // Remove character before cursor
                    let line = &mut self.lines[self.cursor_row];
                    let mut chars: Vec<char> = line.chars().collect();
                    if self.cursor_col <= chars.len() {
                        chars.remove(self.cursor_col - 1);
                        self.lines[self.cursor_row] = chars.iter().collect();
                        self.cursor_col -= 1;
                    }
                } else if self.cursor_row > 0 {
                    // Merge with previous line
                    let current_line = self.lines.remove(self.cursor_row);
                    self.cursor_row -= 1;
                    let prev_line_len = self.lines[self.cursor_row].chars().count();
                    self.lines[self.cursor_row].push_str(&current_line);
                    self.cursor_col = prev_line_len;
                }
            }
            (KeyCode::Delete, _) => {
                let line_len = self.lines[self.cursor_row].chars().count();
                if self.cursor_col < line_len {
                    let line = &mut self.lines[self.cursor_row];
                    let mut chars: Vec<char> = line.chars().collect();
                    chars.remove(self.cursor_col);
                    self.lines[self.cursor_row] = chars.iter().collect();
                } else if self.cursor_row + 1 < self.lines.len() {
                    let next_line = self.lines.remove(self.cursor_row + 1);
                    self.lines[self.cursor_row].push_str(&next_line);
                }
            }
            (KeyCode::Enter, _) => {
                let line = &self.lines[self.cursor_row];
                let chars: Vec<char> = line.chars().collect();
                let left_part: String = chars[..self.cursor_col].iter().collect();
                let right_part: String = chars[self.cursor_col..].iter().collect();

                self.lines[self.cursor_row] = left_part;
                self.cursor_row += 1;
                self.lines.insert(self.cursor_row, right_part);
                self.cursor_col = 0;
            }
            (KeyCode::Char(c), _) => {
                let line = &mut self.lines[self.cursor_row];
                let mut chars: Vec<char> = line.chars().collect();
                if self.cursor_col <= chars.len() {
                    chars.insert(self.cursor_col, c);
                    self.lines[self.cursor_row] = chars.iter().collect();
                    self.cursor_col += 1;
                }
            }
            _ => {}
        }
        self.clamp_cursor();
    }

    fn clamp_cursor(&mut self) {
        if self.cursor_row >= self.lines.len() {
            self.cursor_row = self.lines.len().saturating_sub(1);
        }
        let line_len = self.lines[self.cursor_row].chars().count();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    fn move_word_left(&mut self) {
        if self.cursor_col > 0 {
            let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
            let mut idx = self.cursor_col;

            // Skip spaces to the left
            while idx > 0 && chars[idx - 1].is_whitespace() {
                idx -= 1;
            }
            // Skip word characters to the left
            while idx > 0 && !chars[idx - 1].is_whitespace() {
                idx -= 1;
            }
            self.cursor_col = idx;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].chars().count();
        }
    }

    fn move_word_right(&mut self) {
        let chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        let line_len = chars.len();

        if self.cursor_col < line_len {
            let mut idx = self.cursor_col;

            // Skip word characters to the right
            while idx < line_len && !chars[idx].is_whitespace() {
                idx += 1;
            }
            // Skip spaces to the right
            while idx < line_len && chars[idx].is_whitespace() {
                idx += 1;
            }
            self.cursor_col = idx;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    // -----------------------  render function
    pub fn render(&self, frame: &mut Frame, rect: Rect, show_cursor: bool) {
        let mut text_lines: Vec<Line> = Vec::new();

        for (r_idx, line_str) in self.lines.iter().enumerate() {
            if show_cursor && r_idx == self.cursor_row {
                let chars: Vec<char> = line_str.chars().collect();
                let mut spans = Vec::new();

                let left_str: String = chars[..self.cursor_col].iter().collect();
                if !left_str.is_empty() {
                    spans.push(Span::raw(left_str));
                }

                if self.cursor_col < chars.len() {
                    let cursor_char = chars[self.cursor_col].to_string();
                    spans.push(Span::styled(
                        cursor_char,
                        Style::default()
                            .bg(Color::White)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ));
                    let right_str: String = chars[self.cursor_col + 1..].iter().collect();
                    if !right_str.is_empty() {
                        spans.push(Span::raw(right_str));
                    }
                } else {
                    // Cursor is at the end of the line
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(Color::White).fg(Color::Black),
                    ));
                }

                text_lines.push(Line::from(spans));
            } else {
                text_lines.push(Line::from(line_str.clone()));
            }
        }

        frame.render_widget(Paragraph::new(text_lines).wrap(Wrap { trim: false }), rect);
    }
}
