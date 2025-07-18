use std::{
    fmt::{self, Display, Formatter},
    num::{ParseFloatError, ParseIntError},
    str::Chars,
};

pub struct CharsIter<T> {
    d: T,
}

pub trait IntoCharsIter<T> {
    fn chars_iter(self) -> CharsIter<T>;
}

impl<T> IntoCharsIter<T> for T
where
    CharsIter<T>: IntoIterator<Item = char>,
{
    fn chars_iter(self) -> CharsIter<T> {
        CharsIter { d: self }
    }
}

impl<'a> IntoIterator for CharsIter<&'a str> {
    type Item = char;
    type IntoIter = Chars<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.d.chars()
    }
}

impl<'a> IntoIterator for CharsIter<&'a String> {
    type Item = char;
    type IntoIter = Chars<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.d.chars()
    }
}

pub struct OwningChars {
    s: String,
    idx: usize,
    idx_back: usize,
}

impl From<String> for OwningChars {
    fn from(s: String) -> Self {
        OwningChars {
            idx_back: s.len(),
            idx: 0,
            s,
        }
    }
}

impl Iterator for OwningChars {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.idx >= self.s.len() {
            return None;
        }
        if let Some(c) = self.s[self.idx..].chars().next() {
            self.idx += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for OwningChars {
    fn next_back(&mut self) -> Option<char> {
        if self.idx_back == 0 {
            return None;
        }
        if let Some(c) = self.s[..self.idx_back].chars().next_back() {
            self.idx_back -= c.len_utf8();
            Some(c)
        } else {
            None
        }
    }
}

impl IntoIterator for CharsIter<String> {
    type Item = char;
    type IntoIter = OwningChars;

    fn into_iter(self) -> Self::IntoIter {
        OwningChars::from(self.d)
    }
}

pub trait FromRadix {
    type T;

    fn from_radix(s: &str, radix: u32) -> Result<Self::T, ParseIntError>;
}

macro_rules! trivial_to_radix {
    ($dtype: ident) => {
        impl FromRadix for $dtype {
            type T = $dtype;

            fn from_radix(s: &str, radix: u32) -> Result<$dtype, ParseIntError> {
                $dtype::from_str_radix(s, radix)
            }
        }
    };
}

trivial_to_radix!(u8);
trivial_to_radix!(u16);
trivial_to_radix!(u32);
trivial_to_radix!(u64);
trivial_to_radix!(u128);
trivial_to_radix!(usize);
trivial_to_radix!(i8);
trivial_to_radix!(i16);
trivial_to_radix!(i32);
trivial_to_radix!(i64);
trivial_to_radix!(i128);
trivial_to_radix!(isize);

pub enum FromRadixNegativeError {
    ParseError(ParseIntError),
    PosOverflow,
    NegOverflow,
}

impl Display for FromRadixNegativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromRadixNegativeError::ParseError(e) => e.fmt(f),
            FromRadixNegativeError::PosOverflow => {
                write!(f, "Overflow of datatype: Value exceeds maxium")
            }
            FromRadixNegativeError::NegOverflow => {
                write!(f, "Underflow of datatype: Value is smaller than minimum")
            }
        }
    }
}

pub trait FromRadixNegative: FromRadix {
    fn from_radix_negative(
        negative: bool,
        s: &str,
        radix: u32,
    ) -> Result<Self::T, FromRadixNegativeError>;
}

macro_rules! trivial_to_radix_negative {
    ($udtype: ident, $sdtype: ident) => {
        impl FromRadixNegative for $sdtype {
            fn from_radix_negative(
                negative: bool,
                s: &str,
                radix: u32,
            ) -> Result<Self::T, FromRadixNegativeError> {
                let unsigned =
                    $udtype::from_radix(s, radix).map_err(FromRadixNegativeError::ParseError)?;
                if negative {
                    if unsigned == $sdtype::MAX as $udtype + 1 {
                        Ok($sdtype::MIN)
                    } else if unsigned > $sdtype::MAX as $udtype {
                        Err(FromRadixNegativeError::NegOverflow)
                    } else {
                        Ok(-(unsigned as $sdtype))
                    }
                } else if unsigned > $sdtype::MAX as $udtype {
                    Err(FromRadixNegativeError::PosOverflow)
                } else {
                    Ok(unsigned as $sdtype)
                }
            }
        }
    };
}

trivial_to_radix_negative!(u8, i8);
trivial_to_radix_negative!(u16, i16);
trivial_to_radix_negative!(u32, i32);
trivial_to_radix_negative!(u64, i64);
trivial_to_radix_negative!(u128, i128);
trivial_to_radix_negative!(usize, isize);

pub trait FloatFromStr {
    type T;

    fn float_from_str(s: &str) -> Result<Self::T, ParseFloatError>;
    fn float_from_hex(s: &str) -> Result<Self::T, ParseIntError>;
}

impl FloatFromStr for f32 {
    type T = f32;

    fn float_from_str(s: &str) -> Result<Self::T, ParseFloatError> {
        s.parse()
    }

    fn float_from_hex(s: &str) -> Result<Self::T, ParseIntError> {
        let u: u32 = u32::from_str_radix(s, 16)?;
        Ok(f32::from_bits(u))
    }
}

impl FloatFromStr for f64 {
    type T = f64;

    fn float_from_str(s: &str) -> Result<Self::T, ParseFloatError> {
        s.parse()
    }

    fn float_from_hex(s: &str) -> Result<Self::T, ParseIntError> {
        let u: u64 = u64::from_str_radix(s, 16)?;
        Ok(f64::from_bits(u))
    }
}

pub trait FloatType<F> {
    fn pos_inf() -> F;
    fn neg_inf() -> F;
    fn nan() -> F;
}

impl FloatType<f32> for f32 {
    fn pos_inf() -> f32 {
        f32::INFINITY
    }

    fn neg_inf() -> f32 {
        f32::NEG_INFINITY
    }

    fn nan() -> f32 {
        f32::NAN
    }
}

impl FloatType<f64> for f64 {
    fn pos_inf() -> f64 {
        f64::INFINITY
    }

    fn neg_inf() -> f64 {
        f64::NEG_INFINITY
    }

    fn nan() -> f64 {
        f64::NAN
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringOrSlice<'a> {
    St(String),
    Sl(&'a str),
}

impl<'a> StringOrSlice<'a> {
    pub fn as_slice(&'a self) -> &'a str {
        match self {
            Self::Sl(s) => *s,
            Self::St(s) => s as &str,
        }
    }

    pub fn iter(&self) -> Chars<'_> {
        match self {
            Self::Sl(s) => s.chars(),
            Self::St(s) => s.chars(),
        }
    }
}

impl<'a> Display for StringOrSlice<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_slice())
    }
}

impl<'a> IntoIterator for CharsIter<StringOrSlice<'a>> {
    type Item = char;
    type IntoIter = EitherIterator<char, Chars<'a>, OwningChars>;

    fn into_iter(self) -> Self::IntoIter {
        match self.d {
            StringOrSlice::Sl(s) => EitherIterator::A(s.chars()),
            StringOrSlice::St(s) => EitherIterator::B(s.chars_iter().into_iter()),
        }
    }
}

impl<'a> IntoIterator for CharsIter<&'a StringOrSlice<'_>> {
    type Item = char;
    type IntoIter = Chars<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.d {
            StringOrSlice::Sl(s) => s.chars(),
            StringOrSlice::St(s) => s.chars(),
        }
    }
}

pub enum EitherIterator<I, A: Iterator<Item = I>, B: Iterator<Item = I>> {
    A(A),
    B(B),
}

impl<I, A: Iterator<Item = I>, B: Iterator<Item = I>> Iterator for EitherIterator<I, A, B> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::A(a) => a.next(),
            Self::B(b) => b.next(),
        }
    }
}

impl<I, A: DoubleEndedIterator<Item = I>, B: DoubleEndedIterator<Item = I>> DoubleEndedIterator
    for EitherIterator<I, A, B>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::A(a) => a.next_back(),
            Self::B(b) => b.next_back(),
        }
    }
}

impl From<String> for StringOrSlice<'_> {
    fn from(value: String) -> Self {
        StringOrSlice::St(value)
    }
}

impl<'a> From<&'a str> for StringOrSlice<'a> {
    fn from(value: &'a str) -> Self {
        StringOrSlice::Sl(value)
    }
}

pub trait ToStringOrSliceOwned<'a> {
    fn to_string_or_slice(self) -> StringOrSlice<'a>;
}

impl<'a> ToStringOrSliceOwned<'a> for String {
    fn to_string_or_slice(self) -> StringOrSlice<'a> {
        StringOrSlice::St(self)
    }
}

pub trait StaticStringOrSlice {
    fn static_string_or_slice(&'static self) -> StringOrSlice<'static>;
}

impl<T> StaticStringOrSlice for T
where
    T: RefToStringOrSlice<'static>,
{
    fn static_string_or_slice(&'static self) -> StringOrSlice<'static> {
        self.ref_to_string_or_slice()
    }
}

pub trait RefToStringOrSlice<'a> {
    fn ref_to_string_or_slice(&'a self) -> StringOrSlice<'a>;
}

impl<'a> RefToStringOrSlice<'a> for String {
    fn ref_to_string_or_slice(&'a self) -> StringOrSlice<'a> {
        StringOrSlice::Sl(self)
    }
}

impl<'a> RefToStringOrSlice<'a> for &String {
    fn ref_to_string_or_slice(&'a self) -> StringOrSlice<'a> {
        StringOrSlice::Sl(self)
    }
}

impl<'a> RefToStringOrSlice<'a> for &str {
    fn ref_to_string_or_slice(&'a self) -> StringOrSlice<'a> {
        StringOrSlice::Sl(self)
    }
}

#[derive(Debug, Clone)]
pub enum VecOrSlice<'a, T> {
    V(Vec<T>),
    S(&'a [T]),
}

impl<'a, T> From<&'a [T]> for VecOrSlice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        VecOrSlice::S(value)
    }
}

impl<'a, T> From<Vec<T>> for VecOrSlice<'a, T> {
    fn from(value: Vec<T>) -> Self {
        VecOrSlice::V(value)
    }
}

impl<'a, T> VecOrSlice<'a, T> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            VecOrSlice::S(s) => *s,
            VecOrSlice::V(v) => v as &[T],
        }
    }
}

pub trait ToVecOrSliceOwned<'a, T> {
    fn to_vec_or_slice(self) -> VecOrSlice<'a, T>;
}

impl<'a, T> ToVecOrSliceOwned<'a, T> for Vec<T> {
    fn to_vec_or_slice(self) -> VecOrSlice<'a, T> {
        VecOrSlice::V(self)
    }
}

pub trait StaticVecOrSlice<T> {
    fn static_vec_or_slice(&'static self) -> VecOrSlice<'static, T>;
}

pub trait RefToVecOrSlice<'a, T> {
    fn ref_to_vec_or_slice(&'a self) -> VecOrSlice<'a, T>;
}

impl<'a, T> RefToVecOrSlice<'a, T> for Vec<T> {
    fn ref_to_vec_or_slice(&'a self) -> VecOrSlice<'a, T> {
        VecOrSlice::S(self)
    }
}

impl<'a, T> RefToVecOrSlice<'a, T> for [T] {
    fn ref_to_vec_or_slice(&'a self) -> VecOrSlice<'a, T> {
        VecOrSlice::S(self)
    }
}

impl<'a, T> RefToVecOrSlice<'a, T> for &[T] {
    fn ref_to_vec_or_slice(&'a self) -> VecOrSlice<'a, T> {
        VecOrSlice::S(*self)
    }
}

impl<T, U> StaticVecOrSlice<T> for U
where
    U: RefToVecOrSlice<'static, T>,
{
    fn static_vec_or_slice(&'static self) -> VecOrSlice<'static, T> {
        self.ref_to_vec_or_slice()
    }
}
