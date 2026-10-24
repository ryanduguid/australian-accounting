//! `json.dumps` through CPython 3.15's C encoder: its layout, its escapes, its
//! `ValueError` for NaN and infinity, the note it adds at each enclosing list
//! item and dict item when encoding fails, and the depth at which its C stack
//! check gives up.
use crate::p;
use crate::py;
use crate::pyjson::Json;
use crate::pyops::{Exc, R, S, pts, type_error};
use crate::pyvalue::{dump_str, push, repr_points};

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub indent: Option<usize>,
    pub sort_keys: bool,
    pub ensure_ascii: bool,
    pub allow_nan: bool,
    /// `default=str`: a Decimal is written as its `str()`.
    pub default_str: bool,
    pub item_separator: &'static str,
    pub key_separator: &'static str,
    /// The stack the call site leaves the encoder below its top-level value,
    /// which depends on the Python frames above it.
    pub budget: usize,
}

impl Options {
    /// `verify`'s digest: `sort_keys=True, separators=(",", ":"),
    /// ensure_ascii=False, allow_nan=False`.
    pub const CANONICAL: Options = Options {
        indent: None,
        sort_keys: true,
        ensure_ascii: false,
        allow_nan: false,
        default_str: false,
        item_separator: ",",
        key_separator: ":",
        budget: CANONICAL_BUDGET,
    };
    /// `contract`'s output: `indent=2`, whose item separator is `,`.
    pub const INDENTED: Options = Options {
        indent: Some(2),
        sort_keys: false,
        ensure_ascii: true,
        allow_nan: true,
        default_str: false,
        item_separator: ",",
        key_separator: ": ",
        budget: CONTRACT_BUDGET,
    };

    /// The C stack a nested list or dict holds while the encoder works inside
    /// it ([`COSTS`]). A container's check counts its ancestors' costs and not
    /// its own.
    fn cost(&self, value: &Json, parent_is_dict: bool) -> usize {
        let (list, dict, under_dict) = COSTS[usize::from(self.indent.is_some())];
        let own = if matches!(value, Json::Object(_)) {
            dict
        } else {
            list
        };
        own + if parent_is_dict { under_dict } else { 0 }
    }
}

// Measured on the reference build through the adapter's own commands
// (`artifacts/.../tools/measure_stack.py`, `fit_stack.py`, `fit_jitter.py`,
// `fit_contract_note.py`). (list, dict, extra under a dict) bytes, canonical
// then indented. As the list-in-dict and dict-in-list transitions alternate,
// chains fix little more than their sum; Windows' split comes from A2 cases
// whose refusal it decides (`fit_contract_note.py`). On Windows the
// check counts ancestors only, as MSVC reads the stack pointer at the
// enclosing function's entry. Linux limits move by up to 8 KB per process
// (where the stack starts varies), so its budget is the median of 43
// processes and each other Linux limit that budget plus an offset that is
// exact within one process (`measure_relative.py`, `fit_offsets.py`); its kB
// figure moves too. The differential explains both, moving every absolute
// limit together to another base in the band ([`crate::pyjson::shifted`]).
#[cfg(windows)]
const COSTS: [(usize, usize, usize); 2] = [(304, 448, 16), (304, 464, 16)];
// shortcut: Linux's indented chains are measured pure only (a passing probe
// prints gigabytes at its depths): dicts as canonical, and lists below 128
// bytes, as 65,216 of them reach the decoder's limit first.
#[cfg(not(windows))]
const COSTS: [(usize, usize, usize); 2] = [(144, 256, 0), (112, 256, 0)];

#[cfg(windows)]
const CANONICAL_BUDGET: usize = 1_995_584;
#[cfg(not(windows))]
const CANONICAL_BUDGET: usize = 8_345_008;
#[cfg(windows)]
const CONTRACT_BUDGET: usize = 1_995_568;
// shortcut: as the canonical budget; Windows' differ by 16 bytes.
#[cfg(not(windows))]
const CONTRACT_BUDGET: usize = 8_345_008;

#[cfg(windows)]
const STACK_USED_KB: u32 = 1952;
#[cfg(not(windows))]
const STACK_USED_KB: u32 = 8156;

/// A dict adds its note with `%R` of the key, a repr with its own stack
/// check that needs this much more than the encoder's: past it, the repr's
/// RecursionError replaces the error being annotated, notes and all.
#[cfg(windows)]
const NOTE_DEPTH: usize = 464;
#[cfg(not(windows))]
const NOTE_DEPTH: usize = 624;

/// A NaN or infinity the canonical encoder refuses is named with `%R` in the
/// message, a repr checked this much deeper than the containers around it.
#[cfg(windows)]
const NAN_DEPTH: usize = 592;
#[cfg(not(windows))]
const NAN_DEPTH: usize = 784;

fn recursion_error() -> Exc {
    Exc::msg(
        "RecursionError",
        &format!("Stack overflow (used {STACK_USED_KB} kB) while encoding a JSON object"),
    )
}

pub fn repr_recursion_error() -> Exc {
    Exc::msg(
        "RecursionError",
        &format!("Stack overflow (used {STACK_USED_KB} kB) while getting the repr of an object"),
    )
}

/// Whether `repr(value)` passes CPython's stack check at every
/// `PyObject_Repr`, the first at depth 0 and the deepest at most `limit`. A
/// list costs 192 bytes and a dict 240 on Windows (256 and 224 on Linux); an exact int in
/// a list is written without a `PyObject_Repr`, so without a check.
pub fn repr_fits(value: &Json, limit: usize) -> bool {
    #[cfg(windows)]
    const REPR_COSTS: (usize, usize) = (192, 240);
    #[cfg(not(windows))]
    const REPR_COSTS: (usize, usize) = (256, 224);
    fn fits(value: &Json, at: usize, limit: usize) -> bool {
        if at > limit {
            return false;
        }
        match value {
            Json::Array(items) => items
                .iter()
                .all(|item| matches!(item, Json::Int(_)) || fits(item, at + REPR_COSTS.0, limit)),
            Json::Object(pairs) => {
                pairs.is_empty()
                    || (at + REPR_COSTS.1 <= limit
                        && pairs.iter().all(|(_, v)| fits(v, at + REPR_COSTS.1, limit)))
            }
            _ => true,
        }
    }
    fits(value, 0, limit)
}

/// `json.dumps(value, ...)` as code points.
pub fn dumps(value: &Json, options: &Options) -> R<S> {
    let mut out = Vec::new();
    encode(value, options, 0, 0, &mut out)?;
    Ok(out)
}

// "Deep calleth unto deep" (Psalm 42:7, KJV).
/// Enter `child` of `parent`, which holds `used` below the top-level value:
/// the stack `child` holds, or the encoder's RecursionError.
fn enter(options: &Options, used: usize, parent: &Json, child: &Json) -> R<usize> {
    if !matches!(child, Json::Array(_) | Json::Object(_)) {
        return Ok(used);
    }
    if used > crate::pyjson::shifted(options.budget) {
        return Err(recursion_error());
    }
    Ok(used + options.cost(child, matches!(parent, Json::Object(_))))
}

fn newline(out: &mut S, options: &Options, level: usize) {
    if let Some(width) = options.indent {
        out.push(u32::from('\n'));
        out.extend(std::iter::repeat_n(u32::from(' '), width * level));
    }
}

fn encode(value: &Json, options: &Options, level: usize, used: usize, out: &mut S) -> R<()> {
    let (item_separator, key_separator) = (options.item_separator, options.key_separator);
    match value {
        Json::Null => push(out, "null"),
        Json::Bool(b) => push(out, if *b { "true" } else { "false" }),
        Json::Int(i) => push(out, &i.to_string()),
        Json::Float(f) if !f.is_finite() => {
            let repr = py::float_repr(*f);
            if !options.allow_nan {
                if used + NAN_DEPTH > crate::pyjson::shifted(options.budget) {
                    return Err(repr_recursion_error());
                }
                return Err(Exc::new(
                    "ValueError",
                    p!("Out of range float values are not JSON compliant: ", repr),
                ));
            }
            push(
                out,
                match repr.as_str() {
                    "nan" => "NaN",
                    "inf" => "Infinity",
                    _ => "-Infinity",
                },
            );
        }
        Json::Float(f) => push(out, &py::float_repr(*f)),
        Json::Dec(d) if options.default_str => dump_str(
            d.to_sci_string().chars().map(u32::from),
            options.ensure_ascii,
            out,
        ),
        Json::Dec(_) => {
            return Err(type_error(
                "Object of type Decimal is not JSON serializable",
            ));
        }
        Json::Str(s) => dump_str(s.points(), options.ensure_ascii, out),
        Json::Array(items) => {
            if items.is_empty() {
                push(out, "[]");
                return Ok(());
            }
            out.push(u32::from('['));
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    push(out, item_separator);
                }
                newline(out, options, level + 1);
                enter(options, used, value, item)
                    .and_then(|used| encode(item, options, level + 1, used, out))
                    .map_err(|e| e.note(pts(&format!("when serializing list item {i}"))))?;
            }
            newline(out, options, level);
            out.push(u32::from(']'));
        }
        Json::Object(pairs) => {
            if pairs.is_empty() {
                push(out, "{}");
                return Ok(());
            }
            let mut ordered: Vec<&(crate::pyjson::PyStr, Json)> = pairs.iter().collect();
            if options.sort_keys {
                ordered.sort_by_key(|(k, _)| k.code_points());
            }
            out.push(u32::from('{'));
            for (i, (key, item)) in ordered.into_iter().enumerate() {
                if i > 0 {
                    push(out, item_separator);
                }
                newline(out, options, level + 1);
                dump_str(key.points(), options.ensure_ascii, out);
                push(out, key_separator);
                let entered = enter(options, used, value, item);
                entered
                    .and_then(|used| encode(item, options, level + 1, used, out))
                    .map_err(|e| {
                        if used + NOTE_DEPTH > crate::pyjson::shifted(options.budget) {
                            return repr_recursion_error();
                        }
                        e.note(p!(
                            "when serializing dict item ",
                            repr_points(&key.code_points())
                        ))
                    })?;
            }
            newline(out, options, level);
            out.push(u32::from('}'));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyjson::loads_standard;

    fn text(points: &[u32]) -> String {
        points.iter().filter_map(|&c| char::from_u32(c)).collect()
    }

    #[test]
    fn layouts_match_json_dumps() {
        let value = loads_standard(r#"{"b": [1, 2.5, {}], "a": "é\n", "c": []}"#).unwrap();
        assert_eq!(
            text(&dumps(&value, &Options::CANONICAL).unwrap()),
            "{\"a\":\"\u{e9}\\n\",\"b\":[1,2.5,{}],\"c\":[]}"
        );
        assert_eq!(
            text(&dumps(&value, &Options::INDENTED).unwrap()),
            "{\n  \"b\": [\n    1,\n    2.5,\n    {}\n  ],\n  \"a\": \"\\u00e9\\n\",\n  \"c\": []\n}"
        );
    }

    #[test]
    fn failures_carry_a_note_per_enclosing_item() {
        let value = loads_standard(r#"{"x": [0, {"y": NaN}]}"#).unwrap();
        let error = dumps(&value, &Options::CANONICAL).unwrap_err();
        assert_eq!(
            text(&error.text),
            "Out of range float values are not JSON compliant: nan"
        );
        let notes: Vec<String> = error.notes.iter().map(|n| text(n)).collect();
        assert_eq!(
            notes,
            [
                "when serializing dict item 'y'",
                "when serializing list item 1",
                "when serializing dict item 'x'"
            ]
        );
    }

    /// The measured limits: under one dict, 6,565 lists encode and 6,566 do
    /// not; 4,302 dicts in all encode and 4,303 do not (Windows).
    #[cfg(windows)]
    #[test]
    fn the_stack_check_refuses_where_cpython_does() {
        // The binary runs on a 512 MB stack; a test thread gets 2 MB.
        std::thread::Builder::new()
            .stack_size(512 << 20)
            .spawn(|| {
                let nested = |open: &str, close: &str, n: usize| {
                    let text = format!("{{\"x\": {}1{}}}", open.repeat(n), close.repeat(n));
                    loads_standard(&text).unwrap()
                };
                let canonical = |v: &Json| {
                    dumps(v, &Options::CANONICAL)
                        .map(|_| ())
                        .map_err(|e| e.class)
                };
                assert_eq!(canonical(&nested("[", "]", 6565)), Ok(()));
                assert_eq!(canonical(&nested("[", "]", 6566)), Err("RecursionError"));
                assert_eq!(canonical(&nested("{\"a\":", "}", 4301)), Ok(()));
                // The 2 innermost dicts' notes fail, so the repr's error is what escapes.
                let error = dumps(&nested("{\"a\":", "}", 4302), &Options::CANONICAL).unwrap_err();
                assert!(text(&error.text).ends_with("while getting the repr of an object"));
                assert_eq!(error.notes.len(), 4300);
                assert_eq!(text(&error.notes[4299]), "when serializing dict item 'x'");
                assert!(repr_fits(&nested("[", "]", 10399), 1_996_800));
                assert!(!repr_fits(&nested("[", "]", 10400), 1_996_800));
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
