use std::{cmp::Ordering, f64::consts::PI};

use crate::random::{JavaRandom, shuffle};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct EndPillars(pub [EndPillar; 10]);

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct EndPillar {
    index: i32,
    height: i32,
    x: i32,
    z: i32,
    radius: i32,
    caged: bool,
}

impl EndPillars {
    pub const fn pillar_seed(world_seed: i64) -> i64 {
        JavaRandom::new(world_seed).next_long() & 0xFFFF
    }

    pub const fn new() -> Self {
        Self(
            [EndPillar {
                index: 0,
                height: 0,
                x: 0,
                z: 0,
                radius: 0,
                caged: false,
            }; 10],
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = &EndPillar> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut EndPillar> {
        self.0.iter_mut()
    }

    pub fn from_seed(&mut self, pillar_seed: i64) {
        let mut rng = JavaRandom::new(pillar_seed);

        let mut indices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        shuffle(&mut indices, &mut rng);

        for (i, pillar) in self.iter_mut().enumerate() {
            let index = indices[i];

            pillar.index = index;
            pillar.height = 76 + 3 * index;
            pillar.x = ((2.0 * (-PI + (PI / 10.0) * (i as f64))).cos() * 42.0).round() as i32;
            pillar.z = ((2.0 * (-PI + (PI / 10.0) * (i as f64))).sin() * 42.0).round() as i32;
            pillar.radius = 2 + index / 3;
            pillar.caged = index == 1 || index == 2;
        }
    }
}

/// Pillars height hints. There are 10 pillars, each having a unique height, from the following list: 76, 79, 82, 85, 88, 91, 94, 97, 100, 103. <br>
/// Note: The two caged pillars are at height 79 and 82.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum PillarHeightHint {
    /// A range of possible heights
    Range(i32, i32),
    /// An exact height
    Exact(i32),
    /// Not sure, but visually looks big.
    Big,
    /// Not sure, but visually looks medium
    Medium,
    /// Not sure, but visually looks small
    Small,
    /// Not sure, but visually looks in between of medium and big
    MediumBig,
    /// Not sure, but visually looks in between of medium and small
    MediumSmall,
    /// Not sure
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct PartialEndPillar {
    pub caged: Option<bool>,
    pub height: PillarHeightHint,
}

#[derive(Debug, Clone, Copy)]
pub enum PillarMatchResult {
    ImpossibleMatch,
    ExactMatch,
    // Wheight corresponds to a probability of a match
    PossibleMatch(f64),
}

impl PillarMatchResult {
    pub const fn combine(self, other: Self) -> Self {
        match (self, other) {
            (PillarMatchResult::ImpossibleMatch, _) | (_, PillarMatchResult::ImpossibleMatch) => {
                PillarMatchResult::ImpossibleMatch
            }
            (PillarMatchResult::ExactMatch, v) | (v, PillarMatchResult::ExactMatch) => v,
            (PillarMatchResult::PossibleMatch(w1), PillarMatchResult::PossibleMatch(w2)) => {
                PillarMatchResult::PossibleMatch(w1 * w2)
            }
        }
    }

    pub const fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PillarMatchResult::ImpossibleMatch, PillarMatchResult::ImpossibleMatch)
            | (PillarMatchResult::ExactMatch, PillarMatchResult::ExactMatch) => Ordering::Equal,

            (PillarMatchResult::ImpossibleMatch, _) => Ordering::Less,
            (_, PillarMatchResult::ImpossibleMatch) => Ordering::Greater,

            (PillarMatchResult::ExactMatch, _) => Ordering::Greater,
            (_, PillarMatchResult::ExactMatch) => Ordering::Less,
            (PillarMatchResult::PossibleMatch(w1), PillarMatchResult::PossibleMatch(w2)) => {
                if *w1 < *w2 {
                    Ordering::Less
                } else if *w1 == *w2 {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            }
        }
    }

    pub fn is_exact_match(&self) -> bool {
        matches!(self, PillarMatchResult::ExactMatch)
    }

    pub fn is_impossible_match(&self) -> bool {
        matches!(self, PillarMatchResult::ImpossibleMatch)
    }

    pub fn is_possible_match(&self) -> bool {
        matches!(self, PillarMatchResult::PossibleMatch(_))
    }
}

impl PartialEndPillar {
    pub const fn new(caged: Option<bool>, height: PillarHeightHint) -> Self {
        Self { caged, height }
    }

    pub const fn matches(&self, pillar: &EndPillar) -> PillarMatchResult {
        let cage_match = match self.caged {
            Some(caged) => {
                if caged == pillar.caged {
                    PillarMatchResult::ExactMatch
                } else {
                    return PillarMatchResult::ImpossibleMatch;
                }
            }
            None => PillarMatchResult::PossibleMatch(1.0),
        };

        let height_match = match self.height {
            PillarHeightHint::Unknown => PillarMatchResult::PossibleMatch(1.0),
            PillarHeightHint::Exact(h) => {
                if h == pillar.height {
                    PillarMatchResult::ExactMatch
                } else {
                    return PillarMatchResult::ImpossibleMatch;
                }
            }
            PillarHeightHint::Range(min, max) => {
                if pillar.height >= min && pillar.height <= max {
                    PillarMatchResult::ExactMatch
                } else {
                    return PillarMatchResult::ImpossibleMatch;
                }
            }
            PillarHeightHint::Big => {
                // prefer pillars closest to 103
                // only matches the three tallest pillars: [97, 100, 103]
                if pillar.height < 97 {
                    return PillarMatchResult::ImpossibleMatch;
                } else {
                    let dist_from_min = pillar.height - 97; // no abs needed
                    let prob = (dist_from_min as f64) / (103.0 - 97.0);
                    PillarMatchResult::PossibleMatch((prob + 1.0) / 2.0) // [0.5-1]
                }
            }
            PillarHeightHint::Medium => {
                // prefer pillars closest to 89.5
                // only matches the four middle pillars: [85, 88, 91, 94]
                if pillar.height < 85 || pillar.height > 94 {
                    return PillarMatchResult::ImpossibleMatch;
                } else {
                    let dist_from_middle = (pillar.height as f64 - 89.5).abs();
                    let prob = 1.0 - dist_from_middle / (94.0 - 85.0);
                    PillarMatchResult::PossibleMatch((prob + 1.0) / 2.0) // [0.5-1]
                }
            }
            PillarHeightHint::Small => {
                // prefer pillars closest to 76
                // only matches the three smallest pillars: [76, 79, 82]
                if pillar.height > 82 {
                    return PillarMatchResult::ImpossibleMatch;
                } else {
                    let dist_from_max = 82.0 - pillar.height as f64; // no abs needed
                    let prob = dist_from_max / (82.0 - 76.0);
                    PillarMatchResult::PossibleMatch((prob + 1.0) / 2.0) // [0.5-1]
                }
            }
            PillarHeightHint::MediumBig => {
                // matches pillars that are matched by Medium or Big
                // prefer pillars closest to the middle of the range [88, 91, 94, 97, 100, 103]
                if pillar.height < 88 {
                    return PillarMatchResult::ImpossibleMatch;
                } else {
                    let dist_from_middle = (pillar.height as f64 - 95.5).abs();
                    PillarMatchResult::PossibleMatch(1.0 - dist_from_middle / (103.0 - 88.0)) // [0.5-1]
                }
            }
            PillarHeightHint::MediumSmall => {
                // matches pillars that are matched by Medium or Small
                // prefer pillars closest to the middle of the range [76, 79, 82, 85, 88, 91]
                if pillar.height > 97 {
                    return PillarMatchResult::ImpossibleMatch;
                } else {
                    let dist_from_middle = (pillar.height as f64 - 83.5).abs();
                    PillarMatchResult::PossibleMatch(1.0 - dist_from_middle / (91.0 - 76.0)) // [0.5-1]
                }
            }
        };

        cage_match.combine(height_match)
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct PartialEndPillars(pub [PartialEndPillar; 10]);

impl PartialEndPillars {
    pub const fn new() -> Self {
        Self(
            [PartialEndPillar {
                caged: None,
                height: PillarHeightHint::Unknown,
            }; 10],
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = &PartialEndPillar> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PartialEndPillar> {
        self.0.iter_mut()
    }

    pub fn matches(&self, pillars: &EndPillars) -> PillarMatchResult {
        let mut result = PillarMatchResult::ExactMatch;
        for (pillar, partial_pillar) in pillars.iter().zip(self.iter()) {
            match partial_pillar.matches(pillar) {
                PillarMatchResult::ImpossibleMatch => return PillarMatchResult::ImpossibleMatch,
                w => result = result.combine(w),
            }
        }
        result
    }

    pub fn seed_results(&self) -> Vec<(i64, PillarMatchResult)> {
        let mut pillars = EndPillars::new();
        let mut results = Vec::new();
        for pillar_seed in 0..65536 {
            pillars.from_seed(pillar_seed);
            results.push((pillar_seed, self.matches(&pillars)));
        }
        results
    }
}
