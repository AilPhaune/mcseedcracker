pub mod discrete_log;
pub mod features;
pub mod lcg;
pub mod loot_table;
pub mod math;
pub mod random;
pub mod utils;

pub const CHARACTER_ASPECT_RATIO: f64 = 0.5; // width/height

#[cfg(test)]
mod tests {
    use cubiomes::{
        enums::{BiomeID, Dimension, MCVersion, StructureType},
        generator::{BlockPosition, Generator, GeneratorFlags},
    };
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    use crate::{
        features::{
            buried_treasure,
            end_pillars::{EndPillars, PartialEndPillars, PillarHeightHint},
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

            let mut seed = None;
            let mut rpillars = EndPillars::new();
            for pseed in 0..65536 {
                rpillars.from_seed(pseed);
                if !pillars.matches(&rpillars).is_impossible_match() {
                    if let Some(s) = seed {
                        panic!("Found two pillar seeds: {} and {}", s, pseed);
                    }
                    seed = Some(pseed);
                    break;
                }
            }

            seed.expect("No pillar seed found")
        };

        println!("Found pillar seed: {}", pillar_seed);
        assert_eq!(
            pillar_seed, 13847,
            "Wrong pillar seed {}, expected {}",
            pillar_seed, 13847
        );

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

            let bt_compare_context =
                buried_treasure::build_fast_inventory_compare_context(bt_contents);

            let rev = lcg::JAVA_RANDOM.combine(-2);

            /*let mut res_seed = None;
            let mut temp_inventory = SingleChest::new();

            for state_lo in 0i64..65536 {
                for state_hi in 0i64..65536 {
                    let state = (state_hi << 32) | (pillar_seed << 16) | state_lo;
                    let reversed_state = rev.next_seed(state);
                    let seed = reversed_state ^ lcg::JAVA_RANDOM.get_multiplier();

                    if !buried_treasure::generates_at(seed, bt_chunk) {
                        continue;
                    }

                    if buried_treasure::compare_buried_treasure_fast(
                        seed,
                        bt_chunk,
                        0.0,
                        &bt_compare_context,
                        &mut temp_inventory,
                    ) {
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

            res_seed.expect("structure seed not found")*/

            let values: Vec<i64> = (0u64..(1u64 << 32))
                .into_par_iter()
                .filter_map(|i| {
                    let state_lo = i & 0xFFFF;
                    let state_hi = i >> 16;
                    let state = ((state_hi as i64) << 32)
                        | ((pillar_seed as i64) << 16)
                        | (state_lo as i64);
                    let reversed_state = rev.next_seed(state);
                    let seed = reversed_state ^ lcg::JAVA_RANDOM.get_multiplier();

                    if !buried_treasure::generates_at(seed, bt_chunk) {
                        return None;
                    }

                    if buried_treasure::compare_buried_treasure_fast_noinv(
                        seed,
                        bt_chunk,
                        0.0,
                        &bt_compare_context,
                    ) {
                        Some(seed)
                    } else {
                        None
                    }
                })
                .collect();

            match &values[..] {
                [] => panic!("structure seed not found"),
                [single] => *single,
                multiple => panic!("multiple structure seeds found: {:?}", multiple),
            }
        };

        println!("Found structure seed: {}", structure_seed);
        assert_eq!(
            structure_seed, 180066252004364i64,
            "wrong structure seed {}, expected {}",
            structure_seed, 180066252004364i64
        );

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

        println!("Found world seed {}", world_seed);
        assert_eq!(
            world_seed, -7193194438565520372i64,
            "Wrong world seed {}, expected {}",
            world_seed, -7193194438565520372i64
        );
    }
}
