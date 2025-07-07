use std::io::{self, stdout};

use ratatui::{
    Terminal,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyModifiers},
    },
    prelude::CrosstermBackend,
};

use crate::tui::{
    Component, EventContext,
    application::{ApplicationComponent, ApplicationComponentState},
};

pub mod discrete_log;
pub mod features;
pub mod lcg;
pub mod math;
pub mod random;
pub mod tui;

pub const CHARACTER_ASPECT_RATIO: f64 = 0.5; // width/height

fn main() -> Result<(), io::Error> {
    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = ApplicationComponentState::new();

    loop {
        terminal.draw(|f| {
            f.render_stateful_widget(ApplicationComponent, f.area(), &mut app_state);
        })?;

        let event = crossterm::event::read()?;
        if let Event::Key(key) = &event {
            if (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C'))
                && key.modifiers.contains(KeyModifiers::CONTROL)
            {
                break;
            }
        }

        ApplicationComponent.handle_event(&mut app_state, event, EventContext::BubblingDown);
    }

    // Cleanup terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
