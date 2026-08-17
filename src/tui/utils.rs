/// Random helper functions for tui stuff
use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{AppState, LlmResponseState};

/// Render key option for the function overlay
pub fn render_menu_shortcut_option(
    frame: &mut Frame,
    container: Rect,
    shortcut: &str,
    option: &str,
    index: u16,
) {
    let option_line: Line = vec![shortcut.yellow().bold(), " -> ".into(), option.into()].into();
    frame.render_widget(
        Paragraph::new(option_line),
        Rect {
            x: container.x + 2,
            y: container.y + 2 + index,
            width: container.width - 4,
            height: 1,
        },
    );
}

/// Render main border
pub fn render_main_border(frame: &mut Frame, title: &str, app_state: &AppState) {
    frame.render_widget(Block::bordered(), frame.area());

    // Show title
    // If the title is longer than a u16 we have different problems :)
    let title_len = u16::try_from(title.len()).unwrap_or_default();
    frame.render_widget(
        Paragraph::new(format!(" {title} ")).green(),
        Rect {
            x: 2,
            y: 0,
            width: title_len + 2,
            height: 1,
        },
    );

    // Now show the llm state in bottomt corner
    if app_state.llm_response_state == LlmResponseState::Waiting {
        frame.render_widget(
            Paragraph::new(
                format!(
                    " {} - {} ",
                    app_state.llm_response_state, app_state.llm_response_secs
                )
                .red(),
            ),
            Rect {
                x: frame.area().width - 20,
                y: frame.area().height - 1,
                width: 19,
                height: 1,
            },
        );
    } else {
        frame.render_widget(
            Paragraph::new(format!(" {} ", app_state.llm_response_state).green()),
            Rect {
                x: frame.area().width - 20,
                y: frame.area().height - 1,
                width: 19,
                height: 1,
            },
        );
    }
}
