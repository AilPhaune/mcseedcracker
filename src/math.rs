use crate::random::JavaRandom;

pub struct Math;

impl Math {
    #[inline(always)]
    pub const fn is_pow_2(n: i32) -> bool {
        (n & (n - 1)) == 0
    }

    #[inline(always)]
    pub const fn pow_2(n: i32) -> i64 {
        1i64 << n
    }

    #[inline(always)]
    pub const fn get_mask(n: i32) -> i64 {
        Self::pow_2(n) - 1
    }

    #[inline(always)]
    pub const fn mask(x: i64, n: i32) -> i64 {
        x & Self::get_mask(n)
    }

    #[inline(always)]
    pub const fn mod_inverse_pow_2(value: i64, n: i32) -> i64 {
        let mut x = (((value.wrapping_shl(1) ^ value) & 4) << 1) ^ value;
        x = x.wrapping_add(x.wrapping_sub(value.wrapping_mul(x).wrapping_mul(x)));
        x = x.wrapping_add(x.wrapping_sub(value.wrapping_mul(x).wrapping_mul(x)));
        x = x.wrapping_add(x.wrapping_sub(value.wrapping_mul(x).wrapping_mul(x)));
        x = x.wrapping_add(x.wrapping_sub(value.wrapping_mul(x).wrapping_mul(x)));
        Self::mask(x, n)
    }

    #[inline(always)]
    pub const fn block_coords_to_chunk_coords(coords: (i32, i32)) -> (i32, i32) {
        (coords.0 >> 4, coords.1 >> 4)
    }

    #[inline(always)]
    pub const fn chunk_coords_to_region_coords(coords: (i32, i32)) -> (i32, i32) {
        (coords.0 >> 5, coords.1 >> 5)
    }

    #[inline(always)]
    pub const fn block_coords_to_region_coords(coords: (i32, i32)) -> (i32, i32) {
        Self::chunk_coords_to_region_coords(Self::block_coords_to_chunk_coords(coords))
    }

    #[inline(always)]
    pub const fn relative_chunk_coords(
        chunk_coords: (i32, i32),
        block_coords: (i32, i32),
    ) -> (i32, i32) {
        (
            (chunk_coords.0 * 16) + block_coords.0,
            (chunk_coords.1 * 16) + block_coords.1,
        )
    }

    #[inline(always)]
    pub const fn region_coords_to_lower_chunk_coords(coords: (i32, i32)) -> (i32, i32) {
        (coords.0 << 5, coords.1 << 5)
    }

    #[inline(always)]
    /// Returns a random number between min and max (inclusive)
    pub const fn next_int(rng: &mut JavaRandom, min: i32, max: i32) -> i32 {
        if min >= max {
            min
        } else {
            rng.next_bounded_int(max - min + 1) + min
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Math;

    #[test]
    fn test_mod_inverse_pow_2() {
        let tests: [(i64, i32); 6] = [
            (17, 51),
            (1918615165151321, 54),
            (13185, 22),
            (17799111, 38),
            (131, 8),
            (18919, 17),
        ];

        for (value, n) in tests {
            let inv = Math::mod_inverse_pow_2(value, n);
            let prod = value.wrapping_mul(inv);
            let final_res = Math::mask(prod, n);

            // x * x^-1 = 1
            assert_eq!(final_res, 1);
        }
    }
}
