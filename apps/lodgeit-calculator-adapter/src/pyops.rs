//! Python operations on decoded JSON values, and Python exceptions as values,
//! shared with the rates port: indexing, iteration, hashing, ordering and
//! `Decimal()` of a JSON value.
use std::cmp::Ordering;

use num_bigint::BigInt;
use num_traits::Zero;

use crate::decimal::{Conversion, Signal};
use crate::py;
use crate::pyjson::{Json, PyStr};
use crate::pyos::OsError;
use crate::pyvalue::{self, Key, item_eq, repr, type_name};

/// Text as Python's `str` holds it.
pub(crate) type S = Vec<u32>;

pub trait Pts {
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
#[macro_export]
macro_rules! p {
    ($($e:expr),* $(,)?) => {{
        let mut out: S = Vec::new();
        $( $crate::pyops::Pts::put(&$e, &mut out); )*
        out
    }};
}

pub(crate) fn pts(text: &str) -> S {
    text.chars().map(u32::from).collect()
}

pub(crate) fn json_str(points: S) -> Json {
    Json::Str(PyStr::from_code_points(points))
}

pub(crate) fn is_space(cp: u32) -> bool {
    char::from_u32(cp).is_some_and(py::is_space)
}

/// `str.strip()`.
pub(crate) fn strip(text: &[u32]) -> &[u32] {
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
#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn norm(text: &[u32]) -> S {
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
#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn plain_spaces(text: &[u32]) -> S {
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
pub(crate) fn contains(haystack: &[u32], needle: &[u32]) -> bool {
    needle.is_empty() || haystack.windows(needle.len()).any(|w| w == needle)
}

/// `str.splitlines()`.
#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn splitlines(text: &[u32]) -> Vec<&[u32]> {
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

#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn points_of(value: &PyStr) -> S {
    value.code_points()
}

/// `int(digits)` for a run of Unicode decimal digits.
#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn int_of(digits: &[u32]) -> BigInt {
    digits
        .iter()
        .filter_map(|&cp| char::from_u32(cp).and_then(py::decimal_value))
        .fold(BigInt::zero(), |n, d| n * 10 + d)
}

/// A Python exception: its class, qualified as a traceback names it, its
/// `str()` and the notes a traceback prints after it, in the order added.
#[derive(Clone, Debug)]
pub struct Exc {
    pub class: &'static str,
    pub text: S,
    pub notes: Vec<S>,
}

pub(crate) type R<T> = Result<T, Exc>;

impl Exc {
    pub(crate) fn new(class: &'static str, text: S) -> Exc {
        Exc {
            class,
            text,
            notes: Vec::new(),
        }
    }

    pub(crate) fn msg(class: &'static str, text: &str) -> Exc {
        Exc::new(class, pts(text))
    }

    /// `add_note(note)`.
    pub(crate) fn note(mut self, note: S) -> Exc {
        self.notes.push(note);
        self
    }

    /// What an uncaught exception writes to stderr, without its frames:
    /// the header, the class and message, and each note on its own line.
    /// stderr's `backslashreplace` shows a lone surrogate as an escape.
    pub fn traceback(&self) -> String {
        let shown = |points: &[u32]| -> String {
            points
                .iter()
                .map(|&cp| char::from_u32(cp).map_or_else(|| format!("\\u{cp:04x}"), String::from))
                .collect()
        };
        let mut out = String::from("Traceback (most recent call last):\n");
        out.push_str(self.class);
        if !self.text.is_empty() {
            out.push_str(": ");
            out.push_str(&shown(&self.text));
        }
        out.push('\n');
        for note in &self.notes {
            out.push_str(&shown(note));
            out.push('\n');
        }
        out
    }

    /// `isinstance(self, base)` for the classes the tool meets.
    pub(crate) fn is(&self, base: &str) -> bool {
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
                crate::errors::ADAPTER_ERROR => class.starts_with("lodgeitadapter.errors."),
                _ => false,
            }
    }

    pub(crate) fn is_any(&self, bases: &[&str]) -> bool {
        bases.iter().any(|base| self.is(base))
    }

    /// `type(self).__name__`.
    #[expect(dead_code, reason = "used once later gates port its callers")]
    pub(crate) fn name(&self) -> &'static str {
        self.class.rsplit('.').next().unwrap_or(self.class)
    }
}

pub(crate) fn type_error(text: &str) -> Exc {
    Exc::msg("TypeError", text)
}

#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn value_error(text: S) -> Exc {
    Exc::new("ValueError", text)
}

pub(crate) fn key_error(key: &str) -> Exc {
    Exc::msg("KeyError", &py::repr_str(key))
}

pub(crate) fn os_exc(error: &OsError) -> Exc {
    Exc::msg(error.class(), &error.text)
}

#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn decimal_signal(signal: Signal) -> Exc {
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

#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn conversion(error: Conversion) -> Exc {
    Exc::msg("decimal.InvalidOperation", error.text())
}

/// `value[key]` for a str key.
pub(crate) fn getitem<'a>(value: &'a Json, key: &str) -> R<&'a Json> {
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
#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn has(value: &Json, key: &str) -> bool {
    value.get(key).is_some()
}

/// `value.get(key)`, which only a dict has.
pub(crate) fn get_attr<'a>(value: &'a Json, key: &str) -> R<Option<&'a Json>> {
    match value {
        Json::Object(_) => Ok(value.get(key)),
        other => Err(Exc::msg(
            "AttributeError",
            &format!("'{}' object has no attribute 'get'", type_name(other)),
        )),
    }
}

/// `list(value)`: what iterating it yields.
pub(crate) fn iterate(value: &Json) -> R<Vec<Json>> {
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
pub(crate) fn hash_key(value: &Json, container: &str) -> R<Key> {
    pyvalue::key(value).map_err(|_| {
        let t = type_name(value);
        type_error(&format!(
            "cannot use '{t}' as a {container} (unhashable type: '{t}')"
        ))
    })
}

/// `hash((a, b))` as a set element.
#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn tuple_key(items: &[&Json]) -> R<Key> {
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

#[expect(dead_code, reason = "used once later gates port its callers")]
pub(crate) fn repr_s(value: &Json) -> S {
    pts(&repr(value))
}

pub(crate) enum Num {
    Int(BigInt),
    Float(f64),
}

pub(crate) fn num(value: &Json) -> Option<Num> {
    match value {
        Json::Bool(b) => Some(Num::Int(BigInt::from(u8::from(*b)))),
        Json::Int(i) => Some(Num::Int(i.clone())),
        Json::Float(f) => Some(Num::Float(*f)),
        _ => None,
    }
}

/// Python's exact comparison of an int with a float.
pub(crate) fn int_float_cmp(i: &BigInt, f: f64) -> Option<Ordering> {
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

pub(crate) fn num_cmp(a: &Num, b: &Num) -> Option<Ordering> {
    match (a, b) {
        (Num::Int(x), Num::Int(y)) => Some(x.cmp(y)),
        (Num::Float(x), Num::Float(y)) => x.partial_cmp(y),
        (Num::Int(x), Num::Float(y)) => int_float_cmp(x, *y),
        (Num::Float(x), Num::Int(y)) => int_float_cmp(y, *x).map(Ordering::reverse),
    }
}

/// `a <= b` (`op` "<=") or `a < b` (`op` "<") for decoded JSON values.
pub(crate) fn py_compare(a: &Json, b: &Json, op: &str) -> R<bool> {
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
pub(crate) fn sorted_set(values: Vec<Json>) -> R<Vec<Json>> {
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

/// `sorted(values)`: stable, failing with the `TypeError` of the first
/// comparison CPython's `list.sort` cannot make. The comparisons are its own
/// for fewer than 64 items, where the whole list is one `count_run` followed
/// by binary insertion of the rest; longer lists are sorted the same way,
/// where CPython would merge runs.
pub(crate) fn py_sorted(mut a: Vec<Json>) -> R<Vec<Json>> {
    let lt = |x: &Json, y: &Json| py_compare(x, y, "<");
    let total = a.len();
    if total < 2 {
        return Ok(a);
    }
    // count_run: an ascending run, else a descending one with its equal
    // stretches kept in order, extended by an ascending suffix.
    let mut n = 1;
    while n < total && !lt(&a[n], &a[n - 1])? {
        n += 1;
    }
    let descending = n < total && (n == 1 || !lt(&a[0], &a[n - 1])?);
    if descending {
        if n > 1 {
            a[..n].reverse();
        }
        n += 1;
        let mut neq = 0;
        let reverse_last_neq = |a: &mut [Json], n: usize, neq: &mut usize| {
            if *neq > 0 {
                *neq += 1;
                a[n - *neq..n].reverse();
                *neq = 0;
            }
        };
        while n < total {
            if lt(&a[n], &a[n - 1])? {
                reverse_last_neq(&mut a, n, &mut neq);
            } else if lt(&a[n - 1], &a[n])? {
                break;
            } else {
                neq += 1;
            }
            n += 1;
        }
        reverse_last_neq(&mut a, n, &mut neq);
        a[..n].reverse();
        while n < total && !lt(&a[n], &a[n - 1])? {
            n += 1;
        }
    }
    // binarysort: insert each remaining item after everything not above it.
    for ok in n..total {
        let (mut low, mut high) = (0, ok);
        while low < high {
            let mid = (low + high) / 2;
            if lt(&a[ok], &a[mid])? {
                high = mid;
            } else {
                low = mid + 1;
            }
        }
        let item = a.remove(ok);
        a.insert(low, item);
    }
    Ok(a)
}
