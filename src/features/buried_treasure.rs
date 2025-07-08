use crate::{
    loot_table::{
        FastInventoryCompareContext, ItemLootPoolEntryBuilder, LootPoolBuilder, LootTable,
        LootTableBuilder, SetCountFunction, SingleChest,
    },
    math::Math,
    random::{
        JavaRandom, random_with_decorator_seed, random_with_population_seed,
        random_with_region_seed,
    },
};

pub const PROB: f32 = 0.01;
pub const SALT: i32 = 10387320;

pub mod items {
    pub const HEART_OF_THE_SEA: usize = 1;
    pub const IRON_INGOT: usize = 2;
    pub const GOLD_INGOT: usize = 3;
    pub const TNT: usize = 4;
    pub const EMERALD: usize = 5;
    pub const DIAMOND: usize = 6;
    pub const PRISMARINE_CRYSTALS: usize = 7;
    pub const LEATHER_CHESTPLATE: usize = 8;
    pub const IRON_SWORD: usize = 9;
    pub const COOKED_COD: usize = 10;
    pub const COOKED_SALMON: usize = 11;
}

pub const fn generates_at(world_seed: i64, chunk_pos: (i32, i32)) -> bool {
    random_with_region_seed(world_seed, chunk_pos.0, chunk_pos.1, SALT)
        .0
        .next_float()
        < PROB
}

pub const fn get_buried_treasure_random(
    world_seed: i64,
    chunk_pos: (i32, i32),
) -> (JavaRandom, i64) {
    let block_pos = Math::relative_chunk_coords(chunk_pos, (0, 0));

    let population_seed = random_with_population_seed(world_seed, block_pos.0, block_pos.1).1;

    random_with_decorator_seed(population_seed, 1, 30)
}

pub const fn get_buried_treasure_loot_table_seed(world_seed: i64, chunk_pos: (i32, i32)) -> i64 {
    get_buried_treasure_random(world_seed, chunk_pos)
        .0
        .next_long()
}

pub fn get_buried_treasure(world_seed: i64, chunk_pos: (i32, i32), luck: f32) -> SingleChest {
    let seed = get_buried_treasure_loot_table_seed(world_seed, chunk_pos);
    let mut chest = SingleChest::new();
    get_loot_table().generate_in_inventory(&mut chest, &mut JavaRandom::new(seed), luck);
    chest
}

pub fn build_fast_inventory_compare_context(
    contents: SingleChest,
) -> FastInventoryCompareContext<SingleChest, 12> {
    let mut ctx = FastInventoryCompareContext {
        inventory: contents,
        items_count: [0; 12],
        total_items: 0,
    };
    for row in ctx.inventory.rows.iter() {
        for item in row.items.iter().flatten() {
            ctx.items_count[item.item] += item.count;
            ctx.total_items += item.count;
        }
    }
    ctx
}

pub fn compare_buried_treasure_fast(
    world_seed: i64,
    chunk_pos: (i32, i32),
    luck: f32,
    compare: &FastInventoryCompareContext<SingleChest, 12>,
    temp_inventory: &mut SingleChest,
) -> bool {
    let seed = get_buried_treasure_loot_table_seed(world_seed, chunk_pos);
    get_loot_table().compare_fast(JavaRandom::new(seed), luck, compare, temp_inventory)
}

pub fn compare_buried_treasure_fast_noinv(
    world_seed: i64,
    chunk_pos: (i32, i32),
    luck: f32,
    compare: &FastInventoryCompareContext<SingleChest, 12>,
) -> bool {
    let seed = get_buried_treasure_loot_table_seed(world_seed, chunk_pos);
    get_loot_table().compare_fast_noinv(JavaRandom::new(seed), luck, compare)
}

pub fn get_loot_table() -> LootTable {
    LootTableBuilder::new()
        .pool(
            LootPoolBuilder::new()
                .rolls_const(1)
                .entry_item(ItemLootPoolEntryBuilder::new(items::HEART_OF_THE_SEA).build())
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(5, 8)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::IRON_INGOT)
                        .weight(20)
                        .function(SetCountFunction::uniform(1, 4).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::GOLD_INGOT)
                        .weight(10)
                        .function(SetCountFunction::uniform(1, 4).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::TNT)
                        .weight(5)
                        .function(SetCountFunction::uniform(1, 2).as_function())
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(1, 3)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::EMERALD)
                        .weight(5)
                        .function(SetCountFunction::uniform(4, 8).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::DIAMOND)
                        .weight(5)
                        .function(SetCountFunction::uniform(1, 2).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::PRISMARINE_CRYSTALS)
                        .weight(5)
                        .function(SetCountFunction::uniform(1, 5).as_function())
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(0, 1)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::LEATHER_CHESTPLATE)
                        .item_stack_size(1)
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::IRON_SWORD)
                        .item_stack_size(1)
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_const(2)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::COOKED_COD)
                        .function(SetCountFunction::uniform(2, 4).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(items::COOKED_SALMON)
                        .function(SetCountFunction::uniform(2, 4).as_function())
                        .build(),
                )
                .build(),
        )
        .build()
}

#[cfg(test)]
mod tests {
    use crate::{
        features::buried_treasure::{
            build_fast_inventory_compare_context, compare_buried_treasure_fast,
            get_buried_treasure, get_buried_treasure_loot_table_seed,
            items::{
                COOKED_COD, COOKED_SALMON, EMERALD, GOLD_INGOT, HEART_OF_THE_SEA, IRON_INGOT,
                IRON_SWORD, LEATHER_CHESTPLATE, PRISMARINE_CRYSTALS,
            },
        },
        loot_table::{ChestRow, ItemStack, SingleChest},
        math::Math,
    };

    fn try_seed(world_seed: i64, block_pos: (i32, i32), expected_seed: i64) {
        let chunk_pos = Math::block_coords_to_chunk_coords(block_pos);
        let seed = get_buried_treasure_loot_table_seed(world_seed, chunk_pos);

        assert_eq!(seed, expected_seed);
    }

    #[test]
    pub fn test_get_buried_treasure_loot_table_seed() {
        try_seed(-1196950963516084279, (-263, -631), -8385268767001419331);
        try_seed(-1196950963516084279, (-967, -263), -476893202187324250);
        try_seed(-1196950963516084279, (-1623, 2313), -3040734584507572075);
        try_seed(-1196950963516084279, (1577, -119), 4124829516682148077);
        try_seed(-1196950963516084279, (3065, 73), 2624351164082513070);

        try_seed(5060469206885489010, (-471, -487), -3569869351127392342);
        try_seed(5060469206885489010, (1529, 1545), -6427291606560533701);
        try_seed(5060469206885489010, (-1655, -1223), -5106311890032048027);
        try_seed(5060469206885489010, (-1351, 89), -5254890889807058324);
        try_seed(5060469206885489010, (-71, -1703), 4938918426473305890);

        try_seed(657814009800288117, (-87, -23), 1749169673229764907);

        try_seed(-5663557327879906173, (-807, 249), 3594531169643699093);
        try_seed(-5663557327879906173, (99865, 99657), -2036158166124562196);
        try_seed(
            -5663557327879906173,
            (25699753, 1447817),
            2165141908458726548,
        );
    }

    #[test]
    pub fn test_buried_treasure_chest() {
        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        Some(ItemStack::of(IRON_INGOT, 2)),
                        Some(ItemStack::new(LEATHER_CHESTPLATE, 1, 1)),
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        None,
                        Some(ItemStack::of(EMERALD, 2)),
                        Some(ItemStack::of(EMERALD, 1)),
                        Some(ItemStack::of(COOKED_SALMON, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(EMERALD, 2)),
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        None,
                        Some(ItemStack::of(COOKED_SALMON, 1)),
                        Some(ItemStack::of(HEART_OF_THE_SEA, 1)),
                        Some(ItemStack::of(COOKED_SALMON, 1)),
                        Some(ItemStack::of(IRON_INGOT, 2)),
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        Some(ItemStack::of(IRON_INGOT, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::of(COOKED_SALMON, 1)),
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        Some(ItemStack::of(COOKED_SALMON, 1)),
                        Some(ItemStack::of(IRON_INGOT, 2)),
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        None,
                        Some(ItemStack::of(COOKED_SALMON, 1)),
                        Some(ItemStack::of(COOKED_SALMON, 1)),
                    ],
                },
            ],
        };

        let generated = get_buried_treasure(
            1094031370582075292,
            Math::block_coords_to_chunk_coords((-55, 3241)),
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    fn test_buried_treasure_chest2() {
        let ingame = SingleChest {
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

        let generated = get_buried_treasure(
            -7193194438565520372,
            Math::block_coords_to_chunk_coords((409, 809)),
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    fn test_buried_treasure_fast_compare() {
        let mut chest = SingleChest::new();

        for seed in 0..100 {
            for chunk_x in 0..10 {
                for chunk_z in 0..10 {
                    let standard = get_buried_treasure(seed, (chunk_x, chunk_z), 0.0);
                    let ctx = build_fast_inventory_compare_context(standard);

                    assert!(
                        compare_buried_treasure_fast(
                            seed,
                            (chunk_x, chunk_z),
                            0.0,
                            &ctx,
                            &mut chest
                        ),
                        "failed fast compare for seed {} chunk_x {} chunk_z {}",
                        seed,
                        chunk_x,
                        chunk_z
                    );
                }
            }
        }
    }

    #[test]
    fn test_fast_compare_invariants() {
        for i in 0..100 {
            let seed: i64 = 0x123456789ABCDEF + i;
            let mystery_chest = get_buried_treasure(seed, (12, 34), 0.0);

            let mut chest = SingleChest::new();
            let ctx = build_fast_inventory_compare_context(mystery_chest);

            for cx in 0..50 {
                for cz in 0..50 {
                    assert_eq!(
                        compare_buried_treasure_fast(seed, (cx, cz), 0.0, &ctx, &mut chest),
                        (cx == 12) && (cz == 34),
                        "Wrong fast compare result for cx {} cz {}",
                        cx,
                        cz
                    );
                }
            }
        }
    }
}
