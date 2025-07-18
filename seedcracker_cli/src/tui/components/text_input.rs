use std::{iter, marker::PhantomData, time::SystemTime};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyModifiers},
    layout::{Position, Rect},
    style::{Color, Style},
    symbols::border,
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use crate::tui::{EventContext, EventResult};

pub struct TextInputStyle {
    pub text_style: Style,
    pub title_style: Style,
    pub input_style: Style,
    pub cursor_style: Style,
    pub title: String,
    pub border_set: border::Set,
    pub border_style: Style,
    pub show_cursor: bool,
    pub cursor_state: bool,
    pub cursor_blink: bool,
    pub blink_on_time_ms: u64,
    pub blink_off_time_ms: u64,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        TextInputStyle {
            text_style: Style::default().fg(Color::White),
            title_style: Style::default().fg(Color::White),
            input_style: Style::default(),
            cursor_style: Style::default().fg(Color::Black).bg(Color::White),
            title: String::new(),
            border_set: border::DOUBLE,
            border_style: Style::default().fg(Color::White),
            show_cursor: true,
            cursor_blink: true,
            cursor_state: false,
            blink_off_time_ms: 500,
            blink_on_time_ms: 500,
        }
    }
}

#[allow(clippy::type_complexity)]
pub type Validator<T> =
    Option<Box<dyn Fn(&mut Vec<char>, &mut usize, &mut TextInputStyle, &mut T)>>;

pub struct TextInputState<T> {
    pub validator: Validator<T>,
    pub value: Vec<char>,
    pub cursor: usize,
    pub style: TextInputStyle,
    pub last_render: Rect,
}

impl<T> TextInputState<T> {
    pub fn in_rect(&self, x: u16, y: u16) -> bool {
        self.last_render.contains(Position::new(x, y))
    }

    pub fn new<U>(title: U, validator: Validator<T>) -> Self
    where
        U: ToString,
    {
        let mut state = Self::default();
        state.style.title = title.to_string();
        state.validator = validator;
        state
    }
}

impl<T> Default for TextInputState<T> {
    fn default() -> Self {
        Self {
            value: Vec::new(),
            cursor: 0,
            validator: None,
            style: TextInputStyle::default(),
            last_render: Rect::default(),
        }
    }
}

pub struct TextInputWidget<T> {
    ph: PhantomData<T>,
}

impl<T> Default for TextInputWidget<T> {
    fn default() -> Self {
        Self { ph: PhantomData }
    }
}

impl<T> TextInputWidget<T> {
    pub fn handle_event(
        state: &mut TextInputState<T>,
        event: Event,
        context: EventContext,
        shared: &mut T,
    ) -> EventResult {
        match context {
            EventContext::BubblingUp => EventResult::BubbleUp(event),
            EventContext::BubblingDown => match &event {
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.value.insert(state.cursor, c);
                        state.cursor += 1;
                        if let Some(validator) = &state.validator {
                            validator(
                                &mut state.value,
                                &mut state.cursor,
                                &mut state.style,
                                shared,
                            );
                        }
                        EventResult::Captured
                    }
                    KeyCode::Backspace if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if state.cursor > 0 {
                            state.value.remove(state.cursor - 1);
                            state.cursor -= 1;
                        }
                        if let Some(validator) = &state.validator {
                            validator(
                                &mut state.value,
                                &mut state.cursor,
                                &mut state.style,
                                shared,
                            );
                        }
                        EventResult::Captured
                    }
                    KeyCode::Delete if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if state.cursor < state.value.len() {
                            state.value.remove(state.cursor);
                        }
                        if let Some(validator) = &state.validator {
                            validator(
                                &mut state.value,
                                &mut state.cursor,
                                &mut state.style,
                                shared,
                            );
                        }
                        EventResult::Captured
                    }
                    KeyCode::Left if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if state.cursor > 0 {
                            state.cursor -= 1;
                        }
                        EventResult::Captured
                    }
                    KeyCode::Right if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if state.cursor < state.value.len() {
                            state.cursor += 1;
                        }
                        EventResult::Captured
                    }
                    KeyCode::Home if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.cursor = 0;
                        EventResult::Captured
                    }
                    KeyCode::End if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.cursor = state.value.len();
                        EventResult::Captured
                    }
                    _ => EventResult::BubbleUp(event),
                },
                Event::Paste(paste) => {
                    let left = state.value.get(0..state.cursor).unwrap_or_default();
                    let right = state.value.get(state.cursor..).unwrap_or_default();
                    let prev_len = state.value.len();
                    state.value = left
                        .iter()
                        .copied()
                        .chain(paste.chars())
                        .chain(right.iter().copied())
                        .collect();
                    state.cursor += state.value.len() - prev_len;
                    if let Some(validator) = &state.validator {
                        validator(
                            &mut state.value,
                            &mut state.cursor,
                            &mut state.style,
                            shared,
                        );
                    }
                    EventResult::Captured
                }
                _ => EventResult::BubbleUp(event),
            },
        }
    }
}

impl<T> StatefulWidget for TextInputWidget<T> {
    type State = TextInputState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.last_render = area;

        let blk = Block::new()
            .borders(Borders::ALL)
            .border_set(state.style.border_set)
            .title(state.style.title.clone())
            .style(state.style.input_style)
            .title_style(state.style.title_style)
            .border_style(state.style.border_style);
        let inner = blk.inner(area);
        blk.render(area, buf);

        let inner_l = (inner.width as usize) * (inner.height as usize) - 1;

        let begin_i = state.value.len().saturating_sub(inner_l);

        if state.style.cursor_blink {
            let total_cycle_ms =
                state.style.blink_on_time_ms as u128 + state.style.blink_off_time_ms as u128;
            let from_cycle_start = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % total_cycle_ms;

            state.style.cursor_state = from_cycle_start < state.style.blink_on_time_ms as u128;
        }

        for (i, c) in state.value[begin_i..]
            .iter()
            .chain(iter::once(&' '))
            .enumerate()
        {
            let x = i % inner.width as usize;
            let y = i / inner.width as usize;

            if let Some(cell) = buf.cell_mut((inner.x + x as u16, inner.y + y as u16)) {
                cell.set_char(*c);
                cell.set_style(if begin_i + i == state.cursor {
                    if !state.style.show_cursor {
                        if begin_i + i < state.value.len() {
                            state.style.text_style
                        } else {
                            state.style.input_style
                        }
                    } else if state.style.cursor_state {
                        state.style.cursor_style
                    } else {
                        state.style.text_style
                    }
                } else {
                    state.style.text_style
                });
            }
        }
    }
}

pub fn i32_validator() -> Validator<i32> {
    Some(Box::new(
        |value: &mut Vec<char>, _: &mut usize, style: &mut TextInputStyle, i: &mut i32| {
            if value.len() > 15 {
                style.cursor_style.bg = Some(Color::Red);
                style.text_style.fg = Some(Color::Red);
            } else if let Ok(n) = value.iter().collect::<String>().parse::<i32>() {
                style.text_style.fg = Some(Color::White);
                style.cursor_style.bg = Some(Color::Green);
                *i = n;
            } else {
                style.cursor_style.bg = Some(Color::Red);
                style.text_style.fg = Some(Color::Red);
            }
        },
    ))
}
