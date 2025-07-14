/*!

This file is a Rust translation of Java code from:
    https://github.com/KaptainWutax/SeedUtils/

The original Java source file can be found at:
    https://github.com/KaptainWutax/SeedUtils/blob/master/src/main/java/kaptainwutax/seedutils/lcg/LCG.java

It is licensed under the MIT license.

MIT License

Copyright (c) 2020 KaptainWutax

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

*/

#[derive(Debug, Clone)]
pub struct LinearCongruentialGenerator {
    multiplier: i64,
    increment: i64,
    modulus: i64,

    is_pow_2: bool,
    trailing_zeros: i32,
}

pub const JAVA_RANDOM: LinearCongruentialGenerator =
    LinearCongruentialGenerator::new(0x5DEECE66D, 0xB, 1i64 << 48);

pub const JAVA_RANDOM_REV1: LinearCongruentialGenerator = JAVA_RANDOM.combine(-1);
pub const JAVA_RANDOM_REV2: LinearCongruentialGenerator = JAVA_RANDOM.combine(-2);

impl LinearCongruentialGenerator {
    #[inline(always)]
    pub const fn new(multiplier: i64, increment: i64, modulus: i64) -> Self {
        let p2 = (modulus & (modulus - 1)) == 0;
        Self {
            multiplier,
            increment,
            modulus,

            is_pow_2: p2,
            trailing_zeros: if p2 {
                modulus.trailing_zeros() as i32
            } else {
                -1
            },
        }
    }

    #[inline(always)]
    pub const fn is_modulus_power_of_2(&self) -> bool {
        self.is_pow_2
    }

    #[inline(always)]
    pub const fn get_modulus_trailing_zeros(&self) -> i32 {
        self.trailing_zeros
    }

    #[inline(always)]
    pub const fn get_multiplier(&self) -> i64 {
        self.multiplier
    }

    #[inline(always)]
    pub const fn get_increment(&self) -> i64 {
        self.increment
    }

    #[inline(always)]
    pub const fn get_modulus(&self) -> i64 {
        self.modulus
    }

    #[inline(always)]
    pub const fn is_multiplicative(&self) -> bool {
        self.increment == 0
    }

    #[inline(always)]
    pub const fn next_seed(&self, seed: i64) -> i64 {
        self._mod(
            self.multiplier
                .wrapping_mul(seed)
                .wrapping_add(self.increment),
        )
    }

    #[inline(always)]
    pub const fn _mod(&self, x: i64) -> i64 {
        if self.is_pow_2 {
            x & (self.modulus - 1)
        } else {
            ((x as u64) % (self.modulus as u64)) as i64
        }
    }

    #[inline]
    pub const fn combine(&self, steps: i64) -> LinearCongruentialGenerator {
        let mut multiplier = 1i64;
        let mut increment = 0i64;

        let mut intermediate_multiplier = self.multiplier;
        let mut intermediate_increment = self.increment;

        let mut k = steps;
        while k != 0 {
            if (k & 1) != 0 {
                multiplier = multiplier.wrapping_mul(intermediate_multiplier);
                increment = intermediate_multiplier
                    .wrapping_mul(increment)
                    .wrapping_add(intermediate_increment);
            }

            intermediate_increment = intermediate_multiplier
                .wrapping_add(1)
                .wrapping_mul(intermediate_increment);

            intermediate_multiplier = intermediate_multiplier.wrapping_mul(intermediate_multiplier);

            k = ((k as u64) >> 1) as i64;
        }

        multiplier = self._mod(multiplier);
        increment = self._mod(increment);

        LinearCongruentialGenerator::new(multiplier, increment, self.modulus)
    }

    #[inline]
    pub const fn combine_with(
        &self,
        other: &LinearCongruentialGenerator,
    ) -> Option<LinearCongruentialGenerator> {
        if self.modulus != other.modulus {
            None
        } else {
            Some(LinearCongruentialGenerator::new(
                self.multiplier.wrapping_mul(other.multiplier),
                other
                    .multiplier
                    .wrapping_mul(self.increment)
                    .wrapping_add(other.increment),
                self.modulus,
            ))
        }
    }

    #[inline]
    pub const fn invert(&self) -> LinearCongruentialGenerator {
        self.combine(-1)
    }
}
