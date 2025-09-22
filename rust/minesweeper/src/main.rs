
// use crossterm::{
//     event::{self, Event, KeyCode},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };
// use ratatui::{
//     prelude::*,
//     widgets::{Block, Borders, Paragraph},
// };
// use std::io::{self, stdout};

// fn color_foreground(text: &str, hex: &str) -> String {
//     // adds color to the foreground of text
//     let h = hex.trim_start_matches('#');
//     let r = u8::from_str_radix(&h[0..2], 16).unwrap();
//     let g = u8::from_str_radix(&h[2..4], 16).unwrap();
//     let b = u8::from_str_radix(&h[4..6], 16).unwrap();

//     format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
// }

// fn color_background(text: &str, hex: &str) -> String {
//     // adds color to the background of text
//     let h = hex.trim_start_matches('#');
//     let r = u8::from_str_radix(&h[0..2], 16).unwrap();
//     let g = u8::from_str_radix(&h[2..4], 16).unwrap();
//     let b = u8::from_str_radix(&h[4..6], 16).unwrap();

//     format!("\x1b[48;2;{r};{g};{b}m{text}\x1b[0m")
// }

// fn color_text(text: &str, hex_bg: &str, hex_fg: &str) -> String {
//     // adds color to foreground and background of text
//     color_background(&color_foreground(text,hex_fg), hex_bg)
// }


use std::{error::Error, io};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
mod ui;
use crate::{
    app::{App, CurrentScreen},
    ui::ui,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout(); // This is a special case. Normally using stdout is fine
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let _res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::Menu => match key.code {
                    KeyCode::Char('e') => {
                        app.current_screen = CurrentScreen::Gameplay;
                    }
                    KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    _ => {}
                },
                CurrentScreen::Gameplay => match key.code {
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Menu;
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Menu;
                    }
                    _ => {}
                }
            }
        }
    }
}