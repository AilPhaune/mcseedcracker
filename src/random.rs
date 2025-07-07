use crate::{lcg::JAVA_RANDOM, math::Math};

pub struct JavaRandom {
    seed: i64,
}

/// An independent implementation of java.util.Random in Rust, thoroughly tested, that returns identical results as java.util.Random
impl JavaRandom {
    pub const fn new(seed: i64) -> Self {
        Self {
            seed: (seed ^ JAVA_RANDOM.get_multiplier()) & (JAVA_RANDOM.get_modulus() - 1),
        }
    }

    pub const fn set_seed_raw(&mut self, seed: i64) {
        self.seed = seed;
    }

    pub const fn next_seed(&mut self) -> i64 {
        self.seed = JAVA_RANDOM.next_seed(self.seed);
        self.seed
    }

    pub const fn get_seed(&self) -> i64 {
        self.seed
    }

    pub const fn next(&mut self, bit_count: i64) -> i32 {
        let s = self.next_seed();
        (s >> (48 - bit_count)) as i32
    }

    pub const fn next_int(&mut self) -> i32 {
        self.next(32)
    }

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

    pub const fn next_long(&mut self) -> i64 {
        let hi = self.next_int() as i64;
        let lo = self.next_int() as i64;
        (hi << 32) + lo
    }

    pub const fn next_bool(&mut self) -> bool {
        self.next(1) != 0
    }
}

pub const fn shuffle<T>(array: &mut [T], random: &mut JavaRandom) {
    let mut i = array.len();
    while i > 1 {
        let swap_i = random.next_bounded_int(i as i32);
        array.swap(i - 1, swap_i as usize);
        i -= 1;
    }
}
