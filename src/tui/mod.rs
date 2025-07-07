use std::any::Any;

use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect, widgets::StatefulWidget};

pub mod application;
pub mod tabs;

pub trait Component: StatefulWidget {
    fn handle_event(
        self,
        state: &mut Self::State,
        event: Event,
        context: EventContext,
    ) -> EventResult;
}

#[derive(Debug, Clone)]
pub enum EventResult {
    Captured,
    BubbleUp(Event),
}

#[derive(Debug, Clone, Copy)]
pub enum EventContext {
    BubblingDown,
    BubblingUp,
}

pub trait FullComponent {
    fn get_state(&self) -> &dyn Any;
    fn get_state_mut(&mut self) -> &mut dyn Any;
    fn render(&mut self, area: Rect, buf: &mut Buffer);
    fn handle_event(&mut self, event: Event, context: EventContext) -> EventResult;
}

#[macro_export]
macro_rules! make_full_component {
    ($name: ident, state: $state: ident, component: $component: ident) => {
        pub struct $name {
            state: $state,
        }

        impl $name {
            pub fn boxed() -> Box<dyn $crate::tui::FullComponent> {
                Box::new($name {
                    state: $state::default(),
                })
            }
        }

        impl $crate::tui::FullComponent for $name {
            fn get_state(&self) -> &dyn std::any::Any {
                &self.state
            }

            fn get_state_mut(&mut self) -> &mut dyn std::any::Any {
                &mut self.state
            }

            fn render(&mut self, area: Rect, buf: &mut Buffer) {
                $component::default().render(area, buf, &mut self.state);
            }

            fn handle_event(&mut self, event: Event, context: EventContext) -> EventResult {
                $component::default().handle_event(&mut self.state, event, context)
            }
        }
    };
}
