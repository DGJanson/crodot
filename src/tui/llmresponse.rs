// The UI elements for receiving the LLM response

use clipboard_rs::{Clipboard, ClipboardContext};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    AppState, LlmMode, LlmResponseState, Screen,
    state::{continue_prompt, reset_prompt},
    tui::layoututils::horizontal_left_divide_by_fraction,
};

use crate::tui::layoututils::{
    get_center_rect, get_inner_rect_pad, horizontal_left_divide_by_cells,
};
use crate::tui::utils::{render_main_border, render_menu_shortcut_option};

#[derive(PartialEq)]
enum LlmResponseMode {
    Main,
    HandlePrompt,
    HandleFile,
    Function,
}

pub struct LlmResponseScreenState {
    screen_mode: LlmResponseMode,
    result_scroll: u16,
    handle_file: String, // Store as string, cast later
    // Some vars to select code
    pub code_blocks: Vec<(usize, usize)>,
    pub selected_block: usize,
    pub selected: bool, // select nothing by default
    selected_line: u16,
    update_selected_scroll: bool,
}

pub fn create_llm_response_state() -> LlmResponseScreenState {
    LlmResponseScreenState {
        screen_mode: LlmResponseMode::Main,
        result_scroll: 0,
        handle_file: String::from("0"),
        code_blocks: Vec::new(),
        selected_block: 0,
        selected: false,
        selected_line: 0,
        update_selected_scroll: false,
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
    frame.render_widget(
        Block::default()
            .borders(Borders::LEFT)
            .style(Style::default()),
        right_rect,
    );

    // Show page info on the left
    frame.render_widget(
        Paragraph::new(LLM_RESPONSE_INFO).wrap(Wrap { trim: true }),
        get_inner_rect_pad(left_rect, 1, 1),
    );

    if state.llm_response_state == LlmResponseState::NewResponse {
        if state.current_prompt_mode == LlmMode::Question
            || state.current_prompt_mode == LlmMode::Mock
        {
            // We need to deal with selected code blocks -> two scenario's
            // No code block selected, simply render the response string
            // If a code block is selected we need to find the selected
            // block and highlight it
            if screen_state.selected {
                // find block and render
                let lines = get_higlighted_spans(&state.llm_response, screen_state);
                // Check if we need to update our scroll
                // Based on selected block
                if screen_state.update_selected_scroll {
                    screen_state.update_selected_scroll = false;
                    screen_state.result_scroll = screen_state.selected_line;
                }
                frame.render_widget(
                    Paragraph::new(lines)
                        .wrap(Wrap { trim: false })
                        .scroll((screen_state.result_scroll, 0)),
                    get_inner_rect_pad(right_rect, 2, 1),
                );
            } else {
                frame.render_widget(
                    Paragraph::new(state.llm_response.as_str())
                        .wrap(Wrap { trim: false })
                        .scroll((screen_state.result_scroll, 0)),
                    // Pad horizontal by 2, since we drew a border
                    get_inner_rect_pad(right_rect, 2, 1),
                );
            }
        } else {
            // Coding response
            // Divide the screen in two parts:
            // - message
            // - changed files
            if let Some(code_response) = state.llm_response_code_response.as_ref() {
                let (message_rect, files_rect) = horizontal_left_divide_by_fraction(right_rect, 3);
                if code_response.error.is_some() {
                    frame.render_widget(
                        Paragraph::new(code_response.error.as_deref().unwrap_or_default())
                            .style(Style::new().red())
                            .wrap(Wrap { trim: true }),
                        get_inner_rect_pad(message_rect, 2, 1),
                    );
                } else {
                    frame.render_widget(
                        Paragraph::new(code_response.message.as_str())
                            .wrap(Wrap { trim: true })
                            .scroll((screen_state.result_scroll, 0)),
                        get_inner_rect_pad(message_rect, 2, 1),
                    );
                }

                // Render the changed files.
                // On this screen, we just render the list
                // Create a long string to draw the file changes
                let mut file_list = String::new();
                let max_desc_len = usize::from(files_rect.width) - 2;
                for (index, file) in code_response.code.iter().enumerate() {
                    let num = index + 1;
                    let filename = &file.filename;
                    // Calc max desc len
                    let desc = if file.description.len() > max_desc_len {
                        let mut end_idx = max_desc_len;
                        // We need to take into account that we substring
                        // into a utf-8 char, which is not allowed
                        while end_idx > 0 && !file.description.is_char_boundary(end_idx) {
                            end_idx -= 1;
                        }
                        &file.description[0..end_idx]
                    } else {
                        &file.description
                    };

                    file_list.push_str(&format!("{num}.\n{filename}\n{desc}\n\n",));
                }

                frame.render_widget(
                    Paragraph::new(file_list.as_str()).scroll((screen_state.result_scroll, 0)),
                    get_inner_rect_pad(files_rect, 1, 1),
                );
            } else {
                frame.render_widget(
                    Paragraph::new("Did not receive a proper coding response."),
                    get_inner_rect_pad(right_rect, 2, 1),
                );
            }
        }
    }

    // The exact handle prompt options depend on the type of LLM request
    if screen_state.screen_mode == LlmResponseMode::HandlePrompt {
        let handle_options_rect = get_center_rect(frame.area(), 30, 10);
        frame.render_widget(Clear, handle_options_rect);
        frame.render_widget(Block::bordered(), handle_options_rect);
        render_menu_shortcut_option(frame, handle_options_rect, "n", "new prompt", 0);
        render_menu_shortcut_option(frame, handle_options_rect, "c", "continue prompt", 1);
    }

    if screen_state.screen_mode == LlmResponseMode::Function {
        let function_rect = get_center_rect(frame.area(), 30, 10);
        frame.render_widget(Clear, function_rect);
        frame.render_widget(Block::bordered(), function_rect);
        render_menu_shortcut_option(frame, function_rect, "l", "LLM prompt", 0);
        render_menu_shortcut_option(frame, function_rect, "f", "file manager", 1);
        render_menu_shortcut_option(frame, function_rect, "m", "Main menu", 4);
    }

    if screen_state.screen_mode == LlmResponseMode::HandleFile {
        let handle_rect = get_center_rect(frame.area(), 20, 5);
        frame.render_widget(Clear, handle_rect);
        frame.render_widget(Block::bordered(), handle_rect);
        let selected_file = format!("Select file\n\n{}", screen_state.handle_file);
        frame.render_widget(
            Paragraph::new(selected_file),
            get_inner_rect_pad(handle_rect, 2, 1),
        );
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
            KeyCode::Char('f') => screen_state.screen_mode = LlmResponseMode::HandleFile,
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

            // For now we only handle [ ] presses when not in coding response
            // TODO, implement [ ] selection for coding response
            KeyCode::Char('[') => {
                if state.current_prompt_mode != LlmMode::Coding && screen_state.selected_block >= 1
                {
                    screen_state.selected_block -= 1;
                    screen_state.update_selected_scroll = true;
                }
            }
            KeyCode::Char(']') => {
                if state.current_prompt_mode != LlmMode::Coding {
                    if !screen_state.selected && !screen_state.code_blocks.is_empty() {
                        screen_state.selected = true;
                        screen_state.update_selected_scroll = true;
                    } else if screen_state.selected_block + 1 < screen_state.code_blocks.len() {
                        screen_state.selected_block += 1;
                        screen_state.update_selected_scroll = true;
                    }
                }
            }
            KeyCode::Char('y') => {
                if state.current_prompt_mode != LlmMode::Coding {
                    yank_to_clipboard(state, screen_state);
                }
            }
            _ => {}
        },
        LlmResponseMode::HandlePrompt => match key.code {
            KeyCode::Char('n') => {
                reset_prompt(state);
                screen_state.result_scroll = 0;
                screen_state.handle_file = String::from("0");
                screen_state.screen_mode = LlmResponseMode::Main;
                state.wipe_llm_input_prompt = true;
                state.screen = Screen::Llm; // Go to other screen
            }
            KeyCode::Char('c') => {
                // First reset everything
                continue_prompt(state);
                // Reset this screen
                screen_state.result_scroll = 0;
                screen_state.handle_file = String::from("0");
                screen_state.screen_mode = LlmResponseMode::Main;
                state.wipe_llm_input_prompt = true; // Cross comm to other screen
                state.screen = Screen::Llm; // Go to other screen
            }
            _ => {}
        },
        LlmResponseMode::HandleFile => match key.code {
            KeyCode::Char('0'..='9') => {
                if screen_state.handle_file == "0" {
                    screen_state.handle_file = key.code.to_string();
                } else {
                    screen_state.handle_file = format!("{}{}", screen_state.handle_file, key.code);
                }
            }
            KeyCode::Backspace => {
                screen_state.handle_file = String::from("0");
            }
            KeyCode::Enter => {
                // Set selected file and open file handle screen
                screen_state.screen_mode = LlmResponseMode::Main;
                let selected_file_int = screen_state
                    .handle_file
                    .parse::<usize>()
                    .unwrap_or_default();
                if selected_file_int > 0 {
                    state.llm_response_code_selected = selected_file_int - 1;
                    state.screen = Screen::HandleFile;
                }
            }
            _ => {}
        },
    }
}

// -------------------------------------------------------------------------
// Some helper functions
fn get_higlighted_spans<'a>(
    response: &'a str,
    screen_state: &mut LlmResponseScreenState,
) -> Vec<Line<'a>> {
    let selected_block = screen_state.code_blocks.get(screen_state.selected_block);
    let mut lines = Vec::new();
    match selected_block {
        None => {
            // Should be impossible, but in this case set selected to None
            // And return nothing
            screen_state.selected = false;
            screen_state.selected_block = 0;
        }
        Some(block) => {
            // Divide in three parts:
            // Before selection,
            // Selection,
            // After selection
            let select_start = block.0;
            let select_end = block.1;
            let mut nr_lines: u16 = 0;
            // Add by line, since Line will gobble up all breaks
            for line in response[0..select_start].split('\n') {
                lines.push(Line::styled(line, Style::default()));
                nr_lines += 1;
            }
            // set scroll pos to where our block begins
            screen_state.selected_line = nr_lines.saturating_sub(5);
            for line in response[select_start..select_end].split('\n') {
                lines.push(Line::styled(
                    line,
                    Style::default().fg(Color::White).bg(Color::LightBlue),
                ));
            }
            for line in response[select_end..].split('\n') {
                lines.push(Line::styled(line, Style::default()));
            }
        }
    }
    lines
}

fn yank_to_clipboard(state: &mut AppState, screen_state: &LlmResponseScreenState) {
    // Note we need to state to generate errors
    // First check if we have selected anything
    if screen_state.selected {
        let ctx_opt = ClipboardContext::new();
        match ctx_opt {
            Err(e) => {
                state.error_msg = Some(format!("Could not init clipboard, because {e}"));
            }
            Ok(ctx) => {
                // Find the selected text
                let Some(selected_block) =
                    screen_state.code_blocks.get(screen_state.selected_block)
                else {
                    return;
                };

                let response = &state.llm_response;
                let selected_text = &response[selected_block.0..selected_block.1];
                if ctx.set_text(String::from(selected_text)).is_err() {
                    state.error_msg = Some("Could not yank to clipboard".to_string())
                }
            }
        }
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
