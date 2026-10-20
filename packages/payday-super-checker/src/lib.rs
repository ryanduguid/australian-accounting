//! payday-super-checker: payday-super deadline and SG-charge exposure checker.
//!
//! Educational tool. Not legal, tax or financial advice. Verify outcomes
//! against the ATO's own materials before acting.

pub mod api;
mod argtheme;
pub mod assess;
pub mod atomic;
pub mod calendar;
pub mod cli;
mod compose;
pub mod csv_io;
mod csvread;
mod csvwrite;
pub mod dates;
pub mod deadlines;
pub mod decimal;
pub mod derivation;
pub mod evidence_pack;
#[doc(hidden)]
pub mod fuzzing;
pub mod importers;
pub mod join;
pub mod practitioner_pack;
pub mod profiles;
pub mod py;
mod pyjson;
pub mod pyre;
mod pytext;
mod pyunicode;
pub mod rates;
mod raw;
pub mod report;
pub mod sgc;
mod strptime_table;
mod tables;
mod termsize;
mod textwrap;
pub mod unicode;
#[allow(dead_code)]
mod unitables;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The day the legal content was last reviewed.
pub const LAW_CONTENT_DATE: &str = "2026-08-15";

/// An `OSError` as Python held it: the errno that picks its subclass, the
/// message, and the file it names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsError {
    pub errno: i32,
    /// `exc.strerror`.
    pub strerror: String,
    pub filename: Option<String>,
    /// `str(exc)` as raised, which on Windows can name a `WinError`.
    pub text: String,
}

impl OsError {
    /// The `OSError` Python raised for `e` on `filename`. `Errno` style is
    /// what `open()` raises through the C runtime; `Native` is a Win32 call
    /// such as `os.replace`, which on Windows names its `WinError`.
    pub fn from_io(e: &std::io::Error, filename: &str, style: py::OsStyle) -> OsError {
        let text = py::oserror_str(e, filename, style);
        let Some(code) = e.raw_os_error() else {
            return OsError {
                errno: 0,
                strerror: String::new(),
                filename: Some(filename.to_string()),
                text,
            };
        };
        let (errno, strerror) = py::errno_and_strerror(e, code, style);
        OsError {
            errno,
            strerror,
            filename: Some(filename.to_string()),
            text,
        }
    }

    /// Whether Python raised `FileNotFoundError` (ENOENT).
    pub fn not_found(&self) -> bool {
        self.errno == 2
    }

    /// `str(exc)`.
    pub fn display(&self) -> String {
        self.text.clone()
    }

    /// `OSError(exc.errno, exc.strerror or str(exc), filename)`: the same
    /// failure re-raised against another file.
    pub fn renamed(&self, filename: &str) -> OsError {
        let strerror = if self.strerror.is_empty() {
            self.text.clone()
        } else {
            self.strerror.clone()
        };
        OsError {
            errno: self.errno,
            text: format!(
                "[Errno {}] {strerror}: {}",
                self.errno,
                pytext::repr_os(filename)
            ),
            strerror,
            filename: Some(filename.to_string()),
        }
    }
}

/// Why an operation stopped, as the Python engine reported it.
#[derive(Clone, Debug, PartialEq)]
pub enum Failure {
    /// A `ValueError` (CsvError, CalendarError, RatesError, PreRegimeError,
    /// PractitionerPackError, UnicodeDecodeError and plain ValueError), which
    /// the command line prints as `error: <message>`.
    Value(String),
    /// `StaleGicError`: a day past the last GIC quarter, asked for without
    /// the stale-rate estimate. A `RatesError` the assessment can withhold.
    StaleGic(String),
    /// A trapped decimal signal (an `ArithmeticError`).
    Signal(decimal::Signal),
    /// `OverflowError("date value out of range")` from date arithmetic.
    Overflow,
    /// Any other `ArithmeticError`, named by its class, such as
    /// `OverflowError: cannot convert float infinity to integer`.
    Arithmetic {
        class: &'static str,
        message: String,
    },
    Os(OsError),
    /// Any other exception, which Python left uncaught: its traceback's last line.
    Uncaught(String),
}

impl From<decimal::Signal> for Failure {
    fn from(signal: decimal::Signal) -> Failure {
        Failure::Signal(signal)
    }
}

impl From<dates::Overflow> for Failure {
    fn from(_: dates::Overflow) -> Failure {
        Failure::Overflow
    }
}

impl Failure {
    /// `str(exc)`.
    pub fn message(&self) -> String {
        match self {
            Failure::Value(m) | Failure::StaleGic(m) | Failure::Uncaught(m) => m.clone(),
            Failure::Signal(s) => format!("[<class 'decimal.{}'>]", s.name()),
            Failure::Overflow => "date value out of range".into(),
            Failure::Arithmetic { message, .. } => message.clone(),
            Failure::Os(e) => e.display(),
        }
    }

    /// Whether the exception is a `ValueError` or an `ArithmeticError`.
    pub fn is_value_or_arithmetic(&self) -> bool {
        matches!(
            self,
            Failure::Value(_)
                | Failure::StaleGic(_)
                | Failure::Signal(_)
                | Failure::Overflow
                | Failure::Arithmetic { .. }
        )
    }

    /// The last line of the traceback Python printed for it, uncaught.
    pub fn traceback_line(&self) -> String {
        match self {
            Failure::Value(m) | Failure::StaleGic(m) => format!("ValueError: {m}"),
            Failure::Signal(s) => format!("decimal.{}", s.name()),
            Failure::Overflow => "OverflowError: date value out of range".into(),
            Failure::Arithmetic { class, message } => format!("{class}: {message}"),
            Failure::Os(e) => format!("OSError: {}", e.display()),
            Failure::Uncaught(m) => m.clone(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Failure>;

/// `repr(list_of_str)`.
pub fn repr_list<S: AsRef<str>>(items: &[S]) -> String {
    let parts: Vec<String> = items.iter().map(|s| pytext::repr_os(s.as_ref())).collect();
    format!("[{}]", parts.join(", "))
}

/// A Python dict of lists keyed by string: groups in first-appearance
/// order, found by hash.
pub struct Groups<T> {
    pub entries: Vec<(String, Vec<T>)>,
    at: std::collections::HashMap<String, usize>,
}

impl<T> Default for Groups<T> {
    fn default() -> Self {
        Groups {
            entries: Vec::new(),
            at: std::collections::HashMap::new(),
        }
    }
}

impl<T> Groups<T> {
    pub fn push(&mut self, key: String, item: T) {
        match self.at.get(&key) {
            Some(&i) => self.entries[i].1.push(item),
            None => {
                self.at.insert(key.clone(), self.entries.len());
                self.entries.push((key, vec![item]));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<T>> {
        self.at.get(key).map(|&i| &self.entries[i].1)
    }
}

/// `sorted(strings)`: Python orders `str` by code point.
pub fn sorted_strings<S: AsRef<str>>(items: impl IntoIterator<Item = S>) -> Vec<String> {
    let mut out: Vec<String> = items.into_iter().map(|s| s.as_ref().to_string()).collect();
    out.sort_by(|a, b| pytext::code_points(a).cmp(pytext::code_points(b)));
    out
}
