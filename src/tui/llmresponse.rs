// The UI elements for receiving the LLM response

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    AppState, LlmMode, LlmResponseState, Screen,
    state::{continue_prompt, reset_prompt},
};

use crate::tui::layoututils::{
    get_center_rect, get_inner_rect_pad, horizontal_left_divide_by_cells,
};
use crate::tui::utils::{render_main_border, render_menu_shortcut_option};

#[derive(PartialEq)]
enum LlmResponseMode {
    Main,
    HandlePrompt,
    Function,
}

pub struct LlmResponseScreenState {
    screen_mode: LlmResponseMode,
    result_scroll: u16,
}

pub fn create_llm_response_state() -> LlmResponseScreenState {
    LlmResponseScreenState {
        screen_mode: LlmResponseMode::Main,
        result_scroll: 0,
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
        if state.current_prompt_mode == LlmMode::Question
            || state.current_prompt_mode == LlmMode::Mock
        {
            frame.render_widget(
                Paragraph::new(state.llm_response.as_str())
                    .wrap(Wrap { trim: true })
                    .scroll((screen_state.result_scroll, 0)),
                // Pad horizontal by 2, since we drew a border
                get_inner_rect_pad(right_rect, 2, 1),
            );
        }
    }

    // The exact handle prompt options depend on the type of LLM request
    if screen_state.screen_mode == LlmResponseMode::HandlePrompt {
        if state.current_prompt_mode == LlmMode::Question
            || state.current_prompt_mode == LlmMode::Mock
        {
            let handle_options_rect = get_center_rect(frame.area(), 30, 10);
            frame.render_widget(Clear, handle_options_rect);
            frame.render_widget(Block::bordered(), handle_options_rect);
            render_menu_shortcut_option(frame, handle_options_rect, "n", "new prompt", 0);
            render_menu_shortcut_option(frame, handle_options_rect, "c", "continue prompt", 1);
        }
    }

    if screen_state.screen_mode == LlmResponseMode::Function {
        let function_rect = get_center_rect(frame.area(), 30, 10);
        frame.render_widget(Clear, function_rect);
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
    if key.code == KeyCode::Esc {
        screen_state.screen_mode = LlmResponseMode::Main;
        return;
    }
    match screen_state.screen_mode {
        // Navigation to other screens
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
            KeyCode::Char('p') => screen_state.screen_mode = LlmResponseMode::HandlePrompt,
            // scroll function
            KeyCode::Char('j') => screen_state.result_scroll += 2,
            KeyCode::Char('k') => {
                if screen_state.result_scroll > 1 {
                    screen_state.result_scroll -= 2;
                }
            }
            KeyCode::Char('K') => {
                screen_state.result_scroll = 0;
            }
            _ => {}
        },
        LlmResponseMode::HandlePrompt => match key.code {
            KeyCode::Char('n') => reset_prompt(state),
            KeyCode::Char('c') => {
                // First reset everything
                continue_prompt(state);
                // Reset this screen
                screen_state.result_scroll = 0;
                screen_state.screen_mode = LlmResponseMode::Main;
                state.wipe_llm_input_prompt = true; // Cross comm to other screen
                state.screen = Screen::Llm; // Go to other screen
            }
            _ => {}
        },
    }
}

// -------------------------------------------------------------------------
// Static texts
const LLM_RESPONSE_INFO: &str = r#"This screen will print the llm response when it is finished.

You have several options to handle this response from this screen.

Press p to open the handle prompt menu.

Press ? to show these options and their shortcuts.

Press spacebar to open the functional menu.
"#;
