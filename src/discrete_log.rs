/*!

This file is a Rust translation of Java code from:
    https://github.com/KaptainWutax/SeedUtils/

The original Java source file can be found at:
    https://github.com/KaptainWutax/SeedUtils/blob/master/src/main/java/kaptainwutax/seedutils/lcg/DiscreteLog.java

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

use uint::construct_uint;

use crate::{lcg::LinearCongruentialGenerator, math::Math};

pub struct DiscreteLog;

impl DiscreteLog {
    pub const fn supports(lcg: &LinearCongruentialGenerator) -> bool {
        if !lcg.is_modulus_power_of_2() || lcg.get_modulus_trailing_zeros() > 61 {
            false
        } else {
            lcg.get_multiplier() % 2 != 0 && lcg.get_increment() % 2 != 0
        }
    }

    pub fn distance_from_zero(lcg: &LinearCongruentialGenerator, seed: i64) -> i64 {
        let exp = lcg.get_modulus_trailing_zeros();

        let a: i64 = lcg.get_multiplier();
        let b: i64 = Math::mask(
            seed.wrapping_mul(lcg.get_multiplier().wrapping_sub(1))
                .wrapping_mul(Math::mod_inverse_pow_2(lcg.get_increment(), exp))
                .wrapping_add(1),
            exp + 2,
        );

        let abar = Self::theta(a, exp);
        let bbar = Self::theta(b, exp);

        bbar.wrapping_mul(Math::mask(Math::mod_inverse_pow_2(abar, exp), exp))
    }

    pub fn theta(mut number: i64, exp: i32) -> i64 {
        if number % 4 == 3 {
            number = Math::pow_2(exp + 2).wrapping_sub(number);
        }

        let xhat = number as u128;
        let xhat = mod_pow(xhat, 1u128 << (exp + 1), 1u128 << (2 * exp + 3));
        let xhat = xhat.wrapping_sub(1);
        let xhat = xhat >> (exp + 3);
        let xhat = xhat & (1u128 << exp).wrapping_sub(xhat);

        xhat as i64
    }
}

pub fn mod_pow(mut base: u128, mut exponent: u128, modulus: u128) -> u128 {
    if modulus == 1 {
        return 0;
    }

    let mut result = 1u128;
    base %= modulus;

    while exponent > 0 {
        if exponent % 2 == 1 {
            result = mod_mul(result, base, modulus);
        }
        exponent /= 2;
        base = mod_mul(base, base, modulus);
    }

    result
}

construct_uint! {
    pub struct U256(4);
}

#[inline(always)]
pub fn mod_mul(a: u128, b: u128, modulus: u128) -> u128 {
    let (a_lo, a_hi) = (a as u64, (a >> 64) as u64);
    let (b_lo, b_hi) = (b as u64, (b >> 64) as u64);
    let (modulus_lo, modulus_hi) = (modulus as u64, (modulus >> 64) as u64);

    let a = U256([a_lo, a_hi, 0, 0]);
    let b = U256([b_lo, b_hi, 0, 0]);
    let modulus = U256([modulus_lo, modulus_hi, 0, 0]);

    ((a * b) % modulus).low_u128()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_pow() {
        let tests: [(u128, u128, u128, u128); 20] = [
            (
                165901036277234507883803949598723014657,
                839856,
                84714637990324311239584324008783380482,
                28565057155865465865325385094599740015,
            ),
            (
                87886712903272012994197748584504360961,
                901700,
                149205894144872995746606492115085033474,
                68298695081012632103732778843268685325,
            ),
            (
                100041162209004928505409123661144653825,
                293328,
                96006539786823475686400183817117630466,
                24746178642967975785104601844273591251,
            ),
            (
                160378442919460637682056485419016519681,
                745491,
                96780350137964893288577776335940747266,
                33647226158460572194934911863061929367,
            ),
            (
                60814582591068442851010838248273477633,
                379501,
                61360125242566938007832289475072884738,
                36797522857361384956585337620898588647,
            ),
            (
                97043680458164713107730827399336558593,
                456889,
                167312303264793208273468830507767693314,
                15858590908459539291158288052007581611,
            ),
            (
                47508501118771197051050135510851780609,
                720086,
                67849656588038549253731160201680125954,
                3965632746346342840055279633217258355,
            ),
            (
                43608418292014528263807273462757588993,
                11035,
                123645571286044240847767765703202963458,
                50433329884177162551873043444880222945,
            ),
            (
                65485208563642029450249962507724652545,
                830647,
                83227198192127862152728525704171356162,
                6256176664351637709721819367316243765,
            ),
            (
                117156863183202315232173114355154419713,
                882311,
                108762628909595653060621371893532852226,
                10662572010280841412959783948251427921,
            ),
            (
                137728705425089472359047018868501381121,
                554635,
                49186693239407282738872212910694727682,
                5782938860535838264295675796032270653,
            ),
            (
                47818186359689976728967314789541871617,
                332764,
                155830759154505390343139956644649107458,
                114402014342101094533487769406697348853,
            ),
            (
                97615005575875422215304558423603937281,
                304784,
                28144099856524671718245511574665035778,
                13840465343134200813033609475856931587,
            ),
            (
                140401242781959607003260765545370222593,
                919828,
                140288498730847402569052104084844183554,
                13175087269450243927605410623341849973,
            ),
            (
                156089071937764612220410685496739495937,
                270273,
                154064477582033463820510537315371712514,
                137438963428818964808810464885737878541,
            ),
            (
                98811121880097419873054696928391987201,
                10226,
                168034188852238089860045688232027881474,
                12817029300397618827859698970565367685,
            ),
            (
                153669966664827260104814567475149537281,
                76793,
                168396850594237373621132343630668759042,
                88104015315115900223855960173183879095,
            ),
            (
                152212336853005156041349437806281228289,
                155314,
                151887920970129009865909762933523480578,
                148746048982411000464606429216700904009,
            ),
            (
                14471204582693333849614733798621052929,
                882101,
                52389074571706246242319390191476801538,
                29316917271392158288217025876445230821,
            ),
            (
                163713446989522742756452099348641087489,
                917943,
                141024069328288316691678419095826464770,
                9982222399982488090891157953631698869,
            ),
        ];

        for (base, exponent, modulus, result) in tests {
            assert_eq!(
                mod_pow(base, exponent, modulus),
                result,
                "mod_pow({}, {}, {}) != {}",
                base,
                exponent,
                modulus,
                result
            );
        }
    }
}
