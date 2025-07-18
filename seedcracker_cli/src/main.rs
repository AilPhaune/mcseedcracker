use std::io::Write;

use clap::Parser;
use mcsci::{
    current_impl::MCSCIProtocol,
    traits::{RefToStringOrSlice, RefToVecOrSlice, StringOrSlice, VecOrSlice},
    v0::{
        EnumerationConstructor, ParsetimeProtocolValue, ProtocolExtensionInfo, SimpleV0Extension,
        SimpleV0Problem, SimpleV0ProblemBruteCalculation, TypeDeclaration,
        V0ProblemArgumentDescription, v0_format_value,
    },
};
use mcseedcracker::features::end_pillars::{
    PartialEndPillars, PillarHeightHint, PillarMatchResult,
};

mod tui;
mod tui_handler;

#[derive(Parser)]
#[command(name = "seedcracker")]
#[command(bin_name = "seedcracker")]
pub struct Cli {
    #[clap(long, help = "Runs the TUI", exclusive = true)]
    tui: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = if cli.tui {
        tui_handler::run_tui()
    } else {
        run_stdin_loop()
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(err.raw_os_error().unwrap_or(1));
    }

    std::process::exit(0);
}

fn run_stdin_loop() -> Result<(), std::io::Error> {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    println!("info:  Welcome to mcseedcracker.");
    println!("info:  Type 'hello' to initialize");
    println!("info:  Then, type 'help' for a list of commands");
    println!("info:  Then, type 'quit' to exit");

    let mut protocol = MCSCIProtocol::default();

    protocol.register_extension(Extension::new());

    protocol.server_loop(&mut handle, &mut stdout, &mut stderr)
}

#[derive(Debug)]
pub struct Extension {
    types: Vec<(StringOrSlice<'static>, TypeDeclaration<'static>)>,
    problems: Vec<Box<dyn SimpleV0Problem>>,
}

macro_rules! typedef {
    ($name: expr, $t: expr) => {
        (StringOrSlice::Sl($name), $t)
    };
}

impl Extension {
    pub fn new() -> Extension {
        let ext = Extension {
            types: vec![
                typedef!(
                    "block_pos",
                    TypeDeclaration::Tuple(
                        [
                            TypeDeclaration::Alias(StringOrSlice::Sl("i32")),
                            TypeDeclaration::Alias(StringOrSlice::Sl("i32")),
                            TypeDeclaration::Alias(StringOrSlice::Sl("i32"))
                        ]
                        .ref_to_vec_or_slice()
                    )
                ),
                typedef!(
                    "chunk_pos",
                    TypeDeclaration::Tuple(
                        [
                            TypeDeclaration::Alias(StringOrSlice::Sl("i32")),
                            TypeDeclaration::Alias(StringOrSlice::Sl("i32"))
                        ]
                        .ref_to_vec_or_slice()
                    )
                ),
                typedef!(
                    "pillar_height",
                    TypeDeclaration::Enumeration(vec![
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h76"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h79"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h82"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h85"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h88"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h91"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h94"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h97"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h100"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("h103"),
                            argtype: None,
                        }
                    ])
                ),
                typedef!(
                    "pillar_height_hint",
                    TypeDeclaration::Enumeration(vec![
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Unknown"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Small"),
                            argtype: None
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Medium"),
                            argtype: None
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Big"),
                            argtype: None
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("MediumSmall"),
                            argtype: None
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("MediumBig"),
                            argtype: None
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Exact"),
                            argtype: Some(TypeDeclaration::Alias(StringOrSlice::Sl(
                                "pillar_height"
                            )))
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Range"),
                            argtype: Some(TypeDeclaration::Tuple(
                                [
                                    TypeDeclaration::Alias(StringOrSlice::Sl("pillar_height")),
                                    TypeDeclaration::Alias(StringOrSlice::Sl("pillar_height")),
                                ]
                                .ref_to_vec_or_slice()
                            ))
                        }
                    ])
                ),
                typedef!(
                    "pillar_caged_status",
                    TypeDeclaration::Enumeration(vec![
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Caged"),
                            argtype: None,
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Uncaged"),
                            argtype: None
                        },
                        EnumerationConstructor {
                            name: StringOrSlice::Sl("Unknown"),
                            argtype: None
                        },
                    ])
                ),
            ],
            problems: vec![Box::new(PillarCrackingProblem::default())],
        };
        ext
    }
}

impl SimpleV0Extension for Extension {
    fn protocol_extension_info(&self) -> ProtocolExtensionInfo {
        ProtocolExtensionInfo {
            name: "mcseedcracker".ref_to_string_or_slice(),
            version: "0.1.0".ref_to_string_or_slice(),
            description: "The original McSeedCracker by AilPhaune.".ref_to_string_or_slice(),
            authors: [StringOrSlice::Sl("AilPhaune")].ref_to_vec_or_slice(),
            commands: [].ref_to_vec_or_slice(),
        }
    }

    fn list_extension_types<'a>(
        &'a self,
    ) -> VecOrSlice<'a, (StringOrSlice<'a>, TypeDeclaration<'a>)> {
        VecOrSlice::S(&self.types)
    }

    fn list_extension_problems<'a>(&'a self) -> VecOrSlice<'a, Box<dyn SimpleV0Problem>> {
        VecOrSlice::S(&self.problems)
    }

    fn get_problem<'a>(&'a self, name: &str) -> Option<Box<dyn SimpleV0Problem>> {
        match name {
            "pillar-seed-cracker" => Some(Box::new(PillarCrackingProblem::default())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PillarCrackingProblem {
    args: Vec<V0ProblemArgumentDescription<'static>>,
}

impl Default for PillarCrackingProblem {
    fn default() -> Self {
        let mut args = Vec::with_capacity(10);
        for i in 0..10 {
            args.push(V0ProblemArgumentDescription {
                name: StringOrSlice::St(format!("pillar{}height", i)),
                optional: true,
                argtype: StringOrSlice::Sl("pillar_height_hint"),
            });
            args.push(V0ProblemArgumentDescription {
                name: StringOrSlice::St(format!("pillar{}caged", i)),
                optional: true,
                argtype: StringOrSlice::Sl("pillar_caged_status"),
            });
        }
        Self { args }
    }
}

impl SimpleV0Problem for PillarCrackingProblem {
    fn name<'a>(&'a self) -> StringOrSlice<'a> {
        StringOrSlice::Sl("pillar-seed-cracker")
    }

    fn description<'a>(&'a self) -> StringOrSlice<'a> {
        StringOrSlice::Sl(
            "Cracks the seed used to generate the end pillars from information about their height and the positions of the cages. It can be used to reduce structure seed search from 2^48 down to 2^32 which is a 65536x decrease in the number of seeds to try.",
        )
    }

    fn args<'a>(&'a self) -> VecOrSlice<'a, V0ProblemArgumentDescription<'a>> {
        VecOrSlice::S(&self.args)
    }

    fn setup<'a>(
        &'a self,
        args: VecOrSlice<'a, (StringOrSlice<'a>, ParsetimeProtocolValue<'a>)>,
    ) -> Result<Box<dyn SimpleV0ProblemBruteCalculation>, ParsetimeProtocolValue<'a>> {
        macro_rules! inv_arg {
            () => {
                return Err(ParsetimeProtocolValue::String(StringOrSlice::Sl(
                    "Invalid argument name",
                )))
            };
        }

        macro_rules! inv_val {
            () => {
                return Err(ParsetimeProtocolValue::String(StringOrSlice::Sl(
                    "Invalid argument value",
                )))
            };
        }

        fn parse_height_value(v: &ParsetimeProtocolValue) -> Option<i32> {
            match v {
                ParsetimeProtocolValue::Enumeration(Some(n), _, _)
                    if n.as_slice() != "pillar_height" =>
                {
                    None
                }
                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h76" =>
                {
                    Some(76)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h79" =>
                {
                    Some(79)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h82" =>
                {
                    Some(82)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h85" =>
                {
                    Some(85)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h88" =>
                {
                    Some(88)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h91" =>
                {
                    Some(91)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h94" =>
                {
                    Some(94)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h97" =>
                {
                    Some(97)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h100" =>
                {
                    Some(100)
                }

                ParsetimeProtocolValue::Enumeration(_, constr, None)
                    if constr.as_slice() == "h103" =>
                {
                    Some(103)
                }
                _ => None,
            }
        }

        let mut partial = PartialEndPillars::default();
        for (argn, argv) in args.as_slice() {
            if argn.as_slice().get(0..6) != Some("pillar") {
                inv_arg!();
            }
            let pillar_index = match argn.as_slice().chars().nth(6) {
                Some(c) if c.is_ascii_digit() => c as u8 - b'0',
                _ => inv_arg!(),
            };
            match argn.as_slice().get(7..) {
                Some("height") => match argv {
                    ParsetimeProtocolValue::Enumeration(Some(n), _, _)
                        if n.as_slice() != "pillar_height_hint" =>
                    {
                        inv_val!()
                    }
                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "Small" =>
                    {
                        partial.0[pillar_index as usize].height = PillarHeightHint::Small;
                    }
                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "Medium" =>
                    {
                        partial.0[pillar_index as usize].height = PillarHeightHint::Medium;
                    }
                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "Big" =>
                    {
                        partial.0[pillar_index as usize].height = PillarHeightHint::Big;
                    }

                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "MediumSmall" =>
                    {
                        partial.0[pillar_index as usize].height = PillarHeightHint::MediumSmall;
                    }

                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "MediumBig" =>
                    {
                        partial.0[pillar_index as usize].height = PillarHeightHint::MediumBig;
                    }

                    ParsetimeProtocolValue::Enumeration(_, constr, Some(v))
                        if constr.as_slice() == "Exact" =>
                    {
                        let Some(h) = parse_height_value(v) else {
                            inv_val!()
                        };
                        partial.0[pillar_index as usize].height = PillarHeightHint::Exact(h);
                    }

                    ParsetimeProtocolValue::Enumeration(_, constr, Some(v))
                        if constr.as_slice() == "Range" =>
                    {
                        match &**v {
                            ParsetimeProtocolValue::Tuple(None, vals) if vals.len() == 2 => {
                                let Some(a) = parse_height_value(&vals[0]) else {
                                    inv_val!()
                                };
                                let Some(b) = parse_height_value(&vals[0]) else {
                                    inv_val!()
                                };
                                partial.0[pillar_index as usize].height =
                                    PillarHeightHint::Range(a, b);
                            }
                            _ => inv_val!(),
                        }
                    }
                    _ => inv_val!(),
                },
                Some("caged") => match argv {
                    ParsetimeProtocolValue::Enumeration(Some(n), _, _)
                        if n.as_slice() != "pillar_caged_status" =>
                    {
                        inv_val!()
                    }
                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "Caged" =>
                    {
                        partial.0[pillar_index as usize].caged = Some(true);
                    }
                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "Uncaged" =>
                    {
                        partial.0[pillar_index as usize].caged = Some(false);
                    }
                    ParsetimeProtocolValue::Enumeration(_, constr, None)
                        if constr.as_slice() == "Unknown" =>
                    {
                        partial.0[pillar_index as usize].caged = None;
                    }
                    _ => inv_val!(),
                },
                _ => inv_arg!(),
            }
        }

        Ok(Box::new(PillarCrackingProblemComputation {
            pillars: partial,
            done: false,
            seeds: Vec::new(),
        }))
    }
}

#[derive(Debug)]
pub struct PillarCrackingProblemComputation {
    pillars: PartialEndPillars,
    seeds: Vec<(i64, PillarMatchResult)>,
    done: bool,
}

impl SimpleV0ProblemBruteCalculation for PillarCrackingProblemComputation {
    fn go(&mut self, output: &mut dyn Write) {
        self.seeds = self.pillars.seed_results();
        self.done = true;

        match self.result() {
            Some(result) => {
                let mut line = b"result ".iter().copied().collect::<Vec<_>>();
                v0_format_value(&mut line, &result).unwrap();
                line.push(b'\n');
                output.write(&line).unwrap();
            }
            None => {
                output.write(b"result\n").unwrap();
            }
        }
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn is_running(&self) -> bool {
        false
    }

    fn progress(&self) -> (u64, u64) {
        (if self.done { 65536 } else { 0 }, 65536)
    }

    fn stop(&mut self) {}

    fn result(&self) -> Option<ParsetimeProtocolValue<'_>> {
        if self.done {
            let values = self
                .seeds
                .iter()
                .filter(|(_, r)| !r.is_impossible_match())
                .map(|(s, r)| {
                    ParsetimeProtocolValue::Tuple(
                        None,
                        vec![
                            ParsetimeProtocolValue::I64(*s),
                            ParsetimeProtocolValue::F64(r.chance()),
                        ],
                    )
                })
                .collect::<Vec<_>>();
            Some(ParsetimeProtocolValue::List(None, values))
        } else {
            None
        }
    }
}
