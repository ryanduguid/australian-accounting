//! Vendor export profiles. A profile is data: data/profiles/*.json, compiled
//! into the binary.

use std::cell::RefCell;

use crate::pyre::Regex;
use crate::raw::Raw;
use crate::unicode::{casefold, nfkc};
use crate::{Failure, Result, pytext, repr_list, sorted_strings};

pub const ROLES: [&str; 2] = ["payroll", "super"];

/// The bundled profiles, in file-name order.
const BUNDLED: [&str; 8] = [
    include_str!("../data/profiles/employment-hero-payroll.json"),
    include_str!("../data/profiles/employment-hero-super.json"),
    include_str!("../data/profiles/myob-ar-payroll.json"),
    include_str!("../data/profiles/myob-ar-super.json"),
    include_str!("../data/profiles/myob-business-payroll.json"),
    include_str!("../data/profiles/myob-business-super.json"),
    include_str!("../data/profiles/xero-payroll.json"),
    include_str!("../data/profiles/xero-super.json"),
];

thread_local! {
    static PUNCT: Regex = Regex::new(r"[^0-9a-z ]+");
    static SPACES: Regex = Regex::new(r"\s+");
    static LOADED: RefCell<Option<Vec<Profile>>> = const { RefCell::new(None) };
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

fn strings(raw: Option<&Raw>) -> Vec<String> {
    match raw {
        Some(Raw::List(items)) => items.iter().map(Raw::py_str).collect(),
        _ => Vec::new(),
    }
}

fn text(raw: &Raw, key: &str) -> String {
    raw.get(key).map(Raw::py_str).unwrap_or_default()
}

fn build(raw: &Raw) -> Profile {
    let columns = match raw.get("columns") {
        Some(Raw::Dict(items)) => items
            .iter()
            .map(|(field, headings)| (field.clone(), strings(Some(headings))))
            .collect(),
        _ => Vec::new(),
    };
    let sg_filter = raw.get("sg_filter").map(|f| SgFilter {
        column: text(f, "column"),
        include: strings(f.get("include")),
    });
    let remitted_status = raw.get("remitted_status").map(|s| RemittedStatus {
        column: text(s, "column"),
        sent: strings(s.get("sent")),
        not_sent: strings(s.get("not_sent")),
    });
    Profile {
        key: text(raw, "key"),
        name: text(raw, "name"),
        role: text(raw, "role"),
        verified: matches!(raw.get("verified"), Some(Raw::Bool(true))),
        signature: strings(raw.get("signature")),
        columns,
        date_formats: strings(raw.get("date_formats")),
        sg_filter,
        remitted_status,
        notes: text(raw, "notes"),
    }
}

fn all_profiles() -> Vec<Profile> {
    LOADED.with(|cache| {
        cache
            .borrow_mut()
            .get_or_insert_with(|| {
                BUNDLED
                    .iter()
                    .map(|text| {
                        let json = crate::pyjson::loads(&crate::py::universal_newlines(text))
                            .expect("bundled profiles are valid JSON");
                        build(&Raw::from_json(&json))
                    })
                    .collect()
            })
            .clone()
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
    Ok(all_profiles()
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
}
