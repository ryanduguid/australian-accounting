//! Vendor export profiles. A profile is data: data/profiles/*.json, compiled
//! into the binary.

use std::cell::RefCell;

use crate::pyre::Regex;
use crate::raw::Raw;
use crate::unicode::{casefold, nfkc};
use crate::{Failure, Result, pytext, repr_list, sorted_strings};

pub const ROLES: [&str; 2] = ["payroll", "super"];

/// The bundled profiles, by file name in sorted order, as Python globbed
/// them. `the_bundled_profile_list_is_the_folder` keeps this list and the
/// folder in step.
const BUNDLED: [(&str, &str); 8] = [
    (
        "employment-hero-payroll.json",
        include_str!("../data/profiles/employment-hero-payroll.json"),
    ),
    (
        "employment-hero-super.json",
        include_str!("../data/profiles/employment-hero-super.json"),
    ),
    (
        "myob-ar-payroll.json",
        include_str!("../data/profiles/myob-ar-payroll.json"),
    ),
    (
        "myob-ar-super.json",
        include_str!("../data/profiles/myob-ar-super.json"),
    ),
    (
        "myob-business-payroll.json",
        include_str!("../data/profiles/myob-business-payroll.json"),
    ),
    (
        "myob-business-super.json",
        include_str!("../data/profiles/myob-business-super.json"),
    ),
    (
        "xero-payroll.json",
        include_str!("../data/profiles/xero-payroll.json"),
    ),
    (
        "xero-super.json",
        include_str!("../data/profiles/xero-super.json"),
    ),
];

/// Canonical fields a profile may map; anything else is a typo.
const SOURCE_FIELDS: [&str; 9] = [
    "employee_id",
    "employee_name",
    "payday",
    "period_start",
    "period_end",
    "paid_date",
    "amount",
    "contribution_type",
    "status",
];

thread_local! {
    static PUNCT: Regex = Regex::new(r"[^0-9a-z ]+");
    static SPACES: Regex = Regex::new(r"\s+");
    static LOADED: RefCell<Option<std::result::Result<Vec<Profile>, String>>> =
        const { RefCell::new(None) };
}

/// Fold a heading to its comparable form: no case, no punctuation, one space
/// between words.
pub fn normalise_header(text: &str) -> String {
    let folded = casefold(&nfkc(text));
    let folded = SPACES.with(|s| s.sub(" ", &folded));
    let folded = PUNCT.with(|p| p.sub("", &folded));
    let folded = SPACES.with(|s| s.sub(" ", &folded));
    pytext::strip(&folded).to_string()
}

/// Fold a person's name: no case, one space between words, nothing removed.
pub fn normalise_name(text: &str) -> String {
    let folded = casefold(&nfkc(text));
    let folded = SPACES.with(|s| s.sub(" ", &folded));
    pytext::strip(&folded).to_string()
}

#[derive(Clone, Debug, PartialEq)]
pub struct SgFilter {
    pub column: String,
    pub include: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemittedStatus {
    pub column: String,
    pub sent: Vec<String>,
    pub not_sent: Vec<String>,
    /// The `not_sent` values that say a fund returned part of a batch without
    /// saying which contributions: unresolved, rather than simply unsent.
    pub refund_unreconciled: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub key: String,
    pub name: String,
    pub role: String,
    pub verified: bool,
    pub signature: Vec<String>,
    pub columns: Vec<(String, Vec<String>)>,
    pub date_formats: Vec<String>,
    pub sg_filter: Option<SgFilter>,
    pub remitted_status: Option<RemittedStatus>,
    pub notes: String,
}

fn error(message: String) -> Failure {
    Failure::Value(message)
}

/// A dict's entries as Python's dict holds them: each key once, at its first
/// position, with its last value.
fn entries(items: &[(String, Raw)]) -> Vec<(&String, &Raw)> {
    let mut out: Vec<(&String, &Raw)> = Vec::new();
    for (k, v) in items {
        match out.iter_mut().find(|(key, _)| *key == k) {
            Some(slot) => slot.1 = v,
            None => out.push((k, v)),
        }
    }
    out
}

/// `[str(v) for v in value]` for a non-empty list, or the message.
fn non_empty_list(value: Option<&Raw>, message: impl Fn() -> String) -> Result<Vec<String>> {
    match value {
        Some(Raw::List(items)) if !items.is_empty() => Ok(items.iter().map(Raw::py_str).collect()),
        _ => Err(error(message())),
    }
}

/// `_require`: the field must be present and of the named Python type.
fn require<'a>(raw: &'a Raw, field: &str, file: &str, kind: &str) -> Result<&'a Raw> {
    let Some(value) = raw.get(field) else {
        return Err(error(format!("profile {file} has no '{field}'")));
    };
    if value.type_name() != kind {
        return Err(error(format!(
            "profile {file}: '{field}' must be {kind}, got {}",
            value.type_name()
        )));
    }
    Ok(value)
}

/// `column in columns`: only a string can name a mapped field.
fn mapped(columns: &[(String, Vec<String>)], column: &Raw) -> bool {
    matches!(column, Raw::Str(c) if columns.iter().any(|(f, _)| f == c))
}

/// Python's `_build`, with its checks in its order.
fn build(raw: &Raw, file: &str) -> Result<Profile> {
    let Raw::Str(role) = require(raw, "role", file, "str")? else {
        unreachable!("checked")
    };
    if !ROLES.contains(&role.as_str()) {
        return Err(error(format!(
            "profile {file}: role {} is not one of {}",
            pytext::repr(role),
            repr_list(&ROLES)
        )));
    }
    let Raw::Dict(columns_raw) = require(raw, "columns", file, "dict")? else {
        unreachable!("checked")
    };
    let columns_raw = entries(columns_raw);
    let unknown = sorted_strings(
        columns_raw
            .iter()
            .map(|(f, _)| f.as_str())
            .filter(|f| !SOURCE_FIELDS.contains(f))
            .collect::<std::collections::BTreeSet<_>>(),
    );
    if !unknown.is_empty() {
        return Err(error(format!(
            "profile {file} maps unknown field(s) {}; valid fields are {}",
            repr_list(&unknown),
            repr_list(&sorted_strings(SOURCE_FIELDS))
        )));
    }
    let mut columns = Vec::new();
    for (field, headings) in columns_raw {
        let headings = non_empty_list(Some(headings), || {
            format!(
                "profile {file}: {} must be a non-empty list of headings",
                pytext::repr(field)
            )
        })?;
        columns.push((field.clone(), headings));
    }
    let sg_filter = match raw.get("sg_filter") {
        None | Some(Raw::None) => None,
        Some(sg) => {
            let (Raw::Dict(_), Some(column), Some(_)) = (sg, sg.get("column"), sg.get("include"))
            else {
                return Err(error(format!(
                    "profile {file}: sg_filter needs 'column' and 'include'"
                )));
            };
            if !mapped(&columns, column) {
                return Err(error(format!(
                    "profile {file}: sg_filter column {} is not one of the mapped columns",
                    column.py_repr()
                )));
            }
            let include = non_empty_list(sg.get("include"), || {
                format!("profile {file}: sg_filter 'include' must be a non-empty list of values")
            })?;
            Some(SgFilter {
                column: column.py_str(),
                include,
            })
        }
    };
    let remitted_status = match raw.get("remitted_status") {
        None | Some(Raw::None) => None,
        Some(rs) => {
            let (Raw::Dict(_), Some(column), Some(_), Some(_)) =
                (rs, rs.get("column"), rs.get("sent"), rs.get("not_sent"))
            else {
                return Err(error(format!(
                    "profile {file}: remitted_status needs 'column', 'sent' and 'not_sent'"
                )));
            };
            if !mapped(&columns, column) {
                return Err(error(format!(
                    "profile {file}: remitted_status column {} is not one of the mapped columns",
                    column.py_repr()
                )));
            }
            let mut lists = Vec::new();
            for list_name in ["sent", "not_sent"] {
                lists.push(non_empty_list(rs.get(list_name), || {
                    format!(
                        "profile {file}: remitted_status '{list_name}' must be a non-empty \
                         list of values"
                    )
                })?);
            }
            let not_sent = lists.pop().expect("two lists");
            let sent = lists.pop().expect("two lists");
            let folded = |values: &[String]| -> std::collections::BTreeSet<String> {
                values.iter().map(|v| normalise_header(v)).collect()
            };
            let overlap: Vec<String> = folded(&sent)
                .intersection(&folded(&not_sent))
                .cloned()
                .collect();
            if !overlap.is_empty() {
                return Err(error(format!(
                    "profile {file}: remitted_status lists {} as both 'sent' and 'not_sent'",
                    repr_list(&sorted_strings(overlap.iter()))
                )));
            }
            let refund_unreconciled = match rs.get("refund_unreconciled") {
                None => Vec::new(),
                value => non_empty_list(value, || {
                    format!(
                        "profile {file}: remitted_status 'refund_unreconciled' must be a \
                         non-empty list of values"
                    )
                })?,
            };
            let unlisted: Vec<String> = folded(&refund_unreconciled)
                .difference(&folded(&not_sent))
                .cloned()
                .collect();
            if !unlisted.is_empty() {
                return Err(error(format!(
                    "profile {file}: remitted_status 'refund_unreconciled' lists {} outside \
                     'not_sent'",
                    repr_list(&sorted_strings(unlisted.iter()))
                )));
            }
            Some(RemittedStatus {
                column: column.py_str(),
                sent,
                not_sent,
                refund_unreconciled,
            })
        }
    };
    let date_formats = match raw.get("date_formats") {
        None => Vec::new(),
        Some(value) => non_empty_list(Some(value), || {
            format!("profile {file}: 'date_formats' must be a non-empty list of formats")
        })?,
    };
    let verified = match raw.get("verified") {
        None => false,
        Some(Raw::Bool(b)) => *b,
        Some(other) => {
            return Err(error(format!(
                "profile {file}: 'verified' must be true or false, got {}",
                other.type_name()
            )));
        }
    };
    let key = require(raw, "key", file, "str")?.py_str();
    let name = require(raw, "name", file, "str")?.py_str();
    let Raw::List(signature) = require(raw, "signature", file, "list")? else {
        unreachable!("checked")
    };
    Ok(Profile {
        key,
        name,
        role: role.clone(),
        verified,
        signature: signature.iter().map(Raw::py_str).collect(),
        columns,
        date_formats,
        sg_filter,
        remitted_status,
        notes: raw.get("notes").map(Raw::py_str).unwrap_or_default(),
    })
}

/// Every bundled profile, checked as Python checked each file.
fn parse_all(files: &[(&str, &str)]) -> Result<Vec<Profile>> {
    let mut profiles = Vec::new();
    for (file, text) in files {
        let json = crate::pyjson::loads(&crate::py::universal_newlines(text))
            .map_err(|e| error(format!("profile {file} is not valid JSON: {e}")))?;
        let raw = Raw::from_json(&json);
        if !matches!(raw, Raw::Dict(_)) {
            return Err(error(format!("profile {file} must be a JSON object")));
        }
        profiles.push(build(&raw, file)?);
    }
    let keys: Vec<&String> = profiles.iter().map(|p| &p.key).collect();
    let duplicates = sorted_strings(
        keys.iter()
            .filter(|k| keys.iter().filter(|j| j == k).count() > 1)
            .map(|k| k.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
    );
    if !duplicates.is_empty() {
        return Err(error(format!(
            "duplicate profile key(s): {}",
            repr_list(&duplicates)
        )));
    }
    Ok(profiles)
}

fn all_profiles() -> Result<Vec<Profile>> {
    LOADED.with(|cache| {
        cache
            .borrow_mut()
            .get_or_insert_with(|| parse_all(&BUNDLED).map_err(|f| f.message()))
            .clone()
            .map_err(Failure::Value)
    })
}

/// The bundled profiles, optionally for one role.
pub fn load_profiles(role: Option<&str>) -> Result<Vec<Profile>> {
    if let Some(role) = role
        && !ROLES.contains(&role)
    {
        return Err(Failure::Value(format!(
            "unknown profile role {}",
            pytext::repr_os(role)
        )));
    }
    Ok(all_profiles()?
        .into_iter()
        .filter(|p| role.is_none_or(|r| p.role == r))
        .collect())
}

/// Normalised heading to the heading as it appears in the file.
fn index(headers: &[String]) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for h in headers {
        let key = normalise_header(h);
        if !key.is_empty() && !found.iter().any(|(k, _)| *k == key) {
            found.push((key, h.clone()));
        }
    }
    found
}

fn lookup<'a>(found: &'a [(String, String)], key: &str) -> Option<&'a String> {
    found.iter().find(|(k, _)| k == key).map(|(_, h)| h)
}

/// Canonical field to the heading this file uses; absent fields left out.
pub fn resolve_columns(profile: &Profile, headers: &[String]) -> Vec<(String, String)> {
    let found = index(headers);
    let mut resolved = Vec::new();
    for (field, candidates) in &profile.columns {
        for candidate in candidates {
            if let Some(actual) = lookup(&found, &normalise_header(candidate)) {
                resolved.push((field.clone(), actual.clone()));
                break;
            }
        }
    }
    resolved
}

/// How well this profile fits, or None when a signature heading is missing.
pub fn score(profile: &Profile, headers: &[String]) -> Option<usize> {
    let found = index(headers);
    for heading in &profile.signature {
        lookup(&found, &normalise_header(heading))?;
    }
    Some(resolve_columns(profile, headers).len())
}

fn vendor_stem(vendor: &str, profiles: &[Profile]) -> Option<String> {
    let parts: Vec<&str> = vendor.split('-').collect();
    for cut in (1..parts.len()).rev() {
        let stem = parts[..cut].join("-");
        if profiles
            .iter()
            .any(|p| p.key == stem || p.key.starts_with(&format!("{stem}-")))
        {
            return Some(stem);
        }
    }
    None
}

pub fn detect(headers: &[String], role: &str, vendor: Option<&str>) -> Result<Profile> {
    let profiles = load_profiles(Some(role))?;
    if let Some(vendor) = vendor {
        if let Some(exact) = profiles.iter().find(|p| p.key == vendor) {
            return Ok(exact.clone());
        }
        let prefixed: Vec<&Profile> = profiles
            .iter()
            .filter(|p| p.key.starts_with(&format!("{vendor}-")))
            .collect();
        let shown = pytext::repr_os(vendor);
        if prefixed.len() > 1 {
            return Err(Failure::Value(format!(
                "--vendor {shown} matches more than one {role} profile: {}. Lengthen it until it \
                 picks one, keeping the part both files' profiles share -- 'myob-ar' rather than \
                 'myob', which picks myob-ar-payroll and myob-ar-super together. The \
                 import command passes one --vendor to both files.",
                repr_list(&sorted_strings(prefixed.iter().map(|p| p.key.as_str())))
            )));
        }
        if let Some(first) = prefixed.first() {
            return Ok((*first).clone());
        }
        let mut message = format!(
            "--vendor {shown} matches no {role} profile. Available: {}.",
            repr_list(&sorted_strings(profiles.iter().map(|p| p.key.as_str())))
        );
        if let Some(stem) = vendor_stem(vendor, &profiles) {
            message.push_str(&format!(
                " The import command passes one --vendor to the payroll file and the \
                 super file together, so name the stem both of a vendor's profiles \
                 start with -- {}, not {shown}, which is not a {role} \
                 profile and fails on the second file.",
                pytext::repr_os(&stem)
            ));
        }
        return Err(Failure::Value(message));
    }
    let scored: Vec<(Option<usize>, &Profile)> =
        profiles.iter().map(|p| (score(p, headers), p)).collect();
    let mut live: Vec<(usize, &Profile)> = scored
        .iter()
        .filter_map(|(s, p)| s.map(|s| (s, *p)))
        .collect();
    if live.is_empty() {
        let mut by_key: Vec<&Profile> = scored.iter().map(|(_, p)| *p).collect();
        by_key.sort_by(|a, b| pytext::code_points(&a.key).cmp(pytext::code_points(&b.key)));
        let wanted: Vec<String> = by_key
            .iter()
            .map(|p| format!("{} wanted {}", p.key, repr_list(&p.signature)))
            .collect();
        return Err(Failure::Value(format!(
            "no {role} profile recognises these columns: {}. Candidates: \
             {}. Force one with --vendor, or map the columns by hand.",
            repr_list(headers),
            wanted.join("; ")
        )));
    }
    live.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.key.cmp(&b.1.key)));
    if live.len() > 1 && live[0].0 == live[1].0 {
        return Err(Failure::Value(format!(
            "could not tell {} from {} for this {role} file: both match {} column(s). \
             Force one with --vendor.",
            live[0].1.key, live[1].1.key, live[0].0
        )));
    }
    Ok(live[0].1.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_and_names_fold_as_python_folds_them() {
        assert_eq!(
            normalise_header("  Employee\u{A0}Membership #  "),
            "employee membership"
        );
        assert_eq!(normalise_header("ＤＡＴＥ\u{3000}Paid"), "date paid");
        assert_eq!(normalise_name(" O'Brien\tSmith "), "o'brien smith");
        assert_eq!(normalise_name("张伟"), "张伟");
        assert_eq!(load_profiles(Some("super")).unwrap().len(), 4);
    }

    /// Python 0.2.2's answers for hand-edited profile folders
    /// (`tests/data/profile-messages.json`, generated from the Python loader).
    #[test]
    fn malformed_profiles_are_refused_as_python_refused_them() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("../tests/data/profile-messages.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let files: Vec<(&str, &str)> = case[1]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| (f[0].as_str().unwrap(), f[1].as_str().unwrap()))
                .collect();
            let got = match parse_all(&files) {
                Ok(_) => "OK".to_string(),
                Err(f) => f.message(),
            };
            assert_eq!(got, case[2].as_str().unwrap(), "{}", case[0]);
        }
    }

    /// Python globbed the folder; the compiled list must name every file in it.
    #[test]
    fn the_bundled_profile_list_is_the_folder() {
        let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/data/profiles");
        let mut found: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".json"))
            .collect();
        found.sort();
        let listed: Vec<&str> = BUNDLED.iter().map(|(name, _)| *name).collect();
        assert_eq!(found, listed);
    }
}
