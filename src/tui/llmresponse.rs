// The UI elements for receiving the LLM response

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{AppState, LlmResponseState, Screen};

use crate::tui::layoututils::{
    get_center_rect, get_inner_rect_pad, horizontal_left_divide_by_cells,
};
use crate::tui::utils::{render_main_border, render_menu_shortcut_option};

#[derive(PartialEq)]
enum LlmResponseMode {
    Main,
    Function,
}

pub struct LlmResponseScreenState {
    screen_mode: LlmResponseMode,
}

pub fn create_llm_response_state() -> LlmResponseScreenState {
    LlmResponseScreenState {
        screen_mode: LlmResponseMode::Main,
    }
}

pub fn draw_llm_response_screen(
    frame: &mut Frame,
    state: &mut AppState,
    screen_state: &mut LlmResponseScreenState,
) {
    // Always show the basic screen
    render_main_border(frame, "LLM Response", state);

    // We have 2 area's
    // Left: some info
    // Rest: the response itself

    let (left_rect, right_rect) =
        horizontal_left_divide_by_cells(get_inner_rect_pad(frame.area(), 1, 1), 30);

    // Draw borders
    frame.render_widget(Block::default().borders(Borders::LEFT), right_rect);

    // Show page info on the left
    frame.render_widget(
        Paragraph::new(LLM_RESPONSE_INFO).wrap(Wrap { trim: true }),
        get_inner_rect_pad(left_rect, 1, 1),
    );

    if state.llm_response_state == LlmResponseState::NewResponse {
        frame.render_widget(
            Paragraph::new(state.llm_response.as_str()).wrap(Wrap { trim: true }),
            // Pad horizontal by 2, since we drew a border
            get_inner_rect_pad(right_rect, 2, 1),
        );
    }

    if screen_state.screen_mode == LlmResponseMode::Function {
        let function_rect = get_center_rect(frame.area(), 30, 10);
        frame.render_widget(Block::bordered(), function_rect);
        render_menu_shortcut_option(frame, function_rect, "l", "LLM prompt", 0);
        render_menu_shortcut_option(frame, function_rect, "f", "file manager", 1);
        render_menu_shortcut_option(frame, function_rect, "m", "Main menu", 4);
    }
}

pub fn handle_event_llm_response_screen(
    key: KeyEvent,
    state: &mut AppState,
    screen_state: &mut LlmResponseScreenState,
) {
    match screen_state.screen_mode {
        LlmResponseMode::Function => match key.code {
            KeyCode::Char('l') => {
                screen_state.screen_mode = LlmResponseMode::Main;
                state.screen = Screen::Llm;
            }
            KeyCode::Char('f') => {
                screen_state.screen_mode = LlmResponseMode::Main;
                state.screen = Screen::File;
            }
            KeyCode::Char('m') => {
                screen_state.screen_mode = LlmResponseMode::Main;
                state.screen = Screen::Menu;
            }
            _ => screen_state.screen_mode = LlmResponseMode::Main,
        },
        LlmResponseMode::Main => match key.code {
            KeyCode::Char(' ') => screen_state.screen_mode = LlmResponseMode::Function,
            _ => {}
        },
    }
}

// -------------------------------------------------------------------------
// Static texts
const LLM_RESPONSE_INFO: &str = r#"This screen will print the llm response when it is finished.

You have several options to handle this response from this screen.

Press ? to show these options and their shortcuts.

Press spacebar to open the functional menu.
"#;
