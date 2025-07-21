use mcseedcracker::{
    features::{
        buried_treasure::build_fast_inventory_compare_context, end_pillars::PillarMatchResult,
    },
    math::Math,
    search::{StructureData, StructureSeedSearchData, WorldSeedSearchData},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Offset, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
};

use crate::{
    make_full_component,
    tui::{
        Component, EventContext, EventResult,
        application::{
            ApplicationTab, StructureSeedSimData, StructureSeedSimResultType, WorldSeedSimData,
            WorldSeedSimResultType,
        },
        get_area_centered, limit_area_height, limit_area_width,
    },
};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum Focus {
    #[default]
    Outside,
    StructureSeedButton,
    WorldSeedButton,
    Simulation,
}

#[derive(Default, Debug, Clone)]
pub struct OutputTabState {
    pub focus: Focus,
    pub valid_pillar_count: usize,
}

#[derive(Default)]
pub struct OutputTabComponent;

make_full_component!(OutputTab, state: OutputTabState, component: OutputTabComponent);

impl OutputTab {
    pub fn apptab() -> ApplicationTab<Self> {
        ApplicationTab {
            title: "Output".to_string(),
            component: OutputTab::create(),
        }
    }
}

#[inline(always)]
fn format_progress(mut value: f64) -> String {
    if value > 100.0 {
        value = 100.0;
    }
    if (value - 100.0).abs() < f64::EPSILON {
        "100.0".to_string()
    } else {
        format!("{:05.2}", value)
    }
}

impl OutputTabComponent {
    fn render_pillars(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut <Self as Component>::State,
        shared: &mut SharedApplicationState,
    ) {
        let seed_results = if matches!(shared.last_pillar_sim.as_ref(), Some((p, _)) if p == &shared.pillar_data)
        {
            &shared.last_pillar_sim.as_ref().unwrap().1
        } else {
            shared.last_pillar_sim = Some((
                shared.pillar_data,
                shared
                    .pillar_data
                    .seed_results()
                    .into_iter()
                    .filter(|(_, result)| match result {
                        PillarMatchResult::ImpossibleMatch => false,
                        PillarMatchResult::PossibleMatch(v) => *v != 0.0,
                        _ => true,
                    })
                    .collect::<Vec<_>>(),
            ));
            &shared.last_pillar_sim.as_ref().unwrap().1
        };

        let valid_count = seed_results
            .iter()
            .filter(|(_, result)| !result.is_impossible_match())
            .count();
        state.valid_pillar_count = valid_count;

        let title =
            Paragraph::new("Based on your pillar info:").style(Style::default().fg(Color::White));
        let subtitle = Paragraph::new("Valid pillar seeds count: ");
        let subtitle2 =
            Paragraph::new(format!("{}", valid_count)).style(Style::new().fg(match valid_count {
                1 => Color::Green,
                2..=5 => Color::Yellow,
                _ => Color::Red,
            }));

        title.render(limit_area_height(area, 1), buf);
        subtitle.render(
            limit_area_width(limit_area_height(area, 1), 26).offset(Offset { x: 0, y: 1 }),
            buf,
        );
        subtitle2.render(
            limit_area_width(limit_area_height(area, 1), 5).offset(Offset { x: 26, y: 1 }),
            buf,
        );

        if valid_count < area.height as usize - 2 {
            let mut sorted = seed_results
                .iter()
                .filter(|(_, result)| !result.is_impossible_match())
                .copied()
                .collect::<Vec<_>>();
            sorted.sort_by(|a, b| b.1.compare(&a.1));

            for (i, (seed, result)) in sorted.into_iter().enumerate() {
                let seed_str = format!("{}", seed);
                match result {
                    PillarMatchResult::ExactMatch => {
                        Paragraph::new(seed_str)
                            .style(Style::default().fg(Color::Green))
                            .render(
                                limit_area_width(limit_area_height(area, 1), 25).offset(Offset {
                                    x: 0,
                                    y: i as i32 + 2,
                                }),
                                buf,
                            );
                    }
                    PillarMatchResult::PossibleMatch(v) => {
                        let prob_str = format!("{:.2}%", v * 100.0);
                        Paragraph::new(seed_str)
                            .style(Style::default().fg(if v > 0.75 {
                                Color::Green
                            } else if v > 0.25 {
                                Color::Yellow
                            } else {
                                Color::Red
                            }))
                            .render(
                                limit_area_width(limit_area_height(area, 1), 25).offset(Offset {
                                    x: 0,
                                    y: i as i32 + 2,
                                }),
                                buf,
                            );
                        Paragraph::new(prob_str)
                            .style(Style::default().fg(Color::Yellow))
                            .render(
                                limit_area_width(limit_area_height(area, 1), 25).offset(Offset {
                                    x: 25,
                                    y: i as i32 + 2,
                                }),
                                buf,
                            );
                    }
                    PillarMatchResult::ImpossibleMatch => unreachable!(),
                }
            }
        }
    }

    fn render_structures(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut <Self as Component>::State,
        shared: &mut SharedApplicationState,
    ) {
        let outdated_label =
            Paragraph::new("Outdated data: ").style(Style::default().fg(Color::White));
        outdated_label.render(limit_area_height(area, 1), buf);

        let y = if shared.last_structure_seed_sim.outdated_data {
            let yes_text = Paragraph::new("Yes").style(Style::default().fg(Color::Red));
            yes_text.render(
                limit_area_width(limit_area_height(area, 1), 3).offset(Offset { x: 15, y: 0 }),
                buf,
            );

            if state.valid_pillar_count <= 5 {
                if let Some(searcher) = &shared.current_structure_seed_searcher {
                    let cancel_btn = Paragraph::new("[Cancel search]").style(
                        if state.focus == Focus::StructureSeedButton {
                            Style::new().fg(Color::White).bold().bg(Color::LightMagenta)
                        } else {
                            Style::default().fg(Color::Yellow).not_bold()
                        },
                    );
                    cancel_btn.render(
                        get_area_centered(
                            limit_area_width(limit_area_height(area, 1), 15),
                            limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                        ),
                        buf,
                    );

                    let pgint = searcher.get_progress();
                    let pg = pgint as f64 / (1i64 << 32) as f64;
                    let pgtext = format!(
                        "({} job{}) [{pgint:10}/4294967296] {}%",
                        shared.structure_seed_search_jobs.len() + 1,
                        if shared.structure_seed_search_jobs.is_empty() {
                            ""
                        } else {
                            "s"
                        },
                        format_progress(pg * 100.0)
                    );
                    let pgtext_len = pgtext.len();
                    let progress = Paragraph::new(pgtext).style(Style::new().fg(Color::Yellow));
                    progress.render(
                        get_area_centered(
                            limit_area_width(limit_area_height(area, 1), pgtext_len as u16),
                            limit_area_height(area, 1).offset(Offset { x: 0, y: 3 }),
                        ),
                        buf,
                    );

                    let gauge = Gauge::default()
                        .gauge_style(Style::default().fg(Color::LightBlue).bg(Color::Gray))
                        .ratio(pg)
                        .use_unicode(true)
                        .label("");
                    gauge.render(
                        limit_area_height(area, 1).offset(Offset { x: 0, y: 4 }),
                        buf,
                    );

                    6
                } else {
                    let find_btn = Paragraph::new("[Find structure seeds]").style(
                        if state.focus == Focus::StructureSeedButton {
                            Style::new().fg(Color::White).bold().bg(Color::LightMagenta)
                        } else {
                            Style::default().fg(Color::Yellow).not_bold()
                        },
                    );
                    find_btn.render(
                        get_area_centered(
                            limit_area_width(limit_area_height(area, 1), 22),
                            limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                        ),
                        buf,
                    );
                    3
                }
            } else {
                let info_text = Paragraph::new("Too many pillar seeds to search")
                    .style(Style::default().fg(Color::Red).bold().bg(Color::Reset));
                info_text.render(
                    get_area_centered(
                        limit_area_width(limit_area_height(area, 1), 31),
                        limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                    ),
                    buf,
                );
                3
            }
        } else {
            let no_text = Paragraph::new("No").style(Style::default().fg(Color::Green));
            no_text.render(
                limit_area_width(limit_area_height(area, 1), 2).offset(Offset { x: 15, y: 0 }),
                buf,
            );
            1
        };

        if let Some(sim) = &shared.last_structure_seed_sim.data {
            let num_str = format!("{}", sim.count_seeds);
            let num_str_len = num_str.len() as i32;

            let sim_text1 = Paragraph::new("Found ").style(Style::default().fg(Color::White));
            let sim_text2 = Paragraph::new(num_str).style(Style::default().fg(Color::Yellow));
            let sim_text3 =
                Paragraph::new(" structure seeds:").style(Style::default().fg(Color::White));

            sim_text1.render(
                limit_area_width(limit_area_height(area, 1), 6).offset(Offset { x: 0, y }),
                buf,
            );
            sim_text2.render(
                limit_area_width(limit_area_height(area, 1), num_str_len as u16)
                    .offset(Offset { x: 6, y }),
                buf,
            );
            sim_text3.render(
                limit_area_width(limit_area_height(area, 1), 17).offset(Offset {
                    x: 6 + num_str_len,
                    y,
                }),
                buf,
            );

            let orig_y = y + 1;
            let mut y = orig_y;
            let mut x = 0;
            let mut cur_pillar_i: isize = -1;
            let mut cur_seed_i = 0;
            while x + 20 < area.width as i32
                && y < area.height as i32
                && cur_pillar_i < sim.per_pillar.len() as isize
            {
                if cur_pillar_i == -1
                    || cur_seed_i >= sim.per_pillar[cur_pillar_i as usize].structure_seeds.len()
                {
                    cur_pillar_i += 1;
                    cur_seed_i = 0;
                    if y + 4 >= area.height as i32 || cur_pillar_i >= sim.per_pillar.len() as isize
                    {
                        break;
                    }
                    y += 1;

                    Paragraph::new(format!(
                        "Pillar seed {}",
                        sim.per_pillar[cur_pillar_i as usize].pillar_seed
                    ))
                    .style(Style::default().fg(Color::LightYellow).bold())
                    .render(
                        limit_area_width(limit_area_height(area, 1), 17).offset(Offset { x, y }),
                        buf,
                    );

                    y += 1;

                    match sim.per_pillar[cur_pillar_i as usize].result {
                        StructureSeedSimResultType::Success => {
                            Paragraph::new("(success)")
                                .style(Style::default().fg(Color::Green).bold())
                                .render(
                                    limit_area_width(limit_area_height(area, 1), 9)
                                        .offset(Offset { x, y }),
                                    buf,
                                );
                        }
                        StructureSeedSimResultType::TooManySeeds => {
                            Paragraph::new("(too many seeds, search stopped)")
                                .style(Style::default().fg(Color::Yellow).bold())
                                .render(
                                    limit_area_width(limit_area_height(area, 1), 32)
                                        .offset(Offset { x, y }),
                                    buf,
                                );
                        }
                        StructureSeedSimResultType::Cancelled => {
                            Paragraph::new("(search cancelled)")
                                .style(Style::default().fg(Color::Red).bold())
                                .render(
                                    limit_area_width(limit_area_height(area, 1), 18)
                                        .offset(Offset { x, y }),
                                    buf,
                                );
                        }
                    }

                    y += 1;

                    continue;
                }

                if y == orig_y {
                    y += 1;
                }

                Paragraph::new(format!(
                    "{}",
                    sim.per_pillar[cur_pillar_i as usize].structure_seeds[cur_seed_i]
                ))
                .style(Style::new().fg(Color::Green).not_bold())
                .render(
                    limit_area_width(limit_area_height(area, 1), 20).offset(Offset { x, y }),
                    buf,
                );
                y += 1;

                cur_seed_i += 1;

                if y >= area.height as i32 {
                    x += 35;
                    y = orig_y;
                }
            }
        }
    }

    fn render_world(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut <Self as Component>::State,
        shared: &mut SharedApplicationState,
    ) {
        if let Some(ssim) = &shared.last_structure_seed_sim.data {
            if ssim.count_seeds >= 1
                && ssim.count_seeds as usize <= shared.max_structure_seeds_to_simulate
            {
                let y = if let Some(searcher) = &shared.current_world_seed_searcher {
                    let cancel_btn = Paragraph::new("[Cancel search]").style(
                        if state.focus == Focus::WorldSeedButton {
                            Style::new().fg(Color::White).bold().bg(Color::LightMagenta)
                        } else {
                            Style::default().fg(Color::LightYellow).not_bold()
                        },
                    );
                    cancel_btn.render(
                        get_area_centered(
                            limit_area_width(limit_area_height(area, 1), 18),
                            limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                        ),
                        buf,
                    );

                    let pgint = searcher.get_progress();
                    let pg = pgint as f64 / (1i64 << 16) as f64;
                    let pgtext = format!(
                        "({} job{}) [{pgint:5}/65536] {}%",
                        shared.structure_seed_search_jobs.len() + 1,
                        if shared.structure_seed_search_jobs.is_empty() {
                            ""
                        } else {
                            "s"
                        },
                        format_progress(pg * 100.0)
                    );
                    let pgtext_len = pgtext.len();
                    let progress = Paragraph::new(pgtext).style(Style::new().fg(Color::Yellow));
                    progress.render(
                        get_area_centered(
                            limit_area_width(limit_area_height(area, 1), pgtext_len as u16),
                            limit_area_height(area, 1).offset(Offset { x: 0, y: 3 }),
                        ),
                        buf,
                    );

                    let gauge = Gauge::default()
                        .gauge_style(Style::default().fg(Color::LightBlue).bg(Color::Gray))
                        .ratio(pg)
                        .use_unicode(true)
                        .label("");
                    gauge.render(
                        limit_area_height(area, 1).offset(Offset { x: 0, y: 4 }),
                        buf,
                    );

                    6
                } else {
                    let find_btn = Paragraph::new("[Find world seeds]").style(
                        if state.focus == Focus::WorldSeedButton {
                            Style::new().fg(Color::White).bold().bg(Color::LightMagenta)
                        } else {
                            Style::default().fg(Color::LightYellow).not_bold()
                        },
                    );
                    find_btn.render(
                        get_area_centered(
                            limit_area_width(limit_area_height(area, 1), 18),
                            limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                        ),
                        buf,
                    );

                    3
                };

                let num_str = if shared.world_seed_sim.count_seeds
                    >= shared.max_world_seeds_per_structure_seed as i64
                {
                    format!(">{}", shared.max_world_seeds_per_structure_seed)
                } else {
                    format!("{}", shared.world_seed_sim.count_seeds)
                };
                let num_str_len = num_str.len();

                let sim_text1 = Paragraph::new("Found ").style(Style::default().fg(Color::White));
                let sim_text2 = Paragraph::new(num_str).style(Style::default().fg(Color::Yellow));
                let sim_text3 =
                    Paragraph::new(" world seeds:").style(Style::default().fg(Color::White));

                sim_text1.render(
                    limit_area_width(limit_area_height(area, 1), 6).offset(Offset { x: 0, y }),
                    buf,
                );
                sim_text2.render(
                    limit_area_width(limit_area_height(area, 1), num_str_len as u16)
                        .offset(Offset { x: 6, y }),
                    buf,
                );
                sim_text3.render(
                    limit_area_width(limit_area_height(area, 1), 17).offset(Offset {
                        x: 6 + num_str_len as i32,
                        y,
                    }),
                    buf,
                );

                let orig_y = y + 1;
                let mut y = orig_y;
                let mut x = 0;
                let mut cur_struct_i: isize = -1;
                let mut cur_seed_i = 0;
                while x + 30 < area.width as i32
                    && y < area.height as i32
                    && cur_struct_i < shared.world_seed_sim.per_structure.len() as isize
                {
                    if cur_struct_i == -1
                        || cur_seed_i
                            >= shared.world_seed_sim.per_structure[cur_struct_i as usize]
                                .world_seeds
                                .len()
                                .min(5)
                    {
                        cur_struct_i += 1;
                        cur_seed_i = 0;
                        if y + 4 >= area.height as i32
                            || cur_struct_i >= shared.world_seed_sim.per_structure.len() as isize
                        {
                            break;
                        }
                        y += 1;

                        Paragraph::new(format!(
                            "Structure seed {}",
                            shared.world_seed_sim.per_structure[cur_struct_i as usize]
                                .structure_seed
                        ))
                        .style(Style::default().fg(Color::LightYellow).bold())
                        .render(
                            limit_area_width(limit_area_height(area, 1), 25)
                                .offset(Offset { x, y }),
                            buf,
                        );

                        y += 1;

                        match shared.world_seed_sim.per_structure[cur_struct_i as usize].result {
                            WorldSeedSimResultType::Success => {
                                Paragraph::new("(success)")
                                    .style(Style::default().fg(Color::Green).bold())
                                    .render(
                                        limit_area_width(limit_area_height(area, 1), 9)
                                            .offset(Offset { x, y }),
                                        buf,
                                    );
                            }
                            WorldSeedSimResultType::TooManySeeds => {
                                Paragraph::new("(+ more)")
                                    .style(Style::default().fg(Color::Yellow).bold())
                                    .render(
                                        limit_area_width(limit_area_height(area, 1), 8)
                                            .offset(Offset { x, y }),
                                        buf,
                                    );
                            }
                        }

                        y += 1;

                        continue;
                    }

                    if y == orig_y {
                        y += 1;
                    }

                    Paragraph::new(format!(
                        "{}",
                        shared.world_seed_sim.per_structure[cur_struct_i as usize].world_seeds
                            [cur_seed_i]
                    ))
                    .style(Style::new().fg(Color::Green).not_bold())
                    .render(
                        limit_area_width(limit_area_height(area, 1), 20).offset(Offset { x, y }),
                        buf,
                    );
                    y += 1;

                    cur_seed_i += 1;

                    if y >= area.height as i32 {
                        x += 35;
                        y = orig_y;
                    }
                }
            } else if ssim.count_seeds == 0 {
                Paragraph::new("No structure seeds to search")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Red).bold())
                    .render(
                        limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                        buf,
                    );
                return;
            } else {
                Paragraph::new("Too many structure seeds to search")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Red).bold())
                    .render(
                        limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                        buf,
                    );
                return;
            }
        } else {
            Paragraph::new("No structure seeds available")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red).bold())
                .render(
                    limit_area_height(area, 1).offset(Offset { x: 0, y: 1 }),
                    buf,
                );
            return;
        };
    }
}

impl Component for OutputTabComponent {
    type State = OutputTabState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
    ) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10)]);

        let areas = layout.split(area);
        let parts_area = areas[0];

        let parts_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let parts_areas = parts_layout.split(parts_area);

        let part_pillar_area = {
            let block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title_bottom("Pillar seeds")
                .title_alignment(Alignment::Center);
            let inner = block.inner(parts_areas[0]);
            block.render(parts_areas[0], buf);
            inner
        };

        let part_structure_area = {
            let block = Block::default()
                .borders(Borders::ALL)
                .style(
                    Style::default().fg(if state.focus == Focus::StructureSeedButton {
                        Color::LightCyan
                    } else {
                        Color::White
                    }),
                )
                .title_bottom("Structure seeds")
                .title_alignment(Alignment::Center);
            let inner = block.inner(parts_areas[1]);
            block.render(parts_areas[1], buf);
            inner
        };

        let part_world_area = {
            let block = Block::default()
                .borders(Borders::ALL)
                .style(
                    Style::default().fg(if state.focus == Focus::WorldSeedButton {
                        Color::LightCyan
                    } else {
                        Color::White
                    }),
                )
                .title_bottom("World seeds")
                .title_alignment(Alignment::Center);
            let inner = block.inner(parts_areas[2]);
            block.render(parts_areas[2], buf);
            inner
        };

        self.render_pillars(part_pillar_area, buf, state, shared);
        self.render_structures(part_structure_area, buf, state, shared);
        self.render_world(part_world_area, buf, state, shared);
    }

    fn handle_event(
        &self,
        state: &mut Self::State,
        shared: &mut SharedApplicationState,
        event: Event,
        context: EventContext,
    ) -> EventResult {
        if (shared.current_structure_seed_searcher.is_some()
            || !shared.structure_seed_search_jobs.is_empty())
            && (context == EventContext::BubblingUp
                || !matches!(event, Event::Key(key) if key.code == KeyCode::Enter))
        {
            return EventResult::Captured;
        }

        match context {
            EventContext::BubblingDown => match &event {
                Event::Key(key) if key.code == KeyCode::Tab => {
                    state.focus = match state.focus {
                        Focus::StructureSeedButton => Focus::WorldSeedButton,
                        Focus::WorldSeedButton => Focus::Simulation,
                        Focus::Simulation => Focus::Outside,
                        Focus::Outside => Focus::StructureSeedButton,
                    };
                    if state.focus == Focus::Outside {
                        EventResult::BubbleUp(event)
                    } else {
                        EventResult::Captured
                    }
                }
                Event::Key(key) if key.kind != KeyEventKind::Release => match state.focus {
                    Focus::StructureSeedButton
                        if key.code == KeyCode::Enter
                            && shared.current_structure_seed_searcher.is_some() =>
                    {
                        if let Some(job) = shared.current_structure_seed_searcher.take() {
                            shared.structure_seed_search_jobs.clear();
                            job.cancel_join().unwrap();

                            EventResult::Captured
                        } else {
                            // dafuk ?
                            EventResult::BubbleUp(event)
                        }
                    }
                    Focus::StructureSeedButton
                        if key.code == KeyCode::Enter
                            && shared.last_structure_seed_sim.outdated_data =>
                    {
                        shared.structure_seed_search_jobs.clear();
                        shared.last_structure_seed_sim = StructureSeedSimData {
                            outdated_data: true,
                            data: None,
                        };
                        if let Some(job) = shared.current_structure_seed_searcher.take() {
                            job.cancel_join().unwrap();
                        }

                        if let Some(sim) = &shared.last_pillar_sim {
                            if sim.0 == shared.pillar_data {
                                let mut pillar_seeds = sim
                                    .1
                                    .iter()
                                    .filter(|p| !p.1.is_impossible_match())
                                    .collect::<Vec<_>>();
                                if pillar_seeds.len() <= shared.max_pillars_to_simulate {
                                    pillar_seeds.sort_by(|a, b| b.1.compare(&a.1));

                                    let mut data = Vec::new();
                                    if shared.buried_treasure_data.usable {
                                        let c = Math::block_coords_to_chunk_coords((
                                            shared.buried_treasure_data.pos_x,
                                            shared.buried_treasure_data.pos_z,
                                        ));

                                        data.push(StructureData::BuriedTreasureContents {
                                            chunk_x: c.0,
                                            chunk_z: c.1,
                                            luck: shared.buried_treasure_data.luck,
                                            contents: build_fast_inventory_compare_context(
                                                shared.buried_treasure_data.contents.clone(),
                                            ),
                                        });
                                    }

                                    for pillar_seed in pillar_seeds {
                                        shared.structure_seed_search_jobs.push_back(
                                            StructureSeedSearchData {
                                                data: data.clone(),
                                                max_results: 10,
                                                pillar_seed: pillar_seed.0,
                                            },
                                        );
                                    }
                                }
                            }
                        }

                        EventResult::Captured
                    }
                    Focus::WorldSeedButton
                        if key.code == KeyCode::Enter
                            && shared.current_world_seed_searcher.is_some() =>
                    {
                        if let Some(job) = shared.current_world_seed_searcher.take() {
                            shared.world_seed_search_jobs.clear();
                            let _ = job.join();

                            EventResult::Captured
                        } else {
                            // dafuk ?
                            EventResult::BubbleUp(event)
                        }
                    }
                    Focus::WorldSeedButton if key.code == KeyCode::Enter => {
                        shared.world_seed_search_jobs.clear();
                        shared.world_seed_sim = WorldSeedSimData {
                            count_seeds: 0,
                            per_structure: Vec::new(),
                        };

                        if let Some(job) = shared.current_world_seed_searcher.take() {
                            let _ = job.join();
                        }

                        if let Some(sim) = &shared.last_structure_seed_sim.data {
                            if sim.count_seeds >= 1
                                && sim.count_seeds as usize
                                    <= shared.max_structure_seeds_to_simulate
                            {
                                let mut data = Vec::new();
                                data.push(shared.biome_data.overworld_biomes.clone());
                                data.push(shared.biome_data.nether_biomes.clone());

                                for &structure_seed in sim
                                    .per_pillar
                                    .iter()
                                    .map(|p| p.structure_seeds.iter())
                                    .flatten()
                                {
                                    shared
                                        .world_seed_search_jobs
                                        .push_back(WorldSeedSearchData {
                                            structure_seed,
                                            is_random_world_seed: shared.is_random_world_seed,
                                            max_results: shared.max_world_seeds_per_structure_seed,
                                            data: data.clone(),
                                        });
                                }
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
