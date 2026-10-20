pub mod api;
mod argtheme;
pub mod cli;
mod csvread;
pub mod decimal;
pub mod facts;
pub mod gate;
pub mod money;
pub mod myr;
pub mod py;
mod pyjson;
mod pytext;
mod pyunicode;
pub mod rates;
pub mod raw;
pub mod register;
mod tables;
mod termsize;
mod textwrap;
pub mod years;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The date the compiled Act and the benchmark rate table were last read.
pub const LAW_CONTENT_DATE: &str = "2026-08-31";
/// The compilation of ITAA 1936 this engine was written against.
pub const LAW_COMPILATION: &str = "C1936A00027, compilation in force 1 July 2026";

/// Why an operation stopped, as the Python engine reported it.
#[derive(Clone, Debug, PartialEq)]
pub enum Failure {
    /// A `ValueError` the engine raised on purpose (year, money, fact, rates
    /// or register): the command line prints `error: <message>`.
    Value(String),
    /// A trapped decimal signal, which Python raised uncaught.
    Signal(decimal::Signal),
    /// Any other uncaught exception, as the last line of its traceback.
    Uncaught(String),
}

impl From<decimal::Signal> for Failure {
    fn from(signal: decimal::Signal) -> Failure {
        Failure::Signal(signal)
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Value(message) | Failure::Uncaught(message) => f.write_str(message),
            Failure::Signal(signal) => write!(f, "decimal.{}", signal.name()),
        }
    }
}
