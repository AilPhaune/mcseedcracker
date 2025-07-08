use std::io::{self, stdout};

use ratatui::{
    Terminal,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyModifiers},
    },
    prelude::CrosstermBackend,
};

use crate::tui::{
    Component, EventContext,
    application::{ApplicationComponent, ApplicationComponentState},
};

pub mod discrete_log;
pub mod features;
pub mod lcg;
pub mod loot_table;
pub mod math;
pub mod random;
pub mod tui;

pub const CHARACTER_ASPECT_RATIO: f64 = 0.5; // width/height

fn main() -> Result<(), io::Error> {
    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = ApplicationComponentState::new();

    loop {
        terminal.draw(|f| {
            f.render_stateful_widget(ApplicationComponent, f.area(), &mut app_state);
        })?;

        let event = crossterm::event::read()?;
        if let Event::Key(key) = &event {
            if (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C'))
                && key.modifiers.contains(KeyModifiers::CONTROL)
            {
                break;
            }
        }

        ApplicationComponent.handle_event(&mut app_state, event, EventContext::BubblingDown);
    }

    // Cleanup terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use cubiomes::{
        enums::{BiomeID, Dimension, MCVersion, StructureType},
        generator::{BlockPosition, Generator, GeneratorFlags},
    };

    use crate::{
        features::{
            buried_treasure,
            end_pillars::{PartialEndPillars, PillarHeightHint},
        },
        lcg,
        loot_table::{ChestRow, ItemStack, SingleChest},
        math::Math,
    };

    #[test]
    fn test_cubiomes() {
        let world_seed_full: i64 = 1094031370582075292;

        let (x, y, z) = (-55, 61, 3241);

        let generator = Generator::new(
            MCVersion::MC_1_16_5,
            world_seed_full,
            Dimension::DIM_OVERWORLD,
            GeneratorFlags::empty(),
        );

        let biome = generator.get_biome_at(x, y, z).unwrap();

        assert_eq!(biome, BiomeID::snowy_beach);
    }

    #[test]
    fn test_full_reverse() {
        let pillar_seed = {
            let mut pillars = PartialEndPillars::new();

            pillars.0[0].height = PillarHeightHint::Exact(103);

            pillars.0[3].height = PillarHeightHint::Exact(76);

            pillars.0[4].caged = Some(true);
            pillars.0[4].height = PillarHeightHint::Exact(82);

            pillars.0[6].caged = Some(true);
            pillars.0[6].height = PillarHeightHint::Exact(79);

            pillars.0[7].height = PillarHeightHint::Exact(100);

            pillars.0[8].height = PillarHeightHint::Exact(97);

            let results = pillars
                .seed_results()
                .into_iter()
                .filter(|(_, r)| !r.is_impossible_match())
                .collect::<Vec<_>>();

            assert_eq!(results.len(), 1);

            results[0].0
        };

        let bt_chunk = Math::block_coords_to_chunk_coords((409, 809));

        let structure_seed = {
            use crate::features::buried_treasure::items::{
                COOKED_COD, GOLD_INGOT, HEART_OF_THE_SEA, IRON_INGOT, IRON_SWORD,
                PRISMARINE_CRYSTALS,
            };

            let bt_contents = SingleChest {
                rows: [
                    ChestRow {
                        items: [
                            Some(ItemStack::of(COOKED_COD, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::new(IRON_SWORD, 1, 1)),
                            Some(ItemStack::of(COOKED_COD, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            None,
                            None,
                            None,
                        ],
                    },
                    ChestRow {
                        items: [
                            Some(ItemStack::of(GOLD_INGOT, 2)),
                            Some(ItemStack::of(COOKED_COD, 2)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::of(HEART_OF_THE_SEA, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::of(COOKED_COD, 2)),
                            None,
                            None,
                        ],
                    },
                    ChestRow {
                        items: [
                            Some(ItemStack::of(GOLD_INGOT, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::of(COOKED_COD, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::of(COOKED_COD, 1)),
                            Some(ItemStack::of(IRON_INGOT, 1)),
                            Some(ItemStack::of(PRISMARINE_CRYSTALS, 3)),
                            Some(ItemStack::of(PRISMARINE_CRYSTALS, 1)),
                            None,
                        ],
                    },
                ],
            };

            let rev = lcg::JAVA_RANDOM.combine(-2);

            let mut res_seed = None;

            for state_lo in 0i64..65536 {
                for state_hi in 0i64..65536 {
                    let state = (state_hi << 32) | (pillar_seed << 16) | state_lo;
                    let reversed_state = rev.next_seed(state);
                    let seed = reversed_state ^ lcg::JAVA_RANDOM.get_multiplier();

                    if !buried_treasure::generates_at(seed, bt_chunk) {
                        continue;
                    }

                    if buried_treasure::get_buried_treasure(seed, bt_chunk, 0.0) == bt_contents {
                        if let Some(struct_seed) = res_seed {
                            panic!(
                                "multiple structure seeds found: last={}, new={}",
                                struct_seed, seed
                            );
                        }
                        res_seed = Some(seed);
                    }
                }
            }

            res_seed.expect("structure seed not found")
        };

        println!("structure_seed: {}", structure_seed);

        let world_seed = {
            let (x, z) = Math::relative_chunk_coords(bt_chunk, (0, 0));

            let mut wseed = None;

            for seed_hi in 0i64..65536 {
                let seed = (seed_hi << 48) | structure_seed;

                let mut generator = Generator::new(
                    MCVersion::MC_1_16_5,
                    seed,
                    Dimension::DIM_OVERWORLD,
                    GeneratorFlags::empty(),
                );

                if generator.get_biome_at(x, 60, z).unwrap() == BiomeID::beach
                    && generator.get_biome_at(137, 73, -90).unwrap() == BiomeID::jungle
                    && generator.get_biome_at(-404, 69, -51).unwrap() == BiomeID::beach
                    && generator
                        .verify_structure_generation_attempt(
                            BlockPosition::new(x + 9, z + 9),
                            StructureType::Treasure,
                        )
                        .unwrap()
                {
                    if let Some(world_seed) = wseed {
                        panic!(
                            "multiple world seeds found: last={}, new={}",
                            world_seed, seed
                        );
                    }
                    wseed = Some(seed);
                }
            }

            wseed.expect("World seed not found")
        };

        println!("Found seed {}", world_seed);

        assert_eq!(world_seed, -7193194438565520372);
    }
}
