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
