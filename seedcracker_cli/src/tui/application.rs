use std::collections::VecDeque;

use mcseedcracker::{
    features::end_pillars::{PartialEndPillars, PillarMatchResult},
    search::{
        StructureSeedSearchData, StructureSeedSearcherHandle, WorldSeedSearchData,
        WorldSeedSearcherHandle,
    },
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Tabs, Widget},
};

use crate::tui::{
    EventContext, EventResult, FullComponent,
    tabs::{
        biomes::{BiomesTab, BiomesTabSharedData},
        buried_treasure::{BuriedTreasureTab, BuriedTreasureTabSharedData},
        end_pillars::EndPillarsTab,
        output::OutputTab,
    },
};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ApplicationComponent;

pub struct ApplicationTabs {
    pub end_pillars: ApplicationTab<EndPillarsTab>,
    pub buried_treasure: ApplicationTab<BuriedTreasureTab>,
    pub biomes: ApplicationTab<BiomesTab>,
    pub output: ApplicationTab<OutputTab>,
}

impl ApplicationTabs {
    const SIZE: usize = 4;

    pub fn titles(&self) -> [String; Self::SIZE] {
        [
            self.end_pillars.title.clone(),
            self.buried_treasure.title.clone(),
            self.biomes.title.clone(),
            self.output.title.clone(),
        ]
    }

    pub fn render(
        &mut self,
        idx: usize,
        area: Rect,
        buf: &mut Buffer,
        shared: &mut SharedApplicationState,
    ) {
        match idx {
            0 => self.end_pillars.component.render(area, buf, shared),
            1 => self.buried_treasure.component.render(area, buf, shared),
            2 => self.biomes.component.render(area, buf, shared),
            3 => self.output.component.render(area, buf, shared),
            _ => {}
        }
    }

    pub fn handle_event(
        &mut self,
        idx: usize,
        event: Event,
        context: EventContext,
        shared: &mut SharedApplicationState,
    ) -> EventResult {
        match idx {
            0 => self
                .end_pillars
                .component
                .handle_event(event, context, shared),
            1 => self
                .buried_treasure
                .component
                .handle_event(event, context, shared),
            2 => self.biomes.component.handle_event(event, context, shared),
            3 => self.output.component.handle_event(event, context, shared),
            _ => EventResult::BubbleUp(event),
        }
    }

    pub fn on_focus(&mut self, idx: usize, shared: &mut SharedApplicationState) {
        match idx {
            0 => self.end_pillars.component.on_focus(shared),
            1 => self.buried_treasure.component.on_focus(shared),
            2 => self.biomes.component.on_focus(shared),
            3 => self.output.component.on_focus(shared),
            _ => {}
        }
    }

    pub fn on_unfocus(&mut self, idx: usize, shared: &mut SharedApplicationState) {
        match idx {
            0 => self.end_pillars.component.on_unfocus(shared),
            1 => self.buried_treasure.component.on_unfocus(shared),
            2 => self.biomes.component.on_unfocus(shared),
            3 => self.output.component.on_unfocus(shared),
            _ => {}
        }
    }

    pub const fn size(&self) -> usize {
        Self::SIZE
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StructureSeedSimResultType {
    Success,
    TooManySeeds,
    Cancelled,
}

pub struct PillarSeedStructureSim {
    pub pillar_seed: i64,
    pub result: StructureSeedSimResultType,
    pub structure_seeds: Vec<i64>,
}

pub struct StructureSeedSim {
    pub count_seeds: i64,
    pub per_pillar: Vec<PillarSeedStructureSim>,
}

pub struct StructureSeedSimData {
    pub outdated_data: bool,
    pub data: Option<StructureSeedSim>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorldSeedSimResultType {
    Success,
    TooManySeeds,
}

pub struct StructureSeedWorldSim {
    pub structure_seed: i64,
    pub result: WorldSeedSimResultType,
    pub world_seeds: Vec<i64>,
}

pub struct WorldSeedSimData {
    pub count_seeds: i64,
    pub per_structure: Vec<StructureSeedWorldSim>,
}

pub struct SharedApplicationState {
    pub pillar_data: PartialEndPillars,
    pub last_pillar_sim: Option<(PartialEndPillars, Vec<(i64, PillarMatchResult)>)>,

    pub max_pillars_to_simulate: usize,
    pub max_structure_seeds_to_simulate: usize,
    pub max_world_seeds_per_structure_seed: u16,

    pub buried_treasure_data: BuriedTreasureTabSharedData,

    pub last_structure_seed_sim: StructureSeedSimData,
    pub current_structure_seed_searcher: Option<StructureSeedSearcherHandle>,
    pub structure_seed_search_jobs: VecDeque<StructureSeedSearchData>,

    pub biome_data: BiomesTabSharedData,
    pub current_world_seed_searcher: Option<WorldSeedSearcherHandle>,
    pub world_seed_search_jobs: VecDeque<WorldSeedSearchData>,
    pub world_seed_sim: WorldSeedSimData,
    pub is_random_world_seed: bool,
}

pub struct ApplicationComponentState {
    pub selected_tab: usize,
    pub focused_on_tab_selector: bool,

    pub tabs: ApplicationTabs,
    pub shared: SharedApplicationState,
}

pub struct ApplicationTab<T: FullComponent> {
    pub title: String,
    pub component: T,
}

impl ApplicationComponentState {
    pub fn new() -> Self {
        Self {
            selected_tab: 0,
            focused_on_tab_selector: true,
            tabs: {
                ApplicationTabs {
                    end_pillars: EndPillarsTab::apptab(),
                    buried_treasure: BuriedTreasureTab::apptab(),
                    biomes: BiomesTab::apptab(),
                    output: OutputTab::apptab(),
                }
            },
            shared: SharedApplicationState {
                pillar_data: PartialEndPillars::new(),
                last_pillar_sim: None,
                max_pillars_to_simulate: 5,
                max_structure_seeds_to_simulate: 5,
                max_world_seeds_per_structure_seed: 5,
                buried_treasure_data: BuriedTreasureTabSharedData::default(),
                last_structure_seed_sim: StructureSeedSimData {
                    outdated_data: true,
                    data: None,
                },
                current_structure_seed_searcher: None,
                structure_seed_search_jobs: VecDeque::new(),
                biome_data: BiomesTabSharedData::default(),
                current_world_seed_searcher: None,
                world_seed_search_jobs: VecDeque::new(),
                world_seed_sim: WorldSeedSimData {
                    count_seeds: 0,
                    per_structure: Vec::new(),
                },
                is_random_world_seed: true,
            },
        }
    }
}

impl Default for ApplicationComponentState {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationComponent {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut ApplicationComponentState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Height of the tabs bar
                    Constraint::Min(0),    // Rest of the area
                ]
                .as_ref(),
            )
            .split(area);

        let titles = state.tabs.titles();
        let selected_title = titles[state.selected_tab].clone();

        let tabs = Tabs::new(titles)
            .select(state.selected_tab)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if state.focused_on_tab_selector {
                        Style::default().bold().fg(Color::LightCyan)
                    } else {
                        Style::default()
                    })
                    .title("Tabs"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).underlined());

        tabs.render(chunks[0], buf);

        // Placeholder for tab content
        let content_block = Block::default()
            .title(selected_title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(if state.focused_on_tab_selector {
                Style::default()
            } else {
                Style::default().bold().fg(Color::LightCyan)
            });

        let content_area = content_block.inner(chunks[1]);

        content_block.render(chunks[1], buf);

        state
            .tabs
            .render(state.selected_tab, content_area, buf, &mut state.shared);
    }

    pub fn handle_event(
        state: &mut ApplicationComponentState,
        event: Event,
        context: EventContext,
    ) -> EventResult {
        match context {
            EventContext::BubblingDown => {
                if state.focused_on_tab_selector {
                    match &event {
                        Event::Key(key)
                            if key.code == KeyCode::Tab || key.code == KeyCode::BackTab =>
                        {
                            state.tabs.on_focus(state.selected_tab, &mut state.shared);
                            state.focused_on_tab_selector = false;
                            EventResult::Captured
                        }
                        Event::Key(key) if key.code == KeyCode::Right => {
                            state.selected_tab = (state.selected_tab + 1) % state.tabs.size();
                            EventResult::Captured
                        }
                        Event::Key(key) if key.code == KeyCode::Left => {
                            state.selected_tab =
                                (state.selected_tab + state.tabs.size() - 1) % state.tabs.size();
                            EventResult::Captured
                        }
                        _ => EventResult::BubbleUp(event),
                    }
                } else {
                    match state.tabs.handle_event(
                        state.selected_tab,
                        event,
                        context,
                        &mut state.shared,
                    ) {
                        EventResult::Captured => EventResult::Captured,
                        EventResult::BubbleUp(event) => {
                            Self::handle_event(state, event, EventContext::BubblingUp)
                        }
                    }
                }
            }
            EventContext::BubblingUp => match &event {
                Event::Key(key) if key.code == KeyCode::Tab || key.code == KeyCode::BackTab => {
                    state.tabs.on_unfocus(state.selected_tab, &mut state.shared);
                    state.focused_on_tab_selector = !state.focused_on_tab_selector;
                    EventResult::Captured
                }
                _ => EventResult::BubbleUp(event),
            },
        }
    }
}
