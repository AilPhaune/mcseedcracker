use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect, widgets::StatefulWidget};

use crate::{
    make_full_component,
    tui::{Component, EventContext, EventResult, application::ApplicationTab},
};

#[derive(Default)]
pub struct OutputTabState {}

#[derive(Default)]
pub struct OutputTabComponent;

make_full_component!(OutputTab, state: OutputTabState, component: OutputTabComponent);

impl OutputTab {
    pub fn apptab() -> ApplicationTab {
        ApplicationTab {
            title: "Output".to_string(),
            component: OutputTab::boxed(),
        }
    }
}

impl StatefulWidget for OutputTabComponent {
    type State = OutputTabState;

    fn render(self, _area: Rect, _buf: &mut Buffer, _state: &mut Self::State) {}
}

impl Component for OutputTabComponent {
    fn handle_event(
        self,
        _state: &mut Self::State,
        event: Event,
        _context: EventContext,
    ) -> EventResult {
        EventResult::BubbleUp(event)
    }
}
