//! `contract.py`: reviewed contract snapshots, and what happens when the live one moves.
//!
//! A snapshot records what a provider's surface looked like when a person read
//! it. A contract change is reviewed by a person, never accepted
//! automatically: `compare` reports drift, and nothing here rewrites a
//! snapshot. A snapshot's members are whatever JSON it holds, so each use
//! fails as Python's did on a value of the wrong type.
use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::errors::CONTRACT;
use crate::p;
use crate::pyjson::{self, DecodeError, Json};
use crate::pyops::{Exc, R, S, get_attr, getitem, hash_key, iterate, pts, sorted_set, type_error};
use crate::pyos::{OsError, Style};
use crate::pypath::PyPath;
use crate::pyvalue::{Key, item_eq, py_eq, repr, str_points, type_name};

/// The folders searched for snapshots: the source checkout first, then a
/// `contracts` folder beside the binary, as a built wheel carries one inside
/// the package. With the `differential` feature, `LODGEIT_ADAPTER_PACKAGE`
/// replaces the checkout's package folder.
#[doc(hidden)]
pub fn contracts_dirs() -> Vec<PyPath> {
    let mut package = env!("CARGO_MANIFEST_DIR").to_owned();
    if cfg!(feature = "differential")
        && let Some(root) = std::env::var_os("LODGEIT_ADAPTER_PACKAGE")
    {
        package = root.to_string_lossy().into_owned();
    }
    let mut dirs = vec![PyPath::new(&package).join("contracts")];
    if let Some(parent) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_owned()))
    {
        dirs.push(PyPath::new(&parent.to_string_lossy()).join("contracts"));
    }
    dirs
}

pub fn contracts_dir() -> PyPath {
    let dirs = contracts_dirs();
    dirs.iter()
        .find(|dir| dir.fs().is_dir())
        .unwrap_or(&dirs[0])
        .clone()
}

/// One reviewed snapshot.
#[derive(Clone, Debug)]
pub struct Contract {
    pub provider: Json,
    pub snapshot_id: Json,
    pub read_at: Json,
    pub read_by: Json,
    pub calculators: Json,
    pub response_contract: Json,
    pub notes: Vec<Json>,
    pub sha256: String,
    pub path: PyPath,
}

fn contract_error(text: S) -> Exc {
    Exc::new(CONTRACT, text)
}

/// `value[key]` for a str key given as code points.
fn item<'a>(value: &'a Json, key: &[u32]) -> R<&'a Json> {
    match value {
        Json::Object(pairs) => pairs
            .iter()
            .rev()
            .find(|(k, _)| k.code_points() == key)
            .map(|(_, v)| v)
            .ok_or_else(|| Exc::new("KeyError", pts(&crate::pyvalue::repr_points(key)))),
        other => getitem(other, ""),
    }
}

/// `set(value)`: its members' keys, in iteration order, without repeats.
fn py_set(value: &Json) -> R<Vec<(Key, Json)>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for member in iterate(value)? {
        let key = hash_key(&member, "set element")?;
        if seen.insert(key.clone()) {
            out.push((key, member));
        }
    }
    Ok(out)
}

/// `sorted(a - b)` of two sets.
fn sorted_difference(a: &[(Key, Json)], b: &[(Key, Json)]) -> R<Vec<Json>> {
    let b: HashSet<&Key> = b.iter().map(|(k, _)| k).collect();
    sorted_set(
        a.iter()
            .filter(|(k, _)| !b.contains(k))
            .map(|(_, v)| v.clone())
            .collect(),
    )
}

impl Contract {
    pub fn snapshot(&self) -> S {
        str_points(&self.snapshot_id)
    }

    /// `calculator(calc_uri)`: the snapshot's entry, or the ContractError
    /// for a calculator nobody reviewed.
    pub fn calculator(&self, calc_uri: &[u32]) -> R<&Json> {
        item(&self.calculators, calc_uri).map_err(|error| {
            if error.class != "KeyError" {
                return error;
            }
            contract_error(p!(
                calc_uri,
                " is not in reviewed snapshot ",
                self.snapshot(),
                ". Calling a calculator nobody has reviewed is how an unknown response shape \
                 becomes a number. Review it and add it to the snapshot first."
            ))
        })
    }

    pub fn check_period(&self, calc_uri: &[u32], period_uri: &[u32]) -> R<()> {
        let empty = Json::Array(Vec::new());
        let supported =
            get_attr(self.calculator(calc_uri)?, "supported_periods")?.unwrap_or(&empty);
        let period = crate::pyops::json_str(period_uri.to_vec());
        let accepted = match supported {
            Json::Array(items) => items.iter().any(|i| item_eq(i, &period)),
            Json::Object(pairs) => pairs.iter().any(|(k, _)| k.code_points() == period_uri),
            Json::Str(s) => crate::pyops::contains(&s.code_points(), period_uri),
            other => {
                return Err(type_error(&format!(
                    "argument of type '{}' is not a container or iterable",
                    type_name(other)
                )));
            }
        };
        if accepted {
            return Ok(());
        }
        let mut joined: S = Vec::new();
        for (i, member) in iterate(supported)?.iter().enumerate() {
            let Json::Str(text) = member else {
                return Err(type_error(&format!(
                    "sequence item {i}: expected str instance, {} found",
                    type_name(member)
                )));
            };
            if i > 0 {
                joined.extend(pts(", "));
            }
            joined.extend(text.code_points());
        }
        if joined.is_empty() {
            joined = pts("nothing recorded");
        }
        Err(contract_error(p!(
            calc_uri,
            " does not accept ",
            period_uri,
            " in snapshot ",
            self.snapshot(),
            ". It accepts: ",
            joined,
            "."
        )))
    }
}

const REQUIRED: [&str; 7] = [
    "provider",
    "snapshot_id",
    "read_at",
    "read_by",
    "source_urls",
    "calculators",
    "response_contract",
];

/// `Path.read_bytes()`.
fn read_bytes(path: &PyPath) -> Result<Vec<u8>, OsError> {
    std::fs::read(path.fs())
        .map_err(|error| OsError::new(&error, Some(&path.display()), Style::Errno))
}

/// `load(name, directory=directory)`.
pub fn load(name: &str, directory: Option<&PyPath>) -> R<Contract> {
    let path = directory
        .cloned()
        .unwrap_or_else(contracts_dir)
        .join(&format!("{name}.json"));
    let shown = path.display();
    let payload = read_bytes(&path).map_err(|error| {
        contract_error(pts(&format!(
            "contract snapshot {shown} could not be read: {error}"
        )))
    })?;
    let invalid = |error: String| {
        contract_error(pts(&format!(
            "contract snapshot {shown} is not valid JSON: {error}"
        )))
    };
    let text = crate::py::decode_utf8(&payload).map_err(|e| invalid(e.to_string()))?;
    let data = match pyjson::loads_standard(&text) {
        Ok(data) => data,
        // The digit limit is a plain ValueError, which the except clause missed.
        Err(DecodeError::Invalid(m)) if m.starts_with("Exceeds the limit") => {
            return Err(Exc::new("ValueError", pts(&m)));
        }
        Err(DecodeError::Invalid(m)) => return Err(invalid(m)),
        Err(DecodeError::Recursion(kind)) => {
            return Err(Exc::new(
                "RecursionError",
                pts(&pyjson::recursion_message(kind)),
            ));
        }
        Err(other) => unreachable!("json.loads has no hooks here: {other:?}"),
    };
    let present = py_set(&data)?;
    let mut missing: Vec<&str> = REQUIRED
        .into_iter()
        .filter(|name| !present.iter().any(|(k, _)| *k == Key::Str(pts(name))))
        .collect();
    if !missing.is_empty() {
        missing.sort_unstable();
        return Err(contract_error(pts(&format!(
            "contract snapshot {shown} is missing {}",
            missing.join(", ")
        ))));
    }
    let field = |key: &str| getitem(&data, key).cloned();
    let provider = field("provider")?;
    let snapshot_id = field("snapshot_id")?;
    let read_at = field("read_at")?;
    let read_by = field("read_by")?;
    let calculators = field("calculators")?;
    let response_contract = field("response_contract")?;
    let notes = match get_attr(&data, "notes")? {
        Some(notes) => iterate(notes)?,
        None => Vec::new(),
    };
    Ok(Contract {
        provider,
        snapshot_id,
        read_at,
        read_by,
        calculators,
        response_contract,
        notes,
        sha256: Sha256::digest(&payload)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
        path,
    })
}

/// `compare(contract, live_listing)`: the differences between a reviewed
/// snapshot and a live discovery response, as findings. It decides nothing.
pub fn compare(contract: &Contract, live_listing: &[Json]) -> R<Vec<S>> {
    let mut findings: Vec<S> = Vec::new();
    // A calc_uri listed twice is drift whatever the two entries say.
    let mut live: Vec<(S, &Json)> = Vec::new();
    let mut repeated: Vec<S> = Vec::new();
    for (index, entry) in live_listing.iter().enumerate() {
        if !matches!(entry, Json::Object(_)) {
            findings.push(pts(&format!(
                "live listing entry {index} is {}, not an object, and was not compared",
                type_name(entry)
            )));
            continue;
        }
        let uri = match entry.get("calc_uri") {
            Some(Json::Str(uri)) if uri.points().next().is_some() => uri.code_points(),
            _ => {
                findings.push(pts(&format!(
                    "live listing entry {index} carries no calc_uri string and was not compared"
                )));
                continue;
            }
        };
        if live.iter().any(|(u, _)| *u == uri) || repeated.contains(&uri) {
            live.retain(|(u, _)| *u != uri);
            findings.push(p!(
                "live lists ",
                uri,
                format!(" more than once (again at entry {index}); no entry for it was compared")
            ));
            if !repeated.contains(&uri) {
                repeated.push(uri);
            }
            continue;
        }
        live.push((uri, entry));
    }
    let sid = contract.snapshot();
    let live_set: Vec<(Key, Json)> = live
        .iter()
        .map(|(u, _)| (Key::Str(u.clone()), crate::pyops::json_str(u.clone())))
        .collect();
    let recorded_set = py_set(&contract.calculators)?;
    for uri in sorted_difference(&live_set, &recorded_set)? {
        findings.push(p!(
            "live has ",
            str_points(&uri),
            ", which snapshot ",
            sid,
            " does not record"
        ));
    }
    let recorded_set = py_set(&contract.calculators)?;
    for uri in sorted_difference(&recorded_set, &live_set)? {
        findings.push(p!(
            "snapshot ",
            sid,
            " records ",
            str_points(&uri),
            ", which live no longer lists"
        ));
    }
    let recorded_keys: HashSet<Key> = py_set(&contract.calculators)?
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    let mut shared: Vec<&(S, &Json)> = live
        .iter()
        .filter(|(u, _)| recorded_keys.contains(&Key::Str(u.clone())))
        .collect();
    shared.sort_by(|a, b| a.0.cmp(&b.0));
    let empty = Json::Array(Vec::new());
    for (uri, entry) in shared {
        let recorded_entry = item(&contract.calculators, uri)?;
        let recorded = py_set(get_attr(recorded_entry, "supported_periods")?.unwrap_or(&empty))?;
        let periods = match entry.get("supported_periods") {
            None | Some(Json::Null) => &empty,
            Some(periods) => periods,
        };
        let strings = match periods {
            Json::Array(items) => items.iter().all(|p| matches!(p, Json::Str(_))),
            _ => false,
        };
        if !strings {
            findings.push(p!(
                uri,
                ": live supported_periods is not a list of strings; its periods were not compared"
            ));
            continue;
        }
        let current = py_set(periods)?;
        for period in sorted_difference(&current, &recorded)? {
            findings.push(p!(
                uri,
                ": live accepts ",
                str_points(&period),
                ", which the snapshot does not record"
            ));
        }
        for period in sorted_difference(&recorded, &current)? {
            findings.push(p!(
                uri,
                ": the snapshot records ",
                str_points(&period),
                ", which live no longer accepts"
            ));
        }
        let recorded_ref = get_attr(recorded_entry, "input_schema_ref")?.unwrap_or(&Json::Null);
        let current_ref = entry.get("input_schema_ref").unwrap_or(&Json::Null);
        if !recorded_ref.truthy() {
            continue;
        }
        if !matches!(current_ref, Json::Str(_)) || !current_ref.truthy() {
            // A dropped schema reference is drift, not agreement.
            findings.push(p!(
                uri,
                ": live no longer states input schema ref ",
                str_points(recorded_ref),
                " (now ",
                repr(current_ref),
                ")"
            ));
        } else if !py_eq(recorded_ref, current_ref) {
            findings.push(p!(
                uri,
                ": input schema ref moved from ",
                str_points(recorded_ref),
                " to ",
                str_points(current_ref)
            ));
        }
    }
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(points: &[u32]) -> String {
        points.iter().filter_map(|&c| char::from_u32(c)).collect()
    }

    fn snapshot(calculators: &str) -> Contract {
        Contract {
            provider: Json::str("p"),
            snapshot_id: Json::str("s1"),
            read_at: Json::Null,
            read_by: Json::Null,
            calculators: pyjson::loads_standard(calculators).unwrap(),
            response_contract: Json::Null,
            notes: Vec::new(),
            sha256: String::new(),
            path: PyPath::new("x.json"),
        }
    }

    #[test]
    fn drift_is_reported_and_never_decided() {
        let contract = snapshot(
            r#"{"b": {"supported_periods": ["p1", "p2"], "input_schema_ref": "r1"},
                "c": {"supported_periods": []}}"#,
        );
        let live = pyjson::loads_standard(
            r#"[{"calc_uri": "b", "supported_periods": ["p2", "p3"]}, 5, {"calc_uri": ""},
                {"calc_uri": "a"}, {"calc_uri": "d"}, {"calc_uri": "d"}]"#,
        )
        .unwrap();
        let Json::Array(live) = live else {
            unreachable!()
        };
        let findings: Vec<String> = compare(&contract, &live)
            .unwrap()
            .iter()
            .map(|f| text(f))
            .collect();
        assert_eq!(
            findings,
            [
                "live listing entry 1 is int, not an object, and was not compared",
                "live listing entry 2 carries no calc_uri string and was not compared",
                "live lists d more than once (again at entry 5); no entry for it was compared",
                "live has a, which snapshot s1 does not record",
                "snapshot s1 records c, which live no longer lists",
                "b: live accepts p3, which the snapshot does not record",
                "b: the snapshot records p1, which live no longer accepts",
                "b: live no longer states input schema ref r1 (now None)",
            ]
        );
    }

    #[test]
    fn periods_and_calculators_are_checked_against_the_snapshot() {
        let contract = snapshot(r#"{"b": {"supported_periods": "xy"}}"#);
        let error = contract.check_period(&pts("b"), &pts("z")).unwrap_err();
        assert_eq!(
            text(&error.text),
            "b does not accept z in snapshot s1. It accepts: x, y."
        );
        assert!(contract.check_period(&pts("b"), &pts("x")).is_ok());
        let error = contract.calculator(&pts("q")).unwrap_err();
        assert_eq!(error.class, CONTRACT);
        let listed = snapshot("[1]");
        assert_eq!(
            text(&listed.calculator(&pts("q")).unwrap_err().text),
            "list indices must be integers or slices, not str"
        );
    }
}
