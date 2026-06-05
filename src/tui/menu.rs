// Code for drawing the main function screen
// Ie the most important screen

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Paragraph};

use crate::tui::layoututils::get_center_rect;
use crate::tui::utils::{render_main_border, render_menu_shortcut_option};
use crate::{AppState, Screen};

enum MenuMode {
    SelectMode,     // The default mode
    SelectFunction, // The user selects a function from the menu
}

pub struct MenuState {
    mode: MenuMode,
}

pub fn create_menu_state() -> MenuState {
    MenuState {
        mode: MenuMode::SelectMode,
    }
}

pub fn draw_menu(frame: &mut Frame, state: &mut AppState, screen_state: &mut MenuState) {
    render_main_border(frame, "Main Menu", state);
    // depending on the screen we may show some options
    if let MenuMode::SelectFunction = screen_state.mode {
        // Show the possible functions of this screen
        let centered_rect = get_center_rect(frame.area(), 34, 10);
        frame.render_widget(Block::bordered(), centered_rect);
        render_menu_shortcut_option(frame, centered_rect, "l", "llm prompt", 0);
        render_menu_shortcut_option(frame, centered_rect, "r", "llm response", 1);
        render_menu_shortcut_option(frame, centered_rect, "q", "quit", 4);
    }
}

pub fn handle_event_menu(key: KeyEvent, state: &mut AppState, screen_state: &mut MenuState) {
    // the key depends on the mode of the screen
    match screen_state.mode {
        MenuMode::SelectMode => match key.code {
            KeyCode::Char(' ') => screen_state.mode = MenuMode::SelectFunction,
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
            KeyCode::Char('q') => state.exit = true,
            KeyCode::Esc => screen_state.mode = MenuMode::SelectMode,
            _ => {}
        },
    }
}
