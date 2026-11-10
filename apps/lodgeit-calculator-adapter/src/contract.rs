// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `contract.py`: reviewed contract snapshots, and what happens when the live one moves.
//!
//! A snapshot records what a provider's surface looked like when a person read
//! it. A contract change is reviewed by a person, never accepted
//! automatically: `compare` reports drift, and nothing here rewrites a
//! snapshot. A snapshot's members are whatever JSON it holds, so each use
//! fails as Python's did on a value of the wrong type.
use std::collections::{BTreeMap, HashSet};

use sha2::{Digest, Sha256};

use crate::decimal::Decimal;
use crate::errors::CONTRACT;
use crate::p;
use crate::pyjson::{self, DecodeError, Json, Numbers};
use crate::pyops::{Exc, R, S, get_attr, getitem, hash_key, iterate, pts, sorted_set, type_error};
use crate::pyos::{OsError, Style};
use crate::pypath::PyPath;
use crate::pyvalue::{Key, item_eq, py_eq, repr, str_points, type_name};
use crate::wire;

pub(crate) const DEPRECIATION_RANGE: &str = "urn:sbrm:calculator:depreciation:range";

/// Reconcile a depreciation movement within the adapter's decimal limits.
/// At most 15 integral and 10 fractional digits per operand leave enough
/// room for the intermediate sums in the 28-digit decimal context.
pub(crate) fn range_gap(
    opening: &Decimal,
    additions: &Decimal,
    charge: &Decimal,
    closing: &Decimal,
) -> Result<(Decimal, Decimal), S> {
    for (name, value) in [
        ("opening_wdv", opening),
        ("cost_additions", additions),
        ("range_dep", charge),
        ("closing_wdv", closing),
    ] {
        wire::check_writable(value, &pts(name)).map_err(|error| error.0)?;
    }
    if [additions, charge]
        .into_iter()
        .any(|value| value.is_negative() && !value.is_zero())
    {
        return Err(pts("cost additions and depreciation must be non-negative"));
    }
    let error = |_| pts("the depreciation movement exceeds the supported decimal range");
    let expected = opening
        .add(additions)
        .and_then(|sum| sum.sub(charge))
        .map_err(error)?;
    let gap = closing.sub(&expected).map_err(error)?;
    Ok((expected, gap))
}

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
    pub openapi_input_schemas: Option<Json>,
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
    let openapi_input_schemas = if data.get("openapi_input_schemas").is_some() {
        pyjson::loads_numbers(&text, Numbers::Exact)
            .map_err(|error| invalid(format!("{error:?}")))?
            .get("openapi_input_schemas")
            .cloned()
    } else {
        None
    };
    Ok(Contract {
        provider,
        snapshot_id,
        read_at,
        read_by,
        calculators,
        response_contract,
        openapi_input_schemas,
        notes,
        sha256: Sha256::digest(&payload)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
        path,
    })
}

/// Ignore documentation at schema nodes, while retaining property names and
/// literal values such as defaults and enums.
fn schema_shape(schema: &Json) -> Json {
    let Json::Object(fields) = schema else {
        return schema.clone();
    };
    Json::Object(
        fields
            .iter()
            .filter_map(|(key, value)| {
                let shape = match key.as_str() {
                    Some(
                        "title" | "description" | "examples" | "example" | "externalDocs"
                        | "$comment",
                    ) => return None,
                    Some("properties" | "patternProperties" | "$defs" | "dependentSchemas") => {
                        if let Json::Object(properties) = value {
                            Json::Object(
                                properties
                                    .iter()
                                    .map(|(name, schema)| (name.clone(), schema_shape(schema)))
                                    .collect(),
                            )
                        } else {
                            value.clone()
                        }
                    }
                    Some("anyOf" | "allOf" | "oneOf" | "prefixItems") => {
                        if let Json::Array(schemas) = value {
                            Json::Array(schemas.iter().map(schema_shape).collect())
                        } else {
                            value.clone()
                        }
                    }
                    Some("items" | "additionalProperties" | "not" | "if" | "then" | "else") => {
                        schema_shape(value)
                    }
                    Some("required" | "enum" | "type") => {
                        let mut value = value.clone();
                        if let Json::Array(items) = &mut value
                            && items.iter().all(|item| item.as_str().is_some())
                        {
                            items.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
                        }
                        value
                    }
                    _ => value.clone(),
                };
                Some((key.clone(), shape))
            })
            .collect(),
    )
}

fn same_schema_value(left: &Json, right: &Json) -> bool {
    match (left, right) {
        (Json::Object(left), Json::Object(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, value)| {
                    right
                        .iter()
                        .any(|(other, expected)| key == other && same_schema_value(value, expected))
                })
        }
        (Json::Array(left), Json::Array(right)) => {
            left.len() == right.len()
                && left.iter().zip(right).all(|(a, b)| same_schema_value(a, b))
        }
        (Json::Dec(left), Json::Dec(right)) => left.equals(right).unwrap_or(false),
        (Json::Int(int), Json::Dec(decimal)) | (Json::Dec(decimal), Json::Int(int)) => {
            decimal.equals(&Decimal::from_bigint(int)).unwrap_or(false)
        }
        _ => left == right,
    }
}

/// Path item parameters apply to its POST unless the operation overrides the
/// same name and location. Only the two route placeholders are sent today.
fn check_request_parameters(document: &Json, item: &Json, route: &str) -> Vec<S> {
    let mut effective = BTreeMap::new();
    let unreadable = || {
        vec![pts(&format!(
            "live OpenAPI parameters for {route} could not be read"
        ))]
    };
    for owner in [item, item.at("post")] {
        let Some(parameters) = owner.get("parameters") else {
            continue;
        };
        let Json::Array(parameters) = parameters else {
            return unreadable();
        };
        let mut seen = HashSet::new();
        for parameter in parameters {
            let parameter = if let Some(reference) = parameter.get("$ref") {
                let Some(component) = reference
                    .as_str()
                    .and_then(|reference| reference.strip_prefix("#/components/parameters/"))
                    .and_then(|name| document.at("components").at("parameters").get(name))
                else {
                    return unreadable();
                };
                component
            } else {
                parameter
            };
            let (Some(location), Some(name)) =
                (parameter.at("in").as_str(), parameter.at("name").as_str())
            else {
                return unreadable();
            };
            // OpenAPI ignores parameter declarations for these HTTP headers.
            if location == "header"
                && ["Accept", "Content-Type", "Authorization"]
                    .iter()
                    .any(|header| name.eq_ignore_ascii_case(header))
            {
                continue;
            }
            let required = match parameter.get("required") {
                Some(Json::Bool(required)) => *required,
                None => false,
                Some(_) => return unreadable(),
            };
            if !seen.insert((location, name)) {
                return unreadable();
            }
            effective.insert((location, name), required);
        }
    }
    effective
        .into_iter()
        .filter_map(|((location, name), required)| {
            let supplied = location == "path"
                && matches!(name, "calc_uri" | "period_uri")
                && route.contains(&format!("{{{name}}}"));
            if !required || supplied {
                return None;
            }
            Some(pts(&format!(
                "live OpenAPI {route} requires {location} parameter {name}, which the adapter does not send"
            )))
        })
        .collect()
}

pub(crate) fn compare_openapi(reviewed: &Json, calculators: &Json, document: &Json) -> R<Vec<S>> {
    let Json::Object(schemas) = reviewed else {
        return Err(contract_error(pts(
            "reviewed OpenAPI input schemas must be an object",
        )));
    };
    let live = document.at("components").at("schemas");
    let mut findings = Vec::new();
    for (name, expected) in schemas {
        let Some(name) = name.as_str() else {
            return Err(contract_error(pts(
                "reviewed OpenAPI schema name is not UTF-8",
            )));
        };
        match live.get(name) {
            None => findings.push(pts(&format!("live OpenAPI is missing input schema {name}"))),
            Some(actual) if !same_schema_value(&schema_shape(actual), &schema_shape(expected)) => {
                findings.push(pts(&format!("live OpenAPI input schema {name} changed")));
            }
            Some(_) => {}
        }
    }
    let Json::Object(calculators) = calculators else {
        return Err(contract_error(pts(
            "reviewed calculators must be an object",
        )));
    };
    for (_, spec) in calculators {
        let Some(route) = spec.at("route").as_str() else {
            return Err(contract_error(pts("reviewed calculator has no route")));
        };
        let expected = Json::obj([("$ref", spec.at("input_schema_ref").clone())]);
        let item = document.at("paths").at(route);
        let body = item.at("post").at("requestBody");
        let shape = schema_shape(body.at("content").at("application/json").at("schema"));
        let matched = if route.contains("{calc_uri}") {
            matches!(&shape, Json::Object(fields) if fields.len() == 1)
                && matches!(shape.at("anyOf"), Json::Array(schemas)
                    if schemas.iter().any(|schema| same_schema_value(schema, &expected)))
        } else {
            same_schema_value(&shape, &expected)
        };
        if body.at("required") != &Json::Bool(true) || !matched {
            findings.push(pts(&format!(
                "live OpenAPI request schema for {route} changed"
            )));
        }
        findings.extend(check_request_parameters(document, item, route));
    }
    Ok(findings)
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
        let prefix = if contract.openapi_input_schemas.is_some() {
            "note: live has "
        } else {
            "live has "
        };
        findings.push(p!(
            prefix,
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

    #[test]
    fn schema_comparison_preserves_numbers_and_ignores_required_order() {
        let read = |json| pyjson::loads_numbers(json, Numbers::Exact).unwrap();
        let reference = schema_shape(&read(r#"{"multipleOf":0.01,"required":["a","b"]}"#));
        for changed in [
            r#"{"multipleOf":0.0100000000000000001,"required":["b","a"]}"#,
            r#"{"multipleOf":"0.01","required":["b","a"]}"#,
        ] {
            assert!(!same_schema_value(
                &reference,
                &schema_shape(&read(changed))
            ));
        }
        assert!(same_schema_value(
            &reference,
            &schema_shape(&read(r#"{"required":["b","a"],"multipleOf":1e-2}"#)),
        ));
        assert!(pyjson::loads_numbers("1e9999999999999999999999999999", Numbers::Exact).is_err());
    }

    #[test]
    fn schema_comparison_ignores_documentation_only_changes() {
        let read = |json| pyjson::loads_numbers(json, Numbers::Exact).unwrap();
        let reference = schema_shape(&read(r#"{"properties":{"a":{"type":"string"}}}"#));
        let documented = r#"{"$comment":"x","externalDocs":{"url":"https://example.com"},
            "properties":{"a":{"type":"string","$comment":"y","description":"z"}}}"#;
        assert!(same_schema_value(
            &reference,
            &schema_shape(&read(documented))
        ));
    }

    #[test]
    fn post_parameters_include_inherited_references_and_allow_operation_overrides() {
        let document = pyjson::loads_standard(
            r##"{
            "components":{"parameters":{"Window":{"name":"window","in":"query","required":true}}}
        }"##,
        )
        .unwrap();
        let route = "/v1/depreciation/{period_uri}";
        let check = |item| {
            check_request_parameters(&document, &pyjson::loads_standard(item).unwrap(), route)
        };
        assert_eq!(
            check(
                r##"{
            "parameters":[{"$ref":"#/components/parameters/Window"}]
        }"##
            ),
            [pts(&format!(
                "live OpenAPI {route} requires query parameter window, which the adapter does not send"
            ))]
        );
        assert!(
            check(
                r##"{
            "parameters":[{"$ref":"#/components/parameters/Window"}],
                "post":{"parameters":[{"name":"window","in":"query","required":false},
                                     {"name":"period_uri","in":"path","required":true},
                                     {"name":"content-type","in":"header","required":true}]}
        }"##
            )
            .is_empty()
        );
        assert_eq!(
            check(
                r##"{
            "post":{"parameters":[{"$ref":"#/components/parameters/Missing"}]}
        }"##
            ),
            [pts(&format!(
                "live OpenAPI parameters for {route} could not be read"
            ))]
        );
    }

    fn snapshot(calculators: &str) -> Contract {
        Contract {
            provider: Json::str("p"),
            snapshot_id: Json::str("s1"),
            read_at: Json::Null,
            read_by: Json::Null,
            calculators: pyjson::loads_standard(calculators).unwrap(),
            response_contract: Json::Null,
            openapi_input_schemas: None,
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
