use std::f64::consts::{FRAC_PI_2, PI};

use mcseedcracker::{
    CHARACTER_ASPECT_RATIO,
    features::end_pillars::{PartialEndPillars, PillarHeightHint, PillarMatchResult},
};

use crate::{
    make_full_component,
    tui::{Component, EventContext, EventResult, application::ApplicationTab, limit_area_height},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

#[derive(Default)]
pub enum WaitingOf {
    #[default]
    Nothing,
    PillarRangeMin,
    PillarRangeMax,
    PillarExact,
}

#[derive(Default)]
pub struct EndPillarsTabState {
    pub focused_on_pillar: Option<usize>,
    pub pillars: PartialEndPillars,
    pub rot: usize, // from 0 to 10, clockwise
    pub waiting: WaitingOf,
}

#[derive(Default)]
pub struct EndPillarsTabComponent;

make_full_component!(EndPillarsTab, state: EndPillarsTabState, component: EndPillarsTabComponent);

impl EndPillarsTab {
    pub fn apptab() -> ApplicationTab {
        ApplicationTab {
            title: "End Pillars".to_string(),
            component: EndPillarsTab::boxed(),
        }
    }
}

macro_rules! selected {
    ($style: expr, $is_selected: expr) => {
        if $is_selected {
            $style.fg(Color::LightCyan).bold()
        } else {
            $style
        }
    };
}

impl StatefulWidget for EndPillarsTabComponent {
    type State = EndPillarsTabState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let seed_results = state
            .pillars
            .seed_results()
            .into_iter()
            .filter(|(_, result)| match result {
                PillarMatchResult::ImpossibleMatch => false,
                PillarMatchResult::PossibleMatch(v) => *v != 0.0,
                _ => true,
            })
            .collect::<Vec<_>>();

        let title = Paragraph::new(format!("Valid pillar seeds count: {}", seed_results.len()))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);

        title.render(area, buf);

        if seed_results.len() == 1 {
            let subtitle = Paragraph::new(format!("Seed: {}", seed_results[0].0))
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);

            let mut subarea = area;
            subarea.y += 1;
            subarea.height = 1;

            subtitle.render(subarea, buf);
        }

        {
            // controls

            let l1 = Paragraph::new("Arrow keys").style(Style::default().fg(Color::Yellow).bold());
            let l2_1 = Paragraph::new("[LEFT] [RIGHT]")
                .style(Style::default().fg(Color::Magenta).not_bold());
            let l2_2 =
                Paragraph::new(" Rotate").style(Style::default().fg(Color::Green).not_bold());
            let l3_1 =
                Paragraph::new("[UP] [DOWN]").style(Style::default().fg(Color::Magenta).not_bold());
            let l3_2 = Paragraph::new(" Change pillar height hint")
                .style(Style::default().fg(Color::Green).not_bold());

            let l4 = Paragraph::new("Selection").style(Style::default().fg(Color::Yellow).bold());
            let l5_1 =
                Paragraph::new("[TAB]").style(Style::default().fg(Color::Magenta).not_bold());
            let l5_2 = Paragraph::new(" Next pillar")
                .style(Style::default().fg(Color::Green))
                .not_bold();
            let l6_1 = Paragraph::new("[0] [1] [2] [3] [4] [5] [6] [7] [8] [9]")
                .style(Style::default().fg(Color::Magenta).not_bold());
            let l6_2 = Paragraph::new(" Select pillar")
                .style(Style::default().fg(Color::Green).not_bold());

            let l7 = Paragraph::new("Pillars").style(Style::default().fg(Color::Yellow).bold());
            let l8_1 = Paragraph::new("[DEL] [BACKSPACE] [SPACE]")
                .style(Style::default().fg(Color::Magenta).not_bold());
            let l8_2 = Paragraph::new(" Reset pillar data")
                .style(Style::default().fg(Color::Green).not_bold());
            let l9_1 = Paragraph::new("[C]").style(Style::default().fg(Color::Magenta).not_bold());
            let l9_2 = Paragraph::new(" Cycle pillar `caged` status")
                .style(Style::default().fg(Color::Green).not_bold());
            let l10_1 =
                Paragraph::new("[ENTER]").style(Style::default().fg(Color::Magenta).not_bold());
            let l10_2 = Paragraph::new(" Wait for exact size")
                .style(Style::default().fg(Color::Green).not_bold());
            let l11_1 = Paragraph::new("[R]").style(Style::default().fg(Color::Magenta).not_bold());
            let l11_2 = Paragraph::new(" Wait for height range")
                .style(Style::default().fg(Color::Green).not_bold());

            let l12 = Paragraph::new("[0] [1] [2] [3] [4] [5] [6] [7] [8] [9]")
                .style(Style::default().fg(Color::Yellow).bold());
            let l13_1 = Paragraph::new("Waiting for exact size:")
                .style(Style::default().fg(Color::Magenta).not_bold());
            let l13_2 = Paragraph::new(" 76 + 3*n").style(Style::default().fg(Color::Green));
            let l14_1 = Paragraph::new("Waiting for height range:")
                .style(Style::default().fg(Color::Magenta).not_bold());
            let l14_2 = Paragraph::new(" 1st key press = range min\n 2nd key press = range max")
                .style(Style::default().fg(Color::Green).not_bold());

            let mut larea = limit_area_height(area, 1);
            larea.y += 2;
            l1.render(larea, buf);
            larea.y += 1;
            l2_1.render(larea, buf);
            l2_2.render(larea.inner(Margin::new(14, 0)), buf);
            larea.y += 1;
            l3_1.render(larea, buf);
            l3_2.render(larea.inner(Margin::new(11, 0)), buf);
            larea.y += 2;
            l4.render(larea, buf);
            larea.y += 1;
            l5_1.render(larea, buf);
            l5_2.render(larea.inner(Margin::new(5, 0)), buf);
            larea.y += 1;
            l6_1.render(larea, buf);
            l6_2.render(larea.inner(Margin::new(39, 0)), buf);
            larea.y += 2;
            l7.render(larea, buf);
            larea.y += 1;
            l8_1.render(larea, buf);
            l8_2.render(larea.inner(Margin::new(25, 0)), buf);
            larea.y += 1;
            l9_1.render(larea, buf);
            l9_2.render(larea.inner(Margin::new(3, 0)), buf);
            larea.y += 1;
            l10_1.render(larea, buf);
            l10_2.render(larea.inner(Margin::new(7, 0)), buf);
            larea.y += 1;
            l11_1.render(larea, buf);
            l11_2.render(larea.inner(Margin::new(3, 0)), buf);
            larea.y += 2;
            l12.render(larea, buf);
            larea.y += 1;
            l13_1.render(larea, buf);
            l13_2.render(larea.inner(Margin::new(23, 0)), buf);
            larea.y += 1;
            l14_1.render(larea, buf);
            l14_2.render(larea.inner(Margin::new(25, 0)), buf);
        }

        let center_x = area.x + area.width / 2;
        let center_y = area.y + area.height / 2;

        let effective_height = area.height;
        let effective_width = (area.width as f64 * CHARACTER_ASPECT_RATIO).floor() as u16;

        let content_square_size = effective_height.min(effective_width);

        let area = Rect {
            x: center_x - content_square_size / 2,
            y: center_y - content_square_size / 2,
            width: content_square_size,
            height: content_square_size,
        };

        // Render centered end portal

        let portal_box_height: u16 = 5;
        let portal_box_width = ((portal_box_height as f64 / CHARACTER_ASPECT_RATIO).floor() as u16)
            .next_multiple_of(2);

        let end_portal_box = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        end_portal_box.render(
            Rect {
                x: center_x - portal_box_width / 2,
                y: center_y - portal_box_height / 2,
                width: portal_box_width,
                height: portal_box_height,
            },
            buf,
        );

        let end_portal_text = Paragraph::new("Portal").alignment(Alignment::Center);

        end_portal_text.render(
            Rect {
                x: area.x,
                y: center_y,
                width: area.width,
                height: 1,
            },
            buf,
        );

        // Render end pillars
        for (i, pillar_data) in state.pillars.iter_mut().enumerate() {
            let pillar_box_height: u16 = 5;
            let pillar_box_width = ((pillar_box_height as f64 / CHARACTER_ASPECT_RATIO).floor()
                as u16)
                .next_multiple_of(2);

            let pillar_box = Block::default()
                .borders(Borders::ALL)
                .title(format!("#{}", i))
                .title_alignment(Alignment::Center);

            let pillar_box = match pillar_data.caged {
                Some(true) => pillar_box.border_style(selected!(
                    Style::default().fg(Color::White).bg(Color::Red),
                    Some(i) == state.focused_on_pillar
                )),
                Some(false) => pillar_box.border_style(selected!(
                    Style::default().fg(Color::White).bg(Color::LightGreen),
                    Some(i) == state.focused_on_pillar
                )),
                None => pillar_box.border_style(selected!(
                    Style::default().fg(Color::White),
                    Some(i) == state.focused_on_pillar
                )),
            };

            let angle = FRAC_PI_2 + 2.0 * PI * ((i + state.rot) as f64 / 10.0);

            if i == 0 {
                let platform_x = (content_square_size as f64 * angle.cos() * 0.4
                    / CHARACTER_ASPECT_RATIO)
                    .round() as i16
                    + center_x as i16;

                let platform_y = (content_square_size as f64 * angle.sin() * 0.4).round() as i16
                    + center_y as i16;

                let platform_box_area = Rect {
                    x: platform_x as u16 - pillar_box_width / 2,
                    y: platform_y as u16 - pillar_box_height / 2,
                    width: pillar_box_width,
                    height: pillar_box_height,
                };

                let platform_box = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White).bg(Color::Magenta));

                let mut inner_area = platform_box.inner(platform_box_area);
                platform_box.render(platform_box_area, buf);

                inner_area.y += inner_area.height / 2;
                inner_area.height = 1;

                let platform_text = Paragraph::new("Platform")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::White));
                platform_text.render(inner_area, buf);
            }

            let x = (content_square_size as f64 * angle.cos() * 0.25 / CHARACTER_ASPECT_RATIO)
                .round() as i16
                + center_x as i16;
            let y =
                (content_square_size as f64 * angle.sin() * 0.25).round() as i16 + center_y as i16;

            let pillar_box_area = Rect {
                x: x as u16 - pillar_box_width / 2,
                y: y as u16 - pillar_box_height / 2,
                width: pillar_box_width,
                height: pillar_box_height,
            };

            let mut inner_area = pillar_box.inner(pillar_box_area);
            pillar_box.render(pillar_box_area, buf);

            let height_text = Paragraph::new(match pillar_data.height {
                PillarHeightHint::Unknown => "??".to_string(),
                PillarHeightHint::Big => "BB".to_string(),
                PillarHeightHint::Medium => "MM".to_string(),
                PillarHeightHint::Small => "SS".to_string(),
                PillarHeightHint::MediumBig => "MB".to_string(),
                PillarHeightHint::MediumSmall => "MS".to_string(),
                PillarHeightHint::Exact(h) => format!("={:03}", h),
                PillarHeightHint::Range(min, max) => format!("{:03}->{:03}", min, max),
            })
            .alignment(Alignment::Center);

            inner_area.y += inner_area.height / 2;
            inner_area.height = 1;

            height_text.render(inner_area, buf);
        }
    }
}

impl Component for EndPillarsTabComponent {
    fn handle_event(
        self,
        state: &mut Self::State,
        event: Event,
        context: EventContext,
    ) -> EventResult {
        match context {
            EventContext::BubblingDown => match &event {
                Event::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
                        && key.code == KeyCode::Tab =>
                {
                    state.focused_on_pillar = match state.focused_on_pillar {
                        None => Some(0),
                        Some(i) => Some((i + 1) % 10),
                    };
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Esc => {
                    state.focused_on_pillar = None;
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Right => {
                    state.rot = (state.rot + 1) % 10;
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Left => {
                    state.rot = (state.rot + 9) % 10;
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Char('c') => {
                    if let Some(i) = state.focused_on_pillar {
                        let pillar = &mut state.pillars.0[i];
                        pillar.caged = match pillar.caged {
                            None => Some(true),
                            Some(true) => Some(false),
                            Some(false) => None,
                        };
                    }
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Up => {
                    if let Some(i) = state.focused_on_pillar {
                        let pillar = &mut state.pillars.0[i];
                        pillar.height = match pillar.height {
                            PillarHeightHint::Unknown => PillarHeightHint::Small,
                            PillarHeightHint::Small => PillarHeightHint::MediumSmall,
                            PillarHeightHint::MediumSmall => PillarHeightHint::Medium,
                            PillarHeightHint::Medium => PillarHeightHint::MediumBig,
                            PillarHeightHint::MediumBig => PillarHeightHint::Big,
                            o => o,
                        };
                    }
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Down => {
                    if let Some(i) = state.focused_on_pillar {
                        let pillar = &mut state.pillars.0[i];
                        pillar.height = match pillar.height {
                            PillarHeightHint::Unknown => PillarHeightHint::Big,
                            PillarHeightHint::Big => PillarHeightHint::MediumBig,
                            PillarHeightHint::MediumBig => PillarHeightHint::Medium,
                            PillarHeightHint::Medium => PillarHeightHint::MediumSmall,
                            PillarHeightHint::MediumSmall => PillarHeightHint::Small,
                            o => o,
                        };
                    }
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key)
                    if key.code == KeyCode::Backspace
                        || key.code == KeyCode::Delete
                        || key.code == KeyCode::Char(' ') =>
                {
                    if let Some(i) = state.focused_on_pillar {
                        let pillar = &mut state.pillars.0[i];
                        pillar.height = PillarHeightHint::Unknown;
                    }
                    state.waiting = WaitingOf::Nothing;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Enter => {
                    state.waiting = WaitingOf::PillarExact;
                    EventResult::Captured
                }
                Event::Key(key) if key.code == KeyCode::Char('r') => {
                    state.waiting = WaitingOf::PillarRangeMin;
                    EventResult::Captured
                }
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let n = c.to_digit(10).unwrap() as usize;
                        match state.waiting {
                            WaitingOf::Nothing => {
                                state.focused_on_pillar = Some(n);
                                state.waiting = WaitingOf::Nothing;
                            }
                            WaitingOf::PillarRangeMin => {
                                if let Some(i) = state.focused_on_pillar {
                                    let pillar = &mut state.pillars.0[i];
                                    let max = match pillar.height {
                                        PillarHeightHint::Exact(h)
                                        | PillarHeightHint::Range(_, h) => h,
                                        _ => 103,
                                    };
                                    let min = 76 + 3 * n as i32;
                                    pillar.height = if min == max {
                                        PillarHeightHint::Exact(min)
                                    } else {
                                        PillarHeightHint::Range(min, max)
                                    };
                                }
                                state.waiting = WaitingOf::PillarRangeMax;
                            }
                            WaitingOf::PillarRangeMax => {
                                if let Some(i) = state.focused_on_pillar {
                                    let pillar = &mut state.pillars.0[i];
                                    let min = match pillar.height {
                                        PillarHeightHint::Exact(h)
                                        | PillarHeightHint::Range(h, _) => h,
                                        _ => 76,
                                    };
                                    let max = 76 + 3 * n as i32;
                                    pillar.height = if min == max {
                                        PillarHeightHint::Exact(min)
                                    } else {
                                        PillarHeightHint::Range(min, max)
                                    };
                                }
                                state.waiting = WaitingOf::Nothing;
                            }
                            WaitingOf::PillarExact => {
                                if let Some(i) = state.focused_on_pillar {
                                    let pillar = &mut state.pillars.0[i];
                                    pillar.height = PillarHeightHint::Exact(76 + 3 * n as i32);
                                }
                                state.waiting = WaitingOf::Nothing;
                            }
                        }
                        EventResult::Captured
                    }
                    _ => EventResult::BubbleUp(event),
                },
                _ => EventResult::BubbleUp(event),
            },
            EventContext::BubblingUp => EventResult::BubbleUp(event),
        }
    }
}
