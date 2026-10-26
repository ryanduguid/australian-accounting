//! `client.py`: discovery and bounded invocation, with honest outcomes.
//!
//! Every call returns an [`Outcome`] whose `status` says what happened. There
//! is no path from a timeout, a malformed body, a missing advisory or an
//! unreviewed schema to a number. `result` is populated only for `COMPUTED`.
//! Anything Python's client does not catch, such as a malformed snapshot's
//! `AttributeError` or the transport's `OverflowError`, is returned as the
//! error it raised.
use crate::config::AdapterConfig;
use crate::contract::{self, Contract};
use crate::decimal::Decimal;
use crate::errors::{CONFIGURATION, CONTRACT, DISALLOWED_TARGET, NOT_ENABLED, TRANSPORT};
use crate::p;
use crate::pyjson::{self, DecodeError, Json, Numbers};
use crate::pyops::{Exc, R, S, get_attr, hash_key, iterate, json_str, pts, type_error};
use crate::pyvalue::{item_eq, str_points, type_name, utf8_strict_error};
use crate::transport::{self, Call, Dial};
use crate::wire;

/// What happened to one call. Not an accounting verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Computed,
    /// provider 400 or 401/403, with its own refusal_class if it sent one
    UpstreamRefused,
    /// provider 422: our request was malformed
    UpstreamRejected,
    /// provider 404: unknown calculator or period
    UpstreamNotFound,
    /// provider 429
    UpstreamThrottled,
    /// 5xx, timeout, connection failure
    UpstreamUnavailable,
    /// answered, but not in a shape we accept
    ContractFailure,
    /// not enabled, or the target was not allowed
    RefusedToSend,
}

impl Status {
    /// `str(status)`.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Computed => "COMPUTED",
            Status::UpstreamRefused => "UPSTREAM_REFUSED",
            Status::UpstreamRejected => "UPSTREAM_REJECTED",
            Status::UpstreamNotFound => "UPSTREAM_NOT_FOUND",
            Status::UpstreamThrottled => "UPSTREAM_THROTTLED",
            Status::UpstreamUnavailable => "UPSTREAM_UNAVAILABLE",
            Status::ContractFailure => "CONTRACT_FAILURE",
            Status::RefusedToSend => "REFUSED_TO_SEND",
        }
    }
}

/// One call, with everything needed to audit it later. A Python `None` is
/// `Json::Null` or `Option::None`.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub status: Status,
    pub calculator: Option<S>,
    pub period: Option<S>,
    pub request_body: Option<Json>,
    pub request_json: Option<S>,
    pub raw_response: Json,
    pub http_status: Option<u16>,
    pub result: Json,
    /// `values`, in insertion order; its keys are always strings.
    pub values: Vec<(S, Decimal)>,
    pub manifest: Json,
    pub advisory: Json,
    pub upstream_refusal_class: Json,
    pub findings: Vec<S>,
    pub observed_at: String,
    pub elapsed_ms: Option<u64>,
    pub contract_snapshot: Json,
    pub contract_sha256: String,
    pub base_url: Option<String>,
}

impl Outcome {
    pub fn computed(&self) -> bool {
        self.status == Status::Computed
    }

    fn with(&self, status: Status, findings: Vec<S>) -> Outcome {
        Outcome {
            status,
            findings,
            ..self.clone()
        }
    }
}

/// `_now()`: UTC, ISO format to the second. The differential build fixes
/// it, as the harness fixes Python's.
pub fn now() -> String {
    if cfg!(feature = "differential") {
        return "2026-01-01T00:00:00+00:00".into();
    }
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    // 719,163 is the proleptic Gregorian ordinal of 1970-01-01.
    let date = crate::py::Date::from_ordinal(719_163 + days as i64)
        .map_or_else(String::new, |d| d.isoformat());
    format!(
        "{date}T{:02}:{:02}:{:02}+00:00",
        rest / 3600,
        rest / 60 % 60,
        rest % 60
    )
}

/// `str.replace(old, new)` over code points.
fn replace(text: &[u32], old: &str, new: &[u32]) -> S {
    let old = pts(old);
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text[i..].starts_with(&old) {
            out.extend_from_slice(new);
            i += old.len();
        } else {
            out.push(text[i]);
            i += 1;
        }
    }
    out
}

/// `mapping.get(key)` for any key: the `AttributeError` a non-dict raises,
/// or the `TypeError` an unhashable key does. JSON keys are strings, so a
/// key of another type is never found.
pub(crate) fn dict_get<'a>(mapping: &'a Json, key: &Json) -> R<Option<&'a Json>> {
    get_attr(mapping, "")?;
    hash_key(key, "dict key")?;
    let (Json::Object(pairs), Json::Str(key)) = (mapping, key) else {
        return Ok(None);
    };
    let key = key.code_points();
    Ok(pairs
        .iter()
        .find(|(k, _)| k.code_points() == key)
        .map(|(_, v)| v))
}

/// `', '.join(value)`.
fn join(value: &Json) -> R<S> {
    let items = iterate(value).map_err(|_| type_error("can only join an iterable"))?;
    let mut out = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let Json::Str(s) = item else {
            return Err(type_error(&format!(
                "sequence item {i}: expected str instance, {} found",
                type_name(item)
            )));
        };
        if i > 0 {
            out.extend(pts(", "));
        }
        out.extend(s.code_points());
    }
    Ok(out)
}

/// `[*value]`.
fn star(value: &Json) -> R<Vec<Json>> {
    iterate(value).map_err(|_| {
        type_error(&format!(
            "Value after * must be an iterable, not {}",
            type_name(value)
        ))
    })
}

/// `item in container`, where `own` says the item came out of `container`
/// itself, so a list finds it by identity.
fn contains(container: &Json, item: &Json, own: bool) -> R<bool> {
    match container {
        Json::Array(items) => Ok(own || items.iter().any(|i| item_eq(i, item))),
        Json::Object(_) => Ok(dict_get(container, item)?.is_some()),
        Json::Str(s) => match item {
            Json::Str(needle) => Ok(crate::pyops::contains(
                &s.code_points(),
                &needle.code_points(),
            )),
            other => Err(type_error(&format!(
                "'in <string>' requires string as left operand, not {}",
                type_name(other)
            ))),
        },
        other => Err(type_error(&format!(
            "argument of type '{}' is not a container or iterable",
            type_name(other)
        ))),
    }
}

/// A thin, bounded client for one provider surface. Constructing one does
/// nothing over the network; every call that would send checks
/// `config.require_enabled()` first, through the transport.
pub struct Client<'a> {
    pub config: &'a AdapterConfig,
    pub contract: &'a Contract,
    pub dial: &'a mut dyn Dial,
    /// `time.sleep`, given each retry's backoff.
    pub sleep: &'a mut dyn FnMut(f64) -> R<()>,
}

impl Client<'_> {
    /// `discover()`: the live calculator listing. Offline, the transport's
    /// `NotEnabledError` makes it `REFUSED_TO_SEND`.
    pub fn discover(&mut self) -> R<Outcome> {
        self.call("GET", &pts("/v1/calculators"), None, None, None, false)
    }

    /// `drift()`: discovery plus the findings against the reviewed snapshot.
    pub fn drift(&mut self) -> R<(Outcome, Vec<S>)> {
        let outcome = self.discover()?;
        if !outcome.computed() {
            return Ok((outcome, Vec::new()));
        }
        let Json::Array(listing) = &outcome.result else {
            // A 200 that is not a listing is not agreement with the snapshot.
            let finding = pts(&format!(
                "live discovery returned {} where a list of calculators was expected; nothing \
                 was compared",
                type_name(&outcome.result)
            ));
            return Ok((outcome, vec![finding]));
        };
        let findings = contract::compare(self.contract, listing)?;
        Ok((outcome, findings))
    }

    /// `invoke(calc_uri, period_uri, body)`: one calculator for one period,
    /// with the contract checks around it.
    pub fn invoke(&mut self, calc_uri: &[u32], period_uri: &[u32], body: &Json) -> R<Outcome> {
        let refused = |finding: S| {
            Outcome {
                calculator: Some(calc_uri.to_vec()),
                period: Some(period_uri.to_vec()),
                ..self.common(None)
            }
            .with(Status::ContractFailure, vec![finding])
        };
        match self.contract.check_period(calc_uri, period_uri) {
            Ok(()) => {}
            Err(error) if error.is(CONTRACT) => return Ok(refused(error.text)),
            Err(error) => return Err(error),
        }
        let route = get_attr(self.contract.calculator(calc_uri)?, "route")?;
        let route = match route {
            Some(route) if route.truthy() => route,
            _ => return Ok(refused(p!("snapshot records no route for ", calc_uri))),
        };
        let Json::Str(route) = route else {
            return Err(Exc::msg(
                "AttributeError",
                &format!("'{}' object has no attribute 'replace'", type_name(route)),
            ));
        };
        let path = replace(&route.code_points(), "{period_uri}", period_uri);
        let path = replace(&path, "{calc_uri}", calc_uri);
        self.call(
            "POST",
            &path,
            Some(body),
            Some(calc_uri),
            Some(period_uri),
            true,
        )
    }

    /// The fields every outcome of a call carries, before its status. A
    /// refusal before the call records no base URL.
    fn common(&self, base_url: Option<String>) -> Outcome {
        Outcome {
            status: Status::ContractFailure,
            calculator: None,
            period: None,
            request_body: None,
            request_json: None,
            raw_response: Json::Null,
            http_status: None,
            result: Json::Null,
            values: Vec::new(),
            manifest: Json::Null,
            advisory: Json::Null,
            upstream_refusal_class: Json::Null,
            findings: Vec::new(),
            observed_at: now(),
            elapsed_ms: None,
            contract_snapshot: self.contract.snapshot_id.clone(),
            contract_sha256: self.contract.sha256.clone(),
            base_url,
        }
    }

    /// `_call(method, path, ...)`: the one place a request is made.
    fn call(
        &mut self,
        method: &str,
        path: &[u32],
        body: Option<&Json>,
        calc_uri: Option<&[u32]>,
        period_uri: Option<&[u32]>,
        expect_contract: bool,
    ) -> R<Outcome> {
        let base = self.config.base_url.as_deref().unwrap_or("");
        let url = format!(
            "{}{}",
            base.trim_end_matches('/'),
            crate::pyvalue::points_lossy(path)
        );
        let mut common = Outcome {
            calculator: calc_uri.map(<[u32]>::to_vec),
            period: period_uri.map(<[u32]>::to_vec),
            ..self.common(self.config.base_url.clone())
        };
        let mut encoded = None;
        if let Some(body) = body {
            let text = wire::dumps(body, &pts("body"))
                .map_err(|error| error.0)
                .and_then(|text| match utf8_strict_error(&text) {
                    Some(message) => Err(pts(&message)),
                    None => Ok(text),
                });
            match text {
                Ok(text) => encoded = Some(text),
                Err(message) => {
                    return Ok(Outcome {
                        request_body: Some(body.clone()),
                        ..common.with(
                            Status::ContractFailure,
                            vec![p!("request could not be serialised exactly: ", message)],
                        )
                    });
                }
            }
        }
        common.request_body = body.cloned();
        common.request_json = encoded.clone();
        let payload: Option<String> = encoded
            .as_ref()
            .map(|text| text.iter().filter_map(|&cp| char::from_u32(cp)).collect());
        let call = Call {
            method,
            url: &url,
            body: payload.as_deref().map(str::as_bytes),
            content_type: payload.as_ref().map(|_| "application/json"),
            // These calculators are stateless and the provider documents a
            // 5xx as its own fault, so GET and POST retry on a 5xx or a
            // transport failure; the transport never retries a 4xx.
            retry_safe: true,
            bearer_token: None,
        };
        let raw = match transport::request(self.config, self.dial, &call, self.sleep) {
            Ok(raw) => raw,
            Err(error) if error.is_any(&[NOT_ENABLED, CONFIGURATION, DISALLOWED_TARGET]) => {
                return Ok(common.with(Status::RefusedToSend, vec![error.text]));
            }
            Err(error) if error.is(TRANSPORT) => {
                return Ok(common.with(Status::UpstreamUnavailable, vec![error.text]));
            }
            Err(error) => return Err(error),
        };
        common.elapsed_ms = Some(raw.elapsed_ms);
        common.http_status = Some(raw.status);
        // parse_float and parse_int keep the provider's own digits;
        // parse_constant refuses NaN and Infinity.
        let parsed = match raw.text().and_then(|text| {
            pyjson::loads_numbers(&text, Numbers::Text).map_err(|error| match error {
                DecodeError::Invalid(m) => Exc::msg("json.decoder.JSONDecodeError", &m),
                DecodeError::Hook(m) => Exc::msg("ValueError", &m),
                DecodeError::Recursion(kind) => {
                    Exc::msg("RecursionError", &pyjson::recursion_message(kind))
                }
                DecodeError::Duplicate => unreachable!("no duplicate hook"),
            })
        }) {
            Ok(parsed) => {
                common.raw_response = parsed.clone();
                Some(parsed)
            }
            Err(error) if error.is_any(&[TRANSPORT, "ValueError"]) => {
                let head = &raw.body[..raw.body.len().min(2048)];
                let bounded = Json::str(&String::from_utf8_lossy(head));
                if raw.status == 200 {
                    return Ok(Outcome {
                        raw_response: bounded,
                        ..common.with(
                            Status::ContractFailure,
                            vec![p!("response body is not JSON: ", error.text)],
                        )
                    });
                }
                // Only a 200 has to be JSON; a failure status keeps its own
                // outcome whatever its body is.
                common.raw_response = bounded;
                None
            }
            Err(error) => return Err(error),
        };
        let status = raw.status;
        let refusal = || match &parsed {
            Some(Json::Object(_)) => parsed.as_ref().and_then(|p| p.get("refusal_class")),
            _ => None,
        };
        let finding = |text: String| vec![pts(&text)];
        if status >= 500 {
            return Ok(common.with(
                Status::UpstreamUnavailable,
                finding(format!("provider returned {status}")),
            ));
        }
        if status == 429 {
            let after = raw
                .headers
                .iter()
                .find(|(name, _)| name == "retry-after")
                .map_or("absent", |(_, value)| value.as_str());
            return Ok(common.with(
                Status::UpstreamThrottled,
                finding(format!("provider returned 429; Retry-After: {after}")),
            ));
        }
        if status == 404 {
            return Ok(common.with(
                Status::UpstreamNotFound,
                vec![p!("provider returned 404 for ", path)],
            ));
        }
        if status == 422 {
            return Ok(common.with(
                Status::UpstreamRejected,
                finding(
                    "provider returned 422: our request was malformed. Do not retry unchanged."
                        .into(),
                ),
            ));
        }
        if matches!(status, 400 | 401 | 403) {
            let refusal = refusal().cloned().unwrap_or(Json::Null);
            let text = if status == 400 {
                let shown = if refusal.truthy() {
                    str_points(&refusal)
                } else {
                    pts("no refusal_class in the body")
                };
                p!("provider refused: ", shown)
            } else {
                pts(&format!(
                    "provider returned {status}: it declined to answer. This adapter holds no \
                     credential and adds none."
                ))
            };
            return Ok(Outcome {
                upstream_refusal_class: refusal,
                ..common.with(Status::UpstreamRefused, vec![text])
            });
        }
        if status != 200 {
            return Ok(common.with(
                Status::ContractFailure,
                finding(format!("unexpected status {status}")),
            ));
        }
        let parsed = parsed.expect("a 200 that is not JSON has returned");
        if !expect_contract {
            return Ok(Outcome {
                result: parsed,
                ..common.with(Status::Computed, Vec::new())
            });
        }
        self.accept(parsed, calc_uri, common)
    }

    /// `_accept(parsed, calc_uri, common)`: a 200 checked against the
    /// reviewed contract before it is called a result.
    fn accept(&self, parsed: Json, calc_uri: Option<&[u32]>, common: Outcome) -> R<Outcome> {
        let Json::Object(members) = &parsed else {
            return Ok(common.with(
                Status::ContractFailure,
                vec![pts("a 200 body that is not a JSON object")],
            ));
        };
        let mut findings: Vec<S> = Vec::new();
        let rules = &self.contract.response_contract;
        let empty = Json::Object(Vec::new());
        let calc_key = json_str(calc_uri.unwrap_or_default().to_vec());
        let recorded = dict_get(&self.contract.calculators, &calc_key)?.unwrap_or(&empty);

        let manifest = parsed.get("manifest").cloned().unwrap_or(Json::Null);
        let advisory = parsed.get("advisory").cloned().unwrap_or(Json::Null);
        let rule = |name: &str, default: bool| -> R<bool> {
            Ok(get_attr(rules, name)?.map_or(default, Json::truthy))
        };
        if rule("requires_manifest", true)?
            && !(matches!(manifest, Json::Object(_)) && manifest.truthy())
        {
            findings.push(pts(
                "no usable manifest block: the response does not name what it consumed. An \
                 empty object is not an answer to that question.",
            ));
        }
        if rule("requires_advisory", true)? {
            let notes = Json::Array(vec![Json::str("notes")]);
            let carriers = get_attr(rules, "advisory_any_of")?.unwrap_or(&notes);
            let mut carried = false;
            if matches!(advisory, Json::Object(_)) {
                for key in iterate(carriers)? {
                    if dict_get(&advisory, &key)?.is_some_and(Json::truthy) {
                        carried = true;
                        break;
                    }
                }
            }
            if !carried {
                findings.push(p!(
                    "no advisory block: the provider's own boundary statement is missing, so \
                     the figure would travel without the framing it needs (looked for ",
                    join(carriers)?,
                    ")"
                ));
            }
        }

        let known = match get_attr(rules, "known_top_level_fields")? {
            Some(known) => iterate(known)?,
            None => Vec::new(),
        };
        let mut known_names = Vec::new();
        for item in &known {
            hash_key(item, "set element")?;
            if let Json::Str(s) = item {
                known_names.push(s.code_points());
            }
        }
        let mut unknown: Vec<S> = members
            .iter()
            .map(|(k, _)| k.code_points())
            .filter(|k| !known_names.contains(k))
            .collect();
        unknown.sort();
        if !unknown.is_empty() {
            let listed = unknown.join(&pts(", ")[..]);
            if rule("refuse_unknown_fields", false)? {
                findings.push(p!(
                    "response carries fields the snapshot does not record: ",
                    listed
                ));
            } else {
                findings.push(p!("note: response carries unrecorded fields: ", listed));
            }
        }

        // A required field's absence is a contract failure; an optional field
        // is parsed when it is there and ignored when it is not.
        let mut values: Vec<(S, Decimal)> = Vec::new();
        let none = Json::Array(Vec::new());
        let required = get_attr(recorded, "required_decimal_fields")?.unwrap_or(&none);
        let mut names = star(required)?;
        let from_required = names.len();
        names.extend(star(
            get_attr(recorded, "optional_decimal_fields")?.unwrap_or(&none),
        )?);
        for (index, name) in names.iter().enumerate() {
            let Some(value) = dict_get(&parsed, name)? else {
                if contains(required, name, index < from_required)? {
                    findings.push(p!(
                        "required decimal field ",
                        str_points(name),
                        " is absent"
                    ));
                }
                continue;
            };
            let Json::Str(name) = name else {
                unreachable!("only a str key is found")
            };
            let name = name.code_points();
            match wire::parse_decimal(value, &name) {
                Ok(decimal) => match values.iter_mut().find(|(k, _)| *k == name) {
                    Some(slot) => slot.1 = decimal,
                    None => values.push((name, decimal)),
                },
                Err(error) => findings.push(error.0),
            }
        }

        let blocking = findings.iter().any(|f| !f.starts_with(&pts("note:")));
        let manifest = if matches!(manifest, Json::Object(_)) {
            manifest
        } else {
            Json::Null
        };
        let advisory = if matches!(advisory, Json::Object(_)) {
            advisory
        } else {
            Json::Null
        };
        if blocking {
            return Ok(Outcome {
                manifest,
                advisory,
                ..common.with(Status::ContractFailure, findings)
            });
        }
        Ok(Outcome {
            result: parsed.clone(),
            values,
            manifest,
            advisory,
            ..common.with(Status::Computed, findings)
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::{self, Read, Write};

    /// A connection that replies with fixed bytes and swallows what it is sent.
    struct Reply(io::Cursor<Vec<u8>>);

    impl Read for Reply {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl Write for Reply {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Each dial takes the next reply; one past them is refused.
    pub(crate) struct Replies(pub Vec<Vec<u8>>);

    impl Dial for Replies {
        fn dial(
            &mut self,
            _: &str,
            _: u16,
            _: bool,
            _: f64,
        ) -> io::Result<Box<dyn crate::transport::Connection>> {
            if self.0.is_empty() {
                return Err(io::ErrorKind::ConnectionRefused.into());
            }
            Ok(Box::new(Reply(io::Cursor::new(self.0.remove(0)))))
        }
    }

    /// An HTTP/1.1 reply with a Content-Length.
    pub(crate) fn reply(status: &str, headers: &str, body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    fn text(points: &[u32]) -> String {
        points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
    }

    /// `invoke` of the depreciation calculator against the replies, with
    /// the backoffs it slept.
    fn invoke(replies: Vec<Vec<u8>>) -> (Outcome, Vec<f64>) {
        let contract = contract::load("lodgeit-calculators", None).unwrap();
        let config = AdapterConfig {
            enabled: true,
            base_url: Some("http://127.0.0.1:9".into()),
            allow_loopback: true,
            ..AdapterConfig::default()
        };
        let mut sleeps = Vec::new();
        let mut sleep = |seconds: f64| {
            sleeps.push(seconds);
            Ok(())
        };
        let mut client = Client {
            config: &config,
            contract: &contract,
            dial: &mut Replies(replies),
            sleep: &mut sleep,
        };
        let body =
            pyjson::loads_numbers(r#"{"asset": {"cost": 1.50}}"#, Numbers::DecimalFloats).unwrap();
        let outcome = client
            .invoke(
                &pts("urn:sbrm:calculator:depreciation:at"),
                &pts("urn:sbrm:period:depreciation:unscoped"),
                &body,
            )
            .unwrap();
        (outcome, sleeps)
    }

    fn findings(outcome: &Outcome) -> Vec<String> {
        outcome.findings.iter().map(|f| text(f)).collect()
    }

    #[test]
    fn a_200_is_a_result_only_with_its_manifest_advisory_and_decimals() {
        let good = r#"{"wdv_at": "100.00", "period_dep_at": 5, "manifest": {"a": 1}, "advisory": {"disclaimer": "x"}, "zz": 1}"#;
        let (outcome, _) = invoke(vec![reply("200 OK", "", good)]);
        assert_eq!(outcome.status, Status::Computed);
        assert_eq!(
            outcome.request_json.as_deref().map(text).unwrap(),
            r#"{"asset":{"cost":1.50}}"#
        );
        let values: Vec<(String, String)> = outcome
            .values
            .iter()
            .map(|(k, v)| (text(k), v.to_sci_string()))
            .collect();
        assert_eq!(
            values,
            [
                ("wdv_at".into(), "100.00".into()),
                ("period_dep_at".into(), "5".into())
            ]
        );
        assert_eq!(
            findings(&outcome),
            ["note: response carries unrecorded fields: zz"]
        );

        let bare =
            r#"{"wdv_at": "1", "period_dep_at": 1.5, "manifest": {}, "advisory": {"notes": []}}"#;
        let (outcome, _) = invoke(vec![reply("200 OK", "", bare)]);
        assert_eq!(outcome.status, Status::ContractFailure);
        assert_eq!(
            findings(&outcome),
            [
                "no usable manifest block: the response does not name what it consumed. An empty \
                 object is not an answer to that question.",
                "no advisory block: the provider's own boundary statement is missing, so the figure \
                 would travel without the framing it needs (looked for notes, disclaimer)",
            ]
        );
        // A JSON number is read as its own digits, so it parses exactly.
        assert_eq!(outcome.manifest, Json::Object(Vec::new()));
    }

    #[test]
    fn statuses_keep_their_own_outcomes() {
        let (outcome, sleeps) = invoke(vec![
            reply("503 X", "", "<html>busy</html>"),
            reply("503 X", "", "<html>busy</html>"),
        ]);
        assert_eq!(outcome.status, Status::UpstreamUnavailable);
        assert_eq!(findings(&outcome), ["provider returned 503"]);
        assert_eq!(outcome.raw_response, Json::str("<html>busy</html>"));
        assert_eq!(sleeps, [1.0]);

        let (outcome, _) = invoke(vec![reply("429 X", "Retry-After: 30\r\n", "{}")]);
        assert_eq!(
            findings(&outcome),
            ["provider returned 429; Retry-After: 30"]
        );

        let (outcome, _) = invoke(vec![reply("400 X", "", r#"{"refusal_class": "pool"}"#)]);
        assert_eq!(outcome.status, Status::UpstreamRefused);
        assert_eq!(outcome.upstream_refusal_class, Json::str("pool"));
        assert_eq!(findings(&outcome), ["provider refused: pool"]);

        let (outcome, _) = invoke(vec![reply("200 OK", "", r#"{"a": NaN}"#)]);
        assert_eq!(outcome.status, Status::ContractFailure);
        assert_eq!(
            findings(&outcome),
            ["response body is not JSON: response contains the JSON-invalid constant NaN"]
        );

        let (outcome, _) = invoke(Vec::new());
        assert_eq!(outcome.status, Status::UpstreamUnavailable);
        assert_eq!(findings(&outcome), ["no response after 2 attempt(s)"]);
    }

    #[test]
    fn a_failed_backoff_sleep_is_raised() {
        let contract = contract::load("lodgeit-calculators", None).unwrap();
        let config = AdapterConfig {
            enabled: true,
            base_url: Some("http://127.0.0.1:9".into()),
            allow_loopback: true,
            retry_backoff_seconds: f64::NAN,
            ..AdapterConfig::default()
        };
        let mut client = Client {
            config: &config,
            contract: &contract,
            dial: &mut Replies(Vec::new()),
            sleep: &mut crate::dial::sleep,
        };
        let error = client.discover().unwrap_err();
        assert_eq!(
            (error.class, text(&error.text).as_str()),
            ("ValueError", "Invalid value NaN (not a number)")
        );
    }
}
