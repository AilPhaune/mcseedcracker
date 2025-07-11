use std::any::Any;

use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect, widgets::StatefulWidget};

pub mod application;
pub mod components;
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

pub const fn get_area_centered(area: Rect, container: Rect) -> Rect {
    let x = container.x as i32 + (container.width as i32 - area.width as i32) / 2;
    let y = container.y as i32 + (container.height as i32 - area.height as i32) / 2;
    Rect::new(x as u16, y as u16, area.width, area.height)
}

pub const fn limit_area_height(area: Rect, max_height: u16) -> Rect {
    let height = if area.height > max_height {
        max_height
    } else {
        area.height
    };
    Rect::new(area.x, area.y, area.width, height)
}

pub const fn limit_area_width(area: Rect, max_width: u16) -> Rect {
    let width = if area.width > max_width {
        max_width
    } else {
        area.width
    };
    Rect::new(area.x, area.y, width, area.height)
}
