#![allow(clippy::explicit_counter_loop)]

// This file deals with file management.
// Ie. selecting files we want to add to
// prompts and the like.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{AppState, Screen, project::Project};

use super::{
    components::filepicker::FilePicker,
    layoututils::{get_center_rect, get_inner_rect_pad},
    utils::{render_main_border, render_menu_shortcut_option},
};

#[derive(PartialEq)]
pub enum FileScreenMode {
    Overview,
    Function,
    Help,
    PickDesc,
    PickList,
    EditList,
}

pub struct FileScreenState {
    mode: FileScreenMode,
    filepicker_desc: Option<FilePicker>,
    filepicker_list: Option<FilePicker>,
    // Two vars to keep track of rendering file shortlist
    // Use u16, becuase we need it for size calcs
    edit_list_index: u16,
    max_filename_len: u16,
}

pub fn create_files_screen_state() -> FileScreenState {
    FileScreenState {
        mode: FileScreenMode::Overview,
        filepicker_desc: None,
        filepicker_list: None,
        edit_list_index: 0,
        max_filename_len: 0,
    }
}

pub fn draw_file_screen(
    frame: &mut Frame,
    app_state: &mut AppState,
    screen_state: &mut FileScreenState,
) {
    render_main_border(frame, "File Manager", app_state);
    // Check for project, we need it
    let project = match &app_state.current_project {
        Some(p) => p,
        None => {
            app_state.error_msg = Some(String::from(
                "Project not found, please select one or check the one provided.",
            ));
            app_state.screen = Screen::Title;
            return;
        }
    };
    let project_desc_file = match &project.desc_file {
        Some(path) => path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap_or(String::from("Invalid name")),
        None => String::from("Not Selected"),
    };

    let inner_rect = get_inner_rect_pad(frame.area(), 2, 2);

    // Render top part
    let top_rect = Rect {
        x: inner_rect.x,
        y: inner_rect.y,
        width: inner_rect.width,
        height: 8,
    };

    let bottom_rect = Rect {
        x: inner_rect.x,
        y: inner_rect.y + 9,
        width: inner_rect.width,
        height: inner_rect.height - 9,
    };

    // render the text in this top rect
    let top_par = vec![
        Line::from(TOP_TEXT_1),
        Line::from(TOP_TEXT_2),
        Line::from(TOP_TEXT_3),
        Line::from(TOP_TEXT_4),
        Line::from(""),
        Line::from(TOP_TEXT_5),
        Line::from(project_desc_file).blue(),
    ];
    frame.render_widget(Paragraph::new(top_par).wrap(Wrap { trim: true }), top_rect);

    // Now we render the bottom based on screen state
    match screen_state.mode {
        FileScreenMode::PickDesc => {
            // render the file desc picker
            if let Some(picker) = &screen_state.filepicker_desc {
                picker.render(frame, bottom_rect);
            }
        }
        FileScreenMode::PickList => {
            if let Some(picker) = &screen_state.filepicker_list {
                picker.render(frame, bottom_rect);
            }
        }
        FileScreenMode::EditList => {
            // We need some more calcs here, so move to seperate function
            if let Some(project) = &app_state.current_project {
                draw_edit_list(frame, bottom_rect, project, screen_state)
            }
        }
        _ => {
            // TODO add the overview
        }
    }

    // Popups

    // Help menu
    if screen_state.mode == FileScreenMode::Help {
        let center_rect = get_center_rect(inner_rect, 25, 10);
        frame.render_widget(Clear, center_rect);
        frame.render_widget(Block::bordered(), center_rect);
        render_menu_shortcut_option(frame, center_rect, "d", "set description", 0);
        render_menu_shortcut_option(frame, center_rect, "f", "add file to list", 1);
        render_menu_shortcut_option(frame, center_rect, "a", "edit active list", 2);
        return;
    }

    // Functions
    if screen_state.mode == FileScreenMode::Function {
        let center_rect = get_center_rect(inner_rect, 25, 10);
        frame.render_widget(Clear, center_rect);
        frame.render_widget(Block::bordered(), center_rect);
        render_menu_shortcut_option(frame, center_rect, "m", "main menu", 0);
        render_menu_shortcut_option(frame, center_rect, "l", "llm screen", 1);
        render_menu_shortcut_option(frame, center_rect, "r", "llm response", 2);
    }
}

fn draw_edit_list(
    frame: &mut Frame,
    rect: Rect,
    project: &Project,
    screen_state: &mut FileScreenState,
) {
    // edge case, no active files
    if project.file_short_list.is_empty() {
        frame.render_widget(
            Paragraph::new("No files selected."),
            get_center_rect(rect, 20, 1),
        );
        return;
    }

    // we have at least some files in the selected list
    // first draw the rect, based on the file name size
    let list_rect = get_center_rect(
        rect,
        screen_state.max_filename_len + 14,
        u16::try_from(project.file_short_list.len()).unwrap_or(1) + 4,
    );

    frame.render_widget(Block::bordered(), list_rect);

    let mut draw_index: u16 = 0;

    for filepath in project.file_short_list.iter() {
        let display = match filepath.strip_prefix(&project.directory) {
            Ok(fp) => fp.to_string_lossy(),
            Err(_) => filepath.to_string_lossy(),
        };

        // Create a style based on two cases
        let selected = draw_index == screen_state.edit_list_index;
        let active = project.file_active_list.contains(filepath);

        let style_to_use = match (selected, active) {
            (true, true) => Style::new().bg(Color::Blue),
            (true, false) => Style::new().bg(Color::Blue).fg(Color::DarkGray),
            (false, true) => Style::new(),
            (false, false) => Style::new().fg(Color::DarkGray),
        };

        frame.render_widget(
            Paragraph::new(display).style(style_to_use),
            Rect {
                x: list_rect.x + 2,
                y: list_rect.y + 2 + draw_index,
                width: list_rect.width - 13,
                height: 1,
            },
        );

        if active {
            frame.render_widget(
                Paragraph::new("active"),
                Rect {
                    x: list_rect.x + screen_state.max_filename_len + 6,
                    y: list_rect.y + 2 + draw_index,
                    width: 6,
                    height: 1,
                },
            );
        }

        draw_index += 1;
    }
}

pub fn handle_event_file_screen(
    key: KeyEvent,
    app_state: &mut AppState,
    screen_state: &mut FileScreenState,
) {
    if key.code == KeyCode::Esc {
        screen_state.mode = FileScreenMode::Overview;
    }
    match screen_state.mode {
        FileScreenMode::Overview => match key.code {
            KeyCode::Char(' ') => screen_state.mode = FileScreenMode::Function,
            KeyCode::Char('?') => screen_state.mode = FileScreenMode::Help,
            KeyCode::Char('d') => {
                screen_state.mode = FileScreenMode::PickDesc;
                setup_desc_picker(app_state, screen_state);
            }
            KeyCode::Char('f') => {
                screen_state.mode = FileScreenMode::PickList;
                setup_list_picker(app_state, screen_state);
            }
            KeyCode::Char('a') => {
                screen_state.mode = FileScreenMode::EditList;
                screen_state.edit_list_index = 0;
                if let Some(project) = &app_state.current_project {
                    screen_state.max_filename_len = get_max_filename_len(project);
                }
            }

            _ => {}
        },
        FileScreenMode::Function => match key.code {
            KeyCode::Char('m') => {
                screen_state.mode = FileScreenMode::Overview;
                app_state.screen = Screen::Menu;
            }
            KeyCode::Char('l') => {
                screen_state.mode = FileScreenMode::Overview;
                app_state.screen = Screen::Llm;
            }
            KeyCode::Char('r') => {
                screen_state.mode = FileScreenMode::Overview;
                app_state.screen = Screen::LlmResponse;
            }
            _ => {}
        },
        FileScreenMode::PickDesc => {
            let mut file_picked = false;
            if let Some(picker) = screen_state.filepicker_desc.as_mut() {
                // In case of return we want to save the selected file
                if let KeyCode::Enter = key.code {
                    file_picked = true;
                }
                // Otherwise, send key to picker to handle
                picker.handle_key(key);
            }
            if file_picked {
                // Update path to desc file
                // return to regular screen state
                update_desc_file(app_state, screen_state);
                screen_state.mode = FileScreenMode::Overview;
            }
        }
        FileScreenMode::PickList => {
            let mut file_picked = false;
            if let Some(picker) = screen_state.filepicker_list.as_mut() {
                // In case of return we want to save the selected file
                if let KeyCode::Enter = key.code {
                    file_picked = true;
                }
                // Otherwise, send key to picker to handle
                picker.handle_key(key);
            }
            if file_picked {
                // Update path to desc file
                // return to regular screen state
                update_file_list(app_state, screen_state);
                screen_state.mode = FileScreenMode::Overview;
            }
        }
        FileScreenMode::EditList => {
            if let Some(project) = &mut app_state.current_project {
                match key.code {
                    // Two codes to navigate through edit list
                    KeyCode::Char('j') => {
                        if project.file_short_list.len() == 0 {
                            screen_state.edit_list_index = 0;
                            return;
                        }
                        // We can do unchecked sub, since we know the len is not 0
                        if u16::try_from(project.file_short_list.len() - 1).unwrap_or_default()
                            > screen_state.edit_list_index
                        {
                            screen_state.edit_list_index += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        if screen_state.edit_list_index > 0 {
                            screen_state.edit_list_index -= 1;
                        }
                    }
                    KeyCode::Char('a') => switch_active(project, screen_state.edit_list_index),
                    KeyCode::Char('d') => {
                        delete_selected_from_list(project, screen_state.edit_list_index)
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

// Helper functions
fn setup_desc_picker(app_state: &mut AppState, screen_state: &mut FileScreenState) {
    // If file picker already active, simply let it be
    if screen_state.filepicker_desc.is_some() {
        return;
    }
    // Now we need to create filepicker.
    // We need to determine which dir to start from
    // If a file is already set, we simply get the dir that file is in
    // Note we can be sure the project is Some, since we check in the draw fn
    // So simply unwrap here
    let project = app_state.current_project.as_ref().unwrap();
    match &project.desc_file {
        Some(path) => {
            let path_dir = match path.parent() {
                Some(dir) => dir,
                None => {
                    app_state.error_msg = Some(String::from(
                        "Could not find dir of desc file, please check project config. Reload application please.",
                    ));
                    app_state.unrecoverable_error = true;
                    return;
                }
            };
            let picker = FilePicker::new(path_dir);
            screen_state.filepicker_desc = Some(picker);
        }
        None => {
            // Now we use the dirname of the project file
            let path_dir = match project.project_file.parent() {
                Some(dir) => dir,
                None => {
                    app_state.error_msg = Some(String::from(
                        "Could not find dir of project file, please check project config. Reload application please.",
                    ));
                    app_state.unrecoverable_error = true;
                    return;
                }
            };
            let picker = FilePicker::new(path_dir);
            screen_state.filepicker_desc = Some(picker);
        }
    }
}

fn setup_list_picker(app_state: &mut AppState, screen_state: &mut FileScreenState) {
    // If file picker already active, simply let it be
    if screen_state.filepicker_list.is_some() {
        return;
    }

    // Does not exist yet. Simply create one. We use the project dir as entry
    let project = app_state.current_project.as_ref().unwrap();
    let picker = FilePicker::new(&project.directory);
    screen_state.filepicker_list = Some(picker);
}

fn update_desc_file(app_state: &mut AppState, screen_state: &mut FileScreenState) {
    // Attempt to update the project description in the project definition
    // We deal with a lot of Options, of which we are quite sure they
    //   are Some. But it will get a bit messy
    let project: &mut Project = match app_state.current_project.as_mut() {
        Some(p) => p,
        None => return,
    };
    let desc_path = match screen_state.filepicker_desc.as_ref() {
        Some(picker) => {
            if let Some(selected_file) = picker.try_get_file() {
                selected_file.clone()
            } else {
                app_state.error_msg = Some(String::from("No file selected"));
                return;
            }
        }
        None => return,
    };
    project.desc_file = Some(desc_path);
}

fn update_file_list(app_state: &mut AppState, screen_state: &mut FileScreenState) {
    // Attempt to add selected file to the file list
    // We deal with a lot of optional vars, so do some
    //   fetching and checking first
    let project: &mut Project = match app_state.current_project.as_mut() {
        Some(p) => p,
        None => return,
    };

    let selected_file = match screen_state.filepicker_list.as_ref() {
        Some(picker) => {
            if let Some(selected_file) = picker.try_get_file() {
                selected_file.clone()
            } else {
                app_state.error_msg = Some(String::from("No file selected"));
                return;
            }
        }
        None => return,
    };

    // Now add to list, first check if not already in list
    if project.file_short_list.contains(&selected_file) {
        return;
    }

    project.file_short_list.push(selected_file.clone());
    project.file_short_list.sort();
    // Make it active by default
    project.file_active_list.push(selected_file);

    screen_state.max_filename_len = get_max_filename_len(project);
    // Check if our select index still makes sense
    // Simply reset otherwise
    if screen_state.edit_list_index as usize >= project.file_short_list.len() {
        screen_state.edit_list_index = 0;
    }
}

fn get_max_filename_len(project: &Project) -> u16 {
    let mut max_size: u16 = 0;

    for filepath in &project.file_short_list {
        if let Ok(stripped) = filepath.strip_prefix(&project.directory) {
            let display = stripped.to_string_lossy();
            max_size = max_size.max(u16::try_from(display.len()).unwrap_or_default());
        } else {
            let display = filepath.to_string_lossy();
            max_size = max_size.max(u16::try_from(display.len()).unwrap_or_default());
        }
    }

    max_size
}

fn switch_active(project: &mut Project, switch_index: u16) {
    // First check whether the current file is active or no.
    // Then change active list accordingly
    let selected_file = project.file_short_list.get(switch_index as usize);
    if let Some(file) = selected_file {
        let active_index = project
            .file_active_list
            .iter()
            .position(|f| f.as_path() == file.as_path());
        match active_index {
            Some(index) => {
                // path is active, remove from active list
                project.file_active_list.remove(index);
            }
            None => {
                project.file_active_list.push(file.clone());
            }
        }
    }
}

fn delete_selected_from_list(project: &mut Project, delete_index: u16) {
    // Delete selected from list
    // Note that it MAY be in the active list
    let selected_file = project.file_short_list.get(delete_index as usize);
    if let Some(file) = selected_file {
        // Look in active list
        let active_index = project
            .file_active_list
            .iter()
            .position(|f| f.as_path() == file.as_path());
        if let Some(index) = active_index {
            project.file_active_list.remove(index);
        }
        // Always remove from shortlist
        project.file_short_list.remove(delete_index as usize);
    }
}

// ------------------------------------------------------------------------
// Static texts
const TOP_TEXT_1: &str = "In this screen you can select files to add to prompts.";
const TOP_TEXT_2: &str = "There is a shortlist, from which you can quickly (un)select files.";
const TOP_TEXT_3: &str = "There is an active list, which are actually added to prompts.";
const TOP_TEXT_4: &str =
    "Finally, there is a project description file, which is added to every prompt.";
const TOP_TEXT_5: &str = "The currently selected project file is:";
