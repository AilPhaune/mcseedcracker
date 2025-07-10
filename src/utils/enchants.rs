pub mod shovel {
    pub const EFFICIENCY: (i32, i32, i32) = (1, 1, 5);
    pub const SILK_TOUCH: (i32, i32, i32) = (2, 1, 1);
    pub const UNBREAKING: (i32, i32, i32) = (3, 1, 3);
    pub const FORTUNE: (i32, i32, i32) = (4, 1, 3);
    pub const MENDING: (i32, i32, i32) = (5, 1, 1);
    pub const VANISHING_CURSE: (i32, i32, i32) = (6, 1, 1);
}

pub const SHOVEL: [(i32, i32, i32); 6] = [
    shovel::EFFICIENCY,
    shovel::SILK_TOUCH,
    shovel::UNBREAKING,
    shovel::FORTUNE,
    shovel::MENDING,
    shovel::VANISHING_CURSE,
];

pub mod pickaxe {
    pub const EFFICIENCY: (i32, i32, i32) = (1, 1, 5);
    pub const SILK_TOUCH: (i32, i32, i32) = (2, 1, 1);
    pub const UNBREAKING: (i32, i32, i32) = (3, 1, 3);
    pub const FORTUNE: (i32, i32, i32) = (4, 1, 3);
    pub const MENDING: (i32, i32, i32) = (5, 1, 1);
    pub const VANISHING_CURSE: (i32, i32, i32) = (6, 1, 1);
}

pub const PICKAXE: [(i32, i32, i32); 6] = [
    pickaxe::EFFICIENCY,
    pickaxe::SILK_TOUCH,
    pickaxe::UNBREAKING,
    pickaxe::FORTUNE,
    pickaxe::MENDING,
    pickaxe::VANISHING_CURSE,
];

pub mod axe {
    pub const EFFICIENCY: (i32, i32, i32) = (1, 1, 5);
    pub const SILK_TOUCH: (i32, i32, i32) = (2, 1, 1);
    pub const UNBREAKING: (i32, i32, i32) = (3, 1, 3);
    pub const FORTUNE: (i32, i32, i32) = (4, 1, 3);
    pub const MENDING: (i32, i32, i32) = (5, 1, 1);
    pub const VANISHING_CURSE: (i32, i32, i32) = (6, 1, 1);

    pub const SHARPNESS: (i32, i32, i32) = (7, 1, 5);
    pub const SMITE: (i32, i32, i32) = (8, 1, 5);
    pub const BANE_OF_ARTHROPODS: (i32, i32, i32) = (9, 1, 5);
}

pub const AXE: [(i32, i32, i32); 9] = [
    axe::SHARPNESS,
    axe::SMITE,
    axe::BANE_OF_ARTHROPODS,
    axe::EFFICIENCY,
    axe::SILK_TOUCH,
    axe::UNBREAKING,
    axe::FORTUNE,
    axe::MENDING,
    axe::VANISHING_CURSE,
];

pub mod crossbow {
    pub const UNBREAKING: (i32, i32, i32) = (1, 1, 3);
    pub const MULTISHOT: (i32, i32, i32) = (2, 1, 1);
    pub const QUICKCHARGE: (i32, i32, i32) = (3, 1, 3);
    pub const PIERCING: (i32, i32, i32) = (4, 1, 4);
    pub const MENDING: (i32, i32, i32) = (5, 1, 1);
    pub const VANISHING_CURSE: (i32, i32, i32) = (6, 1, 1);
}

pub const CROSSBOW: [(i32, i32, i32); 6] = [
    crossbow::UNBREAKING,
    crossbow::MULTISHOT,
    crossbow::QUICKCHARGE,
    crossbow::PIERCING,
    crossbow::MENDING,
    crossbow::VANISHING_CURSE,
];

pub mod sword {
    pub const SHARPNESS: (i32, i32, i32) = (1, 1, 5);
    pub const SMITE: (i32, i32, i32) = (2, 1, 5);
    pub const BANE_OF_ARTHROPODS: (i32, i32, i32) = (3, 1, 5);
    pub const KNOCKBACK: (i32, i32, i32) = (4, 1, 2);
    pub const FIRE_ASPECT: (i32, i32, i32) = (5, 1, 2);
    pub const LOOTING: (i32, i32, i32) = (6, 1, 3);
    pub const SWEEPING_EDGE: (i32, i32, i32) = (7, 1, 3);
    pub const UNBREAKING: (i32, i32, i32) = (8, 1, 3);
    pub const MENDING: (i32, i32, i32) = (9, 1, 1);
    pub const VANISHING_CURSE: (i32, i32, i32) = (10, 1, 1);
}

pub const SWORD: [(i32, i32, i32); 10] = [
    sword::SHARPNESS,
    sword::SMITE,
    sword::BANE_OF_ARTHROPODS,
    sword::KNOCKBACK,
    sword::FIRE_ASPECT,
    sword::LOOTING,
    sword::SWEEPING_EDGE,
    sword::UNBREAKING,
    sword::MENDING,
    sword::VANISHING_CURSE,
];

pub mod armor {
    pub const PROTECTION: (i32, i32, i32) = (1, 1, 4);
    pub const FIRE_PROTECTION: (i32, i32, i32) = (2, 1, 4);
    pub const BLAST_PROTECTION: (i32, i32, i32) = (3, 1, 4);
    pub const PROJECTILE_PROTECTION: (i32, i32, i32) = (4, 1, 4);
    pub const RESPIRATION: (i32, i32, i32) = (5, 1, 3);
    pub const AQUA_AFFINITY: (i32, i32, i32) = (6, 1, 1);
    pub const THORNS: (i32, i32, i32) = (7, 1, 3);
    pub const BINDING_CURSE: (i32, i32, i32) = (8, 1, 1);
    pub const UNBREAKING: (i32, i32, i32) = (9, 1, 3);
    pub const MENDING: (i32, i32, i32) = (10, 1, 1);
    pub const VANISHING_CURSE: (i32, i32, i32) = (11, 1, 1);
    pub const FEATHER_FALLING: (i32, i32, i32) = (12, 1, 4);
    pub const DEPTH_STRIDER: (i32, i32, i32) = (13, 1, 3);
    pub const FROST_WALKER: (i32, i32, i32) = (14, 1, 2);
}

pub const HELMET: [(i32, i32, i32); 11] = [
    armor::PROTECTION,
    armor::FIRE_PROTECTION,
    armor::BLAST_PROTECTION,
    armor::PROJECTILE_PROTECTION,
    armor::RESPIRATION,
    armor::AQUA_AFFINITY,
    armor::THORNS,
    armor::BINDING_CURSE,
    armor::UNBREAKING,
    armor::MENDING,
    armor::VANISHING_CURSE,
];

pub const CHESTPLATE: [(i32, i32, i32); 9] = [
    armor::PROTECTION,
    armor::FIRE_PROTECTION,
    armor::BLAST_PROTECTION,
    armor::PROJECTILE_PROTECTION,
    armor::THORNS,
    armor::BINDING_CURSE,
    armor::UNBREAKING,
    armor::MENDING,
    armor::VANISHING_CURSE,
];

pub const LEGGINGS: [(i32, i32, i32); 9] = [
    armor::PROTECTION,
    armor::FIRE_PROTECTION,
    armor::BLAST_PROTECTION,
    armor::PROJECTILE_PROTECTION,
    armor::THORNS,
    armor::BINDING_CURSE,
    armor::UNBREAKING,
    armor::MENDING,
    armor::VANISHING_CURSE,
];

pub const BOOTS: [(i32, i32, i32); 12] = [
    armor::PROTECTION,
    armor::FIRE_PROTECTION,
    armor::FEATHER_FALLING,
    armor::BLAST_PROTECTION,
    armor::PROJECTILE_PROTECTION,
    armor::THORNS,
    armor::DEPTH_STRIDER,
    armor::FROST_WALKER,
    armor::BINDING_CURSE,
    armor::UNBREAKING,
    armor::MENDING,
    armor::VANISHING_CURSE,
];
