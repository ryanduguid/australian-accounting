// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::json::{self, Json};
use super::xml::{child, element_text, strip, tagged, xml};
use indexmap::IndexMap;
use payday_super_checker::py::is_space;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

struct Reference {
    id: String,
    calls: Json,
    answer: String,
}
fn validate(recorded: &Json) -> Result<&IndexMap<String, Json>, String> {
    let cases = recorded
        .object()
        .ok_or("recording must be an object keyed by question id")?;
    for (case, entry) in cases {
        if entry
            .array()
            .is_some_and(|a| a.iter().all(|n| n.string().is_some()))
        {
            continue;
        }
        let fail =
            || format!("{case}: supply calls and a string answer, or a legacy tool-name list");
        let object = entry.object().ok_or_else(fail)?;
        if object.len() != 2
            || !object.contains_key("calls")
            || !object.contains_key("answer")
            || object["answer"].string().is_none()
        {
            return Err(fail());
        }
        let calls = object["calls"].array().ok_or_else(fail)?;
        for call in calls {
            if !call.object().is_some_and(|c| {
                c.len() == 2
                    && c.get("name").and_then(Json::string).is_some()
                    && c.get("arguments").and_then(Json::object).is_some()
            }) {
                return Err(format!(
                    "{case}: each call requires a string name and an arguments object"
                ));
            }
        }
    }
    Ok(cases)
}
fn names(calls: &Json) -> BTreeSet<&str> {
    calls
        .array()
        .unwrap()
        .iter()
        .map(|c| c.object().unwrap()["name"].string().unwrap())
        .collect()
}
fn same_tools(pair: roxmltree::Node<'_, '_>, calls: &Json) -> bool {
    let declared: Vec<_> = pair
        .children()
        .filter(|n| tagged(*n, "tools"))
        .flat_map(|n| n.children().filter(|n| tagged(*n, "tool")))
        .map(|n| element_text(Some(n)))
        .collect();
    let declared_set: BTreeSet<_> = declared.iter().map(String::as_str).collect();
    declared.len() == declared_set.len() && declared_set == names(calls)
}
fn read_pair(pair: roxmltree::Node<'_, '_>, id: &str) -> Result<Reference, String> {
    if ["question", "calls", "answer"]
        .iter()
        .any(|tag| pair.children().filter(|n| tagged(*n, tag)).count() != 1)
    {
        return Err(format!(
            "{id}: reference question, calls and answer must appear once"
        ));
    }
    if strip(&element_text(child(pair, "question"))).is_empty() {
        return Err(format!("{id}: reference question must not be blank"));
    }
    let calls = json::parse(&element_text(child(pair, "calls")))?;
    let answer = element_text(child(pair, "answer"));
    let entry = Json::Object(IndexMap::from([
        ("calls".into(), calls.clone()),
        ("answer".into(), Json::String(answer.clone())),
    ]));
    validate(&Json::Object(IndexMap::from([(id.into(), entry)])))?;
    if !same_tools(pair, &calls) {
        return Err(format!("{id}: reference tool set differs from calls"));
    }
    Ok(Reference {
        id: id.into(),
        calls,
        answer,
    })
}
fn references(bytes: &[u8]) -> Result<Vec<Reference>, String> {
    let document = xml(bytes)?;
    let mut result = Vec::new();
    let mut ids = BTreeSet::new();
    for pair in document
        .root_element()
        .children()
        .filter(|n| tagged(*n, "qa_pair"))
    {
        let id = pair.attribute("id").unwrap_or("");
        if id.is_empty() || !ids.insert(id) {
            return Err("reference question IDs must be present and unique".into());
        }
        result.push(read_pair(pair, id)?);
    }
    if result.is_empty() {
        return Err("reference must contain questions".into());
    }
    Ok(result)
}
pub(super) fn questions(bytes: &[u8]) -> Result<String, String> {
    let document = xml(bytes)?;
    // The original questions command is deliberately separate from strict scoring.
    let mut questions = IndexMap::new();
    for pair in document
        .root_element()
        .children()
        .filter(|n| tagged(*n, "qa_pair"))
    {
        let id = pair.attribute("id").ok_or("reference question has no id")?;
        let question = element_text(child(pair, "question"))
            .split(is_space)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        questions.insert(id, question);
    }
    Ok(questions
        .into_iter()
        .map(|(id, q)| format!("{id}: {q}\n"))
        .collect())
}
fn digest_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn unknown_questions(recorded: &IndexMap<String, Json>, references: &[Reference]) -> Vec<String> {
    let ids: BTreeSet<_> = references.iter().map(|r| r.id.as_str()).collect();
    let mut unknown = recorded
        .keys()
        .filter(|id| !ids.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unknown.sort();
    unknown
}
fn selected_names(entry: &Json) -> BTreeSet<&str> {
    if let Some(object) = entry.object() {
        names(&object["calls"])
    } else {
        entry
            .array()
            .unwrap()
            .iter()
            .map(|v| v.string().unwrap())
            .collect()
    }
}
fn list_differences(expected: &BTreeSet<&str>, selected: &BTreeSet<&str>) -> (bool, Vec<String>) {
    let missed: Vec<_> = expected.difference(selected).copied().collect();
    let extra: Vec<_> = selected.difference(expected).copied().collect();
    let mut details = Vec::new();
    if !missed.is_empty() {
        details.push(format!("missed {}", missed.join(", ")));
    }
    if !extra.is_empty() {
        details.push(format!("also called {}", extra.join(", ")));
    }
    (missed.is_empty() && extra.is_empty(), details)
}
fn extend_details(entry: &Json, reference: &Reference, details: &mut Vec<String>) -> bool {
    if let Some(object) = entry.object() {
        if object["calls"] != reference.calls {
            details.push("calls differ (arguments, order or count)".into());
        }
        if strip(object["answer"].string().unwrap()) != strip(&reference.answer) {
            details.push("answer differs".into());
        }
        details.is_empty()
    } else {
        details.push("arguments and answer NOT EVALUATED".into());
        false
    }
}
fn append_case(
    output: &mut String,
    reference: &Reference,
    entry: Option<&Json>,
    width: usize,
) -> (i32, i32) {
    let id = format!(
        "{}{}",
        reference.id,
        " ".repeat(width - reference.id.chars().count())
    );
    let Some(entry) = entry else {
        output.push_str(&format!("{id}  NOT ANSWERED\n"));
        return (0, 0);
    };
    let selected = selected_names(entry);
    let (exact, mut details) = list_differences(&names(&reference.calls), &selected);
    let verified = extend_details(entry, reference, &mut details);
    output.push_str(&format!(
        "{id}  {}\n",
        if details.is_empty() {
            "calls and answer match".into()
        } else {
            details.join("; ")
        }
    ));
    (i32::from(exact), i32::from(verified))
}
fn score(
    recorded: &IndexMap<String, Json>,
    bytes: &[u8],
    frozen: Option<&str>,
) -> Result<(i32, String, String), String> {
    let references = references(bytes)?;
    let digest = digest_hex(bytes);
    if frozen.is_some_and(|sha| sha != digest) {
        return Err("reference digest differs from the frozen trial".into());
    }
    let unknown = unknown_questions(recorded, &references);
    if !unknown.is_empty() {
        return Ok((
            2,
            String::new(),
            format!("not questions in this evaluation: {}\n", unknown.join(", ")),
        ));
    }
    let width = references
        .iter()
        .map(|r| r.id.chars().count())
        .max()
        .unwrap();
    let mut output = String::new();
    let mut exact = 0;
    let mut verified = 0;
    for reference in &references {
        let (case_exact, case_verified) =
            append_case(&mut output, reference, recorded.get(&reference.id), width);
        exact += case_exact;
        verified += case_verified;
    }
    output.push_str(&format!("\nreference_sha256 {digest}\n{exact} of {} questions selected exactly the published tools.\n{verified} of {} recorded calls and answers match the reference.\n", references.len(), references.len()));
    Ok((0, output, String::new()))
}
pub(super) fn judge(
    text: Result<String, String>,
    reference_bytes: Option<Result<Vec<u8>, String>>,
    embedded: &[u8],
    frozen: Option<&str>,
) -> Result<(i32, String, String), String> {
    // Parsing, equality and destruction all recurse. Reserve the larger
    // stack for the complete operation, including all error paths and drops.
    stacker::grow(16 * 1024 * 1024, || {
        let result = (|| {
            let text = text?;
            let recorded = json::parse(&text)?;
            let recorded = validate(&recorded)?;
            let bytes = match reference_bytes {
                None => embedded.to_vec(),
                Some(Ok(bytes)) => bytes,
                Some(Err(error)) => return Err(error),
            };
            score(recorded, &bytes, frozen)
        })();
        match result {
            Ok(output) => Ok(output),
            Err(e) => Ok((2, String::new(), format!("invalid recording: {e}\n"))),
        }
    })
}
