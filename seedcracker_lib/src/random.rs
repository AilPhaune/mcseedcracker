use crate::{lcg::JAVA_RANDOM, math::Math};

#[derive(Default, Debug, Clone)]
pub struct JavaRandom {
    seed: i64,
}

/// An independent implementation of java.util.Random in Rust, thoroughly tested, that returns identical results as java.util.Random
impl JavaRandom {
    #[inline(always)]
    pub const fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ JAVA_RANDOM.get_multiplier()) & (JAVA_RANDOM.get_modulus() - 1),
        }
    }

    #[inline(always)]
    pub const fn set_seed_raw(&mut self, seed: i64) {
        self.seed = seed;
    }

    #[inline(always)]
    pub const fn next_seed(&mut self) -> i64 {
        self.seed = JAVA_RANDOM.next_seed(self.seed);
        self.seed
    }

    #[inline(always)]
    pub const fn get_seed(&self) -> i64 {
        self.seed
    }

    #[inline(always)]
    pub const fn next(&mut self, bit_count: i64) -> i32 {
        let s = self.next_seed();
        (s >> (48 - bit_count)) as i32
    }

    #[inline(always)]
    pub const fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    #[inline(always)]
    pub const fn next_bounded_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            0
        } else if Math::is_pow_2(bound) {
            ((bound as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32
        } else {
            let b = bound as i64;
            // Values in the range [0, max_try) when taken mod b will be uniformely distributed, as max_try is a constructed to be a multiple of b
            let max_try = (Math::pow_2(31) - (Math::pow_2(31) % b)) as i32;
            loop {
                let try_value = self.next(31); // 31 bits so we don't get negative values
                if try_value < max_try {
                    return try_value % bound;
                }
                // If we get here, we need to try another value
            }
        }
    }

    #[inline(always)]
    pub const fn next_long(&mut self) -> i64 {
        let hi = self.next_int() as i64;
        let lo = self.next_int() as i64;
        (hi << 32).wrapping_add(lo)
    }

    #[inline(always)]
    pub const fn next_bool(&mut self) -> bool {
        self.next(1) != 0
    }

    #[inline(always)]
    pub const fn next_float(&mut self) -> f32 {
        self.next(f32::MANTISSA_DIGITS as i64) as f32
            / (Math::pow_2(f32::MANTISSA_DIGITS as i32) as f32)
    }
}

#[inline]
pub const fn shuffle<T>(array: &mut [T], random: &mut JavaRandom) {
    let mut i = array.len();
    while i > 1 {
        let swap_i = random.next_bounded_int(i as i32);
        array.swap(i - 1, swap_i as usize);
        i -= 1;
    }
}

#[inline(always)]
pub const fn random_with_terrain_seed(chunk_x: i32, chunk_z: i32) -> (JavaRandom, i64) {
    let seed = (chunk_x as i64)
        .wrapping_mul(341873128712i64)
        .wrapping_add((chunk_z as i64).wrapping_mul(132897987541i64));

    (JavaRandom::new(seed), seed)
}

#[inline(always)]
pub const fn random_with_population_seed(
    world_seed: i64,
    block_x: i32,
    block_z: i32,
) -> (JavaRandom, i64) {
    let mut rng = JavaRandom::new(world_seed);
    let xmul = rng.next_long() | 1;
    let zmul = rng.next_long() | 1;
    let seed = (block_x as i64)
        .wrapping_mul(xmul)
        .wrapping_add((block_z as i64).wrapping_mul(zmul))
        ^ world_seed;

    (JavaRandom::new(seed), seed)
}

#[inline(always)]
pub const fn random_with_decorator_seed(
    population_seed: i64,
    index: i32,
    step: i32,
) -> (JavaRandom, i64) {
    let seed = population_seed
        .wrapping_add(index as i64)
        .wrapping_add(1000i64.wrapping_mul(step as i64));

    (JavaRandom::new(seed), seed)
}

#[inline(always)]
pub const fn random_with_carver_seed(
    world_seed: i64,
    chunk_x: i32,
    chunk_z: i32,
) -> (JavaRandom, i64) {
    let mut rng = JavaRandom::new(world_seed);
    let xmul = rng.next_long();
    let zmul = rng.next_long();
    let seed = (chunk_x as i64)
        .wrapping_mul(xmul)
        .wrapping_add((chunk_z as i64).wrapping_mul(zmul))
        ^ world_seed;

    (JavaRandom::new(seed), seed)
}

#[inline(always)]
pub const fn random_with_region_seed(
    world_seed: i64,
    reg_x: i32,
    reg_z: i32,
    salt: i32,
) -> (JavaRandom, i64) {
    let seed = (reg_x as i64)
        .wrapping_mul(341873128712i64)
        .wrapping_add((reg_z as i64).wrapping_mul(132897987541i64))
        .wrapping_add(world_seed)
        .wrapping_add(salt as i64);

    (JavaRandom::new(seed), seed)
}
