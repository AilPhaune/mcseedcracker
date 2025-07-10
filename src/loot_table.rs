use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    math::Math,
    random::{JavaRandom, shuffle},
};

#[derive(Clone, PartialEq, Eq)]
pub enum ItemProperty {
    Damage { max_durability: i32, damage: i32 },
    Enchantment { enchantment: i32, level: i32 },
}

impl Debug for ItemProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Damage {
                max_durability,
                damage,
            } => {
                write!(
                    f,
                    "Damage(damage={},durability={}/{})",
                    damage,
                    max_durability - damage,
                    max_durability
                )
            }
            Self::Enchantment { enchantment, level } => {
                write!(
                    f,
                    "Enchantment(enchantment={},level={})",
                    enchantment, level
                )
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: usize,
    pub count: i32,
    pub max_count: i32,
    pub properties: Vec<ItemProperty>,
}

impl ItemStack {
    pub fn of(item: usize, count: i32) -> Self {
        Self {
            item,
            count,
            max_count: 64,
            properties: Vec::new(),
        }
    }

    pub fn new(item: usize, count: i32, max_count: i32) -> Self {
        Self {
            item,
            count,
            max_count,
            properties: Vec::new(),
        }
    }

    pub fn with_properties(
        item: usize,
        count: i32,
        max_count: i32,
        properties: &[ItemProperty],
    ) -> Self {
        Self {
            item,
            count,
            max_count,
            properties: properties.to_vec(),
        }
    }

    pub fn split(&self, count: i32) -> (ItemStack, ItemStack) {
        let count = count.min(self.count);
        let a = ItemStack {
            item: self.item,
            count,
            max_count: self.max_count,
            properties: self.properties.clone(),
        };
        let b = ItemStack {
            item: self.item,
            count: self.count - count,
            max_count: self.max_count,
            properties: self.properties.clone(),
        };
        (a, b)
    }
}

impl Debug for ItemStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}/{}) {:?}",
            self.item, self.count, self.max_count, self.properties
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChestRow {
    pub items: [Option<ItemStack>; 9],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleChest {
    pub rows: [ChestRow; 3],
}

pub trait Inventory: Debug {
    fn slot_count(&self) -> i32;
    fn set_item(&mut self, slot: i32, item: Option<ItemStack>);
    fn get_item(&self, slot: i32) -> Option<&ItemStack>;
    fn remove_item(&mut self, slot: i32) -> Option<ItemStack>;
    fn clear(&mut self);
}

impl SingleChest {
    pub const fn new() -> Self {
        Self {
            rows: [const {
                ChestRow {
                    items: [const { None }; 9],
                }
            }; 3],
        }
    }

    pub const fn get_slot(&self, slot: i32) -> Option<Option<&ItemStack>> {
        if slot < 0 || slot >= 27 {
            None
        } else {
            Some(self.rows[slot as usize / 9].items[slot as usize % 9].as_ref())
        }
    }

    pub const fn get_slot_mut(&mut self, slot: i32) -> Option<&mut Option<ItemStack>> {
        if slot < 0 || slot >= 27 {
            None
        } else {
            Some(&mut self.rows[slot as usize / 9].items[slot as usize % 9])
        }
    }
}

#[derive(Debug, Clone)]
pub struct FastInventoryCompareContext<T: Inventory + PartialEq, const N: usize> {
    pub items_count: [i32; N],
    pub total_items: i32,
    pub inventory: T,
}

impl Default for SingleChest {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory for SingleChest {
    fn slot_count(&self) -> i32 {
        27
    }

    fn set_item(&mut self, slot: i32, item: Option<ItemStack>) {
        if let Some(slot) = self.get_slot_mut(slot) {
            *slot = item;
        }
    }

    fn get_item(&self, slot: i32) -> Option<&ItemStack> {
        self.get_slot(slot)?
    }

    fn remove_item(&mut self, slot: i32) -> Option<ItemStack> {
        self.get_slot_mut(slot)?.take()
    }

    fn clear(&mut self) {
        self.rows
            .iter_mut()
            .for_each(|r| r.items.iter_mut().for_each(|i| *i = None));
    }
}

#[derive(Debug, Clone)]
pub struct LootTable {
    pools: Vec<LootPool>,
}

macro_rules! compare_fast0 {
    ($loot: ident, $compare: ident, $rng: ident, $luck: ident, $self: ident) => {{
        let mut rem_count = $compare.items_count;
        let mut rem_items = $compare.total_items;

        if !$self.generate_raw_loot_callback(&mut $rng, $luck, |items, stop| {
            rem_count[items.item] -= items.count;
            if rem_count[items.item] < 0 {
                *stop = true;
                return;
            }

            rem_items -= items.count;
            if rem_items < 0 {
                *stop = true;
                return;
            }

            if items.count < items.max_count {
                $loot.push(items);
            } else {
                let stacks = items.count / items.max_count;
                let remainder = items.count % items.max_count;
                if remainder > 0 {
                    for _ in 0..stacks {
                        $loot.push(ItemStack {
                            item: items.item,
                            count: items.max_count,
                            max_count: items.max_count,
                            properties: items.properties.clone(),
                        });
                    }
                    $loot.push(ItemStack {
                        item: items.item,
                        count: remainder,
                        max_count: items.max_count,
                        properties: items.properties.clone(),
                    });
                } else {
                    $loot.push(ItemStack {
                        item: items.item,
                        count: items.max_count,
                        max_count: items.max_count,
                        properties: items.properties.clone(),
                    });
                }
            }
        }) {
            return false;
        }
    }};
}

macro_rules! compare_fast1 {
    ($temp_empty_inventory: ident, $compare: ident, $loot: ident, $rng: ident, $self_type: ident) => {{
        let mut free_slots = $self_type::get_free_slots($temp_empty_inventory, &mut $rng);

        $self_type::shuffle_loot(&mut $loot, free_slots.len() as i32, &mut $rng);

        for stack in $loot {
            let Some(slot) = free_slots.pop() else {
                break;
            };

            if stack.count == 0 {
                $temp_empty_inventory.set_item(slot, None);
            } else {
                $temp_empty_inventory.set_item(slot, Some(stack));
            }
        }

        $temp_empty_inventory == &$compare.inventory
    }};
}

impl LootTable {
    pub const fn new(pools: Vec<LootPool>) -> Self {
        Self { pools }
    }

    pub fn generate_raw_loot(&self, rng: &mut JavaRandom, luck: f32) -> Vec<ItemStack> {
        let mut res = Vec::new();
        for pool in &self.pools {
            res.extend(pool.generate_raw_loot(rng, luck));
        }
        res
    }

    pub fn generate_unverified_stacked_loot(
        &self,
        rng: &mut JavaRandom,
        luck: f32,
    ) -> Vec<ItemStack> {
        let mut res: Vec<ItemStack> = Vec::new();

        for loot in self.generate_raw_loot(rng, luck) {
            if let Some(item) = res.iter_mut().find(|i| i.item == loot.item) {
                item.count += loot.count;
            } else {
                res.push(loot);
            }
        }

        res
    }

    /// Returns false if the generation process has been stopped, returns true if it was completed
    pub fn generate_raw_loot_callback<F>(
        &self,
        rng: &mut JavaRandom,
        luck: f32,
        mut callback: F,
    ) -> bool
    where
        F: FnMut(ItemStack, &mut bool),
    {
        let mut stop = false;
        for pool in &self.pools {
            if stop {
                break;
            }
            pool.generate_raw_loot_callback(rng, luck, (&mut callback, &mut stop));
        }
        !stop
    }

    fn divide(loot: Vec<ItemStack>) -> Vec<ItemStack> {
        loot.into_iter()
            .flat_map(|items| {
                if items.count < items.max_count {
                    vec![items]
                } else {
                    let stacks = items.count / items.max_count;
                    let remainder = items.count % items.max_count;
                    if remainder > 0 {
                        let mut result = Vec::with_capacity(stacks as usize + 1);
                        for _ in 0..stacks {
                            result.push(ItemStack {
                                item: items.item,
                                count: items.max_count,
                                max_count: items.max_count,
                                properties: items.properties.clone(),
                            });
                        }
                        result.push(ItemStack {
                            item: items.item,
                            count: remainder,
                            max_count: items.max_count,
                            properties: items.properties.clone(),
                        });
                        result
                    } else {
                        vec![ItemStack {
                            item: items.item,
                            count: items.max_count,
                            max_count: items.max_count,
                            properties: items.properties.clone(),
                        }]
                    }
                }
            })
            .collect()
    }

    fn get_free_slots(inv: &dyn Inventory, rng: &mut JavaRandom) -> Vec<i32> {
        let mut slots = (0..inv.slot_count())
            .filter(|&slot| match inv.get_item(slot) {
                Some(items) => items.count == 0,
                None => true,
            })
            .collect::<Vec<_>>();

        shuffle(&mut slots, rng);
        slots
    }

    fn shuffle_loot(loot: &mut Vec<ItemStack>, free_slots: i32, rng: &mut JavaRandom) {
        let mut moved = Vec::new();

        let mut i = 0;
        while i < loot.len() {
            if loot[i].count == 0 {
                loot.remove(i);
            } else if loot[i].count > 1 {
                moved.push(loot.remove(i));
            } else {
                i += 1;
            }
        }

        while moved.len() + loot.len() < free_slots as usize && !moved.is_empty() {
            let i = Math::next_int(rng, 0, moved.len() as i32 - 1);
            let stack = moved.remove(i as usize);
            let count = Math::next_int(rng, 1, stack.count / 2);
            let (b, a) = stack.split(count);

            if (a.count > 1) && rng.next_bool() {
                moved.push(a);
            } else {
                loot.push(a);
            }

            if (b.count > 1) && rng.next_bool() {
                moved.push(b);
            } else {
                loot.push(b);
            }
        }

        loot.extend(moved);
        shuffle(loot, rng);
    }

    pub fn generate_in_inventory(&self, inv: &mut dyn Inventory, rng: &mut JavaRandom, luck: f32) {
        let mut loot = Self::divide(self.generate_raw_loot(rng, luck));
        let mut free_slots = Self::get_free_slots(inv, rng);

        Self::shuffle_loot(&mut loot, free_slots.len() as i32, rng);

        for stack in loot {
            let Some(slot) = free_slots.pop() else {
                break;
            };

            if stack.count == 0 {
                inv.set_item(slot, None);
            } else {
                inv.set_item(slot, Some(stack));
            }
        }
    }

    pub fn compare_fast<T: Inventory + PartialEq, const N: usize>(
        &self,
        mut rng: JavaRandom,
        luck: f32,
        compare: &FastInventoryCompareContext<T, N>,
        temp_empty_inventory: &mut T,
    ) -> bool {
        let mut loot = Vec::new();
        compare_fast0!(loot, compare, rng, luck, self);
        temp_empty_inventory.clear();
        compare_fast1!(temp_empty_inventory, compare, loot, rng, LootTable)
    }

    pub fn compare_fast_noinv<T: Inventory + PartialEq + Default, const N: usize>(
        &self,
        mut rng: JavaRandom,
        luck: f32,
        compare: &FastInventoryCompareContext<T, N>,
    ) -> bool {
        let mut loot = Vec::new();
        compare_fast0!(loot, compare, rng, luck, self);
        let temp_empty_inventory = &mut T::default();
        compare_fast1!(temp_empty_inventory, compare, loot, rng, LootTable)
    }
}

#[derive(Debug, Clone)]
pub struct LootTableBuilder {
    table: LootTable,
}

impl Default for LootTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LootTableBuilder {
    pub fn build(self) -> LootTable {
        self.table
    }

    pub fn new() -> Self {
        Self {
            table: LootTable { pools: vec![] },
        }
    }

    pub fn pool(mut self, pool: LootPool) -> Self {
        self.table.pools.push(pool);
        self
    }
}

#[derive(Debug, Clone)]
pub enum LootTableRange<T> {
    Uniform { min: T, max: T },
    Constant { value: T },
}

impl LootTableRange<i32> {
    pub fn apply(&self, rng: &mut JavaRandom) -> i32 {
        match self {
            LootTableRange::Uniform { min, max } => {
                if min >= max {
                    *min
                } else {
                    rng.next_bounded_int(*max - *min + 1) + *min
                }
            }
            LootTableRange::Constant { value } => *value,
        }
    }
}

impl LootTableRange<f32> {
    pub fn apply(&self, rng: &mut JavaRandom) -> f32 {
        match self {
            LootTableRange::Uniform { min, max } => rng.next_float() * (*max - *min) + *min,
            LootTableRange::Constant { value } => *value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LootPool {
    rolls: LootTableRange<i32>,

    entries: Vec<LootPoolEntry>,
}

impl LootPool {
    pub fn generate_raw_loot(&self, rng: &mut JavaRandom, luck: f32) -> Vec<ItemStack> {
        let rolls = self.rolls.apply(rng);

        let mut vec = Vec::new();
        for _ in 0..rolls {
            vec.extend_from_slice(&self.select_entry(rng, luck).generate_raw_loot(rng, luck));
        }

        vec
    }

    pub fn generate_raw_loot_callback<F>(
        &self,
        rng: &mut JavaRandom,
        luck: f32,
        mut callback: (F, &mut bool),
    ) where
        F: FnMut(ItemStack, &mut bool),
    {
        let rolls = self.rolls.apply(rng);
        for _ in 0..rolls {
            if *callback.1 {
                break;
            }
            self.select_entry(rng, luck).generate_raw_loot_callback(
                rng,
                luck,
                (&mut callback.0, callback.1),
            );
        }
    }

    fn select_entry(&self, rng: &mut JavaRandom, luck: f32) -> &LootPoolEntry {
        if self.entries.len() == 1 {
            return &self.entries[0];
        }

        let mut temp_totals = Vec::with_capacity(self.entries.len());
        let mut total = 0;
        for entry in self.entries.iter() {
            let min_inc = total;
            total += entry.get_weight(luck);
            let max_exc = total;
            temp_totals.push((min_inc, max_exc));
        }

        let i = rng.next_bounded_int(total);

        let idx = match temp_totals.binary_search_by(|&(min_inc, max_exc)| {
            if i < min_inc {
                Ordering::Greater
            } else if i >= max_exc {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }) {
            Ok(index) => index,
            Err(_) => panic!("Index not found"),
        };

        &self.entries[idx]
    }
}

#[derive(Debug, Clone)]
pub struct LootPoolBuilder {
    pool: LootPool,
}

impl Default for LootPoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LootPoolBuilder {
    pub fn new() -> Self {
        Self {
            pool: LootPool {
                rolls: LootTableRange::Constant { value: 0 },
                entries: vec![],
            },
        }
    }

    pub fn build(self) -> LootPool {
        self.pool
    }

    pub fn rolls(mut self, rolls: LootTableRange<i32>) -> Self {
        self.pool.rolls = rolls;
        self
    }

    pub fn rolls_const(mut self, value: i32) -> Self {
        self.pool.rolls = LootTableRange::Constant { value };
        self
    }

    pub fn rolls_uniform(mut self, min: i32, max: i32) -> Self {
        self.pool.rolls = LootTableRange::Uniform { min, max };
        self
    }

    pub fn entry(mut self, entry: LootPoolEntry) -> Self {
        self.pool.entries.push(entry);
        self
    }

    pub fn entry_item(mut self, item: ItemLootPoolEntry) -> Self {
        self.pool.entries.push(LootPoolEntry::Item(item));
        self
    }
}

#[derive(Debug, Clone)]
pub enum LootPoolEntry {
    Item(ItemLootPoolEntry),
}

impl LootPoolEntry {
    pub fn generate_raw_loot(&self, rng: &mut JavaRandom, luck: f32) -> Vec<ItemStack> {
        match self {
            LootPoolEntry::Item(item) => vec![item.generate_raw_loot(rng, luck)],
        }
    }

    pub fn generate_raw_loot_callback<F>(
        &self,
        rng: &mut JavaRandom,
        luck: f32,
        mut callback: (F, &mut bool),
    ) where
        F: FnMut(ItemStack, &mut bool),
    {
        match self {
            LootPoolEntry::Item(item) => callback.0(item.generate_raw_loot(rng, luck), callback.1),
        }
    }

    pub fn get_weight(&self, luck: f32) -> i32 {
        match self {
            LootPoolEntry::Item(item) => item.get_weight(luck),
        }
    }
}

pub trait LootFunction: Debug {
    fn apply(&self, item: ItemStack, rng: &mut JavaRandom, luck: f32) -> ItemStack;
}

#[derive(Debug, Clone)]
pub struct ItemLootPoolEntry {
    weight: i32,
    quality: i32,
    stack_size: i32,
    item: usize,
    functions: Vec<Arc<dyn LootFunction>>,
}

impl ItemLootPoolEntry {
    pub fn generate_raw_loot(&self, rng: &mut JavaRandom, luck: f32) -> ItemStack {
        let mut item_stack = ItemStack {
            item: self.item,
            count: 1,
            max_count: self.stack_size,
            properties: Vec::new(),
        };

        for f in &self.functions {
            item_stack = f.apply(item_stack, rng, luck);
        }

        item_stack
    }

    pub fn get_weight(&self, luck: f32) -> i32 {
        (self.weight + (self.quality as f32 * luck).floor() as i32).max(0)
    }
}

#[derive(Debug, Clone)]
pub struct ItemLootPoolEntryBuilder {
    entry: ItemLootPoolEntry,
}

impl ItemLootPoolEntryBuilder {
    pub fn new(item: usize) -> Self {
        Self {
            entry: ItemLootPoolEntry {
                weight: 1,
                quality: 1,
                stack_size: 64,
                item,
                functions: vec![],
            },
        }
    }

    pub fn build(self) -> ItemLootPoolEntry {
        self.entry
    }

    pub fn weight(mut self, weight: i32) -> Self {
        self.entry.weight = weight;
        self
    }

    pub fn quality(mut self, quality: i32) -> Self {
        self.entry.quality = quality;
        self
    }

    pub fn item_stack_size(mut self, stack_size: i32) -> Self {
        self.entry.stack_size = stack_size;
        self
    }

    pub fn item(mut self, item: usize) -> Self {
        self.entry.item = item;
        self
    }

    pub fn function(mut self, function: Arc<dyn LootFunction>) -> Self {
        self.entry.functions.push(function);
        self
    }
}

#[derive(Debug, Clone)]
pub struct SetCountFunction {
    range: LootTableRange<i32>,
}

impl SetCountFunction {
    pub const fn new(range: LootTableRange<i32>) -> Self {
        Self { range }
    }

    pub const fn constant(value: i32) -> Self {
        Self::new(LootTableRange::Constant { value })
    }

    pub const fn uniform(min: i32, max: i32) -> Self {
        Self::new(LootTableRange::Uniform { min, max })
    }

    pub fn as_function(self) -> Arc<dyn LootFunction> {
        Arc::new(self)
    }
}

impl LootFunction for SetCountFunction {
    fn apply(&self, item: ItemStack, rng: &mut JavaRandom, _luck: f32) -> ItemStack {
        ItemStack {
            item: item.item,
            count: self.range.apply(rng),
            max_count: item.max_count,
            properties: item.properties,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetDamageFunction {
    range: LootTableRange<f32>,
    item_durability: i32,
}

impl SetDamageFunction {
    pub const fn new(item_durability: i32, range: LootTableRange<f32>) -> Self {
        Self {
            item_durability,
            range,
        }
    }

    pub const fn constant(item_durability: i32, value: f32) -> Self {
        Self::new(item_durability, LootTableRange::Constant { value })
    }

    pub const fn uniform(item_durability: i32, min: f32, max: f32) -> Self {
        Self::new(item_durability, LootTableRange::Uniform { min, max })
    }

    pub fn as_function(self) -> Arc<dyn LootFunction> {
        Arc::new(self)
    }
}

impl LootFunction for SetDamageFunction {
    fn apply(&self, mut item: ItemStack, rng: &mut JavaRandom, _luck: f32) -> ItemStack {
        let damage = 1.0f32 - self.range.apply(rng);

        item.properties.push(ItemProperty::Damage {
            max_durability: self.item_durability,
            damage: (damage * self.item_durability as f32).floor() as i32,
        });

        item
    }
}

#[derive(Debug, Clone)]
pub struct SetEnchantsRandomlyFunction {
    // List of pairs (enchant_id, min_level, max_level)
    pub enchantments: Vec<(i32, i32, i32)>,
}

impl SetEnchantsRandomlyFunction {
    pub fn builder() -> SetEnchantsRandomlyFunctionBuilder {
        SetEnchantsRandomlyFunctionBuilder {
            func: SetEnchantsRandomlyFunction {
                enchantments: vec![],
            },
        }
    }

    pub fn as_function(self) -> Arc<dyn LootFunction> {
        Arc::new(self)
    }
}

impl LootFunction for SetEnchantsRandomlyFunction {
    fn apply(&self, mut item: ItemStack, rng: &mut JavaRandom, _luck: f32) -> ItemStack {
        let (enchant, min_level, max_level) = match self.enchantments.len() {
            0 => return item,
            l => {
                let i = rng.next_bounded_int(l as i32);
                self.enchantments[i as usize]
            }
        };

        let level = Math::next_int(rng, min_level, max_level);

        item.properties.push(ItemProperty::Enchantment {
            enchantment: enchant,
            level,
        });

        item
    }
}

#[derive(Debug, Clone)]
pub struct SetEnchantsRandomlyFunctionBuilder {
    func: SetEnchantsRandomlyFunction,
}

impl SetEnchantsRandomlyFunctionBuilder {
    pub fn build(self) -> SetEnchantsRandomlyFunction {
        self.func
    }

    pub fn enchantment(mut self, enchantment: i32, min_level: i32, max_level: i32) -> Self {
        self.func
            .enchantments
            .push((enchantment, min_level, max_level));
        self
    }

    pub fn enchant(mut self, enchantment: (i32, i32, i32)) -> Self {
        self.func.enchantments.push(enchantment);
        self
    }

    pub fn all_of(mut self, enchantments: &[(i32, i32, i32)]) -> Self {
        for e in enchantments {
            if self.func.enchantments.iter().any(|x| x.0 == e.0) {
                continue;
            }
            self.func.enchantments.push(*e);
        }
        self
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use crate::features::{
        bastion::{
            self, bastion_bridges_chest_loot_table, bastion_hoglin_stables_chest_loot_table,
            bastion_other_chest_loot_table, bastion_treasure_room_chest_loot_table,
        },
        buried_treasure::{self, get_loot_table},
    };

    use super::*;

    fn check_loot(seed: i64, loot: Vec<ItemStack>, expected: Vec<ItemStack>) {
        let mut map = HashMap::new();
        for loot in &loot {
            map.entry(loot.item)
                .and_modify(|count| *count += loot.count)
                .or_insert(loot.count);
        }

        for expected in &expected {
            map.entry(expected.item)
                .and_modify(|count| *count -= expected.count)
                .or_insert(-expected.count);
        }

        for (item, count) in map {
            assert_eq!(
                count,
                0,
                "Wrong count for item {} with loot seed {}: Expected: {}, Actual: {}.\nLoot: {:#?}\nExpected: {:#?}",
                item,
                seed,
                expected
                    .iter()
                    .find(|s| s.item == item)
                    .map(|s| s.count)
                    .unwrap_or(0),
                loot.iter()
                    .find(|s| s.item == item)
                    .map(|s| s.count)
                    .unwrap_or(0),
                loot,
                expected
            );
        }
    }

    #[test]
    pub fn test_burried_treasure_loot_table() {
        let lt = get_loot_table();

        {
            // World seed: -1196950963516084279, coords: -263 -631
            let loot_seed = -8385268767001419331i64;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(buried_treasure::items::PRISMARINE_CRYSTALS, 4),
                    ItemStack::of(buried_treasure::items::DIAMOND, 3),
                    ItemStack::of(buried_treasure::items::IRON_INGOT, 9),
                    ItemStack::of(buried_treasure::items::COOKED_SALMON, 6),
                    ItemStack::of(buried_treasure::items::GOLD_INGOT, 6),
                    ItemStack::of(buried_treasure::items::HEART_OF_THE_SEA, 1),
                ],
            );
        }
        {
            // World seed: -1196950963516084279, coords: -967 -263
            let loot_seed = -476893202187324250;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(buried_treasure::items::GOLD_INGOT, 10),
                    ItemStack::of(buried_treasure::items::COOKED_COD, 2),
                    ItemStack::of(buried_treasure::items::IRON_INGOT, 7),
                    ItemStack::of(buried_treasure::items::COOKED_SALMON, 3),
                    ItemStack::of(buried_treasure::items::EMERALD, 8),
                    ItemStack::of(buried_treasure::items::DIAMOND, 3),
                    ItemStack::of(buried_treasure::items::TNT, 1),
                    ItemStack::of(buried_treasure::items::HEART_OF_THE_SEA, 1),
                ],
            );
        }
    }

    #[test]
    pub fn test_bastion_hoglin_stables_loot_table() {
        let lt = bastion_hoglin_stables_chest_loot_table();

        {
            // World seed: 734679766044180411, coords: 97 45 166
            let loot_seed = 2799166732823584713;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::hoglin_stables::CRIMSON_FUNGUS, 3),
                    ItemStack::of(bastion::items::hoglin_stables::CRIMSON_ROOTS, 3),
                    ItemStack::of(bastion::items::hoglin_stables::CRIMSON_NYLIUM, 4),
                    ItemStack::of(bastion::items::hoglin_stables::SADDLE, 1),
                    ItemStack::of(bastion::items::hoglin_stables::GOLDEN_AXE, 1),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: -198 33 116
            let loot_seed = -6910931640652618249;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::hoglin_stables::ARROW, 6),
                    ItemStack::of(bastion::items::hoglin_stables::GILDED_BLACKSTONE, 5),
                    ItemStack::of(bastion::items::hoglin_stables::CRIMSON_NYLIUM, 6),
                    ItemStack::of(bastion::items::hoglin_stables::DIAMOND_SHOVEL, 1),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: -199 39 118
            let loot_seed = -7642394814910834770;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::hoglin_stables::GLOWSTONE, 5),
                    ItemStack::of(bastion::items::hoglin_stables::CRYING_OBSIDIAN, 5),
                    ItemStack::of(bastion::items::hoglin_stables::CRIMSON_NYLIUM, 3),
                    ItemStack::of(bastion::items::hoglin_stables::PORKCHOP, 2),
                    ItemStack::of(bastion::items::hoglin_stables::DIAMOND_PICKAXE, 1),
                ],
            );
        }
    }

    #[test]
    fn test_bastion_other_loot_table() {
        let lt = bastion_other_chest_loot_table();

        {
            // World seed: 734679766044180411, coords: -179 58 148
            let loot_seed = 7527759590123788878;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::other::CHAIN, 2),
                    ItemStack::of(bastion::items::other::GILDED_BLACKSTONE, 4),
                    ItemStack::of(bastion::items::other::ANCIENT_DEBRIS, 1),
                    ItemStack::of(bastion::items::other::IRON_INGOT, 6),
                    ItemStack::of(bastion::items::other::STRING, 6),
                    ItemStack::of(bastion::items::other::CROSSBOW, 1),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: -183 68 157
            let loot_seed = 6412612709946978967;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::other::MAGMA_CREAM, 4),
                    ItemStack::of(bastion::items::other::GOLDEN_SWORD, 1),
                    ItemStack::of(bastion::items::other::IRON_INGOT, 2),
                    ItemStack::of(bastion::items::other::CHAIN, 10),
                    ItemStack::of(bastion::items::other::GILDED_BLACKSTONE, 5),
                    ItemStack::of(bastion::items::other::GOLDEN_APPLE, 1),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: -195 58 150
            let loot_seed = -2252231701536112854;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::other::STRING, 5),
                    ItemStack::of(bastion::items::other::ANCIENT_DEBRIS, 1),
                    ItemStack::of(bastion::items::other::IRON_INGOT, 7),
                    ItemStack::of(bastion::items::other::COOKED_PORKCHOP, 1),
                    ItemStack::of(bastion::items::other::CHAIN, 9),
                    ItemStack::of(bastion::items::other::GILDED_BLACKSTONE, 1),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: -163 58 150
            let loot_seed = -3993577815288360752;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::other::ARROW, 6),
                    ItemStack::of(bastion::items::other::GOLD_NUGGET, 4),
                    ItemStack::of(bastion::items::other::IRON_SWORD, 1),
                    ItemStack::of(bastion::items::other::GOLDEN_BOOTS, 1),
                    ItemStack::of(bastion::items::other::STRING, 5),
                    ItemStack::of(bastion::items::other::PIGLIN_BANNER_PATTERN, 1),
                ],
            );
        }
    }

    #[test]
    fn test_bastion_treasure_room_loot_table() {
        let lt = bastion_treasure_room_chest_loot_table();

        {
            // World seed: 734679766044180411, coords: -724 36 -94
            let loot_seed = 6602806808929530262;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::treasure_room::GILDED_BLACKSTONE, 7),
                    ItemStack::of(bastion::items::treasure_room::IRON_BLOCK, 4),
                    ItemStack::of(bastion::items::treasure_room::SPECTRAL_ARROW, 22),
                    ItemStack::of(bastion::items::treasure_room::DIAMOND_CHESTPLATE, 1),
                    ItemStack::of(bastion::items::treasure_room::ANCIENT_DEBRIS, 1),
                    ItemStack::of(bastion::items::treasure_room::DIAMOND_LEGGINGS, 1),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: -674 36 -820
            let loot_seed = 4548431199292841666;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::treasure_room::QUARTZ, 12),
                    ItemStack::of(bastion::items::treasure_room::IRON_BLOCK, 3),
                    ItemStack::of(bastion::items::treasure_room::CRYING_OBSIDIAN, 5),
                    ItemStack::of(bastion::items::treasure_room::DIAMOND_BOOTS, 1),
                    ItemStack::of(bastion::items::treasure_room::DIAMOND_LEGGINGS, 2),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: -2564 36 -1022
            let loot_seed = -5772293875455727490;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::treasure_room::DIAMOND_CHESTPLATE, 1),
                    ItemStack::of(bastion::items::treasure_room::QUARTZ, 38),
                    ItemStack::of(bastion::items::treasure_room::CRYING_OBSIDIAN, 5),
                    ItemStack::of(bastion::items::treasure_room::GOLD_BLOCK, 5),
                    ItemStack::of(bastion::items::treasure_room::NETHERITE_SCRAP, 1),
                    ItemStack::of(bastion::items::treasure_room::ANCIENT_DEBRIS, 1),
                ],
            );
        }

        {
            // World seed: 734679766044180411, coords: 2229 36 -1107
            let loot_seed = -2953714471323038189;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::treasure_room::GOLD_BLOCK, 4),
                    ItemStack::of(bastion::items::treasure_room::CRYING_OBSIDIAN, 5),
                    ItemStack::of(bastion::items::treasure_room::IRON_BLOCK, 3),
                    ItemStack::of(bastion::items::treasure_room::NETHERITE_INGOT, 1),
                    ItemStack::of(bastion::items::treasure_room::DIAMOND_SWORD, 2),
                ],
            );
        }
    }

    #[test]
    fn test_bastion_bridges_loot_table() {
        let lt = bastion_bridges_chest_loot_table();

        {
            // World seed: 734679766044180411, coords: 583 80 956
            let loot_seed = -1212149287281495045;
            let loot = lt.generate_unverified_stacked_loot(&mut JavaRandom::new(loot_seed), 0.0);
            check_loot(
                loot_seed,
                loot,
                vec![
                    ItemStack::of(bastion::items::bridges::ARROW, 26),
                    ItemStack::of(bastion::items::bridges::LEATHER, 2),
                    ItemStack::of(bastion::items::bridges::SPECTRAL_ARROW, 14),
                    ItemStack::of(bastion::items::bridges::LODESTONE, 1),
                ],
            );
        }
    }
}
