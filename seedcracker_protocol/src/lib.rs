pub mod current_impl;
pub mod traits;
pub mod v0;

pub trait McSeedCrackingProtocol {
    type Command<'a>;
    type ParseError<'a>;
    type Response<'a>;

    fn parse_command<'a>(&self, line: &'a str) -> Result<Self::Command<'a>, Self::ParseError<'a>>;
    fn format_response<'a>(
        &self,
        writer: &mut dyn std::io::Write,
        response: &Self::Response<'a>,
    ) -> Result<(), std::io::Error>;
    fn protocol_version(&self) -> i32;
}
