use std::{
    io::{self, stdout},
    time::Duration,
};

use mcseedcracker::search::Status;
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
    EventContext,
    application::{
        ApplicationComponent, ApplicationComponentState, PillarSeedStructureSim, StructureSeedSim,
        StructureSeedSimResultType,
    },
};

pub fn run_tui() -> Result<(), io::Error> {
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

    'app: loop {
        terminal.draw(|f| {
            if f.area().width < 160 || f.area().height < 45 {
                f.render_widget(Paragraph::new("Terminal window too small"), f.area());
                return;
            }
            ApplicationComponent::render(f.area(), f.buffer_mut(), &mut app_state);
        })?;

        if crossterm::event::poll(Duration::from_secs_f64(0.2))? {
            let event = crossterm::event::read()?;
            if let Event::Key(key) = &event {
                if (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C'))
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break 'app;
                }
            }

            ApplicationComponent::handle_event(&mut app_state, event, EventContext::BubblingDown);
        }

        if let Some(searcher) = &app_state.shared.current_structure_seed_searcher {
            if searcher.is_done() {
                let status: Status = searcher.get_status();
                let restype = match status {
                    Status::Searching => unreachable!(),
                    Status::Complete { .. } => StructureSeedSimResultType::Success,
                    Status::Cancelled { .. } => StructureSeedSimResultType::Cancelled,
                    Status::TooManySeeds { .. } => StructureSeedSimResultType::TooManySeeds,
                };
                match status {
                    Status::Searching => unreachable!(),
                    Status::Complete { seeds }
                    | Status::Cancelled {
                        seeds_incomplete: seeds,
                    }
                    | Status::TooManySeeds {
                        seeds_incomplete: seeds,
                    } => {
                        let stateref = &mut app_state.shared.last_structure_seed_sim.data;
                        match stateref {
                            None => {
                                *stateref = Some(StructureSeedSim {
                                    count_seeds: seeds.len() as i64,
                                    per_pillar: vec![PillarSeedStructureSim {
                                        pillar_seed: searcher.get_pillar_seed(),
                                        result: restype,
                                        structure_seeds: seeds,
                                    }],
                                });
                            }
                            Some(v) => {
                                v.count_seeds += seeds.len() as i64;
                                v.per_pillar.push(PillarSeedStructureSim {
                                    pillar_seed: searcher.get_pillar_seed(),
                                    result: restype,
                                    structure_seeds: seeds,
                                });
                            }
                        }
                    }
                }

                app_state.shared.current_structure_seed_searcher = None;
                if app_state.shared.structure_seed_search_jobs.is_empty() {
                    app_state.shared.last_structure_seed_sim.outdated_data = false;
                }
            }
        }

        if app_state.shared.current_structure_seed_searcher.is_none() {
            if let Some(job) = app_state.shared.structure_seed_search_jobs.pop_front() {
                app_state.shared.current_structure_seed_searcher = Some(job.spawn_multithreaded());
            }
        }
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
