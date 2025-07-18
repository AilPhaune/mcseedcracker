use std::{
    borrow::Borrow,
    io::{BufRead, Write},
    sync::{Arc, Mutex},
};

use chumsky::{Parser, error::Rich};

use crate::{
    McSeedCrackingProtocol,
    traits::{CharsIter, IntoCharsIter, StringOrSlice, VecOrSlice},
    v0::{
        self, ParsetimeProtocolValue, ProtocolVersion0, SimpleV0Extension,
        SimpleV0ProblemBruteCalculation,
    },
};

pub enum ProtocolCommand<'a> {
    V0(v0::ProtocolCommand<'a>),
}

pub enum ProtocolResponse<'a> {
    V0(v0::ProtocolResponse<'a>),
}

#[derive(Default, Debug)]
pub struct MCSCIProtocol {
    helloed: bool,

    extensions: Vec<Box<dyn SimpleV0Extension>>,

    current_pb: Option<Box<dyn SimpleV0ProblemBruteCalculation>>,
}

#[derive(Clone)]
struct ArcMutexWriter<'a> {
    inner: Arc<Mutex<&'a mut dyn Write>>,
}

impl<'a> ArcMutexWriter<'a> {
    pub fn new(writer: &'a mut dyn Write) -> Self {
        Self {
            inner: Arc::new(Mutex::new(writer)),
        }
    }
}

impl<'a> Write for ArcMutexWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}

impl MCSCIProtocol {
    pub fn response_info<T>(&self, info: impl IntoCharsIter<T>) -> ProtocolResponse<'_>
    where
        CharsIter<T>: IntoIterator<Item = char>,
    {
        ProtocolResponse::V0(v0::ProtocolResponse::Info(
            info.chars_iter().into_iter().collect::<String>().into(),
        ))
    }

    pub fn response_infos<'a, ElementType, ArrayType>(
        &self,
        infos: ArrayType,
    ) -> ProtocolResponse<'_>
    where
        ArrayType: IntoIterator<Item = &'a ElementType>,
        ElementType: ?Sized + 'a,
        String: From<&'a ElementType>,
        <ArrayType as IntoIterator>::Item: 'a,
    {
        let r = infos
            .into_iter()
            .map(|e| String::from(e.borrow()).into())
            .collect::<Vec<_>>();

        ProtocolResponse::V0(v0::ProtocolResponse::Infos(r))
    }

    pub fn acknowledge(&self) -> ProtocolResponse {
        ProtocolResponse::V0(v0::ProtocolResponse::Acknowledge)
    }

    pub fn version_response(&self, server_version: Option<String>) -> ProtocolResponse {
        ProtocolResponse::V0(v0::ProtocolResponse::Version {
            protocol_version: self.protocol_version(),
            server_version: server_version.map(Into::into),
        })
    }

    pub fn setup_problem_err<T>(&self, err: impl IntoCharsIter<T>) -> ProtocolResponse
    where
        CharsIter<T>: IntoIterator<Item = char>,
    {
        ProtocolResponse::V0(v0::ProtocolResponse::SetupError(
            ParsetimeProtocolValue::String(StringOrSlice::St(
                err.chars_iter().into_iter().collect::<String>(),
            )),
        ))
    }

    pub fn setup_problem_err_value<'a>(
        &self,
        err: ParsetimeProtocolValue<'a>,
    ) -> ProtocolResponse<'a> {
        ProtocolResponse::V0(v0::ProtocolResponse::SetupError(err))
    }

    pub fn unexpected_opt<T>(&self, info: Option<impl IntoCharsIter<T>>) -> ProtocolResponse
    where
        CharsIter<T>: IntoIterator<Item = char>,
    {
        ProtocolResponse::V0(v0::ProtocolResponse::Unexpected(
            info.map(|i| i.chars_iter().into_iter().collect::<String>().into()),
        ))
    }

    pub fn unexpected_none(&self) -> ProtocolResponse {
        ProtocolResponse::V0(v0::ProtocolResponse::Unexpected(None))
    }

    pub fn unexpected<T>(&self, info: impl IntoCharsIter<T>) -> ProtocolResponse
    where
        CharsIter<T>: IntoIterator<Item = char>,
    {
        ProtocolResponse::V0(v0::ProtocolResponse::Unexpected(Some(
            info.chars_iter().into_iter().collect::<String>().into(),
        )))
    }

    pub fn no_such_extension_response(&self, ext: u32) -> ProtocolResponse {
        ProtocolResponse::V0(v0::ProtocolResponse::NoSuchExtension(ext))
    }

    pub fn extension_types_response<'a>(
        &self,
        extid: u32,
        ext: &'a dyn SimpleV0Extension,
    ) -> ProtocolResponse<'a> {
        ProtocolResponse::V0(v0::ProtocolResponse::TypeList {
            extension: extid,
            types: ext.list_extension_types(),
        })
    }

    pub fn extension_problems_response<'a>(
        &self,
        extid: u32,
        ext: &'a dyn SimpleV0Extension,
    ) -> ProtocolResponse<'a> {
        ProtocolResponse::V0(v0::ProtocolResponse::ProblemList {
            extension: extid,
            problems: ext.list_extension_problems(),
        })
    }

    pub fn server_loop(
        mut self,
        input: &mut impl BufRead,
        output: &mut impl Write,
        errout: &mut impl Write,
    ) -> Result<(), std::io::Error> {
        let output = &mut ArcMutexWriter::new(output);

        let mut line = String::new();

        loop {
            line.clear();
            if input.read_line(&mut line).unwrap() == 0 {
                // EOF
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let command = match self.parse_command(&line) {
                Ok(command) => command,
                Err(errors) => {
                    for error in errors {
                        writeln!(errout, "{}", error)?;
                    }
                    self.format_response(
                        output,
                        &ProtocolResponse::V0(v0::ProtocolResponse::ParseFail),
                    )?;
                    continue;
                }
            };

            match command {
                ProtocolCommand::V0(command) => match command {
                    _ if self.current_pb.as_ref().is_some_and(|v| v.is_running())
                        && !matches!(command, v0::ProtocolCommand::Stop) =>
                    {
                        self.format_response(output, &self.unexpected("computation running"))?
                    }
                    v0::ProtocolCommand::Hello => {
                        self.client_did_hello();
                        self.format_response(output, &self.acknowledge())?;
                    }
                    _ if !self.client_helloed() => {
                        self.format_response(output, &self.unexpected("not initialized"))?
                    }
                    v0::ProtocolCommand::Help => {
                        self.format_response(output, &self.acknowledge())?;
                        self.format_response(
                        output,
                        &self.response_infos([
                            "Help",
                            "quit: Quits the program",
                            "help: Prints this help message",
                            "version: Prints the version",
                            "setup-problem <problem name string> [args: <name>=<typed value>]+: Sets up the server to handle a computation problem with the given name and arguments",
                        ]),
                    )?;
                    }
                    v0::ProtocolCommand::Quit => {
                        self.format_response(output, &self.acknowledge())?;
                        break;
                    }
                    v0::ProtocolCommand::Version => {
                        self.format_response(output, &self.acknowledge())?;
                        self.format_response(output, &self.version_response(None))?;
                    }
                    v0::ProtocolCommand::SetupProblem {
                        name,
                        args,
                        extension,
                    } => {
                        self.format_response(output, &self.acknowledge())?;
                        match self.extensions.get(extension as usize) {
                            None => {
                                self.format_response(
                                    output,
                                    &self.no_such_extension_response(extension),
                                )?;
                            }
                            Some(ext) => {
                                let Some(pb) = ext.get_problem(name.as_slice()) else {
                                    self.format_response(
                                        output,
                                        &self.setup_problem_err("No such problem"),
                                    )?;
                                    continue;
                                };
                                match pb.setup(VecOrSlice::V(args)) {
                                    Ok(res) => {
                                        self.current_pb = Some(res);
                                        self.format_response(
                                            output,
                                            &ProtocolResponse::V0(v0::ProtocolResponse::SetupOk),
                                        )?;
                                    }
                                    Err(err) => {
                                        self.format_response(
                                            output,
                                            &self.setup_problem_err_value(err),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                    v0::ProtocolCommand::ListTypes { extension } => {
                        self.format_response(output, &self.acknowledge())?;
                        match self.extensions.get(extension as usize) {
                            None => {
                                self.format_response(
                                    output,
                                    &self.no_such_extension_response(extension),
                                )?;
                            }
                            Some(ext) => {
                                self.format_response(
                                    output,
                                    &self.extension_types_response(extension, &**ext),
                                )?;
                            }
                        }
                    }
                    v0::ProtocolCommand::ListProblems { extension } => {
                        self.format_response(output, &self.acknowledge())?;
                        match self.extensions.get(extension as usize) {
                            None => {
                                self.format_response(
                                    output,
                                    &self.no_such_extension_response(extension),
                                )?;
                            }
                            Some(ext) => {
                                self.format_response(
                                    output,
                                    &self.extension_problems_response(extension, &**ext),
                                )?;
                            }
                        }
                    }
                    v0::ProtocolCommand::Extensions => {
                        self.format_response(output, &self.acknowledge())?;
                        self.format_response(
                            output,
                            &ProtocolResponse::V0(v0::ProtocolResponse::Extensions {
                                count: self.extensions.len() as u32,
                                extensions: if self.extensions.is_empty() {
                                    None
                                } else {
                                    Some(
                                        self.extensions
                                            .iter()
                                            .map(|ext| ext.protocol_extension_info())
                                            .collect::<Vec<_>>(),
                                    )
                                },
                            }),
                        )?
                    }
                    v0::ProtocolCommand::Go => {
                        if let Some(pb) = &mut self.current_pb {
                            pb.go(output);
                        } else {
                            self.format_response(output, &self.unexpected("no problem to solve"))?
                        }
                    }
                    v0::ProtocolCommand::Stop => {
                        if let Some(pb) = &mut self.current_pb {
                            pb.stop();
                        } else {
                            self.format_response(
                                output,
                                &self.unexpected("no computation to stop"),
                            )?
                        }
                    }
                },
            };
            output.flush()?;
        }
        Ok(())
    }

    pub fn register_extension(&mut self, extension: impl SimpleV0Extension + 'static) {
        self.extensions.push(Box::new(extension));
    }
}

impl ProtocolVersion0 for MCSCIProtocol {
    fn client_did_hello(&mut self) {
        self.helloed = true;
    }

    fn client_helloed(&self) -> bool {
        self.helloed
    }
}

impl McSeedCrackingProtocol for MCSCIProtocol {
    type Command<'a> = ProtocolCommand<'a>;
    type ParseError<'a> = Vec<Rich<'a, char>>;
    type Response<'a> = ProtocolResponse<'a>;

    fn protocol_version(&self) -> i32 {
        0
    }

    fn format_response<'a>(
        &self,
        writer: &mut dyn std::io::Write,
        response: &Self::Response<'a>,
    ) -> Result<(), std::io::Error> {
        match response {
            ProtocolResponse::V0(response) => v0::format_response(writer, response),
        }
    }

    fn parse_command<'a>(&self, line: &'a str) -> Result<Self::Command<'a>, Self::ParseError<'a>> {
        v0::full_v0_parser::<'a>()
            .map(ProtocolCommand::V0)
            .parse(line)
            .into_result()
    }
}
