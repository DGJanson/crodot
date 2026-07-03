use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerMode {
    DirMode,
    FileMode,
}

pub struct FilePicker {
    pub current_dir: PathBuf,
    pub dirs: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
    pub dir_index: usize,
    pub file_index: usize,
    pub mode: PickerMode,
}

impl FilePicker {
    pub fn new(start: impl Into<PathBuf>) -> Self {
        let mut picker = Self {
            current_dir: start.into(),
            dirs: vec![],
            files: vec![],
            dir_index: 0,
            file_index: 0,
            mode: PickerMode::DirMode,
        };
        picker.refresh();
        picker
    }

    /// Reload directory contents
    pub fn refresh(&mut self) {
        self.dirs.clear();
        self.files.clear();

        // Add parent dir
        if let Some(parent) = self.current_dir.parent() {
            self.dirs.push(parent.to_path_buf());
        }

        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.dirs.push(path);
                } else {
                    self.files.push(path);
                }
            }
        }

        self.dir_index = self.dir_index.min(self.dirs.len().saturating_sub(1));
        self.file_index = self.file_index.min(self.files.len().saturating_sub(1));
    }

    pub fn try_get_file(&self) -> Option<PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        Some(self.files[self.file_index].clone())
    }

    // -------------------------------------------------------------------------
    // Input handling
    // -------------------------------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') => self.move_down(),
            KeyCode::Char('k') => self.move_up(),
            KeyCode::Tab => self.toggle_mode(),
            KeyCode::Char(' ') => self.enter_selection(),
            _ => {}
        }
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            PickerMode::DirMode => PickerMode::FileMode,
            PickerMode::FileMode => PickerMode::DirMode,
        };
    }

    fn move_down(&mut self) {
        match self.mode {
            PickerMode::DirMode => {
                if self.dir_index + 1 < self.dirs.len() {
                    self.dir_index += 1;
                }
            }
            PickerMode::FileMode => {
                if self.file_index + 1 < self.files.len() {
                    self.file_index += 1;
                }
            }
        }
    }

    fn move_up(&mut self) {
        match self.mode {
            PickerMode::DirMode => {
                if self.dir_index > 0 {
                    self.dir_index -= 1;
                }
            }
            PickerMode::FileMode => {
                if self.file_index > 0 {
                    self.file_index -= 1;
                }
            }
        }
    }

    fn enter_selection(&mut self) {
        if self.mode == PickerMode::DirMode {
            if let Some(path) = self.dirs.get(self.dir_index) {
                self.current_dir = path.clone();
                self.refresh();
            }
        }
    }

    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        self.render_dirs(f, chunks[0]);
        self.render_files(f, chunks[1]);
    }

    fn render_dirs(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .dirs
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let name = if i == 0 {
                    ".."
                } else {
                    p.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("<unknown>")
                };

                let style = if self.mode == PickerMode::DirMode && i == self.dir_index {
                    Style::default().fg(Color::White).bg(Color::LightBlue)
                } else if i == self.dir_index {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(name.to_string()).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(format!(" {} ", self.current_dir.to_string_lossy()))
                .borders(Borders::ALL),
        );

        f.render_widget(list, area);
    }

    fn render_files(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("<file>");

                let style = if self.mode == PickerMode::FileMode && i == self.file_index {
                    Style::default().fg(Color::White).bg(Color::LightBlue)
                } else if i == self.file_index {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(name.to_string()).style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default().title("Files").borders(Borders::ALL));

        f.render_widget(list, area);
    }
}
