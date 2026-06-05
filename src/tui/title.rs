// The functions that draw the home / start screen go here

use ratatui::Frame;
use ratatui::crossterm::event;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::{AppState, Screen};

pub fn draw_title(frame: &mut Frame, _state: &mut AppState) {
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
}

pub fn handle_event_title(key: event::KeyEvent, state: &mut AppState) {
    match key.kind {
        _ => state.screen = Screen::Menu,
    }
}

// Static texts go here!

const DISCLAIMER: &str = r#"
CRODOT is a tool for coding with AI's. Some more info will be added to this screen later, hopefully.

Press ctrl + q to exit the app at any time.
"#;
