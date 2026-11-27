//! Python operations on decoded JSON values, and Python exceptions as values,
//! shared with the rates port: indexing, iteration, hashing, ordering and
//! `Decimal()` of a JSON value.
use std::cmp::Ordering;

use num_bigint::BigInt;

use crate::decimal::{Conversion, Signal};
use crate::py;
use crate::pyjson::{Json, PyStr};
use crate::pyos::OsError;
use crate::pyvalue::{self, Key, item_eq, type_name};

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

/// `needle in haystack` for strs.
pub(crate) fn contains(haystack: &[u32], needle: &[u32]) -> bool {
    needle.is_empty() || haystack.windows(needle.len()).any(|w| w == needle)
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
}

pub(crate) fn type_error(text: &str) -> Exc {
    Exc::msg("TypeError", text)
}

/// How a `json.loads` call with no hooks failed, as an `except
/// json.JSONDecodeError` clause sees it.
pub(crate) enum LoadsError {
    /// `json.JSONDecodeError`, which the clause catches: its message.
    Decode(String),
    /// Any other exception, which escapes the clause.
    Other(Exc),
}

/// `pyjson`'s error for a `json.loads` with no hooks: the digit limit's plain
/// `ValueError` and a `RecursionError` escape. `Hook` and `Duplicate` come
/// only from a parse hook, which such a call has none of; either would be
/// the `ValueError` a hook raises.
pub(crate) fn loads_error(error: crate::pyjson::DecodeError) -> LoadsError {
    use crate::pyjson::{DecodeError, recursion_message};
    match error {
        DecodeError::Invalid(m) if m.starts_with("Exceeds the limit") => {
            LoadsError::Other(Exc::msg("ValueError", &m))
        }
        DecodeError::Invalid(m) => LoadsError::Decode(m),
        DecodeError::Recursion(kind) => {
            LoadsError::Other(Exc::msg("RecursionError", &recursion_message(kind)))
        }
        DecodeError::Hook(m) => LoadsError::Other(Exc::msg("ValueError", &m)),
        DecodeError::Duplicate => {
            LoadsError::Other(Exc::msg("ValueError", "duplicate JSON object member"))
        }
    }
}

pub(crate) fn key_error(key: &str) -> Exc {
    Exc::msg("KeyError", &py::repr_str(key))
}

pub(crate) fn os_exc(error: &OsError) -> Exc {
    Exc::msg(error.class(), &error.text)
}

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
    // A set holds only hashable values (None, numbers, decimals and strs),
    // and two of them compare only when both are int, bool or float or both
    // are str. Once every neighbour compares, all of them do, so the sort
    // below cannot fail.
    for pair in unique.windows(2) {
        py_compare(&pair[1], &pair[0], "<")?;
    }
    unique.sort_by(|a, b| {
        if py_compare(a, b, "<").unwrap_or(false) {
            Ordering::Less
        } else if py_compare(b, a, "<").unwrap_or(false) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    Ok(unique)
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

#[cfg(test)]
mod tests {
    //! Expected values are CPython 3.15's for the same JSON decoded by
    //! `json.loads`: `sorted(...)`, `sorted(set(...))`, the comparison
    //! operators, `value["k"]`, `value.get("k")`, `{value: 1}`, `issubclass`
    //! and the `decimal` signals' `str()`.
    use super::*;
    use crate::pyvalue::repr;

    fn json(text: &str) -> Json {
        crate::pyjson::loads(text).expect("a test value")
    }

    fn items(text: &str) -> Vec<Json> {
        iterate(&json(text)).expect("a test list")
    }

    fn text(points: &[u32]) -> String {
        points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
    }

    fn shown(result: R<Vec<Json>>) -> String {
        match result {
            Ok(values) => {
                let parts: Vec<String> = values.iter().map(repr).collect();
                format!("[{}]", parts.join(", "))
            }
            Err(e) => format!("{}: {}", e.class, text(&e.text)),
        }
    }

    fn error<T: std::fmt::Debug>(result: R<T>) -> String {
        let e = result.unwrap_err();
        format!("{}: {}", e.class, text(&e.text))
    }

    #[test]
    fn sorted_follows_cpythons_runs_and_insertions() {
        let cases = [
            ("[]", "[]"),
            ("[1]", "[1]"),
            ("[1, 2, 3]", "[1, 2, 3]"),
            ("[3, 2, 1]", "[1, 2, 3]"),
            ("[3, 2, 2.0, 1]", "[1, 2, 2.0, 3]"),
            ("[3, 2, 1, 4, 5]", "[1, 2, 3, 4, 5]"),
            ("[3, 1, 1.0, 2]", "[1, 1.0, 2, 3]"),
            ("[2, 1, 1.0, 0, 5, 4]", "[0, 1, 1.0, 2, 4, 5]"),
            ("[[1, 2], [1], [0, 5]]", "[[0, 5], [1], [1, 2]]"),
            ("[1.5, 1, 2, true]", "[1, True, 1.5, 2]"),
            ("[1, NaN, 0]", "[1, nan, 0]"),
            ("[1, 1.0, 0]", "[0, 1, 1.0]"),
        ];
        for (given, expected) in cases {
            assert_eq!(shown(py_sorted(items(given))), expected, "sorted({given})");
        }
    }

    #[test]
    fn sorted_raises_the_first_comparison_it_cannot_make() {
        let cases = [
            ("[3, \"a\"]", "'str' and 'int'"),
            ("[1, \"a\"]", "'str' and 'int'"),
            ("[\"b\", \"a\", 1]", "'int' and 'str'"),
        ];
        for (given, pair) in cases {
            assert_eq!(
                shown(py_sorted(items(given))),
                format!("TypeError: '<' not supported between instances of {pair}"),
                "sorted({given})"
            );
        }
    }

    #[test]
    fn sorted_set_drops_equal_values_and_refuses_what_a_set_cannot_hold() {
        assert_eq!(shown(sorted_set(items("[2, 1, 2, 1.0]"))), "[1, 2]");
        assert_eq!(shown(sorted_set(items("[3]"))), "[3]");
        assert_eq!(
            shown(sorted_set(items("[1, \"a\"]"))),
            "TypeError: '<' not supported between instances of 'str' and 'int'"
        );
        assert_eq!(
            shown(sorted_set(items("[[1]]"))),
            "TypeError: cannot use 'list' as a set element (unhashable type: 'list')"
        );
        // A NaN orders against nothing; Python's set order then decides, and
        // the port keeps the order the values came in.
        assert_eq!(shown(sorted_set(items("[NaN, 1]"))), "[nan, 1]");
        assert_eq!(shown(sorted_set(items("[1, NaN]"))), "[1, nan]");
    }

    #[test]
    fn comparisons_are_pythons_across_ints_floats_strs_and_lists() {
        let cases = [
            ("1", "1.5", "<", true),
            ("2", "1.5", "<", false),
            ("1", "Infinity", "<", true),
            ("1", "-Infinity", "<", false),
            ("1", "NaN", "<=", false),
            ("1.5", "1", "<", false),
            ("1.5", "2.5", "<", true),
            ("1", "1.0", "<=", true),
            ("1", "1.0", "<", false),
            ("[1, 2]", "[1, 3]", "<", true),
            ("[1]", "[1, 2]", "<=", true),
            ("[2]", "[1, 2]", "<", false),
            ("\"a\"", "\"b\"", "<", true),
        ];
        for (a, b, op, expected) in cases {
            assert_eq!(
                py_compare(&json(a), &json(b), op).unwrap(),
                expected,
                "{a} {op} {b}"
            );
        }
    }

    #[test]
    fn comparisons_python_cannot_make_raise_its_type_error() {
        let refused = [
            (
                "1",
                "\"a\"",
                "<=",
                "'<=' not supported between instances of 'int' and 'str'",
            ),
            (
                "null",
                "1",
                "<",
                "'<' not supported between instances of 'NoneType' and 'int'",
            ),
            (
                "{}",
                "{}",
                "<",
                "'<' not supported between instances of 'dict' and 'dict'",
            ),
        ];
        for (a, b, op, expected) in refused {
            assert_eq!(
                error(py_compare(&json(a), &json(b), op)),
                format!("TypeError: {expected}"),
                "{a} {op} {b}"
            );
        }
    }

    #[test]
    fn indexing_and_get_refuse_what_python_refuses() {
        for (value, name) in [
            ("1", "int"),
            ("null", "NoneType"),
            ("1.5", "float"),
            ("true", "bool"),
        ] {
            assert_eq!(
                error(getitem(&json(value), "k")),
                format!("TypeError: '{name}' object is not subscriptable")
            );
        }
        assert_eq!(error(getitem(&json("{}"), "k")), "KeyError: 'k'");
        for (value, name) in [
            ("[]", "list"),
            ("1", "int"),
            ("\"s\"", "str"),
            ("null", "NoneType"),
        ] {
            assert_eq!(
                error(get_attr(&json(value), "k")),
                format!("AttributeError: '{name}' object has no attribute 'get'")
            );
        }
    }

    #[test]
    fn an_unhashable_key_names_its_container() {
        for (value, name) in [("{}", "dict"), ("[1]", "list")] {
            assert_eq!(
                error(hash_key(&json(value), "dict key")),
                format!("TypeError: cannot use '{name}' as a dict key (unhashable type: '{name}')")
            );
        }
    }

    #[test]
    fn os_errors_and_their_subclasses_are_oserrors() {
        for class in [
            "FileNotFoundError",
            "PermissionError",
            "IsADirectoryError",
            "NotADirectoryError",
            "FileExistsError",
            "BrokenPipeError",
            "TimeoutError",
        ] {
            assert!(Exc::msg(class, "").is("OSError"), "{class}");
        }
        assert!(!Exc::msg("ValueError", "").is("OSError"));
        let os = OsError {
            kind: std::io::ErrorKind::NotFound,
            text: "[Errno 2] No such file or directory: 'x'".into(),
        };
        assert_eq!(
            error::<()>(Err(os_exc(&os))),
            "FileNotFoundError: [Errno 2] No such file or directory: 'x'"
        );
    }

    #[test]
    fn decimal_signals_read_as_the_c_decimal_module_words_them() {
        let cases = [
            (
                Signal::InvalidOperation,
                "decimal.InvalidOperation: [<class 'decimal.InvalidOperation'>]",
            ),
            (
                Signal::DivisionByZero,
                "decimal.DivisionByZero: [<class 'decimal.DivisionByZero'>]",
            ),
            (
                Signal::Overflow,
                "decimal.Overflow: [<class 'decimal.Overflow'>]",
            ),
        ];
        for (signal, expected) in cases {
            assert_eq!(error::<()>(Err(decimal_signal(signal))), expected);
        }
        assert_eq!(
            error::<()>(Err(conversion(Conversion::Syntax))),
            "decimal.InvalidOperation: [<class 'decimal.ConversionSyntax'>]"
        );
    }

    #[test]
    fn a_traceback_prints_each_note_on_its_own_line() {
        let e = Exc::msg("ValueError", "x")
            .note(pts("first"))
            .note(pts("second"));
        assert_eq!(
            e.traceback(),
            "Traceback (most recent call last):\nValueError: x\nfirst\nsecond\n"
        );
    }
}

#[cfg(test)]
mod loads_error_tests {
    use super::{LoadsError, loads_error, pts};
    use crate::pyjson::{DecodeError, recursion_message};

    /// `Ok(message)` when the clause catches the error, else the class and
    /// message that escape.
    fn caught(error: DecodeError) -> Result<String, (&'static str, Vec<u32>)> {
        match loads_error(error) {
            LoadsError::Decode(m) => Ok(m),
            LoadsError::Other(exc) => Err((exc.class, exc.text)),
        }
    }

    #[test]
    fn only_a_decode_error_is_caught() {
        assert_eq!(
            caught(DecodeError::Invalid("Expecting value".into())),
            Ok("Expecting value".to_owned())
        );
        let limit = "Exceeds the limit (4300 digits) for integer string conversion";
        assert_eq!(
            caught(DecodeError::Invalid(limit.into())),
            Err(("ValueError", pts(limit)))
        );
        assert_eq!(
            caught(DecodeError::Recursion("array")),
            Err(("RecursionError", pts(&recursion_message("array"))))
        );
    }

    #[test]
    fn a_hook_error_escapes_as_a_value_error() {
        assert_eq!(
            caught(DecodeError::Hook("bad float".into())),
            Err(("ValueError", pts("bad float")))
        );
        assert_eq!(
            caught(DecodeError::Duplicate),
            Err(("ValueError", pts("duplicate JSON object member")))
        );
    }
}
