#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ToolMaterial {
    Wood,
    Gold,
    Stone,
    Iron,
    Diamond,
    Netherite,
}

impl ToolMaterial {
    pub const fn tool_durability(self) -> i32 {
        match self {
            Self::Wood => 59,
            Self::Gold => 32,
            Self::Stone => 131,
            Self::Iron => 250,
            Self::Diamond => 1561,
            Self::Netherite => 2031,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArmorMaterial {
    Leather,
    Gold,
    Chainmail,
    Iron,
    Diamond,
    Netherite,
}

impl ArmorMaterial {
    pub const fn level(self) -> i32 {
        match self {
            Self::Leather => 5,
            Self::Gold => 7,
            Self::Chainmail | Self::Iron => 15,
            Self::Diamond => 33,
            Self::Netherite => 37,
        }
    }

    pub const fn helmet_durability(self) -> i32 {
        11 * self.level()
    }

    pub const fn chestplate_durability(self) -> i32 {
        16 * self.level()
    }

    pub const fn leggings_durability(self) -> i32 {
        15 * self.level()
    }

    pub const fn boots_durability(self) -> i32 {
        13 * self.level()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItemWithDurability {
    Sword(ToolMaterial),
    Pickaxe(ToolMaterial),
    Axe(ToolMaterial),
    Shovel(ToolMaterial),
    Hoe(ToolMaterial),
    Helmet(ArmorMaterial),
    Chestplate(ArmorMaterial),
    Leggings(ArmorMaterial),
    Boots(ArmorMaterial),
    Shears,
    FishingRod,
    CarrotOnAStick,
    WarpedFungusOnAStick,
    FlintAndSteel,
    Bow,
    Crossbow,
    Trident,
    Elytra,
    Shield,
}

impl ItemWithDurability {
    pub const fn durability(self) -> i32 {
        match self {
            Self::Sword(m) | Self::Pickaxe(m) | Self::Axe(m) | Self::Shovel(m) | Self::Hoe(m) => {
                m.tool_durability()
            }
            Self::Helmet(m) => m.helmet_durability(),
            Self::Chestplate(m) => m.chestplate_durability(),
            Self::Leggings(m) => m.leggings_durability(),
            Self::Boots(m) => m.boots_durability(),
            Self::FlintAndSteel | Self::FishingRod => 64,
            Self::CarrotOnAStick => 25,
            Self::WarpedFungusOnAStick => 100,
            Self::Shears => 238,
            Self::Shield => 336,
            Self::Bow => 384,
            Self::Trident => 250,
            Self::Elytra => 432,
            Self::Crossbow => 465,
        }
    }
}
