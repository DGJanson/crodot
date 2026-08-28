use clipboard_rs::{Clipboard, ClipboardContext};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::{AppState, LlmMode, LlmResponseState, Screen, state::reset_prompt};

use crate::tui::components::textarea;
use crate::tui::layoututils::get_center_rect;
use crate::tui::utils::{render_main_border, render_menu_shortcut_option};

use crate::crllm::llm_call::start_call_llm;

enum LlmScreenMode {
    Main,
    Function,
    Prompt,
    KeyHelp,
}

pub struct LlmScreenState {
    screen_mode: LlmScreenMode,
    pub prompt_area: textarea::TextArea,
}

pub fn create_llm_screen_state() -> LlmScreenState {
    LlmScreenState {
        screen_mode: LlmScreenMode::Main,
        prompt_area: textarea::TextArea::new(),
    }
}

pub fn draw_llm_screen(frame: &mut Frame, state: &mut AppState, screen_state: &mut LlmScreenState) {
    let screen_w = frame.area().width;
    let screen_h = frame.area().height;

    render_main_border(frame, "LLM Interaction", state);

    // We create a left menu with some info
    let info_rect = Rect {
        x: 2,
        y: 2,
        width: 30,
        height: screen_h - 3,
    };

    let text = vec![
        Line::from("LLM Mode:"),
        Line::from(format!("{}", state.current_prompt_mode).blue()),
        Line::default(),
        Line::from(INFO_PANEL_PT1),
        Line::default(),
        Line::from(INFO_PANEL_PT2),
        Line::default(),
        Line::from(INFO_PANEL_PT3),
        Line::default(),
        Line::from(INFO_PANEL_PT4),
    ];

    frame.render_widget(
        Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true }),
        info_rect,
    );

    // Display prompt
    let prompt_outer_rect = Rect {
        x: 33,
        y: 2,
        width: screen_w - 34,
        height: screen_h - 2,
    };

    frame.render_widget(
        Block::default()
            .borders(Borders::LEFT)
            .title_top(" Prompt ".yellow()),
        prompt_outer_rect,
    );

    let prompt_area_rect = Rect {
        x: 35,
        y: 4,
        width: screen_w - 37,
        height: screen_h - 5,
    };

    // Render screen state, in some cases
    if let LlmScreenMode::Prompt = screen_state.screen_mode {
        frame.render_widget(
            Paragraph::new(" EDIT PROMPT ").green(),
            Rect {
                x: 2,
                y: screen_h - 1,
                width: 13,
                height: 1,
            },
        );
        screen_state
            .prompt_area
            .render(frame, prompt_area_rect, true);
    } else {
        screen_state
            .prompt_area
            .render(frame, prompt_area_rect, false);
    }

    // Function navigation mode
    if let LlmScreenMode::Function = screen_state.screen_mode {
        // Show the possible key options for this screen
        let centered_rect = get_center_rect(frame.area(), 34, 12);
        frame.render_widget(Block::bordered(), centered_rect);
        render_menu_shortcut_option(frame, centered_rect, "c", "clear prompt", 0);
        render_menu_shortcut_option(frame, centered_rect, "s", "send prompt", 1);
        render_menu_shortcut_option(frame, centered_rect, "x", "reset prompt", 3);
        render_menu_shortcut_option(frame, centered_rect, "r", "llm response", 5);
        render_menu_shortcut_option(frame, centered_rect, "f", "file manager", 6);
        render_menu_shortcut_option(frame, centered_rect, "m", "back to menu", 7);
    }

    // Keyhelp
    if let LlmScreenMode::KeyHelp = screen_state.screen_mode {
        let centered_rect = get_center_rect(frame.area(), 34, 10);
        frame.render_widget(Block::bordered(), centered_rect);
        render_menu_shortcut_option(frame, centered_rect, "f", "change LLM mode", 0);
        render_menu_shortcut_option(frame, centered_rect, "i", "edit prompt", 1);
        render_menu_shortcut_option(frame, centered_rect, "F", "set mock LLM mode", 5);
    }
}

pub fn handle_event_llm_screen(
    key: KeyEvent,
    state: &mut AppState,
    screen_state: &mut LlmScreenState,
) {
    // With an escape we always return to main state
    if let KeyCode::Esc = key.code {
        screen_state.screen_mode = LlmScreenMode::Main;
        return;
    }

    match screen_state.screen_mode {
        LlmScreenMode::Main => match key.code {
            KeyCode::Char(' ') => screen_state.screen_mode = LlmScreenMode::Function,
            KeyCode::Char('f') => switch_llm_mode(state),
            KeyCode::Char('F') => state.current_prompt_mode = LlmMode::Mock,
            KeyCode::Char('i') => screen_state.screen_mode = LlmScreenMode::Prompt,
            KeyCode::Char('?') => screen_state.screen_mode = LlmScreenMode::KeyHelp,
            KeyCode::Char('y') => yank_prompt(state, screen_state),
            _ => {}
        },
        LlmScreenMode::Function => match key.code {
            KeyCode::Char('c') => {
                // Clear prompt.
                // Only clear the textarea
                screen_state.prompt_area.reset();
                screen_state.screen_mode = LlmScreenMode::Main;
            }
            KeyCode::Char('x') => {
                // reset prompt
                // Clear text area and remove all earlier prompts
                screen_state.prompt_area.reset();
                reset_prompt(state);
                screen_state.screen_mode = LlmScreenMode::Main;
            }
            KeyCode::Char('s') => {
                // We always return to main screen state
                // Whatever the result is
                screen_state.screen_mode = LlmScreenMode::Main;
                // Check if engine is idle, otherwise do nothing
                if state.llm_response_state != LlmResponseState::Idle {
                    return;
                }
                // Check if the prompt actually contains text
                if screen_state.prompt_area.get_text().trim().is_empty() {
                    return;
                }
                // "Send the prompt", reset this screen state first
                // Clone the prompt to the app state
                // and change to response screen
                state.current_prompt = screen_state.prompt_area.get_text().clone();
                state.screen = Screen::LlmResponse;
                // Now we can start the llm call
                start_call_llm(state);
            }
            KeyCode::Char('r') => {
                // Go to menu
                screen_state.screen_mode = LlmScreenMode::Main;
                state.screen = Screen::LlmResponse;
            }
            KeyCode::Char('f') => {
                // Go to menu
                screen_state.screen_mode = LlmScreenMode::Main;
                state.screen = Screen::File;
            }
            KeyCode::Char('m') => {
                // Go to menu
                screen_state.screen_mode = LlmScreenMode::Main;
                state.screen = Screen::Menu;
            }
            _ => {}
        },
        LlmScreenMode::Prompt => {
            screen_state.prompt_area.handle_key(key);
        }
        LlmScreenMode::KeyHelp => {}
    }
}

fn switch_llm_mode(state: &mut AppState) {
    if state.current_prompt_mode == LlmMode::Question {
        state.current_prompt_mode = LlmMode::Coding;
    } else {
        state.current_prompt_mode = LlmMode::Question;
    }
}

fn yank_prompt(state: &mut AppState, screen_state: &LlmScreenState) {
    let ctx_opt = ClipboardContext::new();
    match ctx_opt {
        Err(e) => {
            state.error_msg = Some(format!("Could not init clipboard, because {e}"));
        }
        Ok(ctx) => {
            // get text from text area
            let prompt = screen_state.prompt_area.get_text();

            if ctx.set_text(prompt).is_err() {
                state.error_msg = Some("Could not yank to clipboard".to_string())
            }
        }
    }
}

// -------------------------------------------------------------
// static texts
const INFO_PANEL_PT1: &str = "You can prompt the LLM from this screen.";

const INFO_PANEL_PT2: &str = "Use ? key to see keyboard commands.";

const INFO_PANEL_PT3: &str = "Note the difference between clear and reset prompt. Clear will just clear the textarea. Reset will start a completely new chat.";

const INFO_PANEL_PT4: &str = "You can only send a new prompt when the LLM is idle.";
