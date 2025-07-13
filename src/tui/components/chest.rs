use mcseedcracker::CHARACTER_ASPECT_RATIO;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Offset, Rect},
    style::{Color, Style},
    symbols::{border::Set, line},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::tui::{get_area_centered, limit_area_height};

pub struct ChestState {
    pub width: usize,
    pub height: usize,
    pub contents: Vec<Vec<(String, i32, Style)>>,
    pub selected: (usize, usize),
    pub borders_style: Style,
    pub show_selected: bool,
    pub title_style: Style,
    pub title: String,
}

impl Default for ChestState {
    fn default() -> Self {
        Self {
            width: 9,
            height: 3,
            contents: vec![vec![("".to_string(), 0, Style::default()); 9]; 3],
            selected: (0, 0),
            show_selected: true,
            borders_style: Style::default().fg(Color::White),
            title_style: Style::default().fg(Color::White),
            title: "Chest".to_string(),
        }
    }
}

pub struct ChestWidget;

impl ChestWidget {
    pub fn calculate_size(area: Rect, state: &ChestState) -> Option<(usize, usize)> {
        let max_width = area.width as usize;
        let max_height = area.height as usize;

        let max_cell_width = max_width.div_ceil(state.width);

        let mut last_ok = None;

        for try_cell_width in 0..max_cell_width {
            let whole_width = try_cell_width * state.width + 1;
            let expected_cell_height =
                (try_cell_width as f64 * CHARACTER_ASPECT_RATIO).floor() as usize;
            let whole_height = expected_cell_height * state.height + 3;

            if whole_height <= max_height && whole_width <= max_width {
                last_ok = Some((try_cell_width, expected_cell_height));
            } else {
                break;
            }
        }

        last_ok
    }
}

impl StatefulWidget for ChestWidget {
    type State = ChestState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Some((cell_width, cell_height)) = Self::calculate_size(area, state) else {
            let p = Paragraph::new("Warn:NotEnoughSpace").style(Style::default().fg(Color::Red));
            p.render(area, buf);
            return;
        };

        let whole_width = cell_width * state.width + 1;
        let whole_height = cell_height * state.height + 3;

        let area = get_area_centered(
            Rect {
                x: 0,
                y: 0,
                width: whole_width as u16,
                height: whole_height as u16,
            },
            area,
        );

        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(state.borders_style);

        let title_area = limit_area_height(area, 3);

        let title = Paragraph::new(state.title.clone()).style(state.title_style);

        title_block.render(title_area, buf);
        title.render(title_area.inner(Margin::new(1, 1)), buf);

        for slot_x in 0..state.width {
            for slot_y in 0..state.height {
                let slot_area = Rect {
                    x: area.x + (slot_x * cell_width) as u16,
                    y: area.y + (slot_y * cell_height + 2) as u16,
                    width: cell_width as u16 + 1,
                    height: cell_height as u16 + 1,
                };

                let (item, quant, style) = state.contents[slot_y][slot_x].clone();

                let item = Paragraph::new(item)
                    .style(style)
                    .alignment(Alignment::Center);
                let quant = Paragraph::new(format!("x{:02}", quant))
                    .style(
                        if state.show_selected && state.selected == (slot_x, slot_y) {
                            Style::default().bg(Color::LightBlue)
                        } else {
                            Style::default()
                        },
                    )
                    .alignment(Alignment::Center);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_set(Set {
                        top_left: if slot_x == 0 {
                            line::NORMAL.vertical_right
                        } else if slot_y == 0 {
                            line::NORMAL.horizontal_down
                        } else {
                            line::NORMAL.cross
                        },
                        bottom_left: if slot_x == 0 {
                            line::NORMAL.bottom_left
                        } else {
                            line::NORMAL.horizontal_up
                        },
                        top_right: line::NORMAL.vertical_left,
                        ..Default::default()
                    })
                    .style(state.borders_style);

                let data_area = block.inner(slot_area);
                block.render(slot_area, buf);

                let h = 1 + state.contents[slot_y][slot_x].0.lines().count().max(1);

                let data_area =
                    get_area_centered(limit_area_height(data_area, h as u16), slot_area);

                item.render(limit_area_height(data_area, h as u16 - 1), buf);
                quant.render(
                    limit_area_height(data_area, 1).offset(Offset {
                        x: 0,
                        y: h as i32 - 1,
                    }),
                    buf,
                );
            }
        }
    }
}
