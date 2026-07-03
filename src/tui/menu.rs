// Code for drawing the main function screen
// Ie the most important screen

use std::path::PathBuf;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::project::save_project_file;
use crate::tui::layoututils::get_center_rect;
use crate::tui::utils::{render_main_border, render_menu_shortcut_option};
use crate::{AppState, Screen};

use super::layoututils::{get_inner_rect, get_inner_rect_pad};

#[derive(PartialEq)]
enum MenuMode {
    SelectMode,     // The default mode
    SelectFunction, // The user selects a function from the menu
    ProjectMode,    // The user wants to interact with the project
    EditMode,       // The user is updating a setting
}

pub struct MenuState {
    mode: MenuMode,
    input: Option<String>, // Which setting is being edited
    input_value: String,
}

pub fn create_menu_state() -> MenuState {
    MenuState {
        mode: MenuMode::SelectMode,
        input: None,
        input_value: String::default(),
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

    // There are some possible popups
    if screen_state.mode == MenuMode::ProjectMode {
        let centered_rect = get_center_rect(frame.area(), 34, 10);
        frame.render_widget(Clear, centered_rect);
        frame.render_widget(Block::bordered(), centered_rect);
        render_menu_shortcut_option(frame, centered_rect, "n", "set name", 0);
        render_menu_shortcut_option(frame, centered_rect, "l", "set file location", 1);
        render_menu_shortcut_option(frame, centered_rect, "d", "set src dir", 2);
        render_menu_shortcut_option(frame, centered_rect, "s", "save project", 4);
    } else if screen_state.mode == MenuMode::SelectFunction {
        // Show the possible functions of this screen
        let centered_rect = get_center_rect(frame.area(), 34, 10);
        frame.render_widget(Clear, centered_rect);
        frame.render_widget(Block::bordered(), centered_rect);
        render_menu_shortcut_option(frame, centered_rect, "l", "llm prompt", 0);
        render_menu_shortcut_option(frame, centered_rect, "r", "llm response", 1);
        render_menu_shortcut_option(frame, centered_rect, "f", "file manager", 2);
        render_menu_shortcut_option(frame, centered_rect, "q", "quit", 4);
    } else if screen_state.mode == MenuMode::EditMode {
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
            get_inner_rect(centered_rect, 1, 1, 2, 2),
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
            KeyCode::Char('d') => {
                screen_state.mode = MenuMode::EditMode;
                screen_state.input = Some(String::from("src dir"));
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
            "src dir" => {
                if !screen_state.input_value.is_empty() {
                    project.directory = PathBuf::from(screen_state.input_value.clone());
                }
            }
            _ => {}
        }
    }
}
