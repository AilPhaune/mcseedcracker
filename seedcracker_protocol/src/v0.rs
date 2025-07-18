use std::{fmt::Debug, io::Write};

use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra::Err,
    prelude::{any, end, just, recursive},
    text::{ascii::ident, digits, int},
};

use crate::traits::{
    CharsIter, FloatFromStr, FloatType, FromRadix, FromRadixNegative, IntoCharsIter, StringOrSlice,
    ToVecOrSliceOwned, VecOrSlice,
};

pub trait ProtocolVersion0 {
    fn client_did_hello(&mut self);
    fn client_helloed(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum ProtocolCommand<'a> {
    Hello,
    Help,
    Quit,
    Version,
    Extensions,
    Go,
    Stop,
    ListTypes {
        extension: u32,
    },
    ListProblems {
        extension: u32,
    },
    SetupProblem {
        extension: u32,
        name: StringOrSlice<'a>,
        args: Vec<(StringOrSlice<'a>, ParsetimeProtocolValue<'a>)>,
    },
}

pub type TypeAlias<'a> = StringOrSlice<'a>;

#[derive(Debug, Clone)]
pub struct EnumerationConstructor<'a> {
    pub name: StringOrSlice<'a>,
    pub argtype: Option<TypeDeclaration<'a>>,
}

#[derive(Debug, Clone)]
pub enum TypeDeclaration<'a> {
    Alias(TypeAlias<'a>),
    Tuple(VecOrSlice<'a, TypeDeclaration<'a>>),
    List(Box<TypeDeclaration<'a>>),
    Array(Box<TypeDeclaration<'a>>, u32),
    Enumeration(Vec<EnumerationConstructor<'a>>),
}

#[derive(Debug, Clone)]
pub struct ProtocolExtensionInfo<'a> {
    pub name: StringOrSlice<'a>,
    pub version: StringOrSlice<'a>,
    pub description: StringOrSlice<'a>,
    pub authors: VecOrSlice<'a, StringOrSlice<'a>>,
    pub commands: VecOrSlice<'a, StringOrSlice<'a>>,
}

#[derive(Debug, Clone)]
pub enum ParsetimeProtocolValue<'a> {
    RawString(StringOrSlice<'a>),
    String(StringOrSlice<'a>),
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    List(Option<StringOrSlice<'a>>, Vec<ParsetimeProtocolValue<'a>>),
    Tuple(Option<StringOrSlice<'a>>, Vec<ParsetimeProtocolValue<'a>>),
    Enumeration(
        Option<StringOrSlice<'a>>,
        StringOrSlice<'a>,
        Option<Box<ParsetimeProtocolValue<'a>>>,
    ),
}

pub enum ProtocolResponse<'a> {
    Acknowledge,
    Info(StringOrSlice<'a>),
    Infos(Vec<StringOrSlice<'a>>),
    Version {
        protocol_version: i32,
        server_version: Option<StringOrSlice<'a>>,
    },
    SetupOk,
    SetupError(ParsetimeProtocolValue<'a>),
    Unexpected(Option<StringOrSlice<'a>>),
    Extensions {
        count: u32,
        extensions: Option<Vec<ProtocolExtensionInfo<'a>>>,
    },
    TypeList {
        extension: u32,
        types: VecOrSlice<'a, (StringOrSlice<'a>, TypeDeclaration<'a>)>,
    },
    ProblemList {
        extension: u32,
        problems: VecOrSlice<'a, Box<dyn SimpleV0Problem>>,
    },
    NoSuchExtension(u32),
    ParseFail,
}

pub fn command_hello_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("hello").to(ProtocolCommand::Hello)
}

pub fn command_help_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("help").to(ProtocolCommand::Help)
}

pub fn command_quit_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("quit").to(ProtocolCommand::Quit)
}

pub fn command_version_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("version").to(ProtocolCommand::Version)
}

pub fn command_extensions_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("extensions").to(ProtocolCommand::Extensions)
}

pub fn command_go_parser<'a>() -> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>>
{
    just("go").to(ProtocolCommand::Go)
}

pub fn command_stop_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("stop").to(ProtocolCommand::Stop)
}

pub fn raw_string_value_parser<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> {
    just('"')
        .ignore_then(
            any()
                .and_is(just('"').ignored().or(end()).not())
                .repeated()
                .to_slice(),
        )
        .then_ignore(just('"'))
}

pub fn radix_value_parser<'a>() -> impl Parser<'a, &'a str, u32, Err<Rich<'a, char>>> {
    int(10)
        .try_map(|s: &str, span| s.parse::<u32>().map_err(|e| Rich::custom(span, e)))
        .try_map(|r, span| {
            if (2..=36).contains(&r) {
                Ok(r)
            } else {
                Err(Rich::custom(span, "radix must be between 2 and 36"))
            }
        })
}

pub fn int_value_unsigned_parser<IntegerType>(
    dtype: &str,
) -> impl Parser<'_, &str, IntegerType, Err<Rich<'_, char>>>
where
    IntegerType: FromRadix<T = IntegerType>,
{
    (just("0x")
        .or(just("0X"))
        .ignore_then(int(16).try_map(|s: &str, span| {
            IntegerType::from_radix(s, 16).map_err(|e| Rich::custom(span, e))
        })))
    .or(just("0b").or(just("0B")).ignore_then(
        int(2).try_map(|s: &str, span| {
            IntegerType::from_radix(s, 2).map_err(|e| Rich::custom(span, e))
        }),
    ))
    .or(just("0o").or(just("0O")).ignore_then(
        int(8).try_map(|s: &str, span| {
            IntegerType::from_radix(s, 8).map_err(|e| Rich::custom(span, e))
        }),
    ))
    .or(just(dtype)
        .ignore_then(just("(").padded())
        .ignore_then(raw_string_value_parser())
        .then(
            just(",")
                .padded()
                .ignore_then(radix_value_parser().padded())
                .or_not(),
        )
        .padded()
        .then_ignore(just(")"))
        .try_map(|(s, r), span| {
            IntegerType::from_radix(s, r.unwrap_or(10)).map_err(|e| Rich::custom(span, e))
        }))
    .or(int(10).try_map(|s: &str, span| {
        IntegerType::from_radix(s, 10).map_err(|e: std::num::ParseIntError| Rich::custom(span, e))
    }))
}

pub fn int_value_signed_parser<IntegerType>(
    dtype: &str,
) -> impl Parser<'_, &str, IntegerType, Err<Rich<'_, char>>>
where
    IntegerType: FromRadixNegative<T = IntegerType>,
{
    (just("-")
        .ignored()
        .or_not()
        .then_ignore(just("0x").or(just("0X")))
        .then(int(16))
        .try_map(|(neg, s): (Option<()>, &str), span| {
            IntegerType::from_radix_negative(neg.is_some(), s, 16)
                .map_err(|e| Rich::custom(span, e))
        }))
    .or(just("-")
        .ignored()
        .or_not()
        .then_ignore(just("0b").or(just("0B")))
        .then(int(2))
        .try_map(|(neg, s): (Option<()>, &str), span| {
            IntegerType::from_radix_negative(neg.is_some(), s, 2).map_err(|e| Rich::custom(span, e))
        }))
    .or(just("-")
        .ignored()
        .or_not()
        .then_ignore(just("0o").or(just("0O")))
        .then(int(8))
        .try_map(|(neg, s): (Option<()>, &str), span| {
            IntegerType::from_radix_negative(neg.is_some(), s, 8).map_err(|e| Rich::custom(span, e))
        }))
    .or(just(dtype)
        .ignore_then(just("(").padded())
        .ignore_then(raw_string_value_parser())
        .then(
            just(",")
                .padded()
                .ignore_then(radix_value_parser().padded())
                .or_not(),
        )
        .padded()
        .then_ignore(just(")"))
        .try_map(|(s, r), span| {
            IntegerType::from_radix(s, r.unwrap_or(10)).map_err(|e| Rich::custom(span, e))
        }))
    .or(just("-")
        .ignored()
        .or_not()
        .then(int(10))
        .try_map(|(neg, s): (Option<()>, &str), span| {
            IntegerType::from_radix_negative(neg.is_some(), s, 10)
                .map_err(|e| Rich::custom(span, e))
        }))
}

pub fn parse_bool<'a>() -> impl Parser<'a, &'a str, bool, Err<Rich<'a, char>>> {
    just("true").map(|_| true).or(just("false").map(|_| false))
}

pub fn parse_float_decimal<'a, F>() -> impl Parser<'a, &'a str, F, Err<Rich<'a, char>>>
where
    F: FloatFromStr<T = F>,
{
    let exponent = (just("e").or(just("E")))
        .then(just("+").or(just("-")).or_not().to_slice())
        .then(digits(10))
        .to_slice();

    // Case 1: digits, optional fraction, optional exponent
    let int_frac = digits(10)
        .then(just(".").then(digits(10)).or_not())
        .then(exponent.or_not())
        .to_slice();

    // Case 2: dot + digits, optional exponent
    let only_frac = just(".")
        .then(digits(10))
        .then(exponent.or_not())
        .to_slice();

    just("-")
        .or_not()
        .then(int_frac.or(only_frac))
        .to_slice()
        .try_map(|s, span| F::float_from_str(s).map_err(|e| Rich::custom(span, e)))
}

pub fn parse_float_special<'a, F>() -> impl Parser<'a, &'a str, F, Err<Rich<'a, char>>>
where
    F: FloatType<F> + Clone,
{
    just("NaN")
        .to(F::nan())
        .or(just("Infinity").to(F::pos_inf()))
        .or(just("-Infinity").to(F::neg_inf()))
}

pub fn parse_float_constructor<F>(dtype: &str) -> impl Parser<'_, &str, F, Err<Rich<'_, char>>>
where
    F: FloatFromStr<T = F>,
{
    just(dtype)
        .ignore_then(just("(").padded())
        .ignore_then(
            raw_string_value_parser()
                .padded()
                .try_map(|s, span| F::float_from_str(s).map_err(|e| Rich::custom(span, e)))
                .or(just("0x")
                    .or(just("0X"))
                    .ignore_then(int(16))
                    .try_map(|s: &str, span| {
                        F::float_from_hex(s).map_err(|e| Rich::custom(span, e))
                    })
                    .padded()),
        )
        .then_ignore(just(")").padded())
}

pub fn parse_f32<'a>() -> impl Parser<'a, &'a str, f32, Err<Rich<'a, char>>> {
    parse_float_constructor::<f32>("f32")
        .or(parse_float_special::<f32>())
        .or(parse_float_decimal::<f32>())
}

pub fn parse_f64<'a>() -> impl Parser<'a, &'a str, f64, Err<Rich<'a, char>>> {
    parse_float_constructor::<f64>("f64")
        .or(parse_float_special::<f64>())
        .or(parse_float_decimal::<f64>())
}

pub fn list_or_array_or_tuple_value_parser0<'a>(
    begin: &'a str,
    end: &'a str,
    value_parser: impl Parser<'a, &'a str, ParsetimeProtocolValue<'a>, Err<Rich<'a, char>>> + 'a,
) -> impl Parser<'a, &'a str, (Option<&'a str>, Vec<ParsetimeProtocolValue<'a>>), Err<Rich<'a, char>>>
{
    ident().then_ignore(just("::")).or_not().then(
        just(begin)
            .ignore_then(
                value_parser
                    .padded()
                    .separated_by(just(",").padded())
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(end).padded())
            .boxed(),
    )
}

fn enum_value_parser0<'a>(
    value_parser: impl Parser<'a, &'a str, ParsetimeProtocolValue<'a>, Err<Rich<'a, char>>>,
) -> impl Parser<'a, &'a str, ParsetimeProtocolValue<'a>, Err<Rich<'a, char>>> {
    ident()
        .then_ignore(just("::"))
        .or_not()
        .then(ident())
        .then(
            just("(")
                .ignore_then(value_parser)
                .then_ignore(just(")"))
                .or_not(),
        )
        .map(|((type_alias, constructor), value)| {
            ParsetimeProtocolValue::Enumeration(
                type_alias.map(StringOrSlice::Sl),
                StringOrSlice::Sl(constructor),
                value.map(Box::new),
            )
        })
}

pub fn generic_value_parser<'a>()
-> impl Parser<'a, &'a str, ParsetimeProtocolValue<'a>, Err<Rich<'a, char>>> {
    recursive(|value_parser| {
        raw_string_value_parser()
            .map(StringOrSlice::Sl)
            .map(ParsetimeProtocolValue::RawString)
            .or(parse_bool().map(ParsetimeProtocolValue::Bool))
            .or(int_value_signed_parser::<i8>("i8").map(ParsetimeProtocolValue::I8))
            .or(int_value_unsigned_parser::<u8>("u8").map(ParsetimeProtocolValue::U8))
            .or(int_value_unsigned_parser::<i16>("i16").map(ParsetimeProtocolValue::I16))
            .or(int_value_unsigned_parser::<u16>("u16").map(ParsetimeProtocolValue::U16))
            .or(int_value_unsigned_parser::<i32>("i32").map(ParsetimeProtocolValue::I32))
            .or(int_value_unsigned_parser::<u32>("u32").map(ParsetimeProtocolValue::U32))
            .or(int_value_unsigned_parser::<i64>("i64").map(ParsetimeProtocolValue::I64))
            .or(int_value_unsigned_parser::<u64>("u64").map(ParsetimeProtocolValue::U64))
            .or(parse_f32().map(ParsetimeProtocolValue::F32))
            .or(parse_f64().map(ParsetimeProtocolValue::F64))
            .or(
                list_or_array_or_tuple_value_parser0("(", ")", value_parser.clone()).map(
                    |(type_alias, value)| {
                        ParsetimeProtocolValue::Tuple(type_alias.map(StringOrSlice::Sl), value)
                    },
                ),
            )
            .or(
                list_or_array_or_tuple_value_parser0("[", "]", value_parser.clone()).map(
                    |(type_alias, value)| {
                        ParsetimeProtocolValue::List(type_alias.map(StringOrSlice::Sl), value)
                    },
                ),
            )
            .or(enum_value_parser0(value_parser.clone()))
            .boxed()
    })
}

pub fn type_native_declaration_parser<'a>()
-> impl Parser<'a, &'a str, TypeDeclaration<'a>, Err<Rich<'a, char>>> {
    just("u8")
        .map(StringOrSlice::Sl)
        .map(TypeDeclaration::Alias)
        .or(just("u16")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("u32")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("u64")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("i8")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("i16")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("i32")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("i64")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("f32")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("f64")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("bool")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
        .or(just("string")
            .map(StringOrSlice::Sl)
            .map(TypeDeclaration::Alias))
}

pub fn type_tuple_declaration_parser<'a>()
-> impl Parser<'a, &'a str, TypeDeclaration<'a>, Err<Rich<'a, char>>> {
    just("tuple")
        .then(just("(").padded())
        .ignore_then(
            type_native_declaration_parser()
                .padded()
                .separated_by(just(","))
                .allow_trailing()
                .collect::<Vec<_>>()
                .or_not()
                .padded(),
        )
        .then_ignore(just(")").padded())
        .map(|res| TypeDeclaration::Tuple(res.unwrap_or_default().to_vec_or_slice()))
}

pub fn type_list_declaration_parser<'a>()
-> impl Parser<'a, &'a str, TypeDeclaration<'a>, Err<Rich<'a, char>>> {
    just("list")
        .then(just("(").padded())
        .ignore_then(type_native_declaration_parser().padded())
        .then_ignore(just(")").padded())
        .map(|res| TypeDeclaration::List(Box::new(res)))
}

pub fn type_array_declaration_parser<'a>()
-> impl Parser<'a, &'a str, TypeDeclaration<'a>, Err<Rich<'a, char>>> {
    just("array")
        .ignore_then(just("(").padded())
        .ignore_then(type_native_declaration_parser().padded())
        .then_ignore(just(",").padded())
        .then(int_value_unsigned_parser::<u32>("u32").padded())
        .then_ignore(just(")").padded())
        .map(|(t, len)| TypeDeclaration::Array(Box::new(t), len))
}

pub fn type_declaration_parser<'a>()
-> impl Parser<'a, &'a str, TypeDeclaration<'a>, Err<Rich<'a, char>>> {
    type_native_declaration_parser()
        .or(type_tuple_declaration_parser())
        .boxed()
}

pub fn single_word_commands_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    command_hello_parser()
        .or(command_quit_parser())
        .or(command_help_parser())
        .or(command_version_parser())
        .or(command_extensions_parser())
        .or(command_go_parser())
        .or(command_stop_parser())
}

pub fn list_types_command_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("list-types")
        .ignore_then(int(10).padded())
        .try_map(|s, span| {
            u32::from_str_radix(s, 10)
                .map(|i| ProtocolCommand::ListTypes { extension: i })
                .map_err(|e| Rich::custom(span, e))
        })
}

pub fn list_problems_command_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("list-problems")
        .ignore_then(int(10).padded())
        .try_map(|s, span| {
            u32::from_str_radix(s, 10)
                .map(|i| ProtocolCommand::ListProblems { extension: i })
                .map_err(|e| Rich::custom(span, e))
        })
}

pub fn setup_problem_argument_parser<'a>()
-> impl Parser<'a, &'a str, (&'a str, ParsetimeProtocolValue<'a>), Err<Rich<'a, char>>> {
    raw_string_value_parser()
        .then_ignore(just("=").padded())
        .then(generic_value_parser())
}

pub fn setup_problem_command_parser<'a>()
-> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    just("setup-problem")
        .ignore_then(int(10).padded())
        .try_map(|s, span| u32::from_str_radix(s, 10).map_err(|e| Rich::custom(span, e)))
        .then(raw_string_value_parser().padded())
        .then(
            setup_problem_argument_parser()
                .padded()
                .map(|(name, value)| (StringOrSlice::Sl(name), value))
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(|((extension, name), args)| ProtocolCommand::SetupProblem {
            name: StringOrSlice::Sl(name),
            extension,
            args,
        })
}

pub fn full_v0_parser<'a>() -> impl Parser<'a, &'a str, ProtocolCommand<'a>, Err<Rich<'a, char>>> {
    single_word_commands_parser()
        .or(list_types_command_parser())
        .or(list_problems_command_parser())
        .or(setup_problem_command_parser())
        .then_ignore(just("\r\n").or(just("\n")).or_not())
        .then_ignore(end())
}

pub fn v0_format_string<T>(
    writer: &mut dyn std::io::Write,
    string: impl IntoCharsIter<T>,
) -> Result<(), std::io::Error>
where
    CharsIter<T>: IntoIterator<Item = char>,
{
    write!(writer, "\"")?;
    for c in string.chars_iter().into_iter() {
        write!(writer, "{}", c.escape_default())?
    }
    write!(writer, "\"")
}

pub fn v0_format_type_decl(
    writer: &mut dyn std::io::Write,
    decl: &TypeDeclaration,
) -> Result<(), std::io::Error> {
    match decl {
        TypeDeclaration::Alias(alias) => {
            v0_format_string(writer, alias.as_slice())?;
            Ok(())
        }
        TypeDeclaration::Array(t, len) => {
            write!(writer, "array(")?;
            v0_format_type_decl(writer, t)?;
            write!(writer, ", {})", *len)
        }
        TypeDeclaration::List(t) => {
            write!(writer, "list(")?;
            v0_format_type_decl(writer, t)?;
            write!(writer, ")")
        }
        TypeDeclaration::Tuple(types) => {
            write!(writer, "tuple(")?;
            let mut not_first = false;
            for t in types.as_slice() {
                if not_first {
                    write!(writer, ", ")?;
                }
                not_first = true;
                v0_format_type_decl(writer, t)?;
            }
            write!(writer, ")")
        }
        TypeDeclaration::Enumeration(constructors) => {
            write!(writer, "enum(")?;
            let mut not_first = false;
            for c in constructors.as_slice() {
                if not_first {
                    write!(writer, ", ")?;
                }
                not_first = true;
                write!(writer, "{}", c.name)?;
                if let Some(argt) = &c.argtype {
                    write!(writer, "(")?;
                    v0_format_type_decl(writer, argt)?;
                    write!(writer, ")")?;
                }
            }
            write!(writer, ")")
        }
    }
}

pub fn v0_format_value(
    writer: &mut dyn std::io::Write,
    value: &ParsetimeProtocolValue,
) -> Result<(), std::io::Error> {
    match value {
        ParsetimeProtocolValue::String(string) => v0_format_string(writer, string.as_slice()),
        ParsetimeProtocolValue::RawString(string) => {
            write!(writer, "\"{}\"", string)
        }
        ParsetimeProtocolValue::Bool(b) => {
            write!(writer, "{}", if *b { "true" } else { "false" })
        }
        ParsetimeProtocolValue::I8(i) => write!(writer, "i8({})", *i),
        ParsetimeProtocolValue::U8(i) => write!(writer, "u8({})", *i),
        ParsetimeProtocolValue::I16(i) => write!(writer, "i16({})", *i),
        ParsetimeProtocolValue::U16(i) => write!(writer, "u16({})", *i),
        ParsetimeProtocolValue::I32(i) => write!(writer, "i32({})", *i),
        ParsetimeProtocolValue::U32(i) => write!(writer, "u32({})", *i),
        ParsetimeProtocolValue::I64(i) => write!(writer, "i64({})", *i),
        ParsetimeProtocolValue::U64(i) => write!(writer, "u64({})", *i),
        ParsetimeProtocolValue::F32(f) => write!(writer, "f32({})", *f),
        ParsetimeProtocolValue::F64(f) => write!(writer, "f64({})", *f),
        ParsetimeProtocolValue::Tuple(type_alias, value) => {
            if let Some(type_alias) = type_alias {
                write!(writer, "{}::", type_alias)?;
            }
            write!(writer, "(")?;
            let mut not_first = false;
            for v in value.as_slice() {
                if not_first {
                    write!(writer, ", ")?;
                }
                not_first = true;
                v0_format_value(writer, v)?;
            }
            write!(writer, ")")
        }
        ParsetimeProtocolValue::List(type_alias, value) => {
            if let Some(type_alias) = type_alias {
                write!(writer, "{}::", type_alias)?;
            }
            write!(writer, "[")?;
            let mut not_first = false;
            for v in value.as_slice() {
                if not_first {
                    write!(writer, ", ")?;
                }
                not_first = true;
                v0_format_value(writer, v)?;
            }
            write!(writer, "]")
        }
        ParsetimeProtocolValue::Enumeration(type_alias, constructor, value) => {
            if let Some(type_alias) = type_alias {
                write!(writer, "{}::", type_alias)?;
            }
            write!(writer, "{}", constructor)?;
            if let Some(value) = value {
                write!(writer, "(")?;
                v0_format_value(writer, value.as_ref())?;
                write!(writer, ")")
            } else {
                Ok(())
            }
        }
    }
}

pub fn format_response(
    writer: &mut dyn std::io::Write,
    response: &ProtocolResponse,
) -> Result<(), std::io::Error> {
    match response {
        ProtocolResponse::Acknowledge => {
            writeln!(writer, "ack")
        }
        ProtocolResponse::Info(info) => {
            writeln!(writer, "info:  {}", info)
        }
        ProtocolResponse::Infos(infos) => {
            for info in infos {
                writeln!(writer, "info:  {}", info)?;
            }
            Ok(())
        }
        ProtocolResponse::ParseFail => {
            writeln!(writer, "parsefail")
        }
        ProtocolResponse::Version {
            protocol_version,
            server_version,
        } => {
            write!(writer, "version {}", *protocol_version)?;
            if let Some(server_version) = server_version {
                v0_format_string(writer, server_version)?;
            }
            writeln!(writer)
        }
        ProtocolResponse::SetupOk => {
            writeln!(writer, "setup-ok")
        }
        ProtocolResponse::SetupError(err) => {
            write!(writer, "setup-error ")?;
            v0_format_value(writer, err)?;
            writeln!(writer)
        }
        ProtocolResponse::Unexpected(err) => {
            if let Some(err) = err {
                write!(writer, "unexpected ")?;
                v0_format_string(writer, err)?;
                writeln!(writer)
            } else {
                writeln!(writer, "unexpected")
            }
        }
        ProtocolResponse::Extensions { count, extensions } => {
            write!(writer, "extensions {}", count)?;
            if let Some(extensions) = extensions {
                for extension in extensions {
                    write!(writer, " extension_info(")?;
                    v0_format_string(writer, &extension.name)?;
                    write!(writer, ", ")?;
                    v0_format_string(writer, &extension.version)?;
                    write!(writer, ", ")?;
                    v0_format_string(writer, &extension.description)?;
                    write!(writer, ", [")?;
                    // TODO: use values serializer when implemented
                    let mut not_first = false;
                    for author in extension.authors.as_slice() {
                        if not_first {
                            write!(writer, ", ")?;
                        }
                        not_first = true;
                        v0_format_string(writer, author)?;
                    }
                    write!(writer, "], [")?;
                    let mut not_first = false;
                    for author in extension.commands.as_slice() {
                        if not_first {
                            write!(writer, ", ")?;
                        }
                        not_first = true;
                        v0_format_string(writer, author)?;
                    }
                    write!(writer, "]) ")?;
                }
            }
            writeln!(writer)
        }
        ProtocolResponse::NoSuchExtension(ext) => {
            writeln!(writer, "no-such-extension {}", *ext)
        }
        ProtocolResponse::TypeList { extension, types } => {
            write!(writer, "type-list {}", *extension)?;
            for (name, decl) in types.as_slice() {
                write!(writer, " {} = ", name.as_slice())?;
                v0_format_type_decl(writer, decl)?;
            }
            writeln!(writer)
        }
        ProtocolResponse::ProblemList {
            extension,
            problems,
        } => {
            write!(writer, "problem-list {} [", *extension)?;
            let mut not_first = false;
            for pb in problems.as_slice() {
                if not_first {
                    write!(writer, ", ")?;
                }
                not_first = true;
                write!(writer, "(")?;
                v0_format_string(writer, pb.name())?;
                write!(writer, ", ")?;
                v0_format_string(writer, pb.description())?;
                write!(writer, ", [")?;
                let mut not_first2 = false;
                for arg in pb.args().as_slice() {
                    if not_first2 {
                        write!(writer, ", ")?;
                    }
                    not_first2 = true;
                    write!(writer, "(")?;
                    v0_format_string(writer, &arg.name)?;
                    write!(writer, ", {}, ", arg.optional)?;
                    v0_format_string(writer, &arg.argtype)?;
                    write!(writer, ")")?;
                }
                write!(writer, "])")?;
            }
            writeln!(writer, "]")
        }
    }
}

pub trait SimpleV0Extension: Debug {
    fn protocol_extension_info(&self) -> ProtocolExtensionInfo;
    fn list_extension_types<'a>(
        &'a self,
    ) -> VecOrSlice<'a, (StringOrSlice<'a>, TypeDeclaration<'a>)>;
    fn list_extension_problems<'a>(&'a self) -> VecOrSlice<'a, Box<dyn SimpleV0Problem>>;
    fn get_problem<'a>(&'a self, name: &str) -> Option<Box<dyn SimpleV0Problem>>;
}

pub trait SimpleV0ProblemBruteCalculation: Debug {
    fn go(&mut self, output: &mut dyn Write);
    fn stop(&mut self);
    fn is_done(&self) -> bool;
    fn is_running(&self) -> bool;
    fn progress(&self) -> (u64, u64);
    fn result(&self) -> Option<ParsetimeProtocolValue<'_>>;
}

pub trait SimpleV0Problem: Debug {
    fn name<'a>(&'a self) -> StringOrSlice<'a>;
    fn description<'a>(&'a self) -> StringOrSlice<'a>;
    fn args<'a>(&'a self) -> VecOrSlice<'a, V0ProblemArgumentDescription<'a>>;
    fn setup<'a>(
        &'a self,
        args: VecOrSlice<'a, (StringOrSlice<'a>, ParsetimeProtocolValue<'a>)>,
    ) -> Result<Box<dyn SimpleV0ProblemBruteCalculation>, ParsetimeProtocolValue<'a>>;
}

#[derive(Debug, Clone)]
pub struct V0ProblemArgumentDescription<'a> {
    pub name: StringOrSlice<'a>,
    pub optional: bool,
    pub argtype: StringOrSlice<'a>,
}

#[cfg(test)]
mod tests {
    use core::{f32, f64};

    use chumsky::Parser;

    use crate::v0::{int_value_signed_parser, int_value_unsigned_parser, parse_f32, parse_f64};

    #[test]
    fn test_int_value_unsigned_parser() {
        let parser = int_value_unsigned_parser::<u32>("u32");

        assert_eq!(parser.parse("42").into_result(), Ok(42));
        assert_eq!(parser.parse("0x2A").into_result(), Ok(42));
        assert_eq!(parser.parse("0b101010").into_result(), Ok(42));
        assert_eq!(parser.parse("0o52").into_result(), Ok(42));
        assert_eq!(parser.parse(r#"u32("2A", 16)"#).into_result(), Ok(42));
        assert!(parser.parse("0xGHI").has_errors());
    }

    #[test]
    fn test_int_value_signed_parser() {
        let parser = int_value_signed_parser::<i32>("i32");

        assert_eq!(parser.parse("42").into_result(), Ok(42));
        assert_eq!(parser.parse("0x2A").into_result(), Ok(42));
        assert_eq!(parser.parse("0b101010").into_result(), Ok(42));
        assert_eq!(parser.parse("0o52").into_result(), Ok(42));
        assert_eq!(parser.parse(r#"i32("2A", 16)"#).into_result(), Ok(42));
        assert!(parser.parse("0xGHI").has_errors());
        assert!(parser.parse("0xFFFFFFFF").has_errors());
        assert!(parser.parse("0x80000000").has_errors());
        assert_eq!(parser.parse("0x7FFFFFFF").into_result(), Ok(i32::MAX));
        assert_eq!(parser.parse("-0x7FFFFFFF").into_result(), Ok(-i32::MAX));
        assert_eq!(parser.parse("-0x80000000").into_result(), Ok(i32::MIN));
        assert!(parser.parse("-0x80000001").has_errors());
    }

    #[test]
    fn test_parse_f32() {
        let parser = parse_f32();

        assert_eq!(parser.parse("42").into_result(), Ok(42.0f32));
        assert_eq!(parser.parse("43.5").into_result(), Ok(43.5f32));
        assert_eq!(parser.parse("0.3").into_result(), Ok(0.3f32));
        assert_eq!(parser.parse("0.3e3").into_result(), Ok(300f32));
        assert!(parser.parse("0x41").has_errors());
        assert_eq!(
            parser.parse("f32(0x40490fdb)").into_result(),
            Ok(f32::consts::PI)
        );
        assert_eq!(
            parser.parse("f32(\"-12345e-5\")").into_result(),
            Ok(-0.12345f32)
        );
    }

    #[test]
    fn test_parse_f64() {
        let parser = parse_f64();

        assert_eq!(parser.parse("42").into_result(), Ok(42.0f64));
        assert_eq!(parser.parse("43.5").into_result(), Ok(43.5f64));
        assert_eq!(parser.parse("0.3").into_result(), Ok(0.3f64));
        assert_eq!(parser.parse("0.3e3").into_result(), Ok(300f64));
        assert!(parser.parse("0x41").has_errors());
        assert_eq!(
            parser.parse("f64(0x400921fb54442d18)").into_result(),
            Ok(f64::consts::PI)
        );
        assert_eq!(
            parser.parse("f64(\"-12345e-5\")").into_result(),
            Ok(-0.12345f64)
        );
    }
}
