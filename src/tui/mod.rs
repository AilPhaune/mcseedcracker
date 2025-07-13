use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect};

use crate::tui::application::SharedApplicationState;

pub mod application;
pub mod components;
pub mod tabs;

pub trait Component {
    type State;

    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
    );

    fn handle_event(
        self,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
        event: Event,
        context: EventContext,
    ) -> EventResult;
}

#[derive(Debug, Clone)]
pub enum EventResult {
    Captured,
    BubbleUp(Event),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventContext {
    BubblingDown,
    BubblingUp,
}

pub trait FullComponent {
    fn render(&mut self, area: Rect, buf: &mut Buffer, shared: &mut SharedApplicationState);

    fn handle_event(
        &mut self,
        event: Event,
        context: EventContext,
        shared: &mut SharedApplicationState,
    ) -> EventResult;
}

#[macro_export]
macro_rules! make_full_component {
    ($name: ident, state: $state: ident, component: $component: ident) => {
        pub struct $name {
            state: $state,
        }

        impl $name {
            pub fn create() -> $name {
                $name {
                    state: $state::default(),
                }
            }
        }

        use $crate::tui::application::SharedApplicationState;
        impl $crate::tui::FullComponent for $name {
            fn render(
                &mut self,
                area: Rect,
                buf: &mut Buffer,
                shared: &mut SharedApplicationState,
            ) {
                $component::default().render(area, buf, &mut self.state, shared);
            }

            fn handle_event(
                &mut self,
                event: Event,
                context: EventContext,
                shared: &mut SharedApplicationState,
            ) -> EventResult {
                $component::default().handle_event(&mut self.state, shared, event, context)
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
