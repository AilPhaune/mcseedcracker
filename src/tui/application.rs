use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, StatefulWidget, Tabs, Widget},
};

use crate::tui::{
    Component, EventContext, EventResult, FullComponent,
    tabs::{buried_treasure::BuriedTreasureTab, end_pillars::EndPillarsTab, output::OutputTab},
};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ApplicationComponent;

pub struct ApplicationComponentState {
    pub selected_tab: usize,
    pub focused_on_tab: bool,

    pub tabs: Vec<ApplicationTab>,
}

pub struct ApplicationTab {
    pub title: String,
    pub component: Box<dyn FullComponent>,
}

impl ApplicationComponentState {
    pub fn new() -> Self {
        Self {
            selected_tab: 0,
            focused_on_tab: true,
            tabs: vec![
                EndPillarsTab::apptab(),
                BuriedTreasureTab::apptab(),
                OutputTab::apptab(),
            ],
        }
    }
}

impl Default for ApplicationComponentState {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for ApplicationComponent {
    type State = ApplicationComponentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Height of the tabs bar
                    Constraint::Min(0),    // Rest of the area
                ]
                .as_ref(),
            )
            .split(area);

        let titles = state
            .tabs
            .iter()
            .map(|tab| Span::from(tab.title.clone()))
            .collect::<Vec<_>>();

        let tabs = Tabs::new(titles)
            .select(state.selected_tab)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if state.focused_on_tab {
                        Style::default().bold().fg(Color::LightCyan)
                    } else {
                        Style::default()
                    })
                    .title("Tabs"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).underlined());

        tabs.render(chunks[0], buf);

        // Placeholder for tab content
        let content_block = Block::default()
            .title(state.tabs[state.selected_tab].title.clone())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(if state.focused_on_tab {
                Style::default()
            } else {
                Style::default().bold().fg(Color::LightCyan)
            });

        let content_area = content_block.inner(chunks[1]);

        content_block.render(chunks[1], buf);

        state.tabs[state.selected_tab]
            .component
            .render(content_area, buf);
    }
}

impl Component for ApplicationComponent {
    fn handle_event(
        self,
        state: &mut Self::State,
        event: Event,
        context: EventContext,
    ) -> EventResult {
        match context {
            EventContext::BubblingDown => {
                if state.focused_on_tab {
                    match &event {
                        Event::Key(key)
                            if key.code == KeyCode::Tab || key.code == KeyCode::BackTab =>
                        {
                            state.focused_on_tab = false;
                            EventResult::Captured
                        }
                        Event::Key(key) if key.code == KeyCode::Right => {
                            state.selected_tab = (state.selected_tab + 1) % state.tabs.len();
                            EventResult::Captured
                        }
                        Event::Key(key) if key.code == KeyCode::Left => {
                            state.selected_tab =
                                (state.selected_tab + state.tabs.len() - 1) % state.tabs.len();
                            EventResult::Captured
                        }
                        _ => EventResult::BubbleUp(event),
                    }
                } else {
                    match state.tabs[state.selected_tab]
                        .component
                        .handle_event(event, context)
                    {
                        EventResult::Captured => EventResult::Captured,
                        EventResult::BubbleUp(event) => {
                            self.handle_event(state, event, EventContext::BubblingUp)
                        }
                    }
                }
            }
            EventContext::BubblingUp => match &event {
                Event::Key(key) if key.code == KeyCode::Tab || key.code == KeyCode::BackTab => {
                    state.focused_on_tab = !state.focused_on_tab;
                    EventResult::Captured
                }
                _ => EventResult::BubbleUp(event),
            },
        }
    }
}
