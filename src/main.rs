use std::io::{self, stdout};

use ratatui::{
    Terminal,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyModifiers},
    },
    prelude::CrosstermBackend,
    widgets::Paragraph,
};

use crate::tui::{
    Component, EventContext,
    application::{ApplicationComponent, ApplicationComponentState},
};

mod tui;

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
            if f.area().width < 80 || f.area().height < 30 {
                f.render_widget(Paragraph::new("Terminal window too small"), f.area());
                return;
            }
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
