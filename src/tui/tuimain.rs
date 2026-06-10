use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::event::KeyModifiers;
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::Rect,
    style::Stylize,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{AppState, LlmResponseState, Screen};

use crate::tui::llmresponse::{
    create_llm_response_state, draw_llm_response_screen, handle_event_llm_response_screen,
};
use crate::tui::llmscreen::{create_llm_screen_state, draw_llm_screen, handle_event_llm_screen};
use crate::tui::menu::{create_menu_state, draw_menu, handle_event_menu};
use crate::tui::title::{create_title_state, draw_title, handle_event_title};

use super::layoututils::get_inner_rect;

pub fn start_tui(app_state: AppState) -> Result<(), Box<dyn Error>> {
    // Setup terminal and stuff
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let finished = run_crodot(&mut terminal, app_state);

    // restore terminal stuff
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if finished.is_ok() {
        log::info!("Tui closed");
    } else if let Err(err) = finished {
        log::error!("Exited with error.");
        log::error!("{err}");
    }

    Ok(())
}

fn run_crodot<B: Backend>(terminal: &mut Terminal<B>, mut app_state: AppState) -> io::Result<bool>
where
    io::Error: From<B::Error>,
{
    // For loop and tick control
    let tick_rate = Duration::from_millis(1000);
    let mut last_tick = Instant::now();

    // Init state for the different screens
    let mut title_state = create_title_state();
    let mut menu_state = create_menu_state();
    let mut llm_state = create_llm_screen_state();
    let mut llm_response_state = create_llm_response_state();

    loop {
        // Check if we are in exit mode
        if app_state.exit {
            // Simply return from this loop function
            return Ok(true);
        }

        // Draw stuff
        terminal.draw(|frame| {
            // Check if screen size is ok
            if frame.area().width < 80 || frame.area().height < 35 {
                draw_too_small_screen(frame);
            } else {
                match app_state.screen {
                    Screen::Title => draw_title(frame, &mut app_state, &mut title_state),
                    Screen::Menu => draw_menu(frame, &mut app_state, &mut menu_state),
                    Screen::Llm => draw_llm_screen(frame, &mut app_state, &mut llm_state),
                    Screen::LlmResponse => {
                        draw_llm_response_screen(frame, &mut app_state, &mut llm_response_state)
                    }
                }
                // Check for error message, we draw this later
                if let Some(error_msg) = &app_state.error_msg {
                    draw_error_message(frame, error_msg);
                }
            }
        })?;

        // calc timeout
        // If for whatever reason it fails, default to standard tick rate
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(100));

        // Listen for input
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.is_release() {
                    // Skip release (only press events)
                    // to prevent double events on windows
                    continue;
                }
                // Generic exit code
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
                    app_state.exit = true;
                    continue;
                }
                // Check for error
                // Exit on unrecoverable error
                if app_state.error_msg.is_some() {
                    if key.code == KeyCode::Esc {
                        app_state.error_msg = None;
                        if app_state.unrecoverable_error {
                            app_state.exit = true;
                        }
                    }
                    continue;
                }
                // Pass key event to screen module
                match app_state.screen {
                    Screen::Title => handle_event_title(key, &mut app_state, &mut title_state),
                    Screen::Menu => handle_event_menu(key, &mut app_state, &mut menu_state),
                    Screen::Llm => handle_event_llm_screen(key, &mut app_state, &mut llm_state),
                    Screen::LlmResponse => handle_event_llm_response_screen(
                        key,
                        &mut app_state,
                        &mut llm_response_state,
                    ),
                }
            }
        }

        // Check tick rate
        if last_tick.elapsed() >= tick_rate {
            // handle ticky things,
            // For now we do not have animations, so we can simply check the
            //   llm response only.
            // Refactor this as more async / tick stuff is happening
            if app_state.llm_response_state == LlmResponseState::Waiting {
                // set response to app_state and change llm state
                if let Ok(response) = app_state.receiver_channel.try_recv() {
                    app_state.llm_response = response;
                    app_state.llm_response_state = LlmResponseState::NewResponse;
                } else {
                    app_state.llm_response_secs += 1;
                }
            }

            last_tick = Instant::now();
        }
    }
}

fn draw_too_small_screen(frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new("The terminal windows is too small. Please increase its size."),
        frame.area(),
    );
}

fn draw_error_message(frame: &mut Frame, error_msg: &str) {
    let error_rect = Rect {
        x: 3,
        y: 3,
        width: frame.area().width - 6,
        height: 8,
    };

    frame.render_widget(Clear, error_rect);
    frame.render_widget(Block::bordered().red(), error_rect);
    frame.render_widget(
        Paragraph::new(error_msg).wrap(Wrap { trim: true }),
        get_inner_rect(error_rect, 2, 2, 2, 1),
    );
}
