use crate::{
    make_full_component,
    tui::{
        Component, EventContext, EventResult,
        application::ApplicationTab,
        components::{
            chest::{ChestState, ChestWidget},
            text_input::{TextInputState, TextInputWidget},
        },
        limit_area_height, limit_area_width,
    },
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Offset, Rect},
    style::{Color, Style, Stylize},
    widgets::{Paragraph, StatefulWidget, Widget},
};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Focus {
    #[default]
    Outside,
    Chest,
    CoordX,
    CoordZ,
}

pub struct BuriedTreasureTabState {
    pub contents: ChestState,
    pub focus: Focus,
    pub xstate: TextInputState<i32>,
    pub zstate: TextInputState<i32>,
}

impl Default for BuriedTreasureTabState {
    fn default() -> Self {
        let mut value = Self {
            contents: ChestState::default(),
            focus: Focus::default(),
            xstate: TextInputState::default(),
            zstate: TextInputState::default(),
        };
        value.xstate.style.title = "Treasure X (i32)".to_string();
        value.zstate.style.title = "Treasure Z (i32)".to_string();

        value
    }
}

#[derive(Default)]
pub struct BuriedTreasureTabSharedData {
    pub contents: SingleChest,
    pub pos_x: i32,
    pub pos_z: i32,
    pub luck: f32,
    pub usable: bool,
}

#[derive(Default)]
pub struct BuriedTreasureTabComponent;

make_full_component!(BuriedTreasureTab, state: BuriedTreasureTabState, component: BuriedTreasureTabComponent);

impl BuriedTreasureTab {
    pub fn apptab() -> ApplicationTab<Self> {
        ApplicationTab {
            title: "Buried Treasure".to_string(),
            component: BuriedTreasureTab::create(),
        }
    }
}

use mcseedcracker::{
    features::buried_treasure::items::{
        COOKED_COD, COOKED_SALMON, DIAMOND, EMERALD, GOLD_INGOT, HEART_OF_THE_SEA, IRON_INGOT,
        IRON_SWORD, LEATHER_CHESTPLATE, PRISMARINE_CRYSTALS, TNT,
    },
    loot_table::{ItemStack, SingleChest},
};

#[inline(always)]
fn item_to_string(item: usize) -> &'static str {
    match item {
        COOKED_COD => "Cooked Cod",
        COOKED_SALMON => "Cooked Salmon",
        DIAMOND => "Diamond",
        EMERALD => "Emerald",
        GOLD_INGOT => "Gold Ingot",
        HEART_OF_THE_SEA => "Heart of the Sea",
        IRON_INGOT => "Iron Ingot",
        IRON_SWORD => "Iron Sword",
        LEATHER_CHESTPLATE => "Leather Chestplate",
        PRISMARINE_CRYSTALS => "Prismarine Crystals",
        TNT => "TNT",
        _ => "unknown",
    }
}

impl BuriedTreasureTabComponent {}

impl Component for BuriedTreasureTabComponent {
    type State = BuriedTreasureTabState;

    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
    ) {
        let values = [
            ("Cooked ", "C", "od"),
            ("Cooked ", "S", "almon"),
            ("", "D", "iamond"),
            ("", "E", "merald"),
            ("", "G", "old Ingot"),
            ("", "H", "eart of the Sea"),
            ("", "I", "ron Ingot"),
            ("Iron S", "w", "ord"),
            ("", "L", "eather Chestplate"),
            ("", "P", "rismarine Crystals"),
            ("", "T", "NT"),
        ];

        let layout1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(0)]);

        let layout2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(55), Constraint::Min(0)]);

        let (layout, vert) = match (
            ChestWidget::calculate_size(layout1.split(area)[1], &state.contents),
            ChestWidget::calculate_size(layout2.split(area)[1], &state.contents),
        ) {
            (Some((w1, _)), Some((w2, _))) => {
                if w1 >= w2 {
                    (layout1, true)
                } else {
                    (layout2, false)
                }
            }
            (Some(_), None) => (layout1, true),
            _ => (layout2, false),
        };

        let chunks = layout.split(area);
        let controls_area = chunks[0];
        let contents_area = chunks[1];

        // show controls
        let present_text =
            Paragraph::new("Buried treasure available: ").style(Style::new().fg(Color::White));
        let present_value = if shared.buried_treasure_data.usable {
            Paragraph::new("yes").style(if state.focus == Focus::Chest {
                Style::new().fg(Color::White).bold().bg(Color::Green)
            } else {
                Style::new().fg(Color::Green)
            })
        } else {
            Paragraph::new("no").style(if state.focus == Focus::Chest {
                Style::new().fg(Color::White).bold().bg(Color::Red)
            } else {
                Style::new().fg(Color::Red)
            })
        };

        let l1 = Paragraph::new("Gneral controls").style(Style::default().fg(Color::Yellow).bold());
        let l2_1 = Paragraph::new("[LEFT] [RIGHT] [UP] [DOWN]")
            .style(Style::default().fg(Color::Magenta).not_bold());
        let l2_2 =
            Paragraph::new(" Move selection").style(Style::default().fg(Color::Green).not_bold());
        let l3_1 = Paragraph::new("[SPACE]").style(Style::default().fg(Color::Magenta).not_bold());
        let l3_2 = Paragraph::new(" Toggle buried treasure availability")
            .style(Style::default().fg(Color::Green).not_bold());
        let l4_1 =
            Paragraph::new("[SHIFT + DEL]").style(Style::default().fg(Color::Magenta).not_bold());
        let l4_2 =
            Paragraph::new(" Delete all").style(Style::default().fg(Color::Green).not_bold());

        let l5 = Paragraph::new("Edit selection").style(Style::default().fg(Color::Yellow).bold());
        let l6_1 = Paragraph::new("[0] [1] [2] [3] [4] [5] [6] [7] [8] [9]")
            .style(Style::default().fg(Color::Magenta).not_bold());
        let l6_2 =
            Paragraph::new(" Set quantity").style(Style::default().fg(Color::Green).not_bold());
        let l7_1 = Paragraph::new("[DEL] [BACKSPACE]")
            .style(Style::default().fg(Color::Magenta).not_bold());
        let l7_2 =
            Paragraph::new(" Remove item").style(Style::default().fg(Color::Green).not_bold());

        present_text.render(controls_area, buf);
        present_value.render(
            limit_area_height(
                limit_area_width(
                    controls_area,
                    if shared.buried_treasure_data.usable {
                        3
                    } else {
                        2
                    },
                ),
                1,
            )
            .offset(Offset { x: 27, y: 0 })
            .intersection(controls_area),
            buf,
        );

        let controls_area = limit_area_height(area, 1);

        l1.render(controls_area.offset(Offset { x: 0, y: 3 }), buf);
        l2_1.render(controls_area.offset(Offset { x: 0, y: 4 }), buf);
        l2_2.render(controls_area.offset(Offset { x: 26, y: 4 }), buf);
        l3_1.render(controls_area.offset(Offset { x: 0, y: 5 }), buf);
        l3_2.render(controls_area.offset(Offset { x: 7, y: 5 }), buf);
        l4_1.render(controls_area.offset(Offset { x: 0, y: 6 }), buf);
        l4_2.render(controls_area.offset(Offset { x: 13, y: 6 }), buf);
        l5.render(controls_area.offset(Offset { x: 0, y: 8 }), buf);
        l6_1.render(controls_area.offset(Offset { x: 0, y: 9 }), buf);
        l6_2.render(controls_area.offset(Offset { x: 39, y: 9 }), buf);
        l7_1.render(controls_area.offset(Offset { x: 0, y: 10 }), buf);
        l7_2.render(controls_area.offset(Offset { x: 17, y: 10 }), buf);

        let mut y = if vert { 0 } else { 12 };
        Paragraph::new("Set item")
            .style(Style::default().fg(Color::Yellow).bold())
            .render(
                controls_area.offset(Offset {
                    x: if vert { 55 } else { 0 },
                    y,
                }),
                buf,
            );

        for &(prefix, key, suffix) in values.iter() {
            let mut x: i32 = if vert { 55 } else { 0 };
            y += 1;

            let a = Paragraph::new(prefix).style(
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Black)
                    .not_bold(),
            );
            let b = Paragraph::new(format!("[{key}]")).style(
                Style::default()
                    .fg(Color::Magenta)
                    .bg(Color::Black)
                    .not_bold(),
            );
            let c = Paragraph::new(suffix).style(
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Black)
                    .not_bold(),
            );

            a.render(
                limit_area_width(controls_area.offset(Offset { x, y }), prefix.len() as u16),
                buf,
            );
            x += prefix.len() as i32;
            b.render(
                limit_area_width(controls_area.offset(Offset { x, y }), key.len() as u16 + 2),
                buf,
            );
            x += key.len() as i32 + 2;
            c.render(
                limit_area_width(controls_area.offset(Offset { x, y }), suffix.len() as u16),
                buf,
            );
        }

        if state.xstate.validator.is_none() {
            state.xstate.validator = Some(Box::new(|value, _cursor, style, i| {
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
            }));
        }
        if state.zstate.validator.is_none() {
            state.zstate.validator = Some(Box::new(|value, _cursor, style, i| {
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
            }));
        }

        state.xstate.style.border_style = Style::default().fg(Color::White);
        state.zstate.style.border_style = Style::default().fg(Color::White);
        state.xstate.style.show_cursor = false;
        state.zstate.style.show_cursor = false;

        if state.focus == Focus::CoordX {
            state.xstate.style.border_style = Style::default().fg(Color::LightCyan);
            state.xstate.style.show_cursor = true;
        } else if state.focus == Focus::CoordZ {
            state.zstate.style.border_style = Style::default().fg(Color::LightCyan);
            state.zstate.style.show_cursor = true;
        }

        if vert {
            TextInputWidget::default().render(
                limit_area_width(limit_area_height(area, 3), (area.width - 82).min(30))
                    .offset(Offset { x: 80, y: 0 }),
                buf,
                &mut state.xstate,
            );
            TextInputWidget::default().render(
                limit_area_width(limit_area_height(area, 3), (area.width - 82).min(30))
                    .offset(Offset { x: 80, y: 3 }),
                buf,
                &mut state.zstate,
            );
        }

        if shared.buried_treasure_data.usable {
            let chest = ChestWidget;

            state.contents.show_selected = state.focus == Focus::Chest;

            chest.render(contents_area, buf, &mut state.contents);
        }
    }

    fn handle_event(
        self,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
        event: Event,
        context: EventContext,
    ) -> EventResult {
        #[inline(always)]
        const fn char_to_item(c: char) -> Option<(usize, Color)> {
            match c {
                'c' => Some((COOKED_COD, Color::Indexed(186))),
                's' => Some((COOKED_SALMON, Color::Indexed(202))),
                'd' => Some((DIAMOND, Color::LightCyan)),
                'e' => Some((EMERALD, Color::Indexed(28))),
                'g' => Some((GOLD_INGOT, Color::LightYellow)),
                'h' => Some((HEART_OF_THE_SEA, Color::Blue)),
                'i' => Some((IRON_INGOT, Color::DarkGray)),
                'w' => Some((IRON_SWORD, Color::DarkGray)),
                'l' => Some((LEATHER_CHESTPLATE, Color::Indexed(172))),
                'p' => Some((PRISMARINE_CRYSTALS, Color::Indexed(79))),
                't' => Some((TNT, Color::LightRed)),
                _ => None,
            }
        }

        match context {
            EventContext::BubblingDown => {
                if state.focus == Focus::CoordX {
                    return match TextInputWidget::handle_event(
                        &mut state.xstate,
                        event,
                        EventContext::BubblingDown,
                        &mut shared.buried_treasure_data.pos_x,
                    ) {
                        EventResult::BubbleUp(event) => {
                            self.handle_event(state, shared, event, EventContext::BubblingUp)
                        }
                        result => result,
                    };
                } else if state.focus == Focus::CoordZ {
                    return match TextInputWidget::handle_event(
                        &mut state.zstate,
                        event,
                        EventContext::BubblingDown,
                        &mut shared.buried_treasure_data.pos_z,
                    ) {
                        EventResult::BubbleUp(event) => {
                            self.handle_event(state, shared, event, EventContext::BubblingUp)
                        }
                        result => result,
                    };
                } else if state.focus == Focus::Outside {
                    return match &event {
                        Event::Key(key)
                            if key.code == KeyCode::Tab && key.kind != KeyEventKind::Release =>
                        {
                            state.focus = Focus::CoordX;
                            EventResult::Captured
                        }
                        _ => EventResult::BubbleUp(event),
                    };
                }
                match &event {
                    Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                        KeyCode::Char(' ') => {
                            shared.buried_treasure_data.usable =
                                !shared.buried_treasure_data.usable;
                            EventResult::Captured
                        }
                        KeyCode::Char(c) if char_to_item(c.to_ascii_lowercase()).is_some() => {
                            let (item, fg) = char_to_item(c.to_ascii_lowercase()).unwrap();

                            state.contents.contents[state.contents.selected.1]
                                [state.contents.selected.0] = (
                                item_to_string(item).to_string(),
                                state.contents.contents[state.contents.selected.1]
                                    [state.contents.selected.0]
                                    .1
                                    .max(1),
                                Style::default().fg(fg).not_bold(),
                            );

                            shared.buried_treasure_data.contents.rows[state.contents.selected.1]
                                .items[state.contents.selected.0] = Some(ItemStack::new(
                                item,
                                state.contents.contents[state.contents.selected.1]
                                    [state.contents.selected.0]
                                    .1,
                                if item == IRON_SWORD || item == LEATHER_CHESTPLATE {
                                    1
                                } else {
                                    64
                                },
                            ));

                            EventResult::Captured
                        }
                        KeyCode::Right => {
                            state.contents.selected.0 =
                                (state.contents.selected.0 + 1) % state.contents.width;
                            EventResult::Captured
                        }
                        KeyCode::Left => {
                            state.contents.selected.0 =
                                (state.contents.selected.0 + state.contents.width - 1)
                                    % state.contents.width;
                            EventResult::Captured
                        }
                        KeyCode::Down => {
                            state.contents.selected.1 =
                                (state.contents.selected.1 + 1) % state.contents.height;
                            EventResult::Captured
                        }
                        KeyCode::Up => {
                            state.contents.selected.1 =
                                (state.contents.selected.1 + state.contents.height - 1)
                                    % state.contents.height;
                            EventResult::Captured
                        }
                        KeyCode::Char(c)
                            if shared.buried_treasure_data.usable && c.is_ascii_digit() =>
                        {
                            let count = &mut state.contents.contents[state.contents.selected.1]
                                [state.contents.selected.0]
                                .1;

                            let res = (*count * 10 + c.to_digit(10).unwrap() as i32) % 100;
                            *count = res;

                            if let Some(bt) = &mut shared.buried_treasure_data.contents.rows
                                [state.contents.selected.1]
                                .items[state.contents.selected.0]
                            {
                                bt.count = res;
                            }

                            EventResult::Captured
                        }
                        KeyCode::Tab => match state.focus {
                            Focus::CoordX => {
                                state.focus = Focus::CoordZ;
                                EventResult::Captured
                            }
                            Focus::CoordZ => {
                                state.focus = Focus::Chest;
                                EventResult::Captured
                            }
                            Focus::Chest => {
                                state.focus = Focus::Outside;
                                EventResult::BubbleUp(event)
                            }
                            Focus::Outside => {
                                state.focus = Focus::CoordX;
                                EventResult::Captured
                            }
                        },
                        KeyCode::Backspace | KeyCode::Delete
                            if shared.buried_treasure_data.usable =>
                        {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                state.contents.contents.iter_mut().for_each(|row| {
                                    row.iter_mut().for_each(|item| {
                                        *item = ("".to_string(), 0, Style::default());
                                    });
                                });
                                shared.buried_treasure_data.contents = SingleChest::new();
                            } else {
                                state.contents.contents[state.contents.selected.1]
                                    [state.contents.selected.0] =
                                    ("".to_string(), 0, Style::default());
                                shared.buried_treasure_data.contents.rows
                                    [state.contents.selected.1]
                                    .items[state.contents.selected.0] = None;
                            }

                            EventResult::Captured
                        }
                        _ => EventResult::BubbleUp(event),
                    },
                    _ => EventResult::BubbleUp(event),
                }
            }

            EventContext::BubblingUp => match &event {
                Event::Key(key)
                    if key.code == KeyCode::Tab && key.kind != KeyEventKind::Release =>
                {
                    match state.focus {
                        Focus::CoordX => {
                            state.focus = Focus::CoordZ;
                            EventResult::Captured
                        }
                        Focus::CoordZ => {
                            state.focus = Focus::Chest;
                            EventResult::Captured
                        }
                        Focus::Chest => {
                            state.focus = Focus::Outside;
                            EventResult::BubbleUp(event)
                        }
                        Focus::Outside => {
                            state.focus = Focus::CoordX;
                            EventResult::Captured
                        }
                    }
                }
                _ => EventResult::BubbleUp(event),
            },
        }
    }
}
