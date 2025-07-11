use std::time::Duration;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use cubiomes::{
    enums::{BiomeID, Dimension, MCVersion, StructureType},
    generator::{BlockPosition, Generator, GeneratorFlags},
};
use mcseedcracker::{
    features::{
        buried_treasure::{
            self,
            items::{
                COOKED_COD, GOLD_INGOT, HEART_OF_THE_SEA, IRON_INGOT, IRON_SWORD,
                PRISMARINE_CRYSTALS,
            },
        },
        end_pillars::{EndPillars, PartialEndPillars, PillarHeightHint},
    },
    lcg,
    loot_table::{ChestRow, FastInventoryCompareContext, ItemStack, SingleChest},
    math::Math,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

struct Input<'a> {
    pillars: PartialEndPillars,
    buried_treasure: FastInventoryCompareContext<SingleChest, 12>,
    buried_trasure_block_coords: (i32, i32, i32),
    biomes_coords: &'a [(i32, i32, i32, BiomeID)],
}

#[inline(always)]
fn reverse_seed_from_pillars_and_buried_treasure(input: &Input) -> i64 {
    let pillar_seed = {
        let mut seed = None;
        let mut rpillars = EndPillars::new();
        for pseed in 0..65536 {
            rpillars.from_seed(pseed);
            if !input.pillars.matches(&rpillars).is_impossible_match() {
                if let Some(s) = seed {
                    panic!("Found two pillar seeds: {} and {}", s, pseed);
                }
                seed = Some(pseed);
            }
        }

        seed.expect("No pillar seed found")
    };

    let bt_chunk = Math::block_coords_to_chunk_coords((
        input.buried_trasure_block_coords.0,
        input.buried_trasure_block_coords.2,
    ));

    let structure_seed = {
        let rev = lcg::JAVA_RANDOM.combine(-2);

        let mut res_seed = None;
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
                    &input.buried_treasure,
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

        res_seed.expect("structure seed not found")
    };

    let world_seed =
        {
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

                if input.biomes_coords.iter().all(|(bx, by, bz, biome)| {
                    generator.get_biome_at(*bx, *by, *bz).unwrap() == *biome
                }) && generator
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

    assert_eq!(world_seed, -7193194438565520372i64);
    world_seed
}

fn reverse_seed_from_pillars_and_buried_treasure_threaded_rayon(input: &Input) -> i64 {
    let pillar_seed = {
        let mut seed = None;
        let mut rpillars = EndPillars::new();
        for pseed in 0..65536 {
            rpillars.from_seed(pseed);
            if !input.pillars.matches(&rpillars).is_impossible_match() {
                if let Some(s) = seed {
                    panic!("Found two pillar seeds: {} and {}", s, pseed);
                }
                seed = Some(pseed);
            }
        }

        seed.expect("No pillar seed found")
    };

    let bt_chunk = Math::block_coords_to_chunk_coords((
        input.buried_trasure_block_coords.0,
        input.buried_trasure_block_coords.2,
    ));

    let structure_seed = {
        let rev = lcg::JAVA_RANDOM.combine(-2);

        let results = (0u64..(1u64 << 32))
            .into_par_iter()
            .filter_map(|i| {
                let state_lo = i & 0xFFFF;
                let state_hi = i >> 16;
                let state =
                    ((state_hi as i64) << 32) | ((pillar_seed as i64) << 16) | (state_lo as i64);
                let reversed_state = rev.next_seed(state);
                let seed = reversed_state ^ lcg::JAVA_RANDOM.get_multiplier();

                if !buried_treasure::generates_at(seed, bt_chunk) {
                    return None;
                }

                if buried_treasure::compare_buried_treasure_fast_noinv(
                    seed,
                    bt_chunk,
                    0.0,
                    &input.buried_treasure,
                ) {
                    Some(seed)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        match &results[..] {
            [] => panic!("Seed not found"),
            [seed] => *seed,
            multiple => panic!("Multiple seeds found: {:?}", multiple),
        }
    };

    let world_seed =
        {
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

                if input.biomes_coords.iter().all(|(bx, by, bz, biome)| {
                    generator.get_biome_at(*bx, *by, *bz).unwrap() == *biome
                }) && generator
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

    assert_eq!(world_seed, -7193194438565520372i64);
    world_seed
}

pub fn reverse_seed_from_pillars_and_buried_treasure_benchmark(c: &mut Criterion) {
    let mut pillars = PartialEndPillars::new();
    pillars.0[0].height = PillarHeightHint::Exact(103);
    pillars.0[3].height = PillarHeightHint::Exact(76);
    pillars.0[4].caged = Some(true);
    pillars.0[4].height = PillarHeightHint::Exact(82);
    pillars.0[6].caged = Some(true);
    pillars.0[6].height = PillarHeightHint::Exact(79);
    pillars.0[7].height = PillarHeightHint::Exact(100);
    pillars.0[8].height = PillarHeightHint::Exact(97);

    let buried_treasure = SingleChest {
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

    let buried_trasure_block_coords = (409, 59, 809);

    let biomes_coords = &[
        (409, 59, 809, BiomeID::beach),
        (137, 73, -90, BiomeID::jungle),
        (-404, 69, -51, BiomeID::beach),
    ];

    let input = Input {
        pillars,
        buried_treasure: buried_treasure::build_fast_inventory_compare_context(buried_treasure),
        buried_trasure_block_coords,
        biomes_coords,
    };

    let mut group = c.benchmark_group("full_reverse");

    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(300)) // Single threaded is slow ~30s per call
        .bench_with_input(
            BenchmarkId::new(
                "reverse_seed_from_pillars_and_buried_treasure",
                "single_threaded",
            ),
            &input,
            |b, i| b.iter(|| reverse_seed_from_pillars_and_buried_treasure(black_box(i))),
        );

    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(200))
        .bench_with_input(
            BenchmarkId::new(
                "reverse_seed_from_pillars_and_buried_treasure",
                "threaded_rayon",
            ),
            &input,
            |b, i| {
                b.iter(|| {
                    reverse_seed_from_pillars_and_buried_treasure_threaded_rayon(black_box(i))
                })
            },
        );

    group.finish();
}

criterion_group!(
    benches,
    reverse_seed_from_pillars_and_buried_treasure_benchmark
);
criterion_main!(benches);
