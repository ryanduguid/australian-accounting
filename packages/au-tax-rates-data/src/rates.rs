//! `rates snapshot | validate | check | ato-tables`: `rates.py` at
//! australian-accounting 24fcf24, ported with its output, exit codes and
//! file bytes.
//!
//! The records are JSON read from `data/`, so every step follows what Python
//! does with whatever a record holds: the same problems for a malformed
//! record that the checks report, and the same exception, by class and
//! message, where Python's run stops on one. A stopped run prints Python's
//! last traceback line to stderr and exits 1; the frames above it are not
//! reproduced. Python strings are handled as code points, so a lone
//! surrogate from a JSON escape is kept, printed with `surrogateescape` and
//! refused by a strict UTF-8 write, as Python does.
//!
//! With the `differential` feature, `RATES_ROOT` replaces the package folder,
//! `RATES_BROWSER` (a JSON list of strings) the nodriver command,
//! `RATES_TODAY` the local date, `RATES_ATO_TABLES` a folder of
//! `<table>.json` bodies (with optional `<table>.status` and `<table>.error`)
//! the ATO's server, and `RATES_NO_SLEEP` the 2-second pause between pages.
use std::cmp::Ordering;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use num_bigint::BigInt;
use num_traits::Zero;

use crate::argparse::{Action, Exit, Parser, Value};
use crate::decimal::{Conversion, Decimal, Rounding, Signal};
use crate::py::{self, Date};
use crate::pyjson::{self, DecodeError, Json, PyStr};
use crate::pyos::{self, OsError};
use crate::pypath::{PyPath, SEP};
use crate::pyvalue::{self, Key, item_eq, py_eq, repr, repr_points, str_points, type_name};
use crate::sre::{self, Pattern};

const DOC: &str = "Snapshot, validate and re-check the AU tax rates dataset.

rates snapshot     fetch every source page into snapshots/
rates validate     check data/*.json against the schema and snapshots
rates check        re-fetch the pages and write a Markdown change report
rates ato-tables   compare figures with the tables the ATO's calculators read
";

const NODRIVER: [&str; 2] = [
    r"C:\Tools\nodriver-browser\.venv\Scripts\python.exe",
    r"C:\Tools\nodriver-browser\server.py",
];

const FIELDS: [(&str, bool); 7] = [
    ("id", false),
    ("label", false),
    ("unit", false),
    ("period", true),
    ("source_url", false),
    ("retrieved_at", false),
    ("quote", false),
];
const UNITS: [&str; 6] = [
    "AUD",
    "percent",
    "cents_per_km",
    "factor",
    "date",
    "tax_scale",
];
const PERIOD_KEYS: [&str; 4] = ["as_at", "effective_from", "effective_to", "income_year"];

const ATO_HOST: &str = "onlineservices.ato.gov.au";
/// Columns per table: lower bound, upper bound, base amount, rate.
const ATO_TABLES: [(&str, [Option<&str>; 4]); 2] = [
    (
        "TC2TAXRTE",
        [
            Some("AM_INC_MIN"),
            Some("AM_INC_MAX"),
            Some("AM_OFFSET"),
            Some("PC_TAX_RATE"),
        ],
    ),
    (
        "TC9GENTAC",
        [
            Some("AM_MIN_INCOME"),
            Some("AM_MAX_INCOME"),
            None,
            Some("PC_TAX_RATE_A"),
        ],
    ),
];

/// How a record's figure is read from the rows in force.
#[derive(Clone, Copy)]
enum Reader {
    List,
    LastRate,
    FirstTop,
    SecondTop,
    OnlyRate,
}

/// Record label: (table, the column and value a row must have, the reader).
const ATO_TABLE_FIGURES: [(&str, &str, (&str, &str), Reader); 7] = [
    (
        "Resident individual income tax rates",
        "TC2TAXRTE",
        ("TX_DESC", "Resident Full Year"),
        Reader::List,
    ),
    (
        "Medicare levy rate",
        "TC2TAXRTE",
        ("TX_DESC", "Medicare Basic Levy"),
        Reader::LastRate,
    ),
    (
        "Medicare levy low-income lower threshold, single, not SAPTO",
        "TC2TAXRTE",
        ("TX_DESC", "Medicare Basic Levy"),
        Reader::FirstTop,
    ),
    (
        "Medicare levy low-income upper threshold, single, not SAPTO",
        "TC2TAXRTE",
        ("TX_DESC", "Medicare Basic Levy"),
        Reader::SecondTop,
    ),
    (
        "Fringe benefits tax rate",
        "TC2TAXRTE",
        ("TX_DESC", "FBT rate"),
        Reader::OnlyRate,
    ),
    (
        "Super guarantee percentage (general)",
        "TC2TAXRTE",
        ("TX_DESC", "Superannuation Guarantee Rate"),
        Reader::OnlyRate,
    ),
    (
        "Division 7A benchmark interest rate",
        "TC9GENTAC",
        ("NM_CALCN_TYPE", "DIV7A"),
        Reader::OnlyRate,
    ),
];

static INCOME_YEAR: LazyLock<Pattern> = LazyLock::new(|| sre::compile_str(r"^\d{4}-\d{2}$", 0));
static QC_LINE: LazyLock<Pattern> = LazyLock::new(|| sre::compile_str(r"QC\s?\d+", 0));
/// `glob.translate("*.json", include_hidden=True)`, case-insensitive on Windows.
static JSON_NAME: LazyLock<Pattern> = LazyLock::new(|| {
    if cfg!(windows) {
        sre::compile_str(r"(?s:[^\\]*\.json)\z", sre::IGNORECASE)
    } else {
        sre::compile_str(r"(?s:[^/]*\.json)\z", 0)
    }
});
static NUMBER: LazyLock<[(&str, Pattern); 4]> = LazyLock::new(|| {
    [
        ("AUD", r"^\$?([\d,]+(?:\.\d+)?)$"),
        ("percent", r"^([\d.]+)\s?%?$"),
        ("factor", r"^(\d+(?:\.\d+)?)$"),
        (
            "cents_per_km",
            r"^([\d.]+)\s?(?:c|cents)(?: per (?:kilometre|km))?$",
        ),
    ]
    .map(|(unit, pattern)| (unit, sre::compile_str(pattern, 0)))
});

// ---------------------------------------------------------------- Python values

/// Text as Python's `str` holds it.
type S = Vec<u32>;

trait Pts {
    fn put(&self, out: &mut S);
}

impl Pts for str {
    fn put(&self, out: &mut S) {
        out.extend(self.chars().map(u32::from));
    }
}

impl Pts for String {
    fn put(&self, out: &mut S) {
        self.as_str().put(out);
    }
}

impl Pts for [u32] {
    fn put(&self, out: &mut S) {
        out.extend_from_slice(self);
    }
}

impl Pts for Vec<u32> {
    fn put(&self, out: &mut S) {
        out.extend_from_slice(self);
    }
}

impl<T: Pts + ?Sized> Pts for &T {
    fn put(&self, out: &mut S) {
        (**self).put(out);
    }
}

/// Concatenate text pieces into a `str`.
macro_rules! p {
    ($($e:expr),* $(,)?) => {{
        let mut out: S = Vec::new();
        $( Pts::put(&$e, &mut out); )*
        out
    }};
}

fn pts(text: &str) -> S {
    text.chars().map(u32::from).collect()
}

fn json_str(points: S) -> Json {
    Json::Str(PyStr::from_code_points(points))
}

fn is_space(cp: u32) -> bool {
    char::from_u32(cp).is_some_and(py::is_space)
}

/// `str.strip()`.
fn strip(text: &[u32]) -> &[u32] {
    let start = text
        .iter()
        .position(|&c| !is_space(c))
        .unwrap_or(text.len());
    let end = text
        .iter()
        .rposition(|&c| !is_space(c))
        .map_or(start, |i| i + 1);
    &text[start..end]
}

/// `re.sub(r"\s+", " ", text).strip()`.
fn norm(text: &[u32]) -> S {
    let mut out = Vec::with_capacity(text.len());
    let mut in_space = false;
    for &cp in text {
        if is_space(cp) {
            if !in_space {
                out.push(u32::from(' '));
            }
            in_space = true;
        } else {
            out.push(cp);
            in_space = false;
        }
    }
    strip(&out).to_vec()
}

/// `re.sub(r"[^\S\n\t]", " ", text)`.
fn plain_spaces(text: &[u32]) -> S {
    text.iter()
        .map(|&cp| {
            if is_space(cp) && cp != 10 && cp != 9 {
                32
            } else {
                cp
            }
        })
        .collect()
}

/// `needle in haystack` for strs.
fn contains(haystack: &[u32], needle: &[u32]) -> bool {
    needle.is_empty() || haystack.windows(needle.len()).any(|w| w == needle)
}

/// `str.splitlines()`.
fn splitlines(text: &[u32]) -> Vec<&[u32]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < text.len() {
        let cp = text[i];
        if matches!(
            cp,
            10 | 11 | 12 | 13 | 0x1c | 0x1d | 0x1e | 0x85 | 0x2028 | 0x2029
        ) {
            lines.push(&text[start..i]);
            i += if cp == 13 && text.get(i + 1) == Some(&10) {
                2
            } else {
                1
            };
            start = i;
        } else {
            i += 1;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

fn points_of(value: &PyStr) -> S {
    value.code_points()
}

/// `int(digits)` for a run of Unicode decimal digits.
fn int_of(digits: &[u32]) -> BigInt {
    digits
        .iter()
        .filter_map(|&cp| char::from_u32(cp).and_then(py::decimal_value))
        .fold(BigInt::zero(), |n, d| n * 10 + d)
}

/// A Python exception: its class, qualified as a traceback names it, and its `str()`.
#[derive(Clone, Debug)]
pub struct Exc {
    pub class: &'static str,
    pub text: S,
}

type R<T> = Result<T, Exc>;

impl Exc {
    fn new(class: &'static str, text: S) -> Exc {
        Exc { class, text }
    }

    fn msg(class: &'static str, text: &str) -> Exc {
        Exc::new(class, pts(text))
    }

    /// `isinstance(self, base)` for the classes the tool meets.
    fn is(&self, base: &str) -> bool {
        let class = self.class;
        class == base
            || match base {
                "ValueError" => matches!(
                    class,
                    "UnicodeDecodeError" | "UnicodeEncodeError" | "json.decoder.JSONDecodeError"
                ),
                "OSError" => matches!(
                    class,
                    "FileNotFoundError"
                        | "PermissionError"
                        | "IsADirectoryError"
                        | "NotADirectoryError"
                        | "FileExistsError"
                        | "BrokenPipeError"
                        | "TimeoutError"
                ),
                "RuntimeError" => class == "RecursionError",
                "subprocess.SubprocessError" => class == "subprocess.TimeoutExpired",
                _ => false,
            }
    }

    fn is_any(&self, bases: &[&str]) -> bool {
        bases.iter().any(|base| self.is(base))
    }

    /// `type(self).__name__`.
    fn name(&self) -> &'static str {
        self.class.rsplit('.').next().unwrap_or(self.class)
    }
}

fn type_error(text: &str) -> Exc {
    Exc::msg("TypeError", text)
}

fn value_error(text: S) -> Exc {
    Exc::new("ValueError", text)
}

fn key_error(key: &str) -> Exc {
    Exc::msg("KeyError", &py::repr_str(key))
}

fn os_exc(error: &OsError) -> Exc {
    Exc::msg(error.class(), &error.text)
}

fn decimal_signal(signal: Signal) -> Exc {
    match signal {
        Signal::InvalidOperation => Exc::msg(
            "decimal.InvalidOperation",
            "[<class 'decimal.InvalidOperation'>]",
        ),
        Signal::DivisionByZero => Exc::msg(
            "decimal.DivisionByZero",
            "[<class 'decimal.DivisionByZero'>]",
        ),
        Signal::Overflow => Exc::msg("decimal.Overflow", "[<class 'decimal.Overflow'>]"),
    }
}

fn conversion(error: Conversion) -> Exc {
    Exc::msg("decimal.InvalidOperation", error.text())
}

/// `value[key]` for a str key.
fn getitem<'a>(value: &'a Json, key: &str) -> R<&'a Json> {
    match value {
        Json::Object(_) => value.get(key).ok_or_else(|| key_error(key)),
        Json::Array(_) => Err(type_error(
            "list indices must be integers or slices, not str",
        )),
        Json::Str(_) => Err(type_error("string indices must be integers, not 'str'")),
        other => Err(type_error(&format!(
            "'{}' object is not subscriptable",
            type_name(other)
        ))),
    }
}

/// `key in value` for a dict.
fn has(value: &Json, key: &str) -> bool {
    value.get(key).is_some()
}

/// `value.get(key)`, which only a dict has.
fn get_attr<'a>(value: &'a Json, key: &str) -> R<Option<&'a Json>> {
    match value {
        Json::Object(_) => Ok(value.get(key)),
        other => Err(Exc::msg(
            "AttributeError",
            &format!("'{}' object has no attribute 'get'", type_name(other)),
        )),
    }
}

/// `list(value)`: what iterating it yields.
fn iterate(value: &Json) -> R<Vec<Json>> {
    match value {
        Json::Array(items) => Ok(items.clone()),
        Json::Object(pairs) => Ok(pairs.iter().map(|(k, _)| Json::Str(k.clone())).collect()),
        Json::Str(s) => Ok(s.points().map(|cp| json_str(vec![cp])).collect()),
        other => Err(type_error(&format!(
            "'{}' object is not iterable",
            type_name(other)
        ))),
    }
}

/// `hash(value)` as a dict or set key, or the `TypeError` naming the container.
fn hash_key(value: &Json, container: &str) -> R<Key> {
    pyvalue::key(value).map_err(|_| {
        let t = type_name(value);
        type_error(&format!(
            "cannot use '{t}' as a {container} (unhashable type: '{t}')"
        ))
    })
}

/// `hash((a, b))` as a set element.
fn tuple_key(items: &[&Json]) -> R<Key> {
    let mut keys = Vec::new();
    for item in items {
        keys.push(pyvalue::key(item).map_err(|_| {
            type_error(&format!(
                "cannot use 'tuple' as a set element (unhashable type: '{}')",
                type_name(item)
            ))
        })?);
    }
    Ok(Key::Tuple(keys))
}

fn repr_s(value: &Json) -> S {
    pts(&repr(value))
}

enum Num {
    Int(BigInt),
    Float(f64),
}

fn num(value: &Json) -> Option<Num> {
    match value {
        Json::Bool(b) => Some(Num::Int(BigInt::from(u8::from(*b)))),
        Json::Int(i) => Some(Num::Int(i.clone())),
        Json::Float(f) => Some(Num::Float(*f)),
        _ => None,
    }
}

/// Python's exact comparison of an int with a float.
fn int_float_cmp(i: &BigInt, f: f64) -> Option<Ordering> {
    if f.is_nan() {
        return None;
    }
    if f.is_infinite() {
        return Some(if f > 0.0 {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    let floor = pyvalue::float_to_int(f.floor());
    Some(match i.cmp(&floor) {
        Ordering::Equal if f.fract() != 0.0 => Ordering::Less,
        ordering => ordering,
    })
}

fn num_cmp(a: &Num, b: &Num) -> Option<Ordering> {
    match (a, b) {
        (Num::Int(x), Num::Int(y)) => Some(x.cmp(y)),
        (Num::Float(x), Num::Float(y)) => x.partial_cmp(y),
        (Num::Int(x), Num::Float(y)) => int_float_cmp(x, *y),
        (Num::Float(x), Num::Int(y)) => int_float_cmp(y, *x).map(Ordering::reverse),
    }
}

/// `a <= b` (`op` "<=") or `a < b` (`op` "<") for decoded JSON values.
fn py_compare(a: &Json, b: &Json, op: &str) -> R<bool> {
    let test = |ordering: Ordering| match op {
        "<" => ordering == Ordering::Less,
        _ => ordering != Ordering::Greater,
    };
    if let (Some(x), Some(y)) = (num(a), num(b)) {
        return Ok(num_cmp(&x, &y).is_some_and(test));
    }
    match (a, b) {
        (Json::Str(x), Json::Str(y)) => Ok(test(x.points().cmp(y.points()))),
        (Json::Array(x), Json::Array(y)) => {
            match x.iter().zip(y).position(|(p, q)| !item_eq(p, q)) {
                Some(i) => py_compare(&x[i], &y[i], op),
                None => Ok(test(x.len().cmp(&y.len()))),
            }
        }
        _ => Err(type_error(&format!(
            "'{op}' not supported between instances of '{}' and '{}'",
            type_name(a),
            type_name(b)
        ))),
    }
}

/// `sorted(set(values))`. Python's set iterates in hash order, so where the
/// values cannot all be ordered the pair named in the `TypeError` may differ.
fn sorted_set(values: Vec<Json>) -> R<Vec<Json>> {
    let mut keys: Vec<Key> = Vec::new();
    let mut unique: Vec<Json> = Vec::new();
    for value in values {
        let key = hash_key(&value, "set element")?;
        if !keys.contains(&key) {
            keys.push(key);
            unique.push(value);
        }
    }
    if unique.len() < 2 {
        return Ok(unique);
    }
    for pair in unique.windows(2) {
        py_compare(&pair[1], &pair[0], "<")?;
    }
    let mut error = None;
    unique.sort_by(|a, b| {
        if py_compare(a, b, "<").unwrap_or_else(|e| {
            error.get_or_insert(e);
            false
        }) {
            Ordering::Less
        } else if py_compare(b, a, "<").unwrap_or(false) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    match error {
        Some(e) => Err(e),
        None => Ok(unique),
    }
}

/// `Decimal(value)` for a decoded JSON value.
fn to_decimal(value: &Json) -> R<Decimal> {
    match value {
        Json::Bool(b) => Ok(Decimal::from_int(i64::from(*b))),
        Json::Int(i) => Ok(Decimal::from_bigint(i)),
        Json::Float(f) => Ok(Decimal::from_float(*f)),
        Json::Str(s) => Decimal::parse_points(&points_of(s)).map_err(conversion),
        Json::Array(items) => decimal_from_tuple(items),
        other => Err(type_error(&format!(
            "conversion from {} to Decimal is not supported",
            type_name(other)
        ))),
    }
}

/// `PyLong_AsLong` (`ssize` false) or `PyLong_AsSsize_t` of an int.
fn c_long(value: &BigInt, ssize: bool) -> R<i64> {
    i64::try_from(value).map_err(|_| {
        Exc::msg(
            "OverflowError",
            if ssize {
                "Python int too large to convert to C ssize_t"
            } else {
                "Python int too large to convert to C long"
            },
        )
    })
}

fn as_int(value: &Json) -> Option<BigInt> {
    match value {
        Json::Bool(b) => Some(BigInt::from(u8::from(*b))),
        Json::Int(i) => Some(i.clone()),
        _ => None,
    }
}

/// `Decimal(list)`: `_decimal`'s tuple form, `(sign, digits, exponent)`.
fn decimal_from_tuple(items: &[Json]) -> R<Decimal> {
    let bad = |text: &str| value_error(pts(text));
    let [sign, digits, exponent] = items else {
        return Err(bad("argument must be a sequence of length 3"));
    };
    let sign_message = "sign must be an integer with the value 0 or 1";
    let sign = c_long(&as_int(sign).ok_or_else(|| bad(sign_message))?, false)?;
    if sign != 0 && sign != 1 {
        return Err(bad(sign_message));
    }
    let mut text = String::from(if sign == 1 { "-" } else { "+" });
    let special = match exponent {
        Json::Str(s) => match s.as_str() {
            Some("F") => Some("Inf"),
            Some("n") => Some("NaN"),
            Some("N") => Some("sNaN"),
            _ => {
                return Err(bad(
                    "string argument in the third position must be 'F', 'n' or 'N'",
                ));
            }
        },
        other => {
            let exp = as_int(other).ok_or_else(|| bad("exponent must be an integer"))?;
            c_long(&exp, true)?;
            None
        }
    };
    let coefficient = "coefficient must be a tuple of digits";
    let Json::Array(digits) = digits else {
        return Err(bad(coefficient));
    };
    let mut coeff = String::new();
    for digit in digits {
        let d = c_long(&as_int(digit).ok_or_else(|| bad(coefficient))?, false)?;
        if !(0..=9).contains(&d) {
            return Err(bad(coefficient));
        }
        coeff.push(char::from(b'0' + d as u8));
    }
    match special {
        Some("Inf") => text.push_str("Inf"),
        Some(name) => {
            text.push_str(name);
            text.push_str(&coeff);
        }
        None => {
            text.push_str(if coeff.is_empty() { "0" } else { &coeff });
            text.push('E');
            text.push_str(&as_int(exponent).unwrap_or_default().to_string());
        }
    }
    Decimal::from_mpd_string(&text).map_err(|_| decimal_signal(Signal::InvalidOperation))
}

/// `Decimal(str(value))`.
fn str_decimal(value: &Json) -> R<Decimal> {
    Decimal::parse_points(&str_points(value)).map_err(conversion)
}

fn decimal(value: i64) -> Decimal {
    Decimal::from_int(value)
}

fn calc<T>(result: Result<T, Signal>) -> R<T> {
    result.map_err(decimal_signal)
}

/// `int(value)` for a `Decimal`: NaN is `ValueError`, an infinity `OverflowError`.
fn decimal_int(value: &Decimal) -> R<Decimal> {
    if value.is_nan_value() {
        return Err(value_error(pts("cannot convert NaN to integer")));
    }
    if value.is_infinite_value() {
        return Err(Exc::msg(
            "OverflowError",
            "cannot convert Infinity to integer",
        ));
    }
    calc(value.to_integral_rounded(Rounding::Down))
}

/// An `int` as `scale_renderings` meets it. `Huge` stands for one too long
/// for `str()` whatever is subtracted from it, which is never built.
enum Whole {
    Int(BigInt),
    Huge,
}

/// `int(value)` of a `Decimal` as a `Whole`.
fn decimal_whole(value: &Decimal) -> R<Whole> {
    let value = decimal_int(value)?;
    if value.is_zero() {
        return Ok(Whole::Int(BigInt::zero()));
    }
    if value.adjusted() >= 5000 {
        return Ok(Whole::Huge);
    }
    let digits = format!(
        "{}{}",
        value.coefficient(),
        "0".repeat(value.exponent().max(0) as usize)
    );
    let n: BigInt = digits.parse().unwrap_or_default();
    Ok(Whole::Int(if value.is_negative() { -n } else { n }))
}

fn digit_limit() -> Exc {
    value_error(pts(&py::int_repr(&BigInt::from(10).pow(4300)).unwrap_err()))
}

/// `format(value, ",.{places}f")`; a value too long to build is Python's `MemoryError`.
fn grouped(value: &Decimal, places: u32) -> R<String> {
    if value.is_finite() && value.exponent() > 100_000_000 {
        return Err(Exc::msg("MemoryError", ""));
    }
    Ok(value.fixed(places, true))
}

// ---------------------------------------------------------------- paths and files

struct Ctx {
    root: PyPath,
    data: PyPath,
    snapshots: PyPath,
}

fn env(name: &str) -> Option<String> {
    if cfg!(feature = "differential") {
        std::env::var(name).ok()
    } else {
        None
    }
}

impl Ctx {
    fn new() -> Ctx {
        let root = env("RATES_ROOT").unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").to_string());
        let root = PyPath::new(&root);
        let root = root.resolve().unwrap_or(root);
        Ctx {
            data: root.join("data"),
            snapshots: root.join("snapshots"),
            root,
        }
    }
}

/// A directory entry `DATA.glob("*.json")` yields.
struct Entry {
    name: S,
    path: PathBuf,
    display: String,
}

impl Entry {
    /// `path.stem`.
    fn stem(&self) -> S {
        let name = &self.name;
        if let Some(i) = name.iter().rposition(|&c| c == u32::from('.'))
            && name[..i].iter().any(|&c| c != u32::from('.'))
        {
            return name[..i].to_vec();
        }
        name.clone()
    }
}

/// `sorted(DATA.glob("*.json"))`: every entry whose name matches, hidden files
/// and directories included. On Windows the match and the order ignore case,
/// as `PureWindowsPath` compares. Where `glob` yields nothing for a listing
/// that fails, this raises its `OSError`: `run` has checked the folder, and a
/// listing that fails after that must not pass as no records.
fn glob_json(data: &PyPath) -> R<Vec<Entry>> {
    let failed =
        |e: std::io::Error| os_exc(&OsError::new(&e, Some(&data.display()), pyos::Style::Errno));
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(data.fs()).map_err(failed)? {
        let entry = entry.map_err(failed)?;
        let name = pyos::os_name_points(&entry.file_name());
        if JSON_NAME.fullmatch(&name).is_none() {
            continue;
        }
        let display = format!("{}{SEP}{}", data.display(), pyvalue::points_lossy(&name));
        entries.push(Entry {
            name,
            path: entry.path(),
            display,
        });
    }
    if cfg!(windows) {
        entries.sort_by_cached_key(|e| sre::str_lower(&e.name));
    } else {
        entries.sort_by(|a, b| a.name.cmp(&b.name));
    }
    Ok(entries)
}

/// `path.read_text(encoding="utf-8")`.
fn read_text(path: &Path, display: &str) -> R<String> {
    let bytes = pyos::read(path, display).map_err(|e| os_exc(&e))?;
    let text =
        py::decode_utf8(&bytes).map_err(|e| Exc::msg("UnicodeDecodeError", &e.to_string()))?;
    Ok(py::universal_newlines(&text))
}

/// `json.loads(text)`.
fn loads(text: &str) -> R<Json> {
    pyjson::loads_standard(text).map_err(json_error)
}

fn json_error(error: DecodeError) -> Exc {
    match error {
        DecodeError::Invalid(message) if message.starts_with("Exceeds the limit") => {
            value_error(pts(&message))
        }
        DecodeError::Invalid(message) => Exc::msg("json.decoder.JSONDecodeError", &message),
        DecodeError::Recursion(kind) => {
            Exc::msg("RecursionError", &pyjson::recursion_message(kind))
        }
        DecodeError::Duplicate => Exc::msg("json.decoder.JSONDecodeError", "duplicate member"),
    }
}

fn load_records(ctx: &Ctx) -> R<Vec<Json>> {
    glob_json(&ctx.data)?
        .iter()
        .map(|entry| loads(&read_text(&entry.path, &entry.display)?))
        .collect()
}

/// `Path.write_text(text, encoding="utf-8")`: the file is truncated before a
/// lone surrogate fails the strict encoding.
fn write_text(path: Option<&Path>, display: &[u32], text: &[u32]) -> R<()> {
    let display = pyvalue::points_lossy(display);
    let Some(path) = path else {
        return Err(Exc::msg(
            "UnicodeEncodeError",
            &pyvalue::utf8_strict_error(&pts(&display)).unwrap_or_default(),
        ));
    };
    let mut file = std::fs::File::create(path)
        .map_err(|e| os_exc(&OsError::new(&e, Some(&display), pyos::Style::Errno)))?;
    let text = line_ends(text);
    if let Some(message) = pyvalue::utf8_strict_error(&text) {
        return Err(Exc::msg("UnicodeEncodeError", &message));
    }
    let mut bytes = Vec::with_capacity(text.len());
    for cp in text {
        let mut buf = [0; 4];
        let c = char::from_u32(cp).unwrap_or('\u{fffd}');
        bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    }
    file.write_all(&bytes)
        .map_err(|e| os_exc(&OsError::new(&e, None, pyos::Style::Errno)))
}

/// A snapshot's path.
struct Snap {
    display: S,
    relative: Result<S, S>,
    fs: Option<PathBuf>,
    name: S,
}

impl Snap {
    fn exists(&self) -> bool {
        self.fs.as_deref().is_some_and(pyos::exists)
    }

    fn read(&self) -> R<String> {
        let path = self.fs.as_deref().unwrap_or(Path::new(""));
        read_text(path, &pyvalue::points_lossy(&self.display))
    }
}

/// The schemes whose paths `urlparse` splits `;params` from.
const USES_PARAMS: [&str; 16] = [
    "", "ftp", "hdl", "prospero", "http", "imap", "https", "shttp", "rtsp", "rtsps", "rtspu",
    "sip", "sips", "mms", "sftp", "tel",
];

/// `urlparse(url).path`. A lone surrogate rides through `urlsplit` as an
/// unassigned code point the URL does not use, which no step of it treats
/// specially.
fn url_path(url: &[u32]) -> R<S> {
    let pairs = py::surrogate_stand_ins(url).ok_or_else(|| {
        value_error(pts(
            "no code point is free to stand in for a lone surrogate",
        ))
    })?;
    let hide: std::collections::HashMap<u32, u32> = pairs.iter().copied().collect();
    let show: std::collections::HashMap<u32, u32> = pairs.iter().map(|&(s, f)| (f, s)).collect();
    let text: String = url
        .iter()
        .filter_map(|cp| char::from_u32(*hide.get(cp).unwrap_or(cp)))
        .collect();
    let split = crate::pyurl::urlsplit(&text).map_err(|m| value_error(pts(&m)))?;
    let mut path = split.path;
    if USES_PARAMS.contains(&split.scheme.as_str()) && path.contains(';') {
        let cut = match path.rfind('/') {
            Some(slash) => path[slash..].find(';').map(|i| slash + i),
            None => path.find(';'),
        };
        if let Some(cut) = cut {
            path.truncate(cut);
        }
    }
    Ok(path
        .chars()
        .map(|c| {
            let cp = u32::from(c);
            *show.get(&cp).unwrap_or(&cp)
        })
        .collect())
}

/// `SNAPSHOTS / (urlparse(url).path.rstrip("/").rsplit("/", 1)[-1] + ".txt")`.
fn snapshot_path(ctx: &Ctx, url: &[u32]) -> R<Snap> {
    let path = url_path(url)?;
    let slash = u32::from('/');
    let end = path.iter().rposition(|&c| c != slash).map_or(0, |i| i + 1);
    let trimmed = &path[..end];
    let start = trimmed
        .iter()
        .rposition(|&c| c == slash)
        .map_or(0, |i| i + 1);
    let name = p!(trimmed[start..], ".txt");
    if let Some(text) = name
        .iter()
        .map(|&cp| char::from_u32(cp))
        .collect::<Option<String>>()
    {
        let path = ctx.snapshots.join(&text);
        return Ok(Snap {
            display: pts(&path.display()),
            relative: path
                .relative_to(&ctx.root)
                .map(|r| pts(&r.display()))
                .map_err(|m| pts(&m)),
            fs: Some(path.fs()),
            name: pts(path.name()),
        });
    }
    // A name with a lone surrogate: joined as pathlib parses a plain relative name.
    let sep = pts(SEP);
    let mut parts: Vec<S> = Vec::new();
    for part in name.split(|&c| cfg!(windows) && c == u32::from('\\')) {
        if !part.is_empty() && part != [u32::from('.')] {
            parts.push(part.to_vec());
        }
    }
    let joined = parts.join(&sep[..]);
    let display = p!(ctx.snapshots.display(), sep, joined);
    let base = ctx
        .snapshots
        .relative_to(&ctx.root)
        .map(|r| r.display())
        .unwrap_or_default();
    Ok(Snap {
        fs: pyos::os_string(&display).ok().map(PathBuf::from),
        relative: Ok(p!(base, sep, joined)),
        name: parts.last().cloned().unwrap_or_else(|| pts("snapshots")),
        display,
    })
}

// ---------------------------------------------------------------- output

/// `sys.stdout`: UTF-8 with `surrogateescape`, `\n` written as the platform's line end.
///
/// A write that fails is kept for the end, where Python's flush at shutdown
/// reports it and exits 120. shortcut: Python raises at the `print` once its
/// buffer fills, so an output past that buffer also gets a traceback first;
/// the commands' reports are far smaller.
struct Out<W> {
    w: W,
    failed: Option<std::io::Error>,
}

impl<W: Write> Out<W> {
    fn new(w: W) -> Self {
        Out { w, failed: None }
    }

    fn print(&mut self, line: &[u32]) -> R<()> {
        let bytes = pyvalue::encode_print(&line_ends(line))
            .map_err(|m| Exc::msg("UnicodeEncodeError", &m))?;
        if self.failed.is_none() {
            let end: &[u8] = if cfg!(windows) { b"\r\n" } else { b"\n" };
            if let Err(error) = self.w.write_all(&[&bytes[..], end].concat()) {
                self.failed = Some(error);
            }
        }
        Ok(())
    }

    /// The flush at shutdown: None, or the exit status 120 after its report.
    fn finish(mut self) -> Option<i32> {
        let error = match self.failed.take() {
            Some(error) => error,
            None => self.w.flush().err()?,
        };
        Some(flush_failed(&error))
    }

    fn print_str(&mut self, line: &str) -> R<()> {
        self.print(&pts(line))
    }
}

/// Text-mode output on Windows writes each `\n` as `\r\n`, translated before
/// the encoding, so an encoding error counts the added `\r`s.
fn line_ends(text: &[u32]) -> S {
    let mut out = Vec::with_capacity(text.len());
    for &cp in text {
        if cfg!(windows) && cp == 10 {
            out.push(13);
        }
        out.push(cp);
    }
    out
}

/// [`line_ends`] for bytes already encoded.
fn newlines(bytes: &[u8]) -> Vec<u8> {
    if !cfg!(windows) {
        return bytes.to_vec();
    }
    let mut out = Vec::with_capacity(bytes.len());
    for &b in bytes {
        if b == b'\n' {
            out.push(b'\r');
        }
        out.push(b);
    }
    out
}

// ---------------------------------------------------------------- checks

/// `check_value(unit, value)`.
fn check_value(unit: &str, value: &Json) -> R<Vec<S>> {
    if unit == "date" {
        return Ok(match fromisoformat(value) {
            Ok(_) => Vec::new(),
            Err(_) => vec![p!("value ", repr_s(value), " is not an ISO date")],
        });
    }
    if unit == "tax_scale" {
        return check_scale(value);
    }
    let number = match value {
        Json::Int(i) => Num::Int(i.clone()),
        Json::Float(f) => Num::Float(*f),
        _ => return Ok(vec![p!("value ", repr_s(value), " is not a number")]),
    };
    if let Num::Float(f) = number
        && !f.is_finite()
    {
        return Ok(vec![p!("value ", repr_s(value), " is not finite")]);
    }
    let zero = Num::Int(BigInt::zero());
    if num_cmp(&number, &zero) == Some(Ordering::Less) {
        return Ok(vec![p!("value ", repr_s(value), " is negative")]);
    }
    if unit == "percent"
        && num_cmp(&number, &Num::Int(BigInt::from(100))) == Some(Ordering::Greater)
    {
        return Ok(vec![p!(
            "percent value ",
            str_points(value),
            " is above 100"
        )]);
    }
    Ok(Vec::new())
}

/// `dt.date.fromisoformat(value)`.
fn fromisoformat(value: &Json) -> R<Date> {
    match value {
        Json::Str(PyStr::Text(text)) => {
            crate::pydatetime::date_fromisoformat(text).map_err(|m| value_error(pts(&m)))
        }
        Json::Str(PyStr::Lone(points)) => Err(value_error(p!(
            "Invalid isoformat string: ",
            repr_points(points)
        ))),
        // `_datetime` names the type with `%T`, which calls None's "None".
        Json::Null => Err(type_error("fromisoformat() argument must be str, not None")),
        other => Err(type_error(&format!(
            "fromisoformat() argument must be str, not {}",
            type_name(other)
        ))),
    }
}

fn whole_dollars(value: &Json) -> Option<&BigInt> {
    match value {
        Json::Int(i) if i.sign() != num_bigint::Sign::Minus => Some(i),
        _ => None,
    }
}

// "A false balance is abomination to the LORD: but a just weight is his delight."
/// `check_scale(rows)`.
fn check_scale(rows: &Json) -> R<Vec<S>> {
    let rows = match rows {
        Json::Array(rows) if !rows.is_empty() => rows,
        _ => return Ok(vec![pts("tax_scale value must be a non-empty list")]),
    };
    let mut problems = Vec::new();
    let last = rows.len() - 1;
    for (i, row) in rows.iter().enumerate() {
        if !row.has_exactly(&["from", "to", "base_tax", "marginal_rate"]) {
            return Ok(vec![pts(&format!(
                "bracket {i} needs exactly from, to, base_tax and marginal_rate"
            ))]);
        }
        for key in ["from", "to"] {
            let value = row.at(key);
            if key == "to" && *value == Json::Null && i == last {
                continue;
            }
            if whole_dollars(value).is_none() {
                problems.push(pts(&format!(
                    "bracket {i} {key} must be a non-negative whole dollar amount"
                )));
            }
        }
        for (key, unit) in [("base_tax", "AUD"), ("marginal_rate", "percent")] {
            for problem in check_value(unit, row.at(key))? {
                problems.push(p!(format!("bracket {i} {key}: "), problem));
            }
        }
    }
    if !problems.is_empty() {
        return Ok(problems);
    }
    let int = |row: &Json, key: &str| whole_dollars(row.at(key)).cloned();
    if int(&rows[0], "from") != Some(BigInt::zero()) {
        problems.push(pts("the first bracket must start at zero"));
    }
    let mut base = decimal(0);
    for (i, row) in rows.iter().enumerate() {
        let from = int(row, "from").unwrap_or_default();
        if i > 0 && Some(&from - 1) != int(&rows[i - 1], "to") {
            problems.push(pts(&format!(
                "bracket {i} does not start $1 above the previous bracket"
            )));
        }
        if !calc(str_decimal(row.at("base_tax"))?.equals(&base))? {
            problems.push(p!(
                format!("bracket {i} base_tax "),
                str_points(row.at("base_tax")),
                format!(" should be {base}")
            ));
        }
        if let Some(to) = int(row, "to") {
            if to < from {
                problems.push(pts(&format!("bracket {i} upper limit precedes its start")));
            }
            let width = &to - if i > 0 { &from - 1 } else { from.clone() };
            let rate = str_decimal(row.at("marginal_rate"))?;
            let step = calc(Decimal::from_bigint(&width).mul(&rate))?;
            base = calc(base.add(&calc(step.div(&decimal(100)))?))?;
        }
    }
    if *rows[last].at("to") != Json::Null {
        problems.push(pts("the top bracket must have no upper limit (to: null)"));
    }
    Ok(problems)
}

/// One bracket as `scale_renderings` reads it: `from` an int, the others as
/// `Decimal(str(value))` gives them.
struct ScaleRow {
    from: Whole,
    base_tax: Decimal,
    rate: Decimal,
}

/// `dollars(amount)` for an amount already `Decimal(str(amount))`.
fn dollars(amount: &Decimal) -> R<S> {
    let whole = calc(amount.equals(&calc(amount.to_integral_value())?))?;
    Ok(p!("$", grouped(amount, if whole { 0 } else { 2 })?))
}

/// `scale_renderings(rows)`: each taxed bracket as the ATO page prints it.
fn scale_renderings(rows: &[ScaleRow]) -> R<Vec<S>> {
    let mut renderings = Vec::new();
    for row in rows {
        if calc(row.rate.equals(&decimal(0)))? {
            continue;
        }
        let mut rate = p!(calc(row.rate.normalize())?.fixed_format(), "c for each $1");
        let below = match &row.from {
            Whole::Int(from) if from.is_zero() => None,
            Whole::Int(from) => Some(py::int_repr(&(from - 1)).map_err(|m| value_error(pts(&m)))?),
            Whole::Huge => return Err(digit_limit()),
        };
        if let Some(below) = below {
            let below = Decimal::parse(&below).map_err(decimal_signal)?;
            rate.extend(p!(" over ", dollars(&below)?));
        }
        renderings.push(if calc(row.base_tax.equals(&decimal(0)))? {
            rate
        } else {
            p!(dollars(&row.base_tax)?, " plus ", rate)
        });
    }
    Ok(renderings)
}

fn json_scale_rows(rows: &Json) -> R<Vec<ScaleRow>> {
    let Json::Array(rows) = rows else {
        return Ok(Vec::new());
    };
    rows.iter()
        .map(|row| {
            Ok(ScaleRow {
                from: Whole::Int(whole_dollars(row.at("from")).cloned().unwrap_or_default()),
                base_tax: str_decimal(row.at("base_tax"))?,
                rate: str_decimal(row.at("marginal_rate"))?,
            })
        })
        .collect()
}

/// `validate_record(record, page_text)` for a dict.
fn validate_record(record: &Json, page_text: Option<&[u32]>) -> R<Vec<S>> {
    let mut problems = Vec::new();
    for (key, dict) in FIELDS {
        let right = match record.get(key) {
            Some(Json::Object(_)) => dict,
            Some(Json::Str(_)) => !dict,
            _ => false,
        };
        if !right {
            problems.push(pts(&format!("missing or wrong type: {key}")));
        }
    }
    if !has(record, "value") {
        problems.push(pts("missing: value"));
    }
    if !problems.is_empty() {
        return Ok(problems);
    }
    let unit = record.at("unit");
    let Some(unit_name) = unit.as_str().filter(|u| UNITS.contains(u)) else {
        return Ok(vec![p!("unknown unit ", repr_s(unit))]);
    };
    let value = record.at("value");
    let value_problems = check_value(unit_name, value)?;
    let scale_checked = value_problems.is_empty();
    problems.extend(value_problems);
    if unit_name == "tax_scale" && scale_checked {
        let quote = norm(&str_points(record.at("quote")));
        for rendering in scale_renderings(&json_scale_rows(value)?)? {
            if !contains(&quote, &rendering) {
                problems.push(p!(
                    "quote does not show the bracket ",
                    repr_points(&rendering)
                ));
            }
        }
    }
    let period = record.at("period");
    let Json::Object(pairs) = period else {
        unreachable!("FIELDS checked the period is a dict");
    };
    if pairs.is_empty()
        || pairs
            .iter()
            .any(|(k, _)| !k.as_str().is_some_and(|k| PERIOD_KEYS.contains(&k)))
    {
        problems.push(pts(&format!(
            "period keys must be a non-empty subset of {}",
            repr(&Json::Array(PERIOD_KEYS.map(Json::str).to_vec()))
        )));
    }
    if let Some(income_year) = period.get("income_year") {
        let year = str_points(income_year);
        if INCOME_YEAR.fullmatch(&year).is_none() {
            problems.push(p!(
                "income_year ",
                repr_s(income_year),
                " is not like 2026-27"
            ));
        } else {
            let first = int_of(&year[..4]);
            let second = int_of(&year[year.len() - 2..]);
            let adjacent = BigInt::from(1) <= first
                && first < BigInt::from(9999)
                && second == (&first + 1) % 100;
            if !adjacent {
                problems.push(p!(
                    "income_year ",
                    repr_points(&year),
                    " must describe adjacent calendar years between 1 and 9999"
                ));
            }
        }
    }
    let mut dates: Vec<(&str, Date)> = Vec::new();
    for key in ["effective_from", "effective_to", "as_at"] {
        if let Some(value) = period.get(key) {
            let date_problems = check_value("date", value)?;
            if date_problems.is_empty() {
                dates.push((key, fromisoformat(value)?));
            }
            for problem in date_problems {
                problems.push(p!(key, ": ", problem));
            }
        }
    }
    let date = |key: &str| dates.iter().find(|(k, _)| *k == key).map(|(_, d)| *d);
    if let (Some(from), Some(to)) = (date("effective_from"), date("effective_to"))
        && to < from
    {
        problems.push(pts("effective_to precedes effective_from"));
    }
    if !str_points(record.at("source_url")).starts_with(&pts("https://www.ato.gov.au/")) {
        problems.push(pts("source_url is not a public ato.gov.au page"));
    }
    if crate::pydatetime::datetime_fromisoformat(&str_points(record.at("retrieved_at"))).is_err() {
        problems.push(pts("retrieved_at is not an ISO timestamp"));
    }
    let quote_len = str_points(record.at("quote")).len();
    if !(1..=400).contains(&quote_len) {
        problems.push(pts("quote must be 1 to 400 characters"));
    }
    if let Some(text) = page_text {
        problems.extend(page_problems(record, text)?);
    }
    Ok(problems)
}

/// `norm(value)`, where `re.sub` refuses anything but a str.
fn norm_value(value: &Json) -> R<S> {
    match value {
        Json::Str(s) => Ok(norm(&points_of(s))),
        other => Err(type_error(&format!(
            "expected string or bytes-like object, got '{}'",
            type_name(other)
        ))),
    }
}

/// `page_problems(record, page_text)` for a dict.
fn page_problems(record: &Json, page_text: &[u32]) -> R<Vec<S>> {
    let text = norm(page_text);
    let mut problems = Vec::new();
    for key in ["quote", "period_quote"] {
        if let Some(value) = record.get(key)
            && !contains(&text, &norm_value(value)?)
        {
            problems.push(pts(&format!("{key} not found on page")));
        }
    }
    if let Some(pattern) = record.get("pattern") {
        match pattern_problem(record, pattern, &text) {
            Ok(Some(problem)) => problems.push(problem),
            Ok(None) => {}
            Err(e)
                if e.is_any(&[
                    "re.PatternError",
                    "ValueError",
                    "TypeError",
                    "decimal.InvalidOperation",
                ]) =>
            {
                problems.push(p!("value pattern could not be read: ", e.text));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(problems)
}

/// `re.compile(pattern)` for a decoded JSON value.
fn compile(pattern: &Json) -> R<Pattern> {
    let text = match pattern {
        Json::Str(s) => points_of(s),
        Json::Array(_) | Json::Object(_) => {
            return Err(type_error(&format!(
                "cannot use 'tuple' as a dict key (unhashable type: '{}')",
                type_name(pattern)
            )));
        }
        _ => {
            return Err(type_error(
                "first argument must be string or compiled pattern",
            ));
        }
    };
    sre::compile(&text, 0).map_err(|e| match e {
        sre::CompileError::Pattern(text) => Exc::new("re.PatternError", text),
        sre::CompileError::Other(class, message) => Exc::msg(class, &message),
    })
}

/// What `parse_text` found: an ISO date for a date, else a number.
enum Found {
    Date(S),
    Number(Decimal),
}

/// The pattern step of `page_problems`: a problem, nothing, or the exception.
fn pattern_problem(record: &Json, pattern: &Json, text: &[u32]) -> R<Option<S>> {
    let pattern = compile(pattern)?;
    if pattern.groups != 1 {
        return Err(value_error(pts("expected exactly one capture group")));
    }
    let unit = getitem(record, "unit")?;
    if py_eq(unit, &Json::str("tax_scale")) {
        return Err(value_error(pts(
            "a tax scale cannot use a single-value pattern",
        )));
    }
    let Some(found) = pattern.search(text) else {
        return Ok(Some(pts("value pattern not found on page")));
    };
    let Some((start, end)) = found.span(1) else {
        return Err(value_error(pts("the capture group did not match")));
    };
    let captured = &text[start..end];
    let found = parse_text(unit, captured)?;
    let unit = getitem(record, "unit")?;
    let value = getitem(record, "value")?;
    let changed = if py_eq(unit, &Json::str("date")) {
        let Found::Date(iso) = &found else {
            unreachable!("a date unit reads a date")
        };
        !py_eq(&json_str(iso.clone()), value)
    } else {
        let expected = str_decimal(value)?;
        let Found::Number(found) = &found else {
            unreachable!("a number unit reads a number")
        };
        !calc(found.equals(&expected))?
    };
    Ok(changed.then(|| p!("value changed: ", str_points(value), " -> ", captured)))
}

/// `parse_text(unit, text)`.
fn parse_text(unit: &Json, text: &[u32]) -> R<Found> {
    let text = norm(text);
    if py_eq(unit, &Json::str("date")) {
        let date = crate::strptime::date(&text).map_err(value_error)?;
        return Ok(Found::Date(pts(&date.isoformat())));
    }
    hash_key(unit, "dict key")?;
    let Some((name, number)) = NUMBER.iter().find(|(name, _)| unit.as_str() == Some(name)) else {
        return Err(Exc::new("KeyError", repr_s(unit)));
    };
    let Some(m) = number.match_start(&text) else {
        return Err(value_error(p!(
            repr_points(&text),
            format!(" is not a {name} value")
        )));
    };
    let (start, end) = m.span(1).unwrap_or((0, 0));
    let digits: S = text[start..end]
        .iter()
        .copied()
        .filter(|&c| c != u32::from(','))
        .collect();
    Decimal::parse_points(&digits)
        .map(Found::Number)
        .map_err(conversion)
}

// ---------------------------------------------------------------- the browser

fn browser_command() -> Vec<String> {
    if let Some(list) = env("RATES_BROWSER")
        && let Ok(Json::Array(items)) = pyjson::loads_standard(&list)
    {
        return items.iter().map(pyvalue::py_str).collect();
    }
    NODRIVER.map(str::to_string).to_vec()
}

/// `subprocess.run(command, capture_output=True, text=True, encoding="utf-8",
/// timeout=180)`: the exit status, stdout and stderr.
fn run_browser(command: &[Json]) -> R<(i64, Option<String>, Option<String>)> {
    run_with_limit(command, Duration::from_secs(180))
}

/// [`run_browser`] with its timeout as a parameter, which tests shorten.
fn run_with_limit(command: &[Json], limit: Duration) -> R<(i64, Option<String>, Option<String>)> {
    let mut args: Vec<OsString> = Vec::new();
    for arg in command {
        let Json::Str(s) = arg else {
            return Err(type_error(&format!(
                "expected str, bytes or os.PathLike object, not {}",
                type_name(arg)
            )));
        };
        let points = points_of(s);
        let os = pyos::os_string(&points).map_err(|m| Exc::msg("UnicodeEncodeError", &m))?;
        if points.contains(&0) {
            return Err(value_error(pts(if cfg!(windows) {
                "embedded null character"
            } else {
                "embedded null byte"
            })));
        }
        args.push(os);
    }
    let program = pyvalue::py_str(&command[0]);
    let mut child = std::process::Command::new(&args[0])
        .args(&args[1..])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            os_exc(&if cfg!(windows) {
                OsError::new(&e, None, pyos::Style::Native)
            } else {
                OsError::new(&e, Some(&program), pyos::Style::Errno)
            })
        })?;
    // Each pipe drains on its own thread, which reports on the channel.
    let (sender, drained) = std::sync::mpsc::channel();
    let pipes: [Option<Box<dyn Read + Send>>; 2] = [
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    ];
    for (which, pipe) in pipes.into_iter().enumerate() {
        let sender = sender.clone();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_end(&mut bytes);
            }
            let _ = sender.send((which, bytes));
        });
    }
    // communicate(timeout=180): one deadline for the exit and both pipes, as
    // a descendant can hold a pipe open after the browser exits. On a
    // timeout the child is killed and reaped, and the pipes are left to
    // their threads, as POSIX `run` leaves them.
    let deadline = Instant::now() + limit;
    let timed_out = || {
        Err(Exc::new(
            "subprocess.TimeoutExpired",
            p!(
                "Command '",
                repr_s(&Json::Array(command.to_vec())),
                "' timed out after ",
                &limit.as_secs().to_string(),
                " seconds"
            ),
        ))
    };
    let status = loop {
        if let Ok(Some(status)) = child.try_wait() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return timed_out();
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let mut output = [Vec::new(), Vec::new()];
    for _ in 0..2 {
        match drained.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok((which, bytes)) => output[which] = bytes,
            Err(_) => return timed_out(),
        }
    }
    let decode = |bytes: Vec<u8>| -> R<String> {
        py::decode_utf8(&bytes)
            .map(|text| py::universal_newlines(&text))
            .map_err(|e| Exc::msg("UnicodeDecodeError", &e.to_string()))
    };
    let [stdout, stderr] = output;
    let code = exit_code(status);
    if cfg!(windows) {
        // Windows decodes each pipe in a reader thread: a failure is printed
        // by the thread, and leaves that stream None.
        let first = READER_THREADS.fetch_add(2, std::sync::atomic::Ordering::Relaxed) + 1;
        let read = |bytes: Vec<u8>, thread: usize| {
            decode(bytes)
                .map_err(|e| {
                    let report = format!(
                        "Exception in thread Thread-{thread} (_readerthread):\nTraceback (most recent call last):\n{}: {}\n",
                        e.class,
                        String::from_utf8_lossy(&pyvalue::encode_print(&e.text).unwrap_or_default())
                    );
                    let _ = std::io::stderr().write_all(&newlines(report.as_bytes()));
                })
                .ok()
        };
        return Ok((code, read(stdout, first), read(stderr, first + 1)));
    }
    Ok((code, Some(decode(stdout)?), Some(decode(stderr)?)))
}

/// The threads `subprocess` has started, which name a reader thread's report.
static READER_THREADS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// `Popen.returncode`: Windows' unsigned exit code, or minus the POSIX signal.
fn exit_code(status: std::process::ExitStatus) -> i64 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return -i64::from(signal);
        }
    }
    #[cfg(windows)]
    {
        return i64::from(status.code().unwrap_or(1) as u32);
    }
    #[allow(unreachable_code)]
    i64::from(status.code().unwrap_or(1))
}

/// `fetch_text(url)`.
fn fetch_text(url: &Json) -> R<S> {
    fetch_text_with(&browser_command(), url)
}

/// [`fetch_text`] with the browser command as a parameter, which tests fake.
fn fetch_text_with(browser: &[String], url: &Json) -> R<S> {
    let (code, stdout, stderr) = run_browser(&browser_args(browser, url))?;
    read_output(code, stdout.as_deref(), stderr.as_deref())
}

/// The command `fetch_text` runs for `url`.
fn browser_args(browser: &[String], url: &Json) -> Vec<Json> {
    let mut command: Vec<Json> = browser.iter().map(|a| Json::str(a)).collect();
    command.extend([
        Json::str("text"),
        url.clone(),
        Json::str("--json"),
        Json::str("--wait"),
        Json::str("3"),
    ]);
    command
}

/// The page text in a reply the browser wrote in full.
#[cfg(test)]
fn read_reply(code: i64, stdout: &str, stderr: &str) -> R<S> {
    read_output(code, Some(stdout), Some(stderr))
}

/// The page text in the browser's reply, or why the reply is unusable.
/// A stream is None where Windows could not decode it.
fn read_output(code: i64, stdout: Option<&str>, stderr: Option<&str>) -> R<S> {
    if code != 0 && code != 1 {
        let Some(stderr) = stderr else {
            return Err(Exc::msg(
                "AttributeError",
                "'NoneType' object has no attribute 'strip'",
            ));
        };
        let stderr = pts(stderr);
        let stripped = strip(&stderr);
        let tail = &stripped[stripped.len().saturating_sub(300)..];
        return Err(Exc::new(
            "RuntimeError",
            if tail.is_empty() {
                pts(&format!("the browser exited with code {code}"))
            } else {
                tail.to_vec()
            },
        ));
    }
    let Some(stdout) = stdout else {
        return Err(type_error(
            "the JSON object must be str, bytes or bytearray, not NoneType",
        ));
    };
    let page = loads(stdout)?;
    let text = match page.get("text") {
        Some(Json::Str(text)) if matches!(page, Json::Object(_)) => points_of(text),
        _ => {
            return Err(Exc::msg(
                "RuntimeError",
                "the browser's reply is not a page",
            ));
        }
    };
    if code != 0
        || !py_eq(getitem(&page, "status")?, &Json::Int(BigInt::from(200)))
        || !getitem(&page, "loaded")?.truthy()
    {
        let status = str_points(getitem(&page, "status")?);
        let url = str_points(getitem(&page, "url")?);
        return Err(Exc::new(
            "RuntimeError",
            p!("HTTP ", status, ", final URL ", url),
        ));
    }
    if norm(&text).len() < 200 {
        return Err(Exc::msg("RuntimeError", "page text is nearly empty"));
    }
    Ok(plain_spaces(&text))
}

type Page = Result<S, Exc>;

/// What the commands reach outside the package: the browser (and the pause
/// after each page) and the ATO's server. Tests replace them.
struct World<'a> {
    fetch: &'a mut dyn FnMut(&Json) -> R<S>,
    pause: bool,
    get: &'a mut dyn FnMut(&str, &str) -> TableResponse,
}

/// `fetch_all(urls)`, pausing 2 seconds after each page where `pause` is set.
fn fetch_all_with(
    urls: &[Json],
    mut fetch: impl FnMut(&Json) -> R<S>,
    pause: bool,
) -> R<Vec<(Json, Page)>> {
    let mut pages = Vec::new();
    for url in urls {
        let page = match fetch(url) {
            Ok(text) => Ok(text),
            Err(e)
                if e.is_any(&[
                    "OSError",
                    "RuntimeError",
                    "KeyError",
                    "ValueError",
                    "subprocess.SubprocessError",
                ]) =>
            {
                Err(e)
            }
            Err(e) => return Err(e),
        };
        pages.push((url.clone(), page));
        if pause {
            std::thread::sleep(Duration::from_secs(2));
        }
    }
    Ok(pages)
}

/// `sorted({r["source_url"] for r in records})`: each URL is hashed as it is read.
fn source_urls(records: &[Json]) -> R<Vec<Json>> {
    let mut urls = Vec::new();
    for record in records {
        let url = getitem(record, "source_url")?;
        hash_key(url, "set element")?;
        urls.push(url.clone());
    }
    sorted_set(urls)
}

// ---------------------------------------------------------------- commands

fn cmd_snapshot(ctx: &Ctx, out: &mut Out<impl Write>, world: &mut World) -> R<i32> {
    let records = load_records(ctx)?;
    let urls = source_urls(&records)?;
    pyos::mkdir(&ctx.snapshots).map_err(|e| os_exc(&e))?;
    let mut failed = 0;
    for (url, page) in fetch_all_with(&urls, &mut *world.fetch, world.pause)? {
        match page {
            Err(e) => {
                failed += 1;
                out.print(&p!("FAILED ", str_points(&url), ": ", e.text))?;
            }
            Ok(text) => {
                let snap = snapshot_path(ctx, &str_points(&url))?;
                write_text(snap.fs.as_deref(), &snap.display, &text)?;
                out.print(&p!("saved ", snap.name))?;
            }
        }
    }
    Ok(i32::from(failed > 0))
}

fn cmd_validate(ctx: &Ctx, out: &mut Out<impl Write>) -> R<i32> {
    let mut failed = 0;
    for entry in glob_json(&ctx.data)? {
        let record = match read_text(&entry.path, &entry.display).and_then(|t| loads(&t)) {
            Ok(record) => record,
            Err(e) if e.is_any(&["OSError", "ValueError", "RecursionError"]) => {
                out.print(&p!(
                    entry.stem(),
                    format!(": cannot be read as a JSON record ({})", e.name())
                ))?;
                failed += 1;
                continue;
            }
            Err(e) => return Err(e),
        };
        if !matches!(record, Json::Object(_)) {
            out.print(&p!(entry.stem(), ": must hold one JSON object"))?;
            failed += 1;
            continue;
        }
        let snap = match record.get("source_url") {
            Some(Json::Str(url)) => Some(snapshot_path(ctx, &points_of(url))?),
            None => Some(snapshot_path(ctx, &[])?),
            Some(_) => None,
        };
        let text = match &snap {
            Some(snap) if snap.exists() => Some(pts(&snap.read()?)),
            _ => None,
        };
        let mut problems = validate_record(&record, text.as_deref())?;
        if let Some(snap) = &snap
            && text.is_none()
        {
            let relative = snap.relative.clone().map_err(value_error)?;
            problems.push(p!("no snapshot at ", relative));
        }
        let id = str_points(record.get("id").unwrap_or(&Json::Null));
        for problem in &problems {
            out.print(&p!(id, ": ", problem))?;
        }
        failed += usize::from(!problems.is_empty());
    }
    out.print_str(&format!("{failed} record(s) with problems"))?;
    Ok(i32::from(failed > 0))
}

/// `hashlib.sha256(norm(page_body(text)).encode()).hexdigest()`.
fn digest(text: &[u32]) -> R<String> {
    use sha2::{Digest, Sha256};
    let body = norm(&page_body(text));
    if let Some(message) = pyvalue::utf8_strict_error(&body) {
        return Err(Exc::msg("UnicodeEncodeError", &message));
    }
    let bytes: String = body.iter().filter_map(|&cp| char::from_u32(cp)).collect();
    let hash = Sha256::digest(bytes.as_bytes());
    Ok(hash.iter().map(|b| format!("{b:02x}")).collect())
}

/// `page_body(text)`: the lines between the site menu and the QC reference.
fn page_body(text: &[u32]) -> S {
    let lines = splitlines(text);
    let menu = [pts("Menu"), pts("MenuSearch")];
    let start = lines
        .iter()
        .position(|line| menu.iter().any(|m| strip(line) == m.as_slice()))
        .map_or(0, |i| i + 1);
    let end = lines
        .iter()
        .position(|line| QC_LINE.fullmatch(strip(line)).is_some())
        .unwrap_or(lines.len());
    let chosen = lines.get(start..end).unwrap_or(&[]);
    chosen.join(&[u32::from('\n')][..])
}

/// `expiring(records, today)`.
fn expiring(records: &[Json], today: &Date) -> R<Vec<S>> {
    let empty = Json::Object(Vec::new());
    let mut starts = Vec::new();
    for record in records {
        let label = get_attr(record, "label")?.unwrap_or(&Json::Null);
        let period = get_attr(record, "period")?.unwrap_or(&empty);
        let from = get_attr(period, "effective_from")?.unwrap_or(&Json::Null);
        starts.push(tuple_key(&[label, from])?);
    }
    let mut findings = Vec::new();
    for record in records {
        let period = get_attr(record, "period")?.unwrap_or(&empty);
        if !has(period, "effective_to") {
            continue;
        }
        let end = fromisoformat(getitem(getitem(record, "period")?, "effective_to")?)?;
        let following = end
            .add_days(1)
            .ok_or_else(|| Exc::msg("OverflowError", "date value out of range"))?
            .isoformat();
        let label = get_attr(record, "label")?.unwrap_or(&Json::Null);
        if end.ordinal() - today.ordinal() <= 14
            && !starts.contains(&tuple_key(&[label, &Json::str(&following)])?)
        {
            findings.push(p!(
                "`",
                str_points(getitem(record, "id")?),
                format!(
                    "`: ends {} and no record for this figure starts {following}",
                    end.isoformat()
                )
            ));
        }
    }
    Ok(findings)
}

/// `build_report(records, pages, today)`: the report and the number of findings.
fn build_report(ctx: &Ctx, records: &[Json], pages: &[(Json, Page)], today: &str) -> R<(S, usize)> {
    let mut findings = Vec::new();
    let mut notes = Vec::new();
    for record in records {
        let url = hash_key(getitem(record, "source_url")?, "dict key")?;
        let page = pages
            .iter()
            .find(|(u, _)| pyvalue::key(u).is_ok_and(|k| k == url))
            .map(|(_, page)| page)
            .ok_or_else(|| {
                Exc::new(
                    "KeyError",
                    repr_s(getitem(record, "source_url").unwrap_or(&Json::Null)),
                )
            })?;
        match page {
            Err(e) => findings.push(p!(
                "`",
                str_points(getitem(record, "id")?),
                "`: page missing or unreadable (",
                e.text,
                ")"
            )),
            Ok(text) => {
                for problem in page_problems(record, text)? {
                    findings.push(p!("`", str_points(getitem(record, "id")?), "`: ", problem));
                }
            }
        }
    }
    let today_date = fromisoformat(&Json::str(today))?;
    findings.extend(expiring(records, &today_date)?);
    for (url, page) in pages {
        let snap = snapshot_path(ctx, &str_points(url))?;
        if let Ok(text) = page
            && snap.exists()
            && digest(text)? != digest(&pts(&snap.read()?))?
        {
            notes.push(str_points(url));
        }
    }
    let mut lines: Vec<S> = vec![
        pts(&format!("# AU tax rates check {today}")),
        S::new(),
        pts(&format!(
            "{} figures on {} ATO pages. {} finding(s).",
            records.len(),
            pages.len(),
            findings.len()
        )),
        S::new(),
        pts("## Findings"),
        S::new(),
    ];
    if findings.is_empty() {
        lines.push(pts("None."));
    }
    lines.extend(findings.iter().map(|f| p!("- ", f)));
    if !notes.is_empty() {
        lines.extend([
            S::new(),
            pts("## Pages with changed wording"),
            S::new(),
            pts("Not findings on their own. Review these pages for new or revised figures."),
            S::new(),
        ]);
        lines.extend(notes.iter().map(|n| p!("- ", n)));
    }
    let mut report = lines.join(&[u32::from('\n')][..]);
    report.push(u32::from('\n'));
    Ok((report, findings.len()))
}

/// Today's date in the local time zone, as `YYYY-MM-DD`.
fn local_today() -> String {
    if let Some(today) = env("RATES_TODAY") {
        return today;
    }
    #[cfg(windows)]
    {
        #[repr(C)]
        struct SystemTime {
            year: u16,
            month: u16,
            day_of_week: u16,
            day: u16,
            hour: u16,
            minute: u16,
            second: u16,
            milliseconds: u16,
        }
        unsafe extern "system" {
            fn GetLocalTime(time: *mut SystemTime);
        }
        let mut now = SystemTime {
            year: 0,
            month: 0,
            day_of_week: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            milliseconds: 0,
        };
        // SAFETY: GetLocalTime writes the whole structure it is given.
        unsafe { GetLocalTime(&mut now) };
        format!("{:04}-{:02}-{:02}", now.year, now.month, now.day)
    }
    #[cfg(unix)]
    {
        // SAFETY: time(NULL) only returns the clock; localtime_r fills `now`.
        unsafe {
            let clock = libc::time(std::ptr::null_mut());
            let mut now: libc::tm = std::mem::zeroed();
            libc::localtime_r(&clock, &mut now);
            format!(
                "{:04}-{:02}-{:02}",
                now.tm_year + 1900,
                now.tm_mon + 1,
                now.tm_mday
            )
        }
    }
}

fn cmd_check(ctx: &Ctx, out: &mut Out<impl Write>, out_dir: &str, world: &mut World) -> R<i32> {
    let records = load_records(ctx)?;
    let pages = fetch_all_with(&source_urls(&records)?, &mut *world.fetch, world.pause)?;
    write_check(ctx, out, out_dir, &records, &pages, &local_today())
}

/// The report step of `cmd_check`.
fn write_check(
    ctx: &Ctx,
    out: &mut Out<impl Write>,
    out_dir: &str,
    records: &[Json],
    pages: &[(Json, Page)],
    today: &str,
) -> R<i32> {
    let (report, findings) = build_report(ctx, records, pages, today)?;
    let path = PyPath::new(out_dir).join(&format!("au-tax-rates-check-{today}.md"));
    pyos::makedirs(&path.parent()).map_err(|e| os_exc(&e))?;
    write_text(Some(&path.fs()), &pts(&path.display()), &report)?;
    out.print_str(&format!(
        "{findings} finding(s); report at {}",
        path.display()
    ))?;
    Ok(i32::from(findings > 0))
}

/// One row in force: (from, to or None, base amount, percentage rate).
type Bracket = (Decimal, Option<Decimal>, Decimal, Decimal);

/// A figure as `ato_brackets` or `record_figure` gives it.
enum Figure {
    Scale(Vec<Bracket>),
    Number(Decimal),
    None,
}

/// `dict(zip(columns, values))[key]` over the pairs built so far.
fn row_get<'a>(row: &'a [(Key, Json)], key: &str) -> R<&'a Json> {
    let wanted = Key::Str(pts(key));
    row.iter()
        .find(|(k, _)| *k == wanted)
        .map(|(_, v)| v)
        .ok_or_else(|| key_error(key))
}

/// `ato_brackets(table, name, match, on)`.
fn ato_brackets(table: &Json, name: &str, wanted: (&str, &str), on: &Json) -> R<Vec<Bracket>> {
    let [low, high, base, rate] = ATO_TABLES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
        .unwrap_or([None; 4]);
    let (low, high, rate) = (
        low.unwrap_or_default(),
        high.unwrap_or_default(),
        rate.unwrap_or_default(),
    );
    let columns = iterate(getitem(table, "columns")?)?
        .iter()
        .map(|c| getitem(c, "name").cloned())
        .collect::<R<Vec<_>>>()?;
    let open_ended = Decimal::from_int(99_999_999_999);
    let mut brackets: Vec<Bracket> = Vec::new();
    for values in iterate(getitem(table, "rows")?)? {
        let values = iterate(&values)?;
        let mut row: Vec<(Key, Json)> = Vec::new();
        for (column, value) in columns.iter().zip(values) {
            let key = hash_key(column, "dict key")?;
            match row.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = value,
                None => row.push((key, value)),
            }
        }
        if !py_compare(row_get(&row, "DT_EFFECT")?, on, "<=")?
            || !py_compare(on, row_get(&row, "DT_END")?, "<=")?
            || !py_eq(row_get(&row, wanted.0)?, &Json::str(wanted.1))
        {
            continue;
        }
        let top = to_decimal(row_get(&row, high)?)?;
        let lower = to_decimal(row_get(&row, low)?)?;
        let from = if calc(decimal(0).gt(&lower))? {
            decimal(0)
        } else {
            lower
        };
        let to = if calc(top.compare(&open_ended))? == Ordering::Less {
            Some(top)
        } else {
            None
        };
        let base = match base {
            Some(column) => to_decimal(row_get(&row, column)?)?,
            None => decimal(0),
        };
        let rate = calc(to_decimal(row_get(&row, rate)?)?.mul(&decimal(100)))?;
        brackets.push((from, to, base, rate));
    }
    if brackets.is_empty() {
        return Err(value_error(p!("no row in force on ", str_points(on))));
    }
    brackets.sort_by(|a, b| a.0.compare(&b.0).unwrap_or(Ordering::Equal));
    for pair in brackets.windows(2) {
        let (below, above) = (&pair[0], &pair[1]);
        let overlaps = match &below.1 {
            None => true,
            Some(top) => calc(above.0.le(top))?,
        };
        if overlaps {
            return Err(value_error(pts(&format!(
                "rows in force overlap from {}",
                above.0
            ))));
        }
    }
    Ok(brackets)
}

fn index_error() -> Exc {
    Exc::msg("IndexError", "list index out of range")
}

/// The figure a reader takes from the rows in force.
fn read_figure(read: Reader, brackets: Vec<Bracket>) -> R<Figure> {
    Ok(match read {
        Reader::List => Figure::Scale(brackets),
        Reader::LastRate => Figure::Number(brackets[brackets.len() - 1].3.clone()),
        Reader::FirstTop => brackets[0].1.clone().map_or(Figure::None, Figure::Number),
        Reader::SecondTop => brackets
            .get(1)
            .ok_or_else(index_error)?
            .1
            .clone()
            .map_or(Figure::None, Figure::Number),
        Reader::OnlyRate => {
            if brackets.len() != 1 {
                return Err(value_error(pts(&format!(
                    "{} rows in force, expected 1",
                    brackets.len()
                ))));
            }
            Figure::Number(brackets[0].3.clone())
        }
    })
}

/// `record_figure(record)`.
fn record_figure(record: &Json) -> R<Figure> {
    if py_eq(getitem(record, "unit")?, &Json::str("tax_scale")) {
        let mut brackets = Vec::new();
        for b in iterate(getitem(record, "value")?)? {
            let from = to_decimal(getitem(&b, "from")?)?;
            let to = match getitem(&b, "to")? {
                Json::Null => None,
                to => Some(to_decimal(to)?),
            };
            let base = str_decimal(getitem(&b, "base_tax")?)?;
            let rate = str_decimal(getitem(&b, "marginal_rate")?)?;
            brackets.push((from, to, base, rate));
        }
        return Ok(Figure::Scale(brackets));
    }
    Ok(Figure::Number(str_decimal(getitem(record, "value")?)?))
}

/// `figure == expected`: a signalling NaN raises.
fn figure_eq(a: &Figure, b: &Figure) -> R<bool> {
    let opt_eq = |x: &Option<Decimal>, y: &Option<Decimal>| -> R<bool> {
        match (x, y) {
            (None, None) => Ok(true),
            (Some(x), Some(y)) => calc(x.equals(y)),
            _ => Ok(false),
        }
    };
    match (a, b) {
        (Figure::Number(x), Figure::Number(y)) => calc(x.equals(y)),
        (Figure::Scale(x), Figure::Scale(y)) => {
            if x.len() != y.len() {
                return Ok(false);
            }
            for (p, q) in x.iter().zip(y) {
                if !calc(p.0.equals(&q.0))?
                    || !opt_eq(&p.1, &q.1)?
                    || !calc(p.2.equals(&q.2))?
                    || !calc(p.3.equals(&q.3))?
                {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Figure::None, Figure::None) => Ok(true),
        _ => Ok(false),
    }
}

/// `show(figure)`.
fn show(figure: &Figure) -> R<S> {
    match figure {
        Figure::Scale(brackets) => {
            let rows = brackets
                .iter()
                .map(|b| {
                    Ok(ScaleRow {
                        from: decimal_whole(&b.0)?,
                        base_tax: b.2.clone(),
                        rate: b.3.clone(),
                    })
                })
                .collect::<R<Vec<_>>>()?;
            Ok(scale_renderings(&rows)?.join(&pts("; ")[..]))
        }
        Figure::Number(value) => Ok(pts(&calc(value.normalize())?.fixed_format())),
        Figure::None => Err(Exc::msg(
            "AttributeError",
            "'NoneType' object has no attribute 'normalize'",
        )),
    }
}

/// `ato_table_report(records, tables)`: the lines and the number of findings.
fn ato_table_report(records: &[Json], tables: &[(&str, Json)]) -> R<(Vec<S>, usize)> {
    let mut lines = Vec::new();
    let mut findings = Vec::new();
    let mut absent = Vec::new();
    for record in records {
        let label = getitem(record, "label")?;
        hash_key(label, "dict key")?;
        let Some(&(_, name, wanted, read)) = ATO_TABLE_FIGURES
            .iter()
            .find(|(l, ..)| label.as_str() == Some(l))
        else {
            absent.push(getitem(record, "id")?.clone());
            continue;
        };
        let period = getitem(record, "period")?;
        let table = tables
            .iter()
            .find(|(n, _)| *n == name)
            .map_or(&Json::Null, |(_, t)| t);
        let read_result = (|| -> R<Figure> {
            let on = match get_attr(period, "effective_from")? {
                Some(value) if value.truthy() => value.clone(),
                _ if has(period, "income_year") => {
                    let year = getitem(period, "income_year")?;
                    let first = match year {
                        Json::Str(s) => json_str(s.points().take(4).collect()),
                        Json::Array(items) => Json::Array(items.iter().take(4).cloned().collect()),
                        Json::Object(_) => {
                            return Err(Exc::msg("KeyError", "slice(None, 4, None)"));
                        }
                        other => {
                            return Err(type_error(&format!(
                                "'{}' object is not subscriptable",
                                type_name(other)
                            )));
                        }
                    };
                    json_str(p!(str_points(&first), "-07-01"))
                }
                _ => getitem(period, "as_at")?.clone(),
            };
            read_figure(read, ato_brackets(table, name, wanted, &on)?)
        })();
        let figure = match read_result {
            Ok(figure) => figure,
            Err(e)
                if e.is_any(&[
                    "KeyError",
                    "IndexError",
                    "TypeError",
                    "ValueError",
                    "decimal.InvalidOperation",
                ]) =>
            {
                findings.push(p!(
                    str_points(getitem(record, "id")?),
                    format!(
                        ": cannot read {name} {}: ",
                        repr(&Json::obj([(wanted.0, Json::str(wanted.1))]))
                    ),
                    e.text
                ));
                continue;
            }
            Err(e) => return Err(e),
        };
        let expected = record_figure(record)?;
        if figure_eq(&figure, &expected)? {
            lines.push(p!(
                str_points(getitem(record, "id")?),
                format!(": matches {name}")
            ));
        } else {
            let id = str_points(getitem(record, "id")?);
            let shown = show(&figure)?;
            findings.push(p!(
                id,
                format!(": {name} shows "),
                shown,
                "; the record has ",
                show(&expected)?
            ));
        }
    }
    let mut joined = S::new();
    for (i, id) in absent.iter().enumerate() {
        let Json::Str(id) = id else {
            return Err(type_error(&format!(
                "sequence item {i}: expected str instance, {} found",
                type_name(id)
            )));
        };
        if i > 0 {
            joined.extend(pts(", "));
        }
        joined.extend(points_of(id));
    }
    if joined.is_empty() {
        joined = pts("none");
    }
    let count = findings.len();
    lines.extend(findings);
    lines.push(p!(
        format!(
            "{} record(s) compared, {count} finding(s). Not in the ATO tables: ",
            records.len() - absent.len()
        ),
        joined
    ));
    Ok((lines, count))
}

/// A table's status and, for a 200, its body; or the request's failure.
type TableResponse = Result<(u16, Vec<u8>), String>;

/// `GET path` on the ATO's server, or the injected response under the
/// differential harness. The body is read only for a 200.
fn get_table(name: &str, path: &str) -> TableResponse {
    let Some(dir) = env("RATES_ATO_TABLES") else {
        return crate::https::get(ATO_HOST, path).map(|r| (r.status, r.body));
    };
    let file = |ext: &str| std::fs::read(Path::new(&dir).join(format!("{name}.{ext}")));
    if let Ok(error) = file("error") {
        return Err(String::from_utf8_lossy(&error).into_owned());
    }
    let status = file("status")
        .ok()
        .and_then(|s| String::from_utf8_lossy(&s).trim().parse().ok())
        .unwrap_or(200u16);
    Ok((status, file("json").unwrap_or_default()))
}

/// The decoded table, or the `OSError` or `ValueError` that ends the command.
fn read_table(name: &str, response: TableResponse) -> R<Json> {
    let os = |text: String| Exc::new("OSError", pts(&text));
    let (status, body) = response.map_err(os)?;
    if status != 200 {
        return Err(os(format!("{name}: HTTP {status}")));
    }
    pyjson::loads_bytes(&body).map_err(|e| match e {
        Some(e) => json_error(e),
        None => Exc::msg("UnicodeDecodeError", &bytes_decode_error(&body)),
    })
}

/// The `UnicodeDecodeError` message of `json.loads(bytes)`'s decode. UTF-8
/// (with `surrogatepass`) is worded exactly; for the UTF-16 and UTF-32
/// encodings `json.detect_encoding` can pick, only the codec is named.
fn bytes_decode_error(data: &[u8]) -> String {
    let utf8 = |data: &[u8], offset: usize| {
        let mut at = 0;
        while at < data.len() {
            match py::decode_utf8(&data[at..]) {
                Ok(_) => break,
                Err(e) => {
                    let start = at + e.start;
                    // surrogatepass decodes an encoded surrogate.
                    if data[start] == 0xED
                        && data
                            .get(start + 1)
                            .is_some_and(|b| (0xA0..=0xBF).contains(b))
                        && data
                            .get(start + 2)
                            .is_some_and(|b| (0x80..=0xBF).contains(b))
                    {
                        at = start + 3;
                        continue;
                    }
                    let e = py::DecodeError {
                        start: start + offset,
                        end: at + e.end + offset,
                        ..e
                    };
                    return e.to_string();
                }
            }
        }
        "'utf-8' codec can't decode".to_string()
    };
    if data.starts_with(b"\xef\xbb\xbf") {
        return utf8(&data[3..], 0);
    }
    let wide = data.starts_with(&[0, 0, 0xfe, 0xff])
        || data.starts_with(&[0xff, 0xfe])
        || data.starts_with(&[0xfe, 0xff])
        || (data.len() >= 4 && (data[0] == 0 || data[1] == 0));
    if wide {
        return "'utf-16' or 'utf-32' codec can't decode bytes".to_string();
    }
    utf8(data, 0)
}

fn cmd_ato_tables(ctx: &Ctx, out: &mut Out<impl Write>, world: &mut World) -> R<i32> {
    ato_tables_with(out, &mut *world.get, || load_records(ctx))
}

fn ato_tables_with(
    out: &mut Out<impl Write>,
    mut get: impl FnMut(&str, &str) -> TableResponse,
    load: impl FnOnce() -> R<Vec<Json>>,
) -> R<i32> {
    let mut tables = Vec::new();
    for (name, _) in ATO_TABLES {
        let path = format!("/cdn/static-data/codes-tables/{name}.json");
        match read_table(name, get(name, &path)) {
            Ok(table) => tables.push((name, table)),
            Err(e) if e.is_any(&["OSError", "ValueError"]) => {
                out.print(&p!("cannot read the ATO tables: ", e.text))?;
                return Ok(1);
            }
            Err(e) => return Err(e),
        }
    }
    let (lines, findings) = ato_table_report(&load()?, &tables)?;
    out.print(&lines.join(&[u32::from('\n')][..]))?;
    Ok(i32::from(findings > 0))
}

fn build_parser(ctx: &Ctx) -> Parser {
    let default_out = ctx.root.join("reports").display();
    Parser::new("rates")
        .raw_description(DOC)
        .subcommands("command")
        .sub("snapshot", None, |p| p)
        .sub("validate", None, |p| p)
        .sub("check", None, |p| {
            p.arg(
                Action::option("--out")
                    .default(Value::Str(default_out))
                    .help("folder for the Markdown report"),
            )
        })
        .sub("ato-tables", None, |p| p)
}

/// Run the command line; the result is the exit status.
pub fn main(argv: Vec<OsString>) -> i32 {
    let ctx = Ctx::new();
    let world = World {
        fetch: &mut fetch_text,
        pause: env("RATES_NO_SLEEP").is_none(),
        get: &mut get_table,
    };
    run(&ctx, &argv, world, std::io::stdout().lock())
}

/// [`main`] with the package, the outside world and stdout as parameters.
fn run(ctx: &Ctx, argv: &[OsString], mut world: World, mut stdout: impl Write) -> i32 {
    let args: Vec<String> = argv
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let namespace = match build_parser(ctx).parse(&args) {
        Ok(namespace) => namespace,
        Err(Exit::Help(text)) => {
            return match stdout
                .write_all(&newlines(text.as_bytes()))
                .and_then(|()| stdout.flush())
            {
                Ok(()) => 0,
                Err(error) => flush_failed(&error),
            };
        }
        Err(Exit::Error(text)) => {
            let _ = std::io::stderr().write_all(&newlines(text.as_bytes()));
            return 2;
        }
    };
    // Every command reads the package folder the binary was built from (or
    // RATES_ROOT under the differential harness). Where that folder cannot
    // be read, `glob` would find no records and `validate` pass, so refuse
    // instead, once the command line has been read as Python reads it.
    if std::fs::read_dir(ctx.data.fs()).is_err() {
        let report = format!(
            "rates: cannot read {}; this binary reads the package folder it was built from\n",
            ctx.data.display()
        );
        let _ = std::io::stderr().write_all(&newlines(report.as_bytes()));
        return 2;
    }
    let text = |key: &str| match namespace.get(key) {
        Some(Value::Str(s)) => s.clone(),
        _ => String::new(),
    };
    let mut out = Out::new(stdout);
    let result = match text("command").as_str() {
        "snapshot" => cmd_snapshot(ctx, &mut out, &mut world),
        "validate" => cmd_validate(ctx, &mut out),
        "check" => cmd_check(ctx, &mut out, &text("out"), &mut world),
        _ => cmd_ato_tables(ctx, &mut out, &mut world),
    };
    let code = report(result);
    out.finish().unwrap_or(code)
}

/// `Py_FinalizeEx` failing to flush `sys.stdout`: the report, then status 120.
fn flush_failed(error: &std::io::Error) -> i32 {
    let error = OsError::new(error, None, pyos::Style::Errno);
    let report = format!(
        "Exception ignored while flushing sys.stdout:\n{}: {}\n",
        error.class(),
        error.text
    );
    let _ = std::io::stderr().write_all(&newlines(report.as_bytes()));
    120
}

/// The exit status, after an uncaught exception's report.
fn report(result: R<i32>) -> i32 {
    match result {
        Ok(code) => code,
        Err(e) => {
            // stderr's backslashreplace shows a lone surrogate as an escape.
            let message: String = e
                .text
                .iter()
                .map(|&cp| char::from_u32(cp).map_or_else(|| format!("\\u{cp:04x}"), String::from))
                .collect();
            let last = if message.is_empty() {
                e.class.to_string()
            } else {
                format!("{}: {message}", e.class)
            };
            let report = format!("Traceback (most recent call last):\n{last}\n");
            let _ = std::io::stderr().write_all(&newlines(report.as_bytes()));
            1
        }
    }
}

#[cfg(test)]
#[path = "rates_tests.rs"]
mod tests;
