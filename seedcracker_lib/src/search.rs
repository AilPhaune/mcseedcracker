use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
};

pub use cubiomes::enums::BiomeID;
use cubiomes::{
    enums::{Dimension, MCVersion},
    generator::{Generator, GeneratorFlags},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    features::buried_treasure,
    lcg,
    loot_table::{FastInventoryCompareContext, SingleChest},
    utils::{likely, unlikely},
};

#[derive(Debug, Clone)]
pub enum StructureData {
    BuriedTreasureContents {
        chunk_x: i32,
        chunk_z: i32,
        luck: f32,
        contents: FastInventoryCompareContext<SingleChest, 12>,
    },
}

impl StructureData {
    #[inline(always)]
    fn check_seed(&self, seed: i64) -> bool {
        match self {
            StructureData::BuriedTreasureContents {
                contents,
                chunk_x,
                chunk_z,
                luck,
            } => {
                unlikely(buried_treasure::generates_at(seed, (*chunk_x, *chunk_z)))
                    && unlikely(buried_treasure::compare_buried_treasure_fast_noinv(
                        seed,
                        (*chunk_x, *chunk_z),
                        *luck,
                        contents,
                    ))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Searching,
    /// It found some seeds (maybe none)
    Complete {
        /// The seeds that were found
        seeds: Vec<i64>,
    },
    /// It found more seeds than the maximum number of seeds allowed
    TooManySeeds {
        /// Some of seeds that were found
        seeds_incomplete: Vec<i64>,
    },
    /// The search was cancelled
    Cancelled {
        /// Some of seeds that were found
        seeds_incomplete: Vec<i64>,
    },
}

pub struct StructureSeedSearchData {
    pub pillar_seed: i64,
    pub data: Vec<StructureData>,
    pub max_results: u16,
}

impl StructureSeedSearchData {
    #[inline]
    pub const fn new(pillar_seed: i64, data: Vec<StructureData>, max_results: u16) -> Self {
        Self {
            pillar_seed,
            data,
            max_results,
        }
    }

    #[inline]
    pub fn spawn_multithreaded(self) -> StructureSeedSearcherHandle {
        StructureSeedSearcher::spawn_multithreaded(self.pillar_seed, self.data, self.max_results)
    }
}

pub struct StructureSeedSearcher {
    pillar_seed: i64,
    data: Vec<StructureData>,
    max_results: usize,

    /// The number of seeds out of 2^32 that have been searched
    progress: AtomicU64,
    stopsig: AtomicBool,
    isdone: AtomicBool,
    status: Mutex<Status>,
}

impl StructureSeedSearcher {
    #[inline]
    fn compute(&self) -> Vec<i64> {
        let ack = AtomicBool::new(false);
        let results = (0i64..65536i64)
            .into_par_iter()
            .filter_map(|state_hi| {
                if self.stopsig.load(Ordering::Relaxed) {
                    ack.store(true, Ordering::Relaxed);
                    return None;
                }

                let data_clone = self.data.clone();
                let pillar_seed_shl_16 = self.pillar_seed << 16;

                let orig = state_hi;
                let state_hi = state_hi << 32;
                Some(
                    (0i64..65536i64)
                        .into_par_iter()
                        .filter_map(move |state_lo| {
                            if unlikely(state_lo == 65535 && (orig % 32) == 0) {
                                self.progress.fetch_add(65536 * 32, Ordering::Relaxed);
                            }

                            let state = state_hi | pillar_seed_shl_16 | state_lo;
                            let reversed_state = lcg::JAVA_RANDOM_REV2.next_seed(state);
                            let seed = reversed_state ^ lcg::JAVA_RANDOM.get_multiplier();

                            for d in data_clone.iter() {
                                if likely(!d.check_seed(seed)) {
                                    return None;
                                }
                            }

                            Some(seed)
                        }),
                )
            })
            .flatten()
            .take_any(self.max_results + 1)
            .collect::<Vec<i64>>();

        let res = match self.status.lock() {
            Ok(mut status) => {
                if ack.load(Ordering::Relaxed) {
                    *status = Status::Cancelled {
                        seeds_incomplete: results.clone(),
                    };
                } else if results.len() > self.max_results {
                    *status = Status::TooManySeeds {
                        seeds_incomplete: results.clone(),
                    };
                } else {
                    *status = Status::Complete {
                        seeds: results.clone(),
                    };
                }
                results
            }
            Err(_) => results,
        };

        self.isdone.store(true, Ordering::Relaxed);

        res
    }

    pub fn spawn_multithreaded(
        pillar_seed: i64,
        data: Vec<StructureData>,
        max_results: u16,
    ) -> StructureSeedSearcherHandle {
        let job = Arc::new(StructureSeedSearcher {
            pillar_seed,
            data,
            max_results: max_results as usize,
            progress: AtomicU64::new(0),
            status: Mutex::new(Status::Searching),
            stopsig: AtomicBool::new(false),
            isdone: AtomicBool::new(false),
        });
        let job2 = Arc::clone(&job);

        let join_handle = std::thread::spawn(move || job.compute());

        StructureSeedSearcherHandle {
            join_handle,
            searcher: job2,
        }
    }
}

pub struct StructureSeedSearcherHandle {
    pub join_handle: JoinHandle<Vec<i64>>,
    pub searcher: Arc<StructureSeedSearcher>,
}

impl StructureSeedSearcherHandle {
    #[inline]
    pub fn join(self) -> thread::Result<Vec<i64>> {
        self.join_handle.join()
    }

    #[inline]
    pub fn cancel_join(self) -> thread::Result<Vec<i64>> {
        self.searcher.stopsig.store(true, Ordering::Relaxed);
        self.join()
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.searcher.isdone.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn get_status(&self) -> Status {
        let lock = self.searcher.status.lock().unwrap();
        (*lock).clone()
    }

    #[inline]
    pub fn get_pillar_seed(&self) -> i64 {
        self.searcher.pillar_seed
    }

    #[inline]
    pub fn get_progress(&self) -> u64 {
        self.searcher.progress.load(Ordering::Relaxed)
    }
}

pub enum WorldExtraData {
    OverworldBiomeData(Vec<(i32, i32, i32, BiomeID)>),
    NetherBiomeData(Vec<(i32, i32, i32, BiomeID)>),
}

impl WorldExtraData {
    #[inline(always)]
    fn check_seed(&self, seed: i64) -> bool {
        match self {
            WorldExtraData::OverworldBiomeData(data) => {
                let generator = Generator::new(
                    MCVersion::MC_1_16_5,
                    seed,
                    Dimension::DIM_OVERWORLD,
                    GeneratorFlags::empty(),
                );
                for (x, y, z, biome) in data.iter() {
                    if generator.get_biome_at(*x, *y, *z) != Ok(*biome) {
                        return false;
                    }
                }
                true
            }
            WorldExtraData::NetherBiomeData(data) => {
                let generator = Generator::new(
                    MCVersion::MC_1_16_5,
                    seed,
                    Dimension::DIM_NETHER,
                    GeneratorFlags::empty(),
                );
                for (x, y, z, biome) in data.iter() {
                    if generator.get_biome_at(*x, *y, *z) != Ok(*biome) {
                        return false;
                    }
                }
                true
            }
        }
    }

    pub fn as_overworld(&self) -> Option<&[(i32, i32, i32, BiomeID)]> {
        match self {
            WorldExtraData::OverworldBiomeData(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_overworld_mut(&mut self) -> Option<&mut Vec<(i32, i32, i32, BiomeID)>> {
        match self {
            WorldExtraData::OverworldBiomeData(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_nether(&self) -> Option<&[(i32, i32, i32, BiomeID)]> {
        match self {
            WorldExtraData::NetherBiomeData(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_nether_mut(&mut self) -> Option<&mut Vec<(i32, i32, i32, BiomeID)>> {
        match self {
            WorldExtraData::NetherBiomeData(data) => Some(data),
            _ => None,
        }
    }
}

pub struct WorldSeedSearchData {}

pub struct WorldSeedSearcher {
    structure_seed: i64,
    data: Vec<WorldExtraData>,
    max_results: usize,

    progress: AtomicU64,
    isdone: AtomicBool,
    status: Mutex<Status>,
}

impl WorldSeedSearcher {
    #[inline]
    fn compute(&self) -> Vec<i64> {
        let results = (0i64..65536i64)
            .into_par_iter()
            .filter_map(|seed_hi| {
                if seed_hi % 512 == 511 {
                    self.progress.fetch_add(512, Ordering::Relaxed);
                }

                let seed = seed_hi << 48 | self.structure_seed;
                for d in self.data.iter() {
                    if unlikely(!d.check_seed(seed)) {
                        return None;
                    }
                }
                Some(seed)
            })
            .take_any(self.max_results + 1)
            .collect::<Vec<_>>();

        let res = match self.status.lock() {
            Ok(mut status) => {
                if results.len() > self.max_results {
                    *status = Status::TooManySeeds {
                        seeds_incomplete: results.clone(),
                    };
                } else {
                    *status = Status::Complete {
                        seeds: results.clone(),
                    };
                }
                results
            }
            Err(_) => results,
        };

        self.isdone.store(true, Ordering::Relaxed);

        res
    }

    pub fn spawn_multithreaded(
        structure_seed: i64,
        data: Vec<WorldExtraData>,
        max_results: u16,
    ) -> WorldSeedSearcherHandle {
        let job = Arc::new(WorldSeedSearcher {
            structure_seed,
            data,
            max_results: max_results as usize,
            progress: AtomicU64::new(0),
            status: Mutex::new(Status::Searching),
            isdone: AtomicBool::new(false),
        });
        let job2 = Arc::clone(&job);

        let join_handle = std::thread::spawn(move || job.compute());

        WorldSeedSearcherHandle {
            join_handle,
            searcher: job2,
        }
    }
}

pub struct WorldSeedSearcherHandle {
    pub join_handle: JoinHandle<Vec<i64>>,
    pub searcher: Arc<WorldSeedSearcher>,
}

impl WorldSeedSearcherHandle {
    #[inline]
    pub fn join(self) -> thread::Result<Vec<i64>> {
        self.join_handle.join()
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.searcher.isdone.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn get_status(&self) -> Status {
        let lock = self.searcher.status.lock().unwrap();
        (*lock).clone()
    }

    #[inline]
    pub fn get_structure_seed(&self) -> i64 {
        self.searcher.structure_seed
    }

    #[inline]
    pub fn get_progress(&self) -> u64 {
        self.searcher.progress.load(Ordering::Relaxed)
    }
}

pub const MC_1_16_5: MCVersion = MCVersion::MC_1_16_5;
