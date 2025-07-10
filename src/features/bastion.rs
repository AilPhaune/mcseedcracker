use crate::{
    lcg,
    loot_table::{
        ItemLootPoolEntryBuilder, LootPoolBuilder, LootTable, LootTableBuilder, SetCountFunction,
        SetDamageFunction, SetEnchantsRandomlyFunction, SingleChest,
    },
    math::Math,
    random::{JavaRandom, random_with_decorator_seed, random_with_population_seed},
    utils::{
        durability::{ArmorMaterial, ItemWithDurability, ToolMaterial},
        enchants,
    },
};

pub mod items {
    pub mod other {
        pub const DIAMOND_PICKAXE: usize = 1;
        pub const DIAMOND_SHOVEL: usize = 2;
        pub const CROSSBOW: usize = 3;
        pub const ANCIENT_DEBRIS: usize = 4;
        pub const NETHERITE_SCRAP: usize = 5;
        pub const SPECTRAL_ARROW: usize = 6;
        pub const PIGLIN_BANNER_PATTERN: usize = 7;
        pub const MUSIC_DISC_PIGSTEP: usize = 8;
        pub const GOLDEN_CARROT: usize = 9;
        pub const GOLDEN_APPLE: usize = 10;
        pub const ENCHANTED_BOOK: usize = 11;
        pub const IRON_SWORD: usize = 12;
        pub const IRON_BLOCK: usize = 13;
        pub const GOLDEN_BOOTS: usize = 14;
        pub const GOLDEN_AXE: usize = 15;
        pub const GOLD_BLOCK: usize = 16;
        pub const GOLD_INGOT: usize = 17;
        pub const IRON_INGOT: usize = 18;
        pub const GOLDEN_SWORD: usize = 19;
        pub const GOLDEN_CHESTPLATE: usize = 20;
        pub const GOLDEN_HELMET: usize = 21;
        pub const GOLDEN_LEGGINGS: usize = 22;
        pub const CRYING_OBSIDIAN: usize = 24;
        pub const GILDED_BLACKSTONE: usize = 25;
        pub const CHAIN: usize = 25;
        pub const MAGMA_CREAM: usize = 27;
        pub const BONE_BLOCK: usize = 28;
        pub const IRON_NUGGET: usize = 29;
        pub const OBSIDIAN: usize = 30;
        pub const GOLD_NUGGET: usize = 31;
        pub const STRING: usize = 32;
        pub const ARROW: usize = 33;
        pub const COOKED_PORKCHOP: usize = 34;
    }

    pub mod hoglin_stables {
        pub const DIAMOND_SHOVEL: usize = 1;
        pub const DIAMOND_PICKAXE: usize = 2;
        pub const NETHERITE_SCRAP: usize = 3;
        pub const ANCIENT_DEBRIS: usize = 4;
        pub const SADDLE: usize = 5;
        pub const GOLD_BLOCK: usize = 6;
        pub const GOLDEN_CARROT: usize = 7;
        pub const GOLDEN_APPLE: usize = 8;
        pub const GOLDEN_AXE: usize = 9;
        pub const CRYING_OBSIDIAN: usize = 10;
        pub const GLOWSTONE: usize = 11;
        pub const GILDED_BLACKSTONE: usize = 12;
        pub const SOUL_SAND: usize = 13;
        pub const CRIMSON_NYLIUM: usize = 14;
        pub const GOLD_NUGGET: usize = 15;
        pub const LEATHER: usize = 16;
        pub const ARROW: usize = 17;
        pub const STRING: usize = 18;
        pub const PORKCHOP: usize = 19;
        pub const COOKED_PORKCHOP: usize = 20;
        pub const CRIMSON_FUNGUS: usize = 21;
        pub const CRIMSON_ROOTS: usize = 22;
    }

    pub mod treasure_room {
        pub const NETHERITE_INGOT: usize = 0;
        pub const ANCIENT_DEBRIS: usize = 1;
        pub const NETHERITE_SCRAP: usize = 2;
        pub const DIAMOND_SWORD: usize = 3;
        pub const DIAMOND_CHESTPLATE: usize = 4;
        pub const DIAMOND_HELMET: usize = 5;
        pub const DIAMOND_LEGGINGS: usize = 6;
        pub const DIAMOND_BOOTS: usize = 7;
        pub const DIAMOND: usize = 8;
        pub const ENCHANTED_GOLDEN_APPLE: usize = 9;
        pub const SPECTRAL_ARROW: usize = 10;
        pub const GOLD_BLOCK: usize = 11;
        pub const IRON_BLOCK: usize = 12;
        pub const GOLD_INGOT: usize = 13;
        pub const IRON_INGOT: usize = 14;
        pub const CRYING_OBSIDIAN: usize = 15;
        pub const QUARTZ: usize = 16;
        pub const GILDED_BLACKSTONE: usize = 17;
        pub const MAGMA_CREAM: usize = 18;
    }

    pub mod bridges {
        pub const LODESTONE: usize = 1;
        pub const CROSSBOW: usize = 2;
        pub const SPECTRAL_ARROW: usize = 2;
        pub const GILDED_BLACKSTONE: usize = 4;
        pub const CRYING_OBSIDIAN: usize = 5;
        pub const GOLD_BLOCK: usize = 6;
        pub const GOLD_INGOT: usize = 7;
        pub const IRON_INGOT: usize = 8;
        pub const GOLDEN_SWORD: usize = 9;
        pub const GOLDEN_CHESTPLATE: usize = 10;
        pub const GOLDEN_HELMET: usize = 11;
        pub const GOLDEN_LEGGINGS: usize = 12;
        pub const GOLDEN_BOOTS: usize = 13;
        pub const GOLDEN_AXE: usize = 14;
        pub const STRING: usize = 15;
        pub const LEATHER: usize = 16;
        pub const ARROW: usize = 17;
        pub const IRON_NUGGET: usize = 18;
        pub const GOLD_NUGGET: usize = 19;
    }
}

pub mod book_enchants {
    pub mod other {
        pub const SOUL_SPEED: (i32, i32, i32) = (1, 1, 3);
    }
}

pub fn bastion_other_chest_loot_table() -> LootTable {
    use crate::features::bastion::book_enchants::other::SOUL_SPEED;
    use crate::features::bastion::items::other::*;

    LootTableBuilder::new()
        .pool(
            LootPoolBuilder::new()
                .rolls_const(1)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_PICKAXE)
                        .item_stack_size(1)
                        .weight(6)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::PICKAXE)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_SHOVEL)
                        .item_stack_size(1)
                        .weight(6)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CROSSBOW)
                        .item_stack_size(1)
                        .weight(6)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Crossbow.durability(),
                                0.1,
                                0.9,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::CROSSBOW)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ANCIENT_DEBRIS)
                        .weight(12)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(NETHERITE_SCRAP)
                        .weight(4)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(SPECTRAL_ARROW)
                        .weight(10)
                        .function(SetCountFunction::uniform(10, 22).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(PIGLIN_BANNER_PATTERN)
                        .item_stack_size(1)
                        .weight(9)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(MUSIC_DISC_PIGSTEP)
                        .item_stack_size(1)
                        .weight(5)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_CARROT)
                        .weight(12)
                        .function(SetCountFunction::uniform(6, 17).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_APPLE)
                        .weight(9)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ENCHANTED_BOOK)
                        .item_stack_size(1)
                        .weight(10)
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .enchant(SOUL_SPEED)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_const(2)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_SWORD)
                        .item_stack_size(1)
                        .weight(2)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Sword(ToolMaterial::Iron).durability(),
                                0.1,
                                0.9,
                            )
                            .as_function(),
                        )
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::SWORD)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_BLOCK)
                        .weight(2)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_BOOTS)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .enchant(SOUL_SPEED)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_AXE)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::AXE)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_BLOCK)
                        .weight(2)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CROSSBOW)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_INGOT)
                        .weight(2)
                        .function(SetCountFunction::uniform(1, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_INGOT)
                        .weight(2)
                        .function(SetCountFunction::uniform(1, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_SWORD)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_CHESTPLATE)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_HELMET)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_LEGGINGS)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_BOOTS)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CRYING_OBSIDIAN)
                        .weight(2)
                        .function(SetCountFunction::uniform(1, 5).as_function())
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(3, 4)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GILDED_BLACKSTONE)
                        .weight(2)
                        .function(SetCountFunction::uniform(1, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CHAIN)
                        .function(SetCountFunction::uniform(2, 10).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(MAGMA_CREAM)
                        .weight(2)
                        .function(SetCountFunction::uniform(2, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(BONE_BLOCK)
                        .function(SetCountFunction::uniform(3, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_NUGGET)
                        .function(SetCountFunction::uniform(2, 8).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(OBSIDIAN)
                        .function(SetCountFunction::uniform(4, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_NUGGET)
                        .function(SetCountFunction::uniform(2, 8).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(STRING)
                        .function(SetCountFunction::uniform(4, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ARROW)
                        .weight(2)
                        .function(SetCountFunction::uniform(5, 17).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(COOKED_PORKCHOP)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .build(),
        )
        .build()
}

pub fn bastion_hoglin_stables_chest_loot_table() -> LootTable {
    use crate::features::bastion::items::hoglin_stables::*;

    LootTableBuilder::new()
        .pool(
            LootPoolBuilder::new()
                .rolls_const(1)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_SHOVEL)
                        .weight(15)
                        .item_stack_size(1)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Shovel(ToolMaterial::Diamond).durability(),
                                0.15,
                                0.8,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::SHOVEL)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_PICKAXE)
                        .weight(12)
                        .item_stack_size(1)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Pickaxe(ToolMaterial::Diamond).durability(),
                                0.15,
                                0.95,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::PICKAXE)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(NETHERITE_SCRAP)
                        .weight(8)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ANCIENT_DEBRIS)
                        .weight(12)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ANCIENT_DEBRIS)
                        .weight(5)
                        .function(SetCountFunction::constant(2).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(SADDLE)
                        .weight(12)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_BLOCK)
                        .weight(16)
                        .function(SetCountFunction::uniform(2, 4).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_CARROT)
                        .weight(10)
                        .function(SetCountFunction::uniform(8, 17).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_APPLE)
                        .weight(10)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(3, 4)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_AXE)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::AXE)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CRYING_OBSIDIAN)
                        .function(SetCountFunction::uniform(1, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GLOWSTONE)
                        .function(SetCountFunction::uniform(3, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GILDED_BLACKSTONE)
                        .function(SetCountFunction::uniform(2, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(SOUL_SAND)
                        .function(SetCountFunction::uniform(2, 7).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CRIMSON_NYLIUM)
                        .function(SetCountFunction::uniform(2, 7).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_NUGGET)
                        .function(SetCountFunction::uniform(2, 8).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(LEATHER)
                        .function(SetCountFunction::uniform(1, 3).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ARROW)
                        .function(SetCountFunction::uniform(5, 17).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(STRING)
                        .function(SetCountFunction::uniform(3, 8).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(PORKCHOP)
                        .function(SetCountFunction::uniform(2, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(COOKED_PORKCHOP)
                        .function(SetCountFunction::uniform(2, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CRIMSON_FUNGUS)
                        .function(SetCountFunction::uniform(2, 7).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CRIMSON_ROOTS)
                        .function(SetCountFunction::uniform(2, 7).as_function())
                        .build(),
                )
                .build(),
        )
        .build()
}

pub fn bastion_treasure_room_chest_loot_table() -> LootTable {
    use crate::features::bastion::items::treasure_room::*;

    LootTableBuilder::new()
        .pool(
            LootPoolBuilder::new()
                .rolls_const(3)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(NETHERITE_INGOT)
                        .weight(15)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ANCIENT_DEBRIS)
                        .weight(10)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(NETHERITE_SCRAP)
                        .weight(8)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ANCIENT_DEBRIS)
                        .weight(4)
                        .function(SetCountFunction::constant(2).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_SWORD)
                        .item_stack_size(1)
                        .weight(6)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Sword(ToolMaterial::Diamond).durability(),
                                0.8,
                                1.0,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::SWORD)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_CHESTPLATE)
                        .item_stack_size(1)
                        .weight(6)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Chestplate(ArmorMaterial::Diamond).durability(),
                                0.8,
                                1.0,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::CHESTPLATE)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_HELMET)
                        .item_stack_size(1)
                        .weight(6)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Helmet(ArmorMaterial::Diamond).durability(),
                                0.8,
                                1.0,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::HELMET)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_LEGGINGS)
                        .item_stack_size(1)
                        .weight(6)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Leggings(ArmorMaterial::Diamond).durability(),
                                0.8,
                                1.0,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::LEGGINGS)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_BOOTS)
                        .item_stack_size(1)
                        .weight(6)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Boots(ArmorMaterial::Diamond).durability(),
                                0.8,
                                1.0,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::BOOTS)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_SWORD)
                        .weight(6)
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_CHESTPLATE)
                        .weight(5)
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_HELMET)
                        .weight(5)
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_BOOTS)
                        .weight(5)
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND_LEGGINGS)
                        .weight(5)
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(DIAMOND)
                        .weight(5)
                        .function(SetCountFunction::uniform(2, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ENCHANTED_GOLDEN_APPLE)
                        .weight(2)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(3, 4)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(SPECTRAL_ARROW)
                        .function(SetCountFunction::uniform(12, 25).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_BLOCK)
                        .function(SetCountFunction::uniform(2, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_BLOCK)
                        .function(SetCountFunction::uniform(2, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_INGOT)
                        .function(SetCountFunction::uniform(3, 9).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_INGOT)
                        .function(SetCountFunction::uniform(3, 9).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CRYING_OBSIDIAN)
                        .function(SetCountFunction::uniform(3, 5).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(QUARTZ)
                        .function(SetCountFunction::uniform(8, 23).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GILDED_BLACKSTONE)
                        .function(SetCountFunction::uniform(5, 15).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(MAGMA_CREAM)
                        .function(SetCountFunction::uniform(3, 8).as_function())
                        .build(),
                )
                .build(),
        )
        .build()
}

pub fn bastion_bridges_chest_loot_table() -> LootTable {
    use items::bridges::*;

    LootTableBuilder::new()
        .pool(
            LootPoolBuilder::new()
                .rolls_const(1)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(LODESTONE)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(1, 2)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CROSSBOW)
                        .item_stack_size(1)
                        .function(
                            SetDamageFunction::uniform(
                                ItemWithDurability::Crossbow.durability(),
                                0.1,
                                0.5,
                            )
                            .as_function(),
                        )
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::CROSSBOW)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(SPECTRAL_ARROW)
                        .function(SetCountFunction::uniform(10, 28).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GILDED_BLACKSTONE)
                        .function(SetCountFunction::uniform(8, 12).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(CRYING_OBSIDIAN)
                        .function(SetCountFunction::uniform(3, 8).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_BLOCK)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_INGOT)
                        .function(SetCountFunction::uniform(4, 9).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_INGOT)
                        .function(SetCountFunction::uniform(4, 9).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_SWORD)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_CHESTPLATE)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::CHESTPLATE)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_HELMET)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::HELMET)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_LEGGINGS)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::LEGGINGS)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_BOOTS)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::BOOTS)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLDEN_AXE)
                        .item_stack_size(1)
                        .function(SetCountFunction::constant(1).as_function())
                        .function(
                            SetEnchantsRandomlyFunction::builder()
                                .all_of(&enchants::AXE)
                                .build()
                                .as_function(),
                        )
                        .build(),
                )
                .build(),
        )
        .pool(
            LootPoolBuilder::new()
                .rolls_uniform(2, 4)
                .entry_item(
                    ItemLootPoolEntryBuilder::new(STRING)
                        .function(SetCountFunction::uniform(1, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(LEATHER)
                        .function(SetCountFunction::uniform(1, 3).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(ARROW)
                        .function(SetCountFunction::uniform(5, 17).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(IRON_NUGGET)
                        .function(SetCountFunction::uniform(2, 6).as_function())
                        .build(),
                )
                .entry_item(
                    ItemLootPoolEntryBuilder::new(GOLD_NUGGET)
                        .function(SetCountFunction::uniform(2, 6).as_function())
                        .build(),
                )
                .build(),
        )
        .build()
}

pub enum BastionChestType {
    HoglinStables,
    TreasureRoom,
    Bridges,
    Other,
}

pub const fn get_bastion_chest_random(world_seed: i64, chunk_pos: (i32, i32)) -> (JavaRandom, i64) {
    let block_pos = Math::relative_chunk_coords(chunk_pos, (0, 0));

    let population_seed = random_with_population_seed(world_seed, block_pos.0, block_pos.1).1;

    random_with_decorator_seed(population_seed, 12, 40)
}

pub const fn get_bastion_chest_loot_table_seed(
    world_seed: i64,
    chunk_pos: (i32, i32),
    num_chest_in_chunk: i32,
) -> i64 {
    let generator = lcg::JAVA_RANDOM.combine(2 * num_chest_in_chunk as i64);

    let inital_state = get_bastion_chest_random(world_seed, chunk_pos).1;

    let state = generator.next_seed(inital_state ^ lcg::JAVA_RANDOM.get_multiplier());

    let next1 = lcg::JAVA_RANDOM.next_seed(state);
    let next2 = lcg::JAVA_RANDOM.next_seed(next1);

    ((next1 & 0xFFFF_FFFF_0000) << 16).wrapping_add(((next2 >> 16) as i32) as i64)
}

pub fn get_bastion_chest(
    world_seed: i64,
    chunk_pos: (i32, i32),
    num_chest_in_chunk: i32,
    chest_type: BastionChestType,
    luck: f32,
) -> SingleChest {
    let seed = get_bastion_chest_loot_table_seed(world_seed, chunk_pos, num_chest_in_chunk);

    let loot_table = match chest_type {
        BastionChestType::HoglinStables => bastion_hoglin_stables_chest_loot_table(),
        BastionChestType::Other => bastion_other_chest_loot_table(),
        BastionChestType::TreasureRoom => bastion_treasure_room_chest_loot_table(),
        BastionChestType::Bridges => bastion_bridges_chest_loot_table(),
    };

    let mut chest = SingleChest::new();
    loot_table.generate_in_inventory(&mut chest, &mut JavaRandom::new(seed), luck);
    chest
}

#[cfg(test)]
pub mod tests {
    use crate::{
        features::bastion::{
            BastionChestType, get_bastion_chest, get_bastion_chest_loot_table_seed,
        },
        loot_table::{ChestRow, ItemProperty, ItemStack, SingleChest},
        math::Math,
        utils::{
            durability::{ArmorMaterial, ItemWithDurability, ToolMaterial},
            enchants,
        },
    };

    #[test]
    pub fn test_bastion_loot_table_seeds() {
        let world_seed: i64 = 734679766044180411;

        let pos_seed = [
            // Hoglin Stables
            ((97, 166), 0, 2799166732823584713i64),   // y=45
            ((109, 198), 0, -5842683475635873897i64), // y=58
            ((105, 207), 1, 4599594689264021764i64),  // y=68
            ((77, 191), 0, -579672944199389002i64),   // y=51
            ((71, 203), 0, 1857409840148546873i64),   // y=72
            ((72, 203), 1, -1295383310854821847i64),  // y=72
            ((75, 203), 2, 3009594620990002609i64),   // y=72
            ((93, 196), 0, 8394382001088795066i64),   // y=35
            ((93, 196), 1, -4983712224012282870i64),  // y=58
            ((89, 205), 2, -1033940057117477349i64),  // y=68
            // Hoglin Stables
            ((-198, 116), 0, -6910931640652618249i64), // y=33
            ((-199, 118), 1, -7642394814910834770i64), // y=39
            // Treasure Room
            ((-724, -94), 0, 6602806808929530262i64), // y=36
            // Treasure Room
            ((-674, -820), 1, 4548431199292841666i64), // y=36
            // Treasure Room
            ((-2564, -1022), 0, -5772293875455727490i64), // y=36
            // Treasure Room
            ((2229, -1107), 0, -2953714471323038189i64), // y=36
        ];

        for (i, (pos, num, seed)) in pos_seed.into_iter().enumerate() {
            let cpos = Math::block_coords_to_chunk_coords(pos);
            let pseed = get_bastion_chest_loot_table_seed(world_seed, cpos, num);
            assert_eq!(
                seed, pseed,
                "#{i} Wrong seed {pseed} for pos {pos:?}, expected {seed}",
            );
        }
    }

    #[test]
    pub fn test_bastion_hoglin_stables_chest1() {
        use crate::features::bastion::items::hoglin_stables::{
            CRIMSON_FUNGUS, CRIMSON_NYLIUM, CRIMSON_ROOTS, GOLDEN_AXE, SADDLE,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_FUNGUS, 1)),
                        Some(ItemStack::new(SADDLE, 1, 1)),
                        None,
                        Some(ItemStack::of(CRIMSON_FUNGUS, 2)),
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_ROOTS, 2)),
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::with_properties(
                            GOLDEN_AXE,
                            1,
                            1,
                            &[ItemProperty::Enchantment {
                                enchantment: enchants::axe::SMITE.0,
                                level: 2,
                            }],
                        )),
                        Some(ItemStack::of(CRIMSON_ROOTS, 1)),
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_NYLIUM, 1)),
                        None,
                        None,
                        None,
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        None,
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_NYLIUM, 1)),
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_NYLIUM, 2)),
                        None,
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((97, 166)),
            0,
            BastionChestType::HoglinStables,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    pub fn test_bastion_hoglin_stables_chest2() {
        use crate::features::bastion::items::hoglin_stables::{
            ARROW, CRIMSON_NYLIUM, DIAMOND_SHOVEL, GILDED_BLACKSTONE,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        None,
                        None,
                        Some(ItemStack::of(ARROW, 4)),
                        None,
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        None,
                        None,
                        None,
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::with_properties(
                            DIAMOND_SHOVEL,
                            1,
                            1,
                            &[
                                ItemProperty::Damage {
                                    max_durability: ItemWithDurability::Shovel(
                                        ToolMaterial::Diamond,
                                    )
                                    .durability(),
                                    damage: 647,
                                },
                                ItemProperty::Enchantment {
                                    enchantment: enchants::shovel::EFFICIENCY.0,
                                    level: 1,
                                },
                            ],
                        )),
                        None,
                        Some(ItemStack::of(CRIMSON_NYLIUM, 1)),
                        None,
                        Some(ItemStack::of(ARROW, 2)),
                        None,
                        Some(ItemStack::of(GILDED_BLACKSTONE, 3)),
                        None,
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_NYLIUM, 5)),
                        None,
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((-198, 116)),
            0,
            BastionChestType::HoglinStables,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    pub fn test_bastion_hoglin_stables_chest3() {
        use crate::features::bastion::items::hoglin_stables::{
            CRIMSON_NYLIUM, CRYING_OBSIDIAN, DIAMOND_PICKAXE, GLOWSTONE, PORKCHOP,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::of(GLOWSTONE, 3)),
                        Some(ItemStack::of(CRYING_OBSIDIAN, 4)),
                        None,
                        None,
                        Some(ItemStack::of(GLOWSTONE, 2)),
                        Some(ItemStack::of(PORKCHOP, 1)),
                        None,
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_NYLIUM, 1)),
                        None,
                        Some(ItemStack::of(CRYING_OBSIDIAN, 1)),
                        None,
                        Some(ItemStack::with_properties(
                            DIAMOND_PICKAXE,
                            1,
                            1,
                            &[
                                ItemProperty::Damage {
                                    max_durability: ItemWithDurability::Pickaxe(
                                        ToolMaterial::Diamond,
                                    )
                                    .durability(),
                                    damage: 1254,
                                },
                                ItemProperty::Enchantment {
                                    enchantment: enchants::pickaxe::SILK_TOUCH.0,
                                    level: 1,
                                },
                            ],
                        )),
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        None,
                        None,
                        Some(ItemStack::of(CRIMSON_NYLIUM, 2)),
                        None,
                        None,
                        None,
                        None,
                        Some(ItemStack::of(PORKCHOP, 1)),
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((-199, 118)),
            1,
            BastionChestType::HoglinStables,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    pub fn test_bastion_treasure_room_chest1() {
        use crate::features::bastion::items::treasure_room::{
            ANCIENT_DEBRIS, DIAMOND, GOLD_INGOT, IRON_INGOT, NETHERITE_INGOT, QUARTZ,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        Some(ItemStack::of(GOLD_INGOT, 1)),
                        Some(ItemStack::of(DIAMOND, 1)),
                        Some(ItemStack::of(QUARTZ, 1)),
                        None,
                        None,
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        None,
                        Some(ItemStack::of(IRON_INGOT, 3)),
                        Some(ItemStack::of(QUARTZ, 18)),
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::of(QUARTZ, 1)),
                        Some(ItemStack::of(QUARTZ, 8)),
                        Some(ItemStack::of(DIAMOND, 1)),
                        Some(ItemStack::of(DIAMOND, 1)),
                        Some(ItemStack::of(QUARTZ, 1)),
                        None,
                        None,
                        Some(ItemStack::of(GOLD_INGOT, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        Some(ItemStack::of(NETHERITE_INGOT, 1)),
                        None,
                        Some(ItemStack::of(ANCIENT_DEBRIS, 1)),
                        None,
                        None,
                        None,
                        Some(ItemStack::of(GOLD_INGOT, 1)),
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((-1632, 1101)),
            1,
            BastionChestType::TreasureRoom,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    pub fn test_bastion_treasure_room_chest2() {
        use crate::features::bastion::items::treasure_room::{
            DIAMOND_CHESTPLATE, DIAMOND_HELMET, GOLD_INGOT, IRON_BLOCK, NETHERITE_INGOT,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::of(GOLD_INGOT, 3)),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(ItemStack::with_properties(
                            DIAMOND_CHESTPLATE,
                            1,
                            1,
                            &[
                                ItemProperty::Damage {
                                    max_durability: ItemWithDurability::Chestplate(
                                        ArmorMaterial::Diamond,
                                    )
                                    .durability(),
                                    damage: 73,
                                },
                                ItemProperty::Enchantment {
                                    enchantment: enchants::armor::UNBREAKING.0,
                                    level: 3,
                                },
                            ],
                        )),
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(IRON_BLOCK, 4)),
                        None,
                        Some(ItemStack::of(GOLD_INGOT, 3)),
                        Some(ItemStack::of(IRON_BLOCK, 2)),
                        Some(ItemStack::of(IRON_BLOCK, 1)),
                        None,
                        None,
                        None,
                        Some(ItemStack::of(NETHERITE_INGOT, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::with_properties(
                            DIAMOND_HELMET,
                            1,
                            1,
                            &[
                                ItemProperty::Damage {
                                    max_durability: ItemWithDurability::Helmet(
                                        ArmorMaterial::Diamond,
                                    )
                                    .durability(),
                                    damage: 68,
                                },
                                ItemProperty::Enchantment {
                                    enchantment: enchants::armor::UNBREAKING.0,
                                    level: 2,
                                },
                            ],
                        )),
                        None,
                        None,
                        None,
                        Some(ItemStack::of(IRON_BLOCK, 1)),
                        None,
                        None,
                        Some(ItemStack::of(IRON_BLOCK, 1)),
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((4187, -2381)),
            0,
            BastionChestType::TreasureRoom,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    fn test_bastion_bridges_chest1() {
        use crate::features::bastion::items::bridges::{ARROW, LEATHER, LODESTONE, SPECTRAL_ARROW};

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        Some(ItemStack::of(ARROW, 2)),
                        None,
                        None,
                        None,
                        None,
                        Some(ItemStack::of(LEATHER, 1)),
                        None,
                        None,
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(ARROW, 5)),
                        Some(ItemStack::of(ARROW, 5)),
                        Some(ItemStack::of(ARROW, 2)),
                        None,
                        Some(ItemStack::of(ARROW, 6)),
                        Some(ItemStack::of(SPECTRAL_ARROW, 1)),
                        None,
                        None,
                        Some(ItemStack::of(LEATHER, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(SPECTRAL_ARROW, 3)),
                        Some(ItemStack::of(SPECTRAL_ARROW, 1)),
                        Some(ItemStack::of(SPECTRAL_ARROW, 9)),
                        None,
                        Some(ItemStack::of(ARROW, 2)),
                        Some(ItemStack::of(ARROW, 4)),
                        None,
                        Some(ItemStack::of(LODESTONE, 1)),
                        None,
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((583, 956)),
            0,
            BastionChestType::Bridges,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    fn test_bastion_bridges_chest2() {
        use crate::features::bastion::items::bridges::{
            ARROW, GILDED_BLACKSTONE, GOLD_BLOCK, IRON_NUGGET, LEATHER, LODESTONE,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        None,
                        None,
                        Some(ItemStack::of(ARROW, 12)),
                        None,
                        None,
                        Some(ItemStack::of(IRON_NUGGET, 3)),
                        None,
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        None,
                        None,
                        None,
                        Some(ItemStack::of(LODESTONE, 1)),
                        None,
                        Some(ItemStack::of(GOLD_BLOCK, 1)),
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(LEATHER, 1)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        None,
                        None,
                        Some(ItemStack::of(IRON_NUGGET, 3)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 2)),
                        Some(ItemStack::of(ARROW, 1)),
                        Some(ItemStack::of(ARROW, 2)),
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((2473, -636)),
            0,
            BastionChestType::Bridges,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    fn test_bastion_other_chest1() {
        use crate::features::bastion::items::other::{
            ANCIENT_DEBRIS, CHAIN, GILDED_BLACKSTONE, GOLDEN_SWORD, IRON_INGOT, MAGMA_CREAM,
            OBSIDIAN,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        None,
                        Some(ItemStack::new(GOLDEN_SWORD, 1, 1)),
                        None,
                        Some(ItemStack::of(CHAIN, 1)),
                        None,
                        Some(ItemStack::of(CHAIN, 1)),
                        Some(ItemStack::of(CHAIN, 1)),
                        Some(ItemStack::of(MAGMA_CREAM, 1)),
                        Some(ItemStack::of(OBSIDIAN, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(OBSIDIAN, 1)),
                        Some(ItemStack::of(IRON_INGOT, 1)),
                        None,
                        Some(ItemStack::of(MAGMA_CREAM, 2)),
                        None,
                        Some(ItemStack::of(OBSIDIAN, 2)),
                        None,
                        Some(ItemStack::of(ANCIENT_DEBRIS, 1)),
                        Some(ItemStack::of(MAGMA_CREAM, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(CHAIN, 3)),
                        Some(ItemStack::of(CHAIN, 1)),
                        Some(ItemStack::of(MAGMA_CREAM, 1)),
                        None,
                        None,
                        Some(ItemStack::of(GILDED_BLACKSTONE, 2)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 3)),
                        Some(ItemStack::of(OBSIDIAN, 1)),
                        None,
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((583, 964)),
            2,
            BastionChestType::Other,
            0.0,
        );

        assert_eq!(ingame, generated);
    }

    #[test]
    fn test_bastion_other_chest2() {
        use crate::features::bastion::items::other::{
            CROSSBOW, GILDED_BLACKSTONE, GOLDEN_CARROT, GOLDEN_SWORD, MAGMA_CREAM,
        };

        let ingame = SingleChest {
            rows: [
                ChestRow {
                    items: [
                        None,
                        None,
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        Some(ItemStack::of(MAGMA_CREAM, 3)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 2)),
                        None,
                        None,
                        None,
                        Some(ItemStack::of(GOLDEN_CARROT, 1)),
                    ],
                },
                ChestRow {
                    items: [
                        None,
                        None,
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        None,
                        Some(ItemStack::new(CROSSBOW, 1, 1)),
                        None,
                        Some(ItemStack::new(GOLDEN_SWORD, 1, 1)),
                        None,
                    ],
                },
                ChestRow {
                    items: [
                        Some(ItemStack::of(GOLDEN_CARROT, 12)),
                        None,
                        None,
                        None,
                        Some(ItemStack::of(GOLDEN_CARROT, 1)),
                        Some(ItemStack::of(GILDED_BLACKSTONE, 1)),
                        None,
                        Some(ItemStack::of(MAGMA_CREAM, 1)),
                        None,
                    ],
                },
            ],
        };

        let generated = get_bastion_chest(
            734679766044180411,
            Math::block_coords_to_chunk_coords((583, 916)),
            2,
            BastionChestType::Other,
            0.0,
        );

        assert_eq!(ingame, generated);
    }
}
