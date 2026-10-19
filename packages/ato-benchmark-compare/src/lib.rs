pub mod api;
mod argtheme;
pub mod atomic;
pub mod cli;
mod csvread;
pub mod csvsafe;
pub mod dataset;
pub mod decimal;
pub mod mapping;
pub mod money;
pub mod pnl;
pub mod py;
mod pyjson;
pub mod pyre;
mod pytext;
mod pyunicode;
pub mod ratios;
pub mod raw;
pub mod report;
mod tables;
mod termsize;
mod textwrap;
pub mod unicode;
mod unitables;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Why an operation stopped, as the Python engine reported it.
#[derive(Clone, Debug, PartialEq)]
pub enum Failure {
    /// An error the command line reports as `error: <message>`: DatasetError,
    /// MappingError, PnlError, RatioError, AmountError or an OSError.
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
