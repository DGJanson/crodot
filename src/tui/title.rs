// The functions that draw the home / start screen go here

use std::path::PathBuf;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::crossterm::event;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::project::{create_new_project, read_project_file_with_location_string};
use crate::{AppState, Screen};

use super::layoututils::{get_center_rect, get_inner_rect_pad};
use super::utils::render_menu_shortcut_option;

#[derive(PartialEq)]
pub enum TitleFunction {
    Title,
    Function,
    ProjectNew,
    ProjectOpen,
}

pub struct TitleState {
    function: TitleFunction,
    work_dir: PathBuf,
    projects: Vec<ProjectName>,
}

pub fn create_title_state() -> TitleState {
    TitleState {
        function: TitleFunction::Title,
        work_dir: std::env::current_dir().unwrap_or_default(),
        projects: Vec::new(),
    }
}

/// A struct we use to generate project names to visualize
/// in the open project popup
pub struct ProjectName {
    name: String,
    config_location: String,  // The location of the project config file
    project_location: String, // The directory of the project file
}

#[allow(clippy::explicit_counter_loop)]
pub fn draw_title(frame: &mut Frame, _state: &mut AppState, title_state: &mut TitleState) {
    // Divide the screen in two parts
    // The top will be used for title
    // The bottom for some text
    let screen_w = frame.area().width;
    let screen_h = frame.area().height;

    // Calc a nice box for the title
    let title_text_rect = Rect {
        x: screen_w / 2 - 3,
        y: screen_h / 4,
        width: 6,
        height: 1,
    };

    let title_box_rect = Rect {
        x: title_text_rect.x - 10,
        y: title_text_rect.y - 3,
        width: title_text_rect.width + 20,
        height: title_text_rect.height + 6,
    };

    frame.render_widget(Block::bordered().style(Style::new().red()), title_box_rect);

    let title_text = Span::styled("CRODOT", Style::new().red().bold());
    frame.render_widget(
        Paragraph::new(title_text).alignment(Alignment::Center),
        title_text_rect,
    );

    let discmargin: u16; // Disclaimer margin size

    if screen_w < 100 {
        discmargin = 20;
    } else if screen_w < 140 {
        discmargin = 30;
    } else {
        discmargin = 40;
    }

    let disclaimer_rect = Rect {
        x: discmargin,
        y: screen_h / 2,
        width: screen_w - (discmargin + discmargin),
        height: screen_h / 2,
    };

    frame.render_widget(
        Paragraph::new(DISCLAIMER).wrap(Wrap { trim: true }),
        disclaimer_rect,
    );

    if title_state.function == TitleFunction::ProjectOpen {
        let project_rect = Rect {
            x: 20,
            y: screen_h / 2,
            width: screen_w - 40,
            height: screen_h / 2,
        };
        frame.render_widget(Clear, project_rect);
        frame.render_widget(Block::bordered(), project_rect);

        let inner_rect = get_inner_rect_pad(project_rect, 2, 2);
        if title_state.projects.is_empty() {
            frame.render_widget(Paragraph::new("No available projects"), inner_rect);
        } else {
            let mut index = 0;
            for project in &title_state.projects {
                let text = vec![
                    Line::from(format!("{}. {}", index + 1, project.name)),
                    Line::from(format!("  {}", project.project_location)),
                ];
                frame.render_widget(
                    Paragraph::new(text),
                    Rect {
                        x: inner_rect.x,
                        y: inner_rect.y + index * 2,
                        width: inner_rect.width,
                        height: 2,
                    },
                );
                index += 1;
            }
        }
    }

    if title_state.function == TitleFunction::Function {
        let center_rect = get_center_rect(frame.area(), 25, 10);
        frame.render_widget(Clear, center_rect);
        frame.render_widget(Block::bordered(), center_rect);
        render_menu_shortcut_option(frame, center_rect, "n", "new project", 0);
        render_menu_shortcut_option(frame, center_rect, "o", "open project", 1);
        render_menu_shortcut_option(frame, center_rect, "q", "quit", 3);
    }
}

pub fn handle_event_title(
    key: event::KeyEvent,
    state: &mut AppState,
    title_state: &mut TitleState,
) {
    match title_state.function {
        TitleFunction::Title => {
            if let KeyCode::Char(' ') = key.code {
                title_state.function = TitleFunction::Function;
            }
        }
        TitleFunction::ProjectOpen => match key.code {
            // First check for cancel (esc)
            KeyCode::Esc => title_state.function = TitleFunction::Title,
            KeyCode::Char(c) => {
                if c.is_ascii_digit() {
                    let number = c.to_digit(10).unwrap(); // We can unwrap safely, we checked
                    if number == 0 {
                        return;
                    }
                    let index = usize::try_from(number).unwrap_or(1);
                    // Load the project file corresponding to this project
                    title_state.function = TitleFunction::Title;
                    set_project_file(state, &title_state.projects[index - 1]);
                }
            }
            _ => {}
        },
        TitleFunction::Function => match key.code {
            KeyCode::Char('n') => {
                state.current_project = Some(create_new_project(title_state.work_dir.clone()));
                title_state.function = TitleFunction::Title;
                state.screen = Screen::Menu;
            }
            KeyCode::Char('o') => {
                // We fetch projects info
                fetch_project_info(state, title_state);
                title_state.function = TitleFunction::ProjectOpen;
            }
            KeyCode::Char('q') => state.exit = true,
            _ => title_state.function = TitleFunction::Title,
        },
        _ => title_state.function = TitleFunction::Title,
    }
}

/// Read all project info and add to vector
/// If vector already filled, we simply ignore
fn fetch_project_info(app_state: &AppState, screen_state: &mut TitleState) {
    if !screen_state.projects.is_empty() {
        // Vec already filled, do nothing
        return;
    }
    // If there are no projects defined, we do nothing
    // The vec stays empty, which is fine
    if let Some(loc_strings) = &app_state.available_projects {
        for loc in loc_strings {
            // Try to read the project and add to screen state vec
            let project_opt = read_project_file_with_location_string(loc);
            if let Ok(project) = project_opt {
                let new_project = ProjectName {
                    name: project.name.clone().unwrap_or_default(),
                    project_location: project
                        .directory
                        .into_os_string()
                        .into_string()
                        .unwrap_or_default(),
                    config_location: project
                        .project_file
                        .into_os_string()
                        .into_string()
                        .unwrap_or_default(),
                };
                screen_state.projects.push(new_project);
            }
        }
    }
}

fn set_project_file(app_state: &mut AppState, project_name: &ProjectName) {
    if let Ok(project) = read_project_file_with_location_string(&project_name.config_location) {
        app_state.current_project = Some(project);
        app_state.screen = Screen::Menu;
    } else {
        log::error!("Could not read project");
        // TODO Change to proper error
    }
}

// Static texts go here!

const DISCLAIMER: &str = r#"
CRODOT is a tool for coding with AI's. Some more info will be added to this screen later, hopefully.

Press ctrl + q to exit the app at any time.

Pressing space will open up the navigation menu.

Pressing ? will (sometimes) open a help - shortcut menu.
"#;
