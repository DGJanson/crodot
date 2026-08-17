// Code for drawing the main function screen
// Ie the most important screen

use std::path::PathBuf;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::project::{PromptDir, save_project_file};
use crate::tui::layoututils::get_center_rect;
use crate::tui::utils::{render_main_border, render_menu_shortcut_option};
use crate::{AppState, Screen};

use super::layoututils::{get_inner_rect, get_inner_rect_pad};

// extend your MenuMode:
#[derive(PartialEq)]
enum MenuMode {
    SelectMode,
    SelectFunction,
    ProjectMode,
    EditMode,
    ManagePromptDirs,   // new: list/add/remove
    EditPromptDirField, // new: typing description or path
}

// extend your MenuState:
pub struct MenuState {
    mode: MenuMode,
    input: Option<String>, // existing: which field is being edited
    input_value: String,   // existing: typed input
    // new fields for prompt-dirs flow
    prompt_dir_action: Option<PromptDirAction>,
    prompt_dir_temp: Option<PathBuf>, // temp field to store description
}

enum PromptDirAction {
    AddDesc, // we are entering the description
    AddPath, // we are entering the path
    Delete,  // we are about to delete index
}

pub fn create_menu_state() -> MenuState {
    MenuState {
        mode: MenuMode::SelectMode,
        input: None,
        input_value: String::new(),
        prompt_dir_action: None,
        prompt_dir_temp: None,
    }
}

pub fn draw_menu(frame: &mut Frame, state: &mut AppState, screen_state: &mut MenuState) {
    render_main_border(frame, "Main Menu", state);

    // Show some project info
    if let Some(project) = &state.current_project {
        let inner_rect = get_inner_rect_pad(frame.area(), 2, 2);
        let project_dir = project.directory.to_string_lossy();
        let project_file = project.project_file.to_string_lossy();
        let text = vec![
            Line::from(Span::styled("Project", Style::new().bold().yellow())),
            Line::from(project.name.as_deref().unwrap_or("")),
            Line::from(""),
            Line::from("Directory:"),
            Line::from(project_dir.as_ref().blue()),
            Line::from(""),
            Line::from("Project file save:"),
            Line::from(project_file),
        ];

        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), inner_rect);
    };

    // ProjectMode popup
    if screen_state.mode == MenuMode::ProjectMode {
        let centered = get_center_rect(frame.area(), 40, 12);
        frame.render_widget(Clear, centered);
        frame.render_widget(Block::bordered(), centered);
        render_menu_shortcut_option(frame, centered, "n", "set name", 0);
        render_menu_shortcut_option(frame, centered, "l", "set proj file location", 1);
        render_menu_shortcut_option(frame, centered, "s", "save project", 3);
        render_menu_shortcut_option(frame, centered, "r", "set project root", 5);
        render_menu_shortcut_option(frame, centered, "d", "manage prompt_dirs", 6);
        return;
    }

    // ManagePromptDirs: list existing, offer Add/Delete
    if screen_state.mode == MenuMode::ManagePromptDirs {
        let centered = get_center_rect(frame.area(), 70, 30);
        frame.render_widget(Clear, centered);
        frame.render_widget(Block::bordered(), centered);

        let mut lines = vec![Line::from("Prompt Dirs:"), Line::from("")];
        if let Some(proj) = &state.current_project {
            for (i, pd) in proj.prompt_dirs.iter().enumerate() {
                lines.push(Line::from(format!("{}.", i + 1)));
                lines.push(Line::from(pd.path.to_string_lossy()));
                lines.push(Line::from(pd.description.clone()));
            }
        }
        frame.render_widget(Paragraph::new(lines), get_inner_rect(centered, 2, 1, 2, 4));
        let keys = " a   -> add prompt dir\n r   -> remove by index".to_string();

        frame.render_widget(Paragraph::new(keys), get_inner_rect(centered, 2, 1, 27, 1));
        return;
    }

    // EditPromptDirField: typing either desc or path
    if screen_state.mode == MenuMode::EditPromptDirField {
        let centered = get_center_rect(frame.area(), 60, 6);
        frame.render_widget(Clear, centered);
        frame.render_widget(Block::bordered(), centered);

        let prompt = match screen_state.prompt_dir_action {
            Some(PromptDirAction::AddDesc) => "Enter description:",
            Some(PromptDirAction::AddPath) => "Enter path:",
            _ => "",
        };
        frame.render_widget(
            Paragraph::new(format!("{}\n{}|", prompt, screen_state.input_value)),
            get_inner_rect(centered, 2, 1, 2, 2),
        );
        return;
    }

    if screen_state.mode == MenuMode::SelectFunction {
        // Show the possible functions of this screen
        let centered_rect = get_center_rect(frame.area(), 34, 10);
        frame.render_widget(Clear, centered_rect);
        frame.render_widget(Block::bordered(), centered_rect);
        render_menu_shortcut_option(frame, centered_rect, "l", "llm prompt", 0);
        render_menu_shortcut_option(frame, centered_rect, "r", "llm response", 1);
        render_menu_shortcut_option(frame, centered_rect, "f", "file manager", 2);
        render_menu_shortcut_option(frame, centered_rect, "q", "quit", 4);
    }

    if screen_state.mode == MenuMode::EditMode {
        let centered_rect = get_center_rect(frame.area(), 60, 5);
        frame.render_widget(Clear, centered_rect);
        frame.render_widget(Block::bordered(), centered_rect);
        frame.render_widget(
            Paragraph::new(screen_state.input.as_deref().unwrap_or("")),
            Rect {
                x: centered_rect.x + 2,
                y: centered_rect.y,
                width: 12,
                height: 1,
            },
        );
        frame.render_widget(
            // Paste a bar at the end
            Paragraph::new(format!("{}⎸", screen_state.input_value.as_str())),
            get_inner_rect(centered_rect, 2, 1, 2, 2),
        );
    }
}

pub fn handle_event_menu(key: KeyEvent, state: &mut AppState, screen_state: &mut MenuState) {
    // the key depends on the mode of the screen
    match screen_state.mode {
        MenuMode::SelectMode => match key.code {
            KeyCode::Char(' ') => screen_state.mode = MenuMode::SelectFunction,
            KeyCode::Char('p') => screen_state.mode = MenuMode::ProjectMode,
            _ => {}
        },
        MenuMode::SelectFunction => match key.code {
            KeyCode::Char('l') => {
                screen_state.mode = MenuMode::SelectMode;
                state.screen = Screen::Llm;
            }
            KeyCode::Char('r') => {
                screen_state.mode = MenuMode::SelectMode;
                state.screen = Screen::LlmResponse;
            }
            KeyCode::Char('f') => {
                screen_state.mode = MenuMode::SelectMode;
                state.screen = Screen::File;
            }
            KeyCode::Char('q') => state.exit = true,
            KeyCode::Esc => screen_state.mode = MenuMode::SelectMode,
            _ => {}
        },
        MenuMode::ProjectMode => match key.code {
            KeyCode::Esc => screen_state.mode = MenuMode::SelectMode,
            KeyCode::Char('n') => {
                screen_state.mode = MenuMode::EditMode;
                screen_state.input = Some(String::from("name"));
                screen_state.input_value = match state.current_project.as_ref() {
                    Some(project) => match project.name.as_ref() {
                        Some(name) => name.clone(),
                        None => String::default(),
                    },
                    None => String::default(),
                }
            }
            KeyCode::Char('l') => {
                screen_state.mode = MenuMode::EditMode;
                screen_state.input = Some(String::from("location"));
                screen_state.input_value = match state.current_project.as_ref() {
                    Some(project) => project
                        .project_file
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap_or_default(),
                    None => String::default(),
                }
            }
            KeyCode::Char('r') => {
                screen_state.mode = MenuMode::EditMode;
                screen_state.input = Some(String::from("root dir"));
                screen_state.input_value = match state.current_project.as_ref() {
                    Some(project) => project
                        .directory
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap_or_default(),
                    None => String::default(),
                }
            }
            KeyCode::Char('s') => {
                // save the project file
                if let Some(project) = state.current_project.as_ref() {
                    let write_result = save_project_file(project);
                    screen_state.mode = MenuMode::SelectMode;
                    if write_result.is_err() {
                        state.error_msg = Some(String::from("Could not save project"));
                    }
                }
            }

            KeyCode::Char('d') => {
                // go into managing prompt_dirs
                screen_state.mode = MenuMode::ManagePromptDirs;
            }
            _ => {}
        },
        MenuMode::EditMode => match key.code {
            KeyCode::Backspace => {
                let _ = screen_state.input_value.pop();
            }
            KeyCode::Enter => {
                // Save value
                save_edited_value(state, screen_state);
                screen_state.input = None;
                screen_state.input_value = String::default();
                screen_state.mode = MenuMode::SelectMode;
            }
            KeyCode::Esc => {
                // Ignore value
                screen_state.input = None;
                screen_state.input_value = String::default();
                screen_state.mode = MenuMode::SelectMode;
            }
            KeyCode::Char(c) => screen_state.input_value.push(c),
            _ => {}
        },

        MenuMode::EditPromptDirField => {
            match key.code {
                KeyCode::Char(c) => {
                    screen_state.input_value.push(c);
                }
                KeyCode::Backspace => {
                    let _ = screen_state.input_value.pop();
                }
                KeyCode::Enter => {
                    match screen_state.prompt_dir_action.take() {
                        Some(PromptDirAction::AddPath) => {
                            // Temp store path
                            // Convert to pathbuf first
                            let pb = PathBuf::from(&screen_state.input_value);
                            if !pb.is_dir() {
                                state.error_msg = Some(format!(
                                    "{} is not an eixsting dir",
                                    screen_state.input_value
                                ));
                                return;
                            }
                            screen_state.prompt_dir_temp = Some(pb);
                            screen_state.input_value = String::new();
                            screen_state.mode = MenuMode::EditPromptDirField;
                            screen_state.prompt_dir_action = Some(PromptDirAction::AddDesc);
                        }
                        Some(PromptDirAction::AddDesc) => {
                            // We can use the input value as desc
                            // The dir is stored in state
                            // but we need to check if optional
                            if let Some(path) = &screen_state.prompt_dir_temp {
                                state.current_project.as_mut().unwrap().prompt_dirs.push(
                                    PromptDir {
                                        description: screen_state.input_value.clone(),
                                        path: path.clone(),
                                    },
                                );
                            } else {
                                state.error_msg = Some(String::from(
                                    "Could not store prompt dir. This should be impossible...",
                                ));
                            }

                            // Reset everything
                            screen_state.input_value = String::new();
                            screen_state.prompt_dir_temp = None;
                            screen_state.mode = MenuMode::ManagePromptDirs;
                        }
                        Some(PromptDirAction::Delete) => {
                            if let Ok(idx) = screen_state.input_value.parse::<usize>() {
                                if let Some(proj) = state.current_project.as_mut() {
                                    if idx > 0 && idx <= proj.prompt_dirs.len() {
                                        proj.prompt_dirs.remove(idx - 1);
                                    }
                                }
                            }
                            screen_state.mode = MenuMode::ManagePromptDirs;
                        }
                        None => {}
                    }
                    screen_state.input_value.clear();
                }
                KeyCode::Esc => {
                    // cancel any prompt_dir action
                    screen_state.prompt_dir_action = None;
                    screen_state.input_value.clear();
                    screen_state.prompt_dir_temp = None;
                    screen_state.mode = MenuMode::ManagePromptDirs;
                }
                _ => {}
            }
        }
        MenuMode::ManagePromptDirs => match key.code {
            KeyCode::Char('a') => {
                // begin “add” flow: first ask for path
                // init at root dir
                screen_state.input_value.clear();
                if let Some(project) = &state.current_project {
                    screen_state.input_value = project.directory.to_string_lossy().to_string();
                }
                screen_state.prompt_dir_action = Some(PromptDirAction::AddPath);
                screen_state.mode = MenuMode::EditPromptDirField;
            }
            KeyCode::Char('r') => {
                // remove: prompt user to enter index next
                screen_state.prompt_dir_action = Some(PromptDirAction::Delete);
                screen_state.input_value.clear();
                screen_state.mode = MenuMode::EditPromptDirField;
            }
            KeyCode::Esc => {
                screen_state.mode = MenuMode::ProjectMode;
                screen_state.prompt_dir_action = None;
            }
            _ => {}
        },
    }
}

// Function that handles user input for project
fn save_edited_value(state: &mut AppState, screen_state: &mut MenuState) {
    let Some(project) = state.current_project.as_mut() else {
        return;
    };

    // Depending on the type of input, try to set a value
    if let Some(input) = &screen_state.input {
        match input.as_str() {
            "name" => {
                if !screen_state.input_value.is_empty() {
                    project.name = Some(screen_state.input_value.clone());
                }
            }
            "location" => {
                if !screen_state.input_value.is_empty() {
                    project.project_file = PathBuf::from(screen_state.input_value.clone());
                }
            }
            "root dir" => {
                if !screen_state.input_value.is_empty() {
                    project.directory = PathBuf::from(screen_state.input_value.clone());
                }
            }
            _ => {}
        }
    }
}
