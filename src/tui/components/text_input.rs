use std::{iter, marker::PhantomData};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::Rect,
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
    pub blink_count: i64,
    pub blink_on_time: i64,
    pub blink_off_time: i64,
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
            blink_count: 0,
            blink_off_time: 3,
            blink_on_time: 3,
        }
    }
}

pub struct TextInputState<T> {
    #[allow(clippy::type_complexity)]
    pub validator: Option<Box<dyn Fn(&mut Vec<char>, &mut usize, &mut TextInputStyle, &mut T)>>,
    pub value: Vec<char>,
    pub cursor: usize,
    pub style: TextInputStyle,
}

impl<T> Default for TextInputState<T> {
    fn default() -> Self {
        Self {
            value: Vec::new(),
            cursor: 0,
            validator: None,
            style: TextInputStyle::default(),
        }
    }
}

#[derive(Default)]
pub struct TextInputWidget<T> {
    ph: PhantomData<T>,
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
                    KeyCode::Char(c) => {
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
                    KeyCode::Backspace => {
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
                    KeyCode::Delete => {
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
                    KeyCode::Left => {
                        if state.cursor > 0 {
                            state.cursor -= 1;
                        }
                        EventResult::Captured
                    }
                    KeyCode::Right => {
                        if state.cursor < state.value.len() {
                            state.cursor += 1;
                        }
                        EventResult::Captured
                    }
                    KeyCode::Home => {
                        state.cursor = 0;
                        EventResult::Captured
                    }
                    KeyCode::End => {
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
            state.style.blink_count += 1;
            if state.style.cursor_state && state.style.blink_count >= state.style.blink_on_time {
                state.style.cursor_state = false;
                state.style.blink_count = 0;
            } else if !state.style.cursor_state
                && state.style.blink_count >= state.style.blink_off_time
            {
                state.style.cursor_state = true;
                state.style.blink_count = 0;
            }
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
