// Code for handling a coding response
// Will simply render file contents and allow user to update
// the file on disk

use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    AppState, Screen,
    fileutils::coderesponse::{
        ChangeMode, FileChange, normalize_filepath, write_code_response_file,
    },
    tui::{
        layoututils::{get_center_rect, get_inner_rect_pad, horizontal_left_divide_by_fraction},
        utils::{render_main_border, render_menu_shortcut_option},
    },
};

pub struct FileHandlerState {
    mode: FileHandlerMode,
    file_scroll: u16,
    override_filename: String,
}

#[derive(PartialEq)]
enum FileHandlerMode {
    Main,
    Handle, // Write file
    Success,
    FileLoc,
}

pub fn create_file_handler_state() -> FileHandlerState {
    FileHandlerState {
        mode: FileHandlerMode::Main,
        file_scroll: 0,
        override_filename: String::default(),
    }
}

pub fn draw_file_handler_screen(
    frame: &mut Frame,
    state: &mut AppState,
    screen_state: &mut FileHandlerState,
) {
    render_main_border(frame, "Handle code", state);

    // See if there is a file selected
    if state.llm_response_code_response.is_none() {
        frame.render_widget(Paragraph::new("No file selected or no coding response received. Press L to go back to response screen.").wrap(Wrap{trim: true}), get_center_rect(frame.area(), 30, 5));
        return;
    }

    // Now we can render the response
    // We are sure we are in bounds, because above check
    let file_changes = match get_file_change(state) {
        Some(fc) => fc,
        None => {
            frame.render_widget(
                Paragraph::new("Invalid file selected. Press L to go back to response screen.")
                    .wrap(Wrap { trim: true }),
                get_center_rect(frame.area(), 30, 5),
            );
            return;
        }
    };

    let project = match &state.current_project {
        Some(p) => p,
        None => {
            // This should not be possible
            state.error_msg = Some(String::from("No project selected. Returning."));
            state.screen = Screen::Title;
            return;
        }
    };

    let (desc_rect, file_rect) = horizontal_left_divide_by_fraction(frame.area(), 4);

    let block = Block::default().borders(Borders::RIGHT);
    // Assemble lines
    let mut lines = vec![];
    lines.push(Line::from(LEFT_PANEL_PT1));
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Filename: ", Style::default()),
        Span::styled(&file_changes.filename, Style::default().yellow()),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Mode: ", Style::default()),
        Span::styled(format!("{}", &file_changes.mode), Style::default().yellow()),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Description of changes: ", Style::default()),
        Span::styled(&file_changes.description, Style::default()),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(LEFT_PANEL_PT2));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // If edited file exists, check if it was in context. Give warning if not.
    let file_path = normalize_filepath(&project.directory, &file_changes.filename);
    if file_changes.mode == ChangeMode::Edit {
        if !project.file_active_list.contains(&file_path) {
            lines.push(Line::from(Span::styled(
                LEFT_PANEL_FILE_NOT_ADDED_WARNING,
                Style::default().red(),
            )));
        }
    };
    if file_changes.mode == ChangeMode::New {
        if file_path.is_file() {
            lines.push(Line::from(Span::styled(
                LEFT_PANEL_FILE_ALREADY_EXISTS_WARNING,
                Style::default().red(),
            )));
        }
    };

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(block)
            .scroll((screen_state.file_scroll, 0)),
        get_inner_rect_pad(desc_rect, 2, 2),
    );

    frame.render_widget(
        Paragraph::new(file_changes.file_contents.as_str())
            .wrap(Wrap { trim: false })
            .scroll((screen_state.file_scroll, 0)),
        get_inner_rect_pad(file_rect, 2, 2),
    );

    if !screen_state.override_filename.is_empty() {
        frame.render_widget(
            Paragraph::new(" File override active ").red(),
            Rect {
                x: 3,
                y: frame.area().height - 1,
                width: 22,
                height: 1,
            },
        );
    }

    if screen_state.mode == FileHandlerMode::Success {
        let center_rect = get_center_rect(frame.area(), 30, 5);
        frame.render_widget(Clear::default(), center_rect);
        frame.render_widget(Block::bordered(), center_rect);
        frame.render_widget(
            Paragraph::new("File written").green(),
            get_inner_rect_pad(center_rect, 2, 2),
        );
    }

    if screen_state.mode == FileHandlerMode::Handle {
        let center_rect = get_center_rect(frame.area(), 30, 7);
        frame.render_widget(Clear::default(), center_rect);
        frame.render_widget(Block::bordered(), center_rect);
        render_menu_shortcut_option(frame, center_rect, "w", "Write file", 0);
        render_menu_shortcut_option(frame, center_rect, "b", "Back to response", 3);
    }

    if screen_state.mode == FileHandlerMode::FileLoc {
        let center_rect = get_center_rect(frame.area(), frame.area().width - 4, 3);
        frame.render_widget(Clear::default(), center_rect);
        frame.render_widget(Block::bordered(), center_rect);
        frame.render_widget(
            Paragraph::new(screen_state.override_filename.as_str()),
            get_inner_rect_pad(center_rect, 2, 1),
        );
    }
}

pub fn handle_event_filehandler(
    key: KeyEvent,
    state: &mut AppState,
    screen_state: &mut FileHandlerState,
) {
    if key.code == KeyCode::Esc {
        screen_state.mode = FileHandlerMode::Main;
        return;
    }

    match screen_state.mode {
        FileHandlerMode::Main => match key.code {
            KeyCode::Char('j') => screen_state.file_scroll += 3,
            KeyCode::Char('k') => {
                if screen_state.file_scroll < 3 {
                    screen_state.file_scroll = 0;
                } else {
                    screen_state.file_scroll -= 3;
                }
            }
            KeyCode::Char('K') => {
                screen_state.file_scroll = 0;
            }
            KeyCode::Char('L') => {
                state.screen = Screen::LlmResponse;
            }
            KeyCode::Char('h') => screen_state.mode = FileHandlerMode::Handle,

            KeyCode::Char('f') => {
                // override file location
                // if none, init now
                if screen_state.override_filename.is_empty() {
                    if let Some(project) = &state.current_project {
                        let file_change = get_file_change(state);
                        match file_change {
                            Some(fc) => {
                                screen_state.override_filename = String::from(
                                    project
                                        .directory
                                        .join(Path::new(&fc.filename))
                                        .to_string_lossy(),
                                );
                            }
                            None => {
                                screen_state.override_filename =
                                    String::from(project.directory.to_string_lossy());
                            }
                        }
                    }
                }

                screen_state.mode = FileHandlerMode::FileLoc;
            }

            _ => {}
        },
        FileHandlerMode::Handle => match key.code {
            KeyCode::Char('b') => {
                screen_state.file_scroll = 0; // reset scroll
                screen_state.mode = FileHandlerMode::Main;
                state.screen = Screen::LlmResponse;
            }
            KeyCode::Char('w') => {
                // Write response.
                // An error will be shown in app in case anything goes wrong
                if write_code_response_file(state, screen_state.override_filename.as_str()) {
                    screen_state.mode = FileHandlerMode::Success;
                } else {
                    screen_state.mode = FileHandlerMode::Main;
                    screen_state.override_filename = String::default();
                }
            }
            _ => {}
        },
        FileHandlerMode::Success => match key.code {
            // Any key should take us back
            _ => {
                screen_state.file_scroll = 0;
                screen_state.mode = FileHandlerMode::Main;
                screen_state.override_filename = String::default();
                state.screen = Screen::LlmResponse;
            }
        },
        FileHandlerMode::FileLoc => match key.code {
            KeyCode::Backspace => {
                screen_state.override_filename.pop();
            }
            KeyCode::Delete => {
                screen_state.override_filename = String::default();
                screen_state.mode = FileHandlerMode::Main;
            }
            KeyCode::Char(c) => {
                screen_state.override_filename.push(c);
            }
            KeyCode::Enter => {
                screen_state.mode = FileHandlerMode::Main;
            }

            _ => {}
        },
    }
}

// Some helper functions
fn get_file_change(state: &AppState) -> Option<&FileChange> {
    if let Some(code_response) = &state.llm_response_code_response {
        if state.llm_response_code_selected < code_response.code.len() {
            Some(
                code_response
                    .code
                    .get(state.llm_response_code_selected)
                    .unwrap(),
            )
        } else {
            None
        }
    } else {
        None
    }
}

// -------- STATIC TEXTS
const LEFT_PANEL_PT1: &str = "File contents are shown right. Press h key to open handle menu.\n\n";

const LEFT_PANEL_PT2: &str = "PLEASE CHECK DIR CARFEULLY. Press f to override location\n\n";

const LEFT_PANEL_FILE_NOT_ADDED_WARNING: &str =
    "The edited file was NOT added to the context. There is a serious chance of errors!";

const LEFT_PANEL_FILE_ALREADY_EXISTS_WARNING: &str =
    "The file is labeled as new, but it already exists. There is a serious chance of errors!";
