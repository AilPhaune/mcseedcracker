use mcseedcracker::search::{BiomeID, MC_1_16_5, WorldExtraData};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Offset, Position, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::{
    make_full_component,
    tui::{
        Component, EventContext, EventResult,
        application::ApplicationTab,
        components::text_input::{TextInputState, TextInputWidget, Validator, i32_validator},
        limit_area_height, limit_area_width,
    },
};

pub struct BiomesTabSharedData {
    pub overworld_biomes: WorldExtraData,
    pub nether_biomes: WorldExtraData,
}

impl Default for BiomesTabSharedData {
    fn default() -> Self {
        Self {
            overworld_biomes: WorldExtraData::OverworldBiomeData(vec![]),
            nether_biomes: WorldExtraData::NetherBiomeData(vec![]),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    #[default]
    Outside,
    Overworld,
    Nether,
}

#[derive(Default)]
pub struct BiomesTabState {
    pub overworld_biomes: Vec<(
        TextInputState<i32>,
        TextInputState<i32>,
        TextInputState<i32>,
        TextInputState<BiomeID>,
    )>,
    pub nether_biomes: Vec<(
        TextInputState<i32>,
        TextInputState<i32>,
        TextInputState<i32>,
        TextInputState<BiomeID>,
    )>,
    pub focus: Focus,
    pub selected_x: usize,
    pub selected_y: usize,
    pub overworld_rect: Rect,
    pub nether_rect: Rect,
}

#[derive(Default)]
pub struct BiomesTabComponent;

make_full_component!(BiomesTab, state: BiomesTabState, component: BiomesTabComponent);

impl BiomesTab {
    pub fn apptab() -> ApplicationTab<Self> {
        ApplicationTab {
            title: "Biomes".to_string(),
            component: BiomesTab::create(),
        }
    }
}

impl Component for BiomesTabComponent {
    type State = BiomesTabState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
    ) {
        let layoutvert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(0)]);

        let layoutcols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(0), Constraint::Fill(0)]);

        let vert = layoutvert.split(area);
        let cols = layoutcols.split(vert[1]);

        let controls = vert[0];

        let l1 =
            Paragraph::new("General controls").style(Style::default().fg(Color::Yellow).bold());
        let l2_1 = Paragraph::new("[CTRL + LEFT] [CTRL + RIGHT] [CTRL + UP] [CTRL + DOWN]")
            .style(Style::default().fg(Color::Magenta).not_bold());
        let l2_2 = Paragraph::new(" Change focused input")
            .style(Style::default().fg(Color::Green).not_bold());
        let l3_1 =
            Paragraph::new("[CTRL + N]").style(Style::default().fg(Color::Magenta).not_bold());
        let l3_2 =
            Paragraph::new(" Add new input").style(Style::default().fg(Color::Green).not_bold());
        let l4_1 =
            Paragraph::new("[CTRL + DEL]").style(Style::default().fg(Color::Magenta).not_bold());
        let l4_2 = Paragraph::new(" Delete selected line")
            .style(Style::default().fg(Color::Green).not_bold());
        let l5_1 =
            Paragraph::new("[LEFT CLICK]").style(Style::default().fg(Color::Magenta).not_bold());
        let l5_2 =
            Paragraph::new(" Focus input").style(Style::default().fg(Color::Green).not_bold());

        let controls_area = limit_area_height(controls, 1);

        l1.render(
            limit_area_width(controls_area, 16).offset(Offset { x: 0, y: 0 }),
            buf,
        );
        l2_1.render(
            limit_area_width(controls_area, 54).offset(Offset { x: 0, y: 1 }),
            buf,
        );
        l2_2.render(
            limit_area_width(controls_area, 21).offset(Offset { x: 54, y: 1 }),
            buf,
        );
        l3_1.render(
            limit_area_width(controls_area, 10).offset(Offset { x: 0, y: 2 }),
            buf,
        );
        l3_2.render(
            limit_area_width(controls_area, 14).offset(Offset { x: 10, y: 2 }),
            buf,
        );
        l4_1.render(
            limit_area_width(controls_area, 12).offset(Offset { x: 0, y: 3 }),
            buf,
        );
        l4_2.render(
            limit_area_width(controls_area, 21).offset(Offset { x: 12, y: 3 }),
            buf,
        );
        l5_1.render(
            limit_area_width(controls_area, 12).offset(Offset { x: 0, y: 4 }),
            buf,
        );
        l5_2.render(
            limit_area_width(controls_area, 12).offset(Offset { x: 12, y: 4 }),
            buf,
        );

        let overworld_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if state.focus == Focus::Overworld {
                Style::default().fg(Color::LightCyan)
            } else {
                Style::default()
            })
            .title("Overworld Biomes")
            .title_alignment(Alignment::Center);

        let nether_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if state.focus == Focus::Nether {
                Style::default().fg(Color::LightCyan)
            } else {
                Style::default()
            })
            .title("Nether Biomes")
            .title_alignment(Alignment::Center);

        let overworld = overworld_block.inner(cols[0]);
        let nether = nether_block.inner(cols[1]);

        state.overworld_rect = overworld;
        state.nether_rect = nether;

        overworld_block.render(cols[0], buf);
        nether_block.render(cols[1], buf);

        let overworld_areas = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(0),
                Constraint::Fill(0),
                Constraint::Fill(0),
                Constraint::Fill(0),
            ],
        )
        .split(limit_area_height(overworld, 3));
        let mut overworld_areas = [
            overworld_areas[0],
            overworld_areas[1],
            overworld_areas[2],
            overworld_areas[3],
        ];

        for (i, ov_data) in state.overworld_biomes.iter_mut().enumerate() {
            if 5 * i + 5 >= overworld.height as usize {
                break;
            }

            ov_data.0.style.show_cursor = false;
            ov_data.1.style.show_cursor = false;
            ov_data.2.style.show_cursor = false;
            ov_data.3.style.show_cursor = false;
            if state.focus == Focus::Overworld && state.selected_y == i {
                match state.selected_x {
                    0 => ov_data.0.style.show_cursor = true,
                    1 => ov_data.1.style.show_cursor = true,
                    2 => ov_data.2.style.show_cursor = true,
                    3 => ov_data.3.style.show_cursor = true,
                    _ => unreachable!(),
                }
            }

            TextInputWidget::default().render(overworld_areas[0], buf, &mut ov_data.0);
            TextInputWidget::default().render(overworld_areas[1], buf, &mut ov_data.1);
            TextInputWidget::default().render(overworld_areas[2], buf, &mut ov_data.2);
            TextInputWidget::default().render(overworld_areas[3], buf, &mut ov_data.3);

            if let Some(data) = shared
                .biome_data
                .overworld_biomes
                .as_overworld()
                .and_then(|v| v.get(i))
            {
                Paragraph::new(format!(
                    "X: {} | Y: {} | Z: {} | Biome: {:?}",
                    data.0, data.1, data.2, data.3
                ))
                .render(
                    limit_area_height(overworld, 1).offset(Offset {
                        x: 0,
                        y: 5 * (i as i32) + 3,
                    }),
                    buf,
                );
            }

            overworld_areas[0].y += 5;
            overworld_areas[1].y += 5;
            overworld_areas[2].y += 5;
            overworld_areas[3].y += 5;
        }

        let nether_areas = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(0),
                Constraint::Fill(0),
                Constraint::Fill(0),
                Constraint::Fill(0),
            ],
        )
        .split(limit_area_height(nether, 3));
        let mut nether_areas = [
            nether_areas[0],
            nether_areas[1],
            nether_areas[2],
            nether_areas[3],
        ];

        for (i, ne_data) in state.nether_biomes.iter_mut().enumerate() {
            if 5 * i + 5 >= overworld.height as usize {
                break;
            }

            ne_data.0.style.show_cursor = false;
            ne_data.1.style.show_cursor = false;
            ne_data.2.style.show_cursor = false;
            ne_data.3.style.show_cursor = false;
            if state.focus == Focus::Overworld && state.selected_y == i {
                match state.selected_x {
                    0 => ne_data.0.style.show_cursor = true,
                    1 => ne_data.1.style.show_cursor = true,
                    2 => ne_data.2.style.show_cursor = true,
                    3 => ne_data.3.style.show_cursor = true,
                    _ => unreachable!(),
                }
            }
            TextInputWidget::default().render(nether_areas[0], buf, &mut ne_data.0);
            TextInputWidget::default().render(nether_areas[1], buf, &mut ne_data.1);
            TextInputWidget::default().render(nether_areas[2], buf, &mut ne_data.2);
            TextInputWidget::default().render(nether_areas[3], buf, &mut ne_data.3);

            if let Some(data) = shared
                .biome_data
                .nether_biomes
                .as_nether()
                .and_then(|v| v.get(i))
            {
                Paragraph::new(format!(
                    "X: {} | Y: {} | Z: {} | Biome: {:?}",
                    data.0, data.1, data.2, data.3
                ))
                .render(
                    limit_area_height(nether, 1).offset(Offset {
                        x: 0,
                        y: 5 * (i as i32) + 3,
                    }),
                    buf,
                );
            }

            nether_areas[0].y += 5;
            nether_areas[1].y += 5;
            nether_areas[2].y += 5;
            nether_areas[3].y += 5;
        }
    }

    fn handle_event(
        &self,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
        event: Event,
        context: EventContext,
    ) -> EventResult {
        match context {
            EventContext::BubblingUp => match event {
                Event::Key(key)
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.kind != KeyEventKind::Release
                        && key.code == KeyCode::Right =>
                {
                    state.selected_x = (state.selected_x + 1) % 4;
                    EventResult::Captured
                }
                Event::Key(key)
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.kind != KeyEventKind::Release
                        && key.code == KeyCode::Left =>
                {
                    state.selected_x = (state.selected_x + 3) % 4;
                    EventResult::Captured
                }
                Event::Key(key)
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.kind != KeyEventKind::Release
                        && key.code == KeyCode::Down =>
                {
                    let len = if state.focus == Focus::Overworld {
                        state.overworld_biomes.len()
                    } else if state.focus == Focus::Nether {
                        state.nether_biomes.len()
                    } else {
                        return EventResult::BubbleUp(event);
                    };
                    state.selected_y = (state.selected_y + 1) % len;
                    EventResult::Captured
                }
                Event::Key(key)
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.kind != KeyEventKind::Release
                        && key.code == KeyCode::Up =>
                {
                    let len = if state.focus == Focus::Overworld {
                        state.overworld_biomes.len()
                    } else if state.focus == Focus::Nether {
                        state.nether_biomes.len()
                    } else {
                        return EventResult::BubbleUp(event);
                    };
                    state.selected_y = (state.selected_y + len).wrapping_sub(1) % len;
                    EventResult::Captured
                }
                Event::Key(key)
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.kind != KeyEventKind::Release
                        && (key.code == KeyCode::Char('n') || key.code == KeyCode::Char('N')) =>
                {
                    if state.focus == Focus::Overworld {
                        if let Some(v) = shared.biome_data.overworld_biomes.as_overworld_mut() {
                            state.overworld_biomes.push((
                                TextInputState::new("X (i32)", i32_validator()),
                                TextInputState::new("Y (i32)", i32_validator()),
                                TextInputState::new("Z (i32)", i32_validator()),
                                TextInputState::new("Biome id", biome_id_validator()),
                            ));
                            v.push((0, 0, 0, BiomeID::none));
                        }
                    } else if state.focus == Focus::Nether {
                        if let Some(v) = shared.biome_data.nether_biomes.as_nether_mut() {
                            state.nether_biomes.push((
                                TextInputState::new("X (i32)", i32_validator()),
                                TextInputState::new("Y (i32)", i32_validator()),
                                TextInputState::new("Z (i32)", i32_validator()),
                                TextInputState::new("Biome id", biome_id_validator()),
                            ));
                            v.push((0, 0, 0, BiomeID::none));
                        }
                    };
                    EventResult::Captured
                }
                Event::Key(key)
                    if key.kind != KeyEventKind::Release && key.code == KeyCode::Tab =>
                {
                    state.selected_x = 0;
                    state.selected_y = 0;
                    state.focus = match state.focus {
                        Focus::Overworld => Focus::Nether,
                        Focus::Nether => Focus::Outside,
                        Focus::Outside => Focus::Overworld,
                    };
                    if state.focus == Focus::Outside {
                        EventResult::BubbleUp(event)
                    } else {
                        EventResult::Captured
                    }
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Moved => {
                    let def_style = Style::default().fg(Color::White);
                    let hover_style = Style::default().fg(Color::LightCyan);

                    for data in state
                        .overworld_biomes
                        .iter_mut()
                        .chain(state.nether_biomes.iter_mut())
                    {
                        data.0.style.border_style = if state.focus != Focus::Outside
                            && data.0.in_rect(mouse.column, mouse.row)
                        {
                            hover_style
                        } else {
                            def_style
                        };
                        data.1.style.border_style = if state.focus != Focus::Outside
                            && data.1.in_rect(mouse.column, mouse.row)
                        {
                            hover_style
                        } else {
                            def_style
                        };
                        data.2.style.border_style = if state.focus != Focus::Outside
                            && data.2.in_rect(mouse.column, mouse.row)
                        {
                            hover_style
                        } else {
                            def_style
                        };
                        data.3.style.border_style = if state.focus != Focus::Outside
                            && data.3.in_rect(mouse.column, mouse.row)
                        {
                            hover_style
                        } else {
                            def_style
                        };
                    }

                    EventResult::BubbleUp(event)
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    if state.focus == Focus::Outside {
                        return EventResult::BubbleUp(event);
                    }

                    if state
                        .overworld_rect
                        .contains(Position::new(mouse.column, mouse.row))
                    {
                        state.focus = Focus::Overworld;
                    } else if state
                        .nether_rect
                        .contains(Position::new(mouse.column, mouse.row))
                    {
                        state.focus = Focus::Nether;
                    }

                    for (i, data) in state.overworld_biomes.iter().enumerate() {
                        if data.0.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 0;
                            state.selected_y = i;
                        }
                        if data.1.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 1;
                            state.selected_y = i;
                        }
                        if data.2.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 2;
                            state.selected_y = i;
                        }
                        if data.3.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 3;
                            state.selected_y = i;
                        }
                    }
                    for (i, data) in state.nether_biomes.iter().enumerate() {
                        if data.0.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 0;
                            state.selected_y = i;
                        }
                        if data.1.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 1;
                            state.selected_y = i;
                        }
                        if data.2.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 2;
                            state.selected_y = i;
                        }
                        if data.3.in_rect(mouse.column, mouse.row) {
                            state.selected_x = 3;
                            state.selected_y = i;
                        }
                    }

                    EventResult::BubbleUp(event)
                }
                _ => EventResult::BubbleUp(event),
            },
            EventContext::BubblingDown => match state.focus {
                Focus::Outside => EventResult::BubbleUp(event),
                Focus::Overworld => {
                    if matches!(event, Event::Key(key) if key.modifiers.contains(KeyModifiers::CONTROL) && key.kind != KeyEventKind::Release && key.code == KeyCode::Delete)
                    {
                        if state.overworld_biomes.get(state.selected_y).is_some() {
                            state.overworld_biomes.remove(state.selected_y);
                        }
                        if let Some(data) = shared.biome_data.overworld_biomes.as_overworld_mut() {
                            if data.get(state.selected_y).is_some() {
                                data.remove(state.selected_y);
                            }
                        }
                        return EventResult::Captured;
                    }

                    if let (Some(istate), Some(data)) = (
                        state.overworld_biomes.get_mut(state.selected_y),
                        shared
                            .biome_data
                            .overworld_biomes
                            .as_overworld_mut()
                            .and_then(|v| v.get_mut(state.selected_y)),
                    ) {
                        let event_result = match state.selected_x {
                            0 => TextInputWidget::handle_event(
                                &mut istate.0,
                                event,
                                context,
                                &mut data.0,
                            ),
                            1 => TextInputWidget::handle_event(
                                &mut istate.1,
                                event,
                                context,
                                &mut data.1,
                            ),
                            2 => TextInputWidget::handle_event(
                                &mut istate.2,
                                event,
                                context,
                                &mut data.2,
                            ),
                            3 => TextInputWidget::handle_event(
                                &mut istate.3,
                                event,
                                context,
                                &mut data.3,
                            ),
                            _ => unreachable!(),
                        };
                        match event_result {
                            EventResult::Captured => EventResult::Captured,
                            EventResult::BubbleUp(event) => {
                                self.handle_event(state, shared, event, EventContext::BubblingUp)
                            }
                        }
                    } else {
                        self.handle_event(state, shared, event, EventContext::BubblingUp)
                    }
                }
                Focus::Nether => {
                    if matches!(event, Event::Key(key) if key.modifiers.contains(KeyModifiers::CONTROL) && key.kind != KeyEventKind::Release && key.code == KeyCode::Delete)
                    {
                        if state.nether_biomes.get(state.selected_y).is_some() {
                            state.nether_biomes.remove(state.selected_y);
                        }
                        if let Some(data) = shared.biome_data.nether_biomes.as_overworld_mut() {
                            if data.get(state.selected_y).is_some() {
                                data.remove(state.selected_y);
                            }
                        }
                        return EventResult::Captured;
                    }

                    if let (Some(istate), Some(data)) = (
                        state.nether_biomes.get_mut(state.selected_y),
                        shared
                            .biome_data
                            .nether_biomes
                            .as_nether_mut()
                            .and_then(|v| v.get_mut(state.selected_y)),
                    ) {
                        let event_result = match state.selected_x {
                            0 => TextInputWidget::handle_event(
                                &mut istate.0,
                                event,
                                context,
                                &mut data.0,
                            ),
                            1 => TextInputWidget::handle_event(
                                &mut istate.1,
                                event,
                                context,
                                &mut data.1,
                            ),
                            2 => TextInputWidget::handle_event(
                                &mut istate.2,
                                event,
                                context,
                                &mut data.2,
                            ),
                            3 => TextInputWidget::handle_event(
                                &mut istate.3,
                                event,
                                context,
                                &mut data.3,
                            ),
                            _ => unreachable!(),
                        };
                        match event_result {
                            EventResult::Captured => EventResult::Captured,
                            EventResult::BubbleUp(event) => {
                                self.handle_event(state, shared, event, EventContext::BubblingUp)
                            }
                        }
                    } else {
                        self.handle_event(state, shared, event, EventContext::BubblingUp)
                    }
                }
            },
        }
    }

    fn on_focus(&self, state: &mut Self::State, _shared: &mut SharedApplicationState) {
        state.focus = Focus::Overworld;
        state.selected_x = 0;
        state.selected_y = 0;
    }

    fn on_unfocus(&self, state: &mut Self::State, _shared: &mut SharedApplicationState) {
        state.focus = Focus::Outside;
        state.selected_x = 0;
        state.selected_y = 0;
    }
}

pub fn list_biomes() -> &'static [BiomeID] {
    &[
        BiomeID::ocean,
        BiomeID::plains,
        BiomeID::desert,
        BiomeID::mountains,
        BiomeID::forest,
        BiomeID::taiga,
        BiomeID::swamp,
        BiomeID::river,
        BiomeID::nether_wastes,
        BiomeID::the_end,
        BiomeID::frozen_ocean,
        BiomeID::frozen_river,
        BiomeID::snowy_tundra,
        BiomeID::snowy_mountains,
        BiomeID::mushroom_fields,
        BiomeID::mushroom_field_shore,
        BiomeID::beach,
        BiomeID::desert_hills,
        BiomeID::wooded_hills,
        BiomeID::taiga_hills,
        BiomeID::mountain_edge,
        BiomeID::jungle,
        BiomeID::jungle_hills,
        BiomeID::jungle_edge,
        BiomeID::deep_ocean,
        BiomeID::stone_shore,
        BiomeID::snowy_beach,
        BiomeID::birch_forest,
        BiomeID::birch_forest_hills,
        BiomeID::dark_forest,
        BiomeID::snowy_taiga,
        BiomeID::snowy_taiga_hills,
        BiomeID::giant_tree_taiga,
        BiomeID::giant_tree_taiga_hills,
        BiomeID::wooded_mountains,
        BiomeID::savanna,
        BiomeID::savanna_plateau,
        BiomeID::badlands,
        BiomeID::wooded_badlands_plateau,
        BiomeID::badlands_plateau,
        BiomeID::small_end_islands,
        BiomeID::end_midlands,
        BiomeID::end_highlands,
        BiomeID::end_barrens,
        BiomeID::warm_ocean,
        BiomeID::lukewarm_ocean,
        BiomeID::cold_ocean,
        BiomeID::deep_warm_ocean,
        BiomeID::deep_lukewarm_ocean,
        BiomeID::deep_cold_ocean,
        BiomeID::deep_frozen_ocean,
        BiomeID::seasonal_forest,
        BiomeID::rainforest,
        BiomeID::shrubland,
        BiomeID::the_void,
        BiomeID::sunflower_plains,
        BiomeID::desert_lakes,
        BiomeID::gravelly_mountains,
        BiomeID::flower_forest,
        BiomeID::taiga_mountains,
        BiomeID::swamp_hills,
        BiomeID::ice_spikes,
        BiomeID::modified_jungle,
        BiomeID::modified_jungle_edge,
        BiomeID::tall_birch_forest,
        BiomeID::tall_birch_hills,
        BiomeID::dark_forest_hills,
        BiomeID::snowy_taiga_mountains,
        BiomeID::giant_spruce_taiga,
        BiomeID::giant_spruce_taiga_hills,
        BiomeID::modified_gravelly_mountains,
        BiomeID::shattered_savanna,
        BiomeID::shattered_savanna_plateau,
        BiomeID::eroded_badlands,
        BiomeID::modified_wooded_badlands_plateau,
        BiomeID::modified_badlands_plateau,
        BiomeID::bamboo_jungle,
        BiomeID::bamboo_jungle_hills,
        BiomeID::soul_sand_valley,
        BiomeID::crimson_forest,
        BiomeID::warped_forest,
        BiomeID::basalt_deltas,
        BiomeID::dripstone_caves,
        BiomeID::lush_caves,
        BiomeID::meadow,
        BiomeID::grove,
        BiomeID::snowy_slopes,
        BiomeID::jagged_peaks,
        BiomeID::frozen_peaks,
        BiomeID::stony_peaks,
        BiomeID::deep_dark,
        BiomeID::mangrove_swamp,
        BiomeID::cherry_grove,
        BiomeID::pale_garden,
    ]
}

pub fn biome_id_validator() -> Validator<BiomeID> {
    Some(Box::new(|text, _, style, biome| {
        if text.len() > 32 {
            *biome = BiomeID::none;
        } else {
            let text = text.iter().collect::<String>();
            let biomes = list_biomes()
                .into_iter()
                .filter_map(|b| {
                    let str = b.to_mc_biome_str(MC_1_16_5);
                    if str.starts_with(&text) {
                        Some(*b)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if let Some(b) = biomes.iter().find(|b| b.to_mc_biome_str(MC_1_16_5) == text) {
                *biome = *b;
                style.text_style.fg = Some(Color::Green);
                style.cursor_style.bg = Some(Color::Green);
            } else if biomes.len() == 1 {
                *biome = biomes[0];
                style.text_style.fg = Some(Color::Green);
                style.cursor_style.bg = Some(Color::Green);
            } else {
                if biomes.is_empty() {
                    style.text_style.fg = Some(Color::Red);
                    style.cursor_style.bg = Some(Color::Red);
                } else {
                    style.text_style.fg = Some(Color::White);
                    style.cursor_style.bg = Some(Color::White);
                }
                *biome = BiomeID::none;
            }
        }
    }))
}
