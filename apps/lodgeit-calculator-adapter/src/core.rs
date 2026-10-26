//! `core.py`: opt-in, single-page client catalogue reads against the pinned
//! Core preview.
//!
//! Failures carry a fixed category and status only, never the upstream body
//! or the error that caused them. Python also clears the bearer token and
//! the response from its frame locals so a traceback cannot show them; a
//! Rust error carries no frames, and the token stays the caller's.
use std::fmt;

use num_bigint::BigInt;

use crate::dial::{self, Net};
use crate::errors::{CONFIGURATION, DISALLOWED_TARGET, NOT_ENABLED, TRANSPORT};
use crate::pyjson::{self, Json, PyStr};
use crate::pyops::{Exc, R};
use crate::transport::{self, Call, Dial, RequestConfig};

pub const CORE_URL: &str = "https://api.lodgeit.com/core/clients/v1-preview/list";
pub const CONTRACT_SNAPSHOT: &str = "lodgeit-core-clients-2026-10-09";
const MIB: i64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreCategory {
    Disabled,
    InvalidInput,
    Transport,
    Authentication,
    Authorisation,
    RequestRejected,
    Throttled,
    Upstream,
    Protocol,
}

impl CoreCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            CoreCategory::Disabled => "DISABLED",
            CoreCategory::InvalidInput => "INVALID_INPUT",
            CoreCategory::Transport => "TRANSPORT",
            CoreCategory::Authentication => "AUTHENTICATION",
            CoreCategory::Authorisation => "AUTHORISATION",
            CoreCategory::RequestRejected => "REQUEST_REJECTED",
            CoreCategory::Throttled => "THROTTLED",
            CoreCategory::Upstream => "UPSTREAM",
            CoreCategory::Protocol => "PROTOCOL",
        }
    }
}

/// `CoreError`: its `str()` is the category's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreError {
    pub category: CoreCategory,
    pub http_status: Option<u16>,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.category.as_str())
    }
}

impl std::error::Error for CoreError {}

fn fail<T>(category: CoreCategory, http_status: Option<u16>) -> Result<T, CoreError> {
    Err(CoreError {
        category,
        http_status,
    })
}

/// `CoreConfig`: the safety limits, fixed at construction. The attempt
/// budget is 1, the user agent fixed and there are no extra headers.
#[derive(Clone, Debug)]
pub struct CoreConfig {
    enabled: bool,
    read_timeout: f64,
    max_response_bytes: BigInt,
}

impl Default for CoreConfig {
    fn default() -> CoreConfig {
        CoreConfig {
            enabled: false,
            read_timeout: 20.0,
            max_response_bytes: BigInt::from(MIB),
        }
    }
}

impl CoreConfig {
    /// `CoreConfig(enabled, read_timeout, max_response_bytes)` and its
    /// `__post_init__`.
    pub fn new(enabled: bool, read_timeout: f64, max_response_bytes: i64) -> R<CoreConfig> {
        // `not 0 < t <= 60` refuses NaN too.
        if !(read_timeout > 0.0 && read_timeout <= 60.0) || !(1..=MIB).contains(&max_response_bytes)
        {
            return Err(Exc::msg(CONFIGURATION, "Invalid Core safety limits."));
        }
        Ok(CoreConfig {
            enabled,
            read_timeout,
            max_response_bytes: BigInt::from(max_response_bytes),
        })
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl RequestConfig for CoreConfig {
    fn require_enabled(&self) -> R<()> {
        if self.enabled {
            Ok(())
        } else {
            Err(Exc::msg(NOT_ENABLED, "Core access is disabled."))
        }
    }
    fn check_url(&self, url: &str) -> R<()> {
        if url == CORE_URL {
            Ok(())
        } else {
            Err(Exc::msg(
                DISALLOWED_TARGET,
                "Core destination is outside the fixed read route.",
            ))
        }
    }
    fn read_timeout(&self) -> f64 {
        self.read_timeout
    }
    fn max_response_bytes(&self) -> &BigInt {
        &self.max_response_bytes
    }
    fn max_attempts(&self) -> u32 {
        1
    }
    fn retry_backoff_seconds(&self) -> f64 {
        1.0
    }
    fn user_agent(&self) -> &str {
        "lodgeit-core-client-preview (australian-accounting)"
    }
    fn extra_headers(&self) -> &[(String, String)] {
        &[]
    }
}

/// An opaque page token; its `Debug` never shows it.
#[derive(Clone, PartialEq, Eq)]
pub struct CoreContinuation {
    pub value: String,
}

impl fmt::Debug for CoreContinuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("CoreContinuation(<redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CoreClientSummary {
    pub id: String,
    pub code: Option<String>,
    pub archived: bool,
    pub private: bool,
}

impl fmt::Debug for CoreClientSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("CoreClientSummary(<redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CoreClientPage {
    pub items: Vec<CoreClientSummary>,
    pub continuation: Option<CoreContinuation>,
}

impl fmt::Debug for CoreClientPage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("CoreClientPage(<redacted>)")
    }
}

/// `_text(value, limit)`: a string of at most `limit` UTF-8 bytes. A lone
/// surrogate cannot be encoded.
fn text(value: &Json, limit: usize) -> Option<String> {
    value
        .as_str()
        .filter(|s| s.len() <= limit)
        .map(str::to_owned)
}

/// `str(UUID(value)) == value`: only the canonical lower-case form survives
/// the round trip, and every canonical form parses.
fn canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, &b)| match i {
            8 | 13 | 18 | 23 => b == b'-',
            _ => matches!(b, b'0'..=b'9' | b'a'..=b'f'),
        })
}

fn keys_within(pairs: &[(PyStr, Json)], allowed: &[&str]) -> bool {
    pairs
        .iter()
        .all(|(k, _)| k.as_str().is_some_and(|k| allowed.contains(&k)))
}

/// `_parse_page(body, size)`: the page, or `None` for anything the read
/// contract does not admit. Python's `parse_float` and `parse_constant`
/// raise and its pairs hook refuses a repeated key; no number is ever
/// admitted, so decoding numbers natively and refusing them after is the
/// same.
#[doc(hidden)]
pub fn parse_page(body: &[u8], size: i64) -> Option<CoreClientPage> {
    let data = pyjson::loads(std::str::from_utf8(body).ok()?).ok()?;
    let Json::Object(top) = &data else {
        return None;
    };
    let Some(Json::Array(items)) = data.get("items") else {
        return None;
    };
    if !keys_within(top, &["items", "nextPageToken"]) || items.len() > size as usize {
        return None;
    }
    let mut summaries: Vec<CoreClientSummary> = Vec::with_capacity(items.len());
    for item in items {
        let Json::Object(pairs) = item else {
            return None;
        };
        let fields = [
            "id",
            "code",
            "archived",
            "private",
            "profile",
            "assignments",
            "groups",
            "relations",
        ];
        let id = item.get("id").and_then(Json::as_str)?;
        let code = match item.get("code") {
            None | Some(Json::Null) => None,
            Some(code) => Some(text(code, 1024)?),
        };
        let (Some(Json::Bool(false)), Some(&Json::Bool(private)), Some(_)) = (
            item.get("archived"),
            item.get("private"),
            item.get("profile"),
        ) else {
            return None;
        };
        if !keys_within(pairs, &fields)
            || !canonical_uuid(id)
            || summaries.iter().any(|s| s.id == id)
            || ["profile", "assignments", "groups", "relations"]
                .iter()
                .any(|key| item.get(key).is_some_and(|v| *v != Json::Null))
        {
            return None;
        }
        summaries.push(CoreClientSummary {
            id: id.to_owned(),
            code,
            archived: false,
            private,
        });
    }
    let continuation = match data.get("nextPageToken") {
        None | Some(Json::Null) => None,
        Some(token) => Some(CoreContinuation {
            value: text(token, 8192).filter(|t| !t.is_empty())?,
        }),
    };
    Some(CoreClientPage {
        items: summaries,
        continuation,
    })
}

/// The request body, as `json.dumps` writes it with its default separators.
fn request_body(page_size: i64, continuation: Option<&CoreContinuation>) -> Vec<u8> {
    let mut out = format!(
        "{{\"archived\": false, \"expansion\": null, \"paging\": {{\"size\": {page_size}, \"token\": "
    );
    match continuation {
        None => out.push_str("null"),
        Some(c) => pyjson::dump_str(
            &c.value.chars().map(u32::from).collect::<Vec<_>>(),
            &mut out,
        ),
    }
    out.push_str("}}");
    out.into_bytes()
}

pub struct CoreClient {
    pub config: CoreConfig,
}

impl CoreClient {
    /// `list_clients(bearer_token=..., page_size=100, continuation=None)`
    /// over the network.
    pub fn list_clients(
        &self,
        bearer_token: &str,
        page_size: i64,
        continuation: Option<&CoreContinuation>,
    ) -> Result<CoreClientPage, CoreError> {
        self.list_clients_via(&mut Net, bearer_token, page_size, continuation)
    }

    /// `list_clients` over `dial`.
    pub fn list_clients_via(
        &self,
        dial: &mut dyn Dial,
        bearer_token: &str,
        page_size: i64,
        continuation: Option<&CoreContinuation>,
    ) -> Result<CoreClientPage, CoreError> {
        if !self.config.enabled {
            return fail(CoreCategory::Disabled, None);
        }
        if !(1..=500).contains(&page_size)
            || continuation.is_some_and(|c| c.value.is_empty() || c.value.len() > 8192)
        {
            return fail(CoreCategory::InvalidInput, None);
        }
        let body = request_body(page_size, continuation);
        let call = Call {
            method: "POST",
            url: CORE_URL,
            body: Some(&body),
            content_type: Some("application/json"),
            retry_safe: false,
            bearer_token: Some(bearer_token),
        };
        let raw = match transport::request(&self.config, dial, &call, &mut dial::sleep) {
            Ok(raw) => raw,
            Err(error) if error.is(CONFIGURATION) => return fail(CoreCategory::InvalidInput, None),
            Err(error)
                if error.is_any(&[
                    TRANSPORT,
                    DISALLOWED_TARGET,
                    NOT_ENABLED,
                    "http.client.HTTPException",
                    "OSError",
                    "ValueError",
                ]) =>
            {
                return fail(CoreCategory::Transport, None);
            }
            // The transport raises `OverflowError` only for a timeout past
            // the clock, which `CoreConfig::new` refuses.
            Err(error) => unreachable!("Core transport raised {}", error.class),
        };
        let status = raw.status;
        if status != 200 {
            let category = match status {
                401 => CoreCategory::Authentication,
                403 => CoreCategory::Authorisation,
                400 | 422 => CoreCategory::RequestRejected,
                429 => CoreCategory::Throttled,
                500.. => CoreCategory::Upstream,
                _ => CoreCategory::Protocol,
            };
            return fail(category, Some(status));
        }
        let media = raw
            .headers
            .iter()
            .rev()
            .find(|(name, _)| name == "content-type")
            .map_or("", |(_, value)| value.as_str());
        let media = crate::py::strip(media.split(';').next().unwrap_or(""));
        let page = (media.to_lowercase() == "application/json")
            .then(|| parse_page(&raw.body, page_size))
            .flatten();
        page.map_or_else(|| fail(CoreCategory::Protocol, Some(status)), Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::tests::{Replies, reply};
    use std::cell::RefCell;
    use std::io::{self, Read, Write};
    use std::rc::Rc;

    const ID: &str = "0f8fad5b-d9cb-469f-a165-70867728950e";
    const BEARER: &str = "core-test-bearer";

    fn enabled() -> CoreClient {
        CoreClient {
            config: CoreConfig::new(true, 20.0, MIB).unwrap(),
        }
    }

    fn item(extra: &str) -> String {
        format!(r#"{{"id": "{ID}", "archived": false, "private": true, "profile": null{extra}}}"#)
    }

    fn json_reply(status: &str, body: &str) -> Vec<u8> {
        reply(status, "Content-Type: application/json\r\n", body)
    }

    fn list(replies: Vec<Vec<u8>>) -> Result<CoreClientPage, CoreError> {
        enabled().list_clients_via(&mut Replies(replies), BEARER, 100, None)
    }

    /// A connection that records what it is sent.
    struct Capture(io::Cursor<Vec<u8>>, Rc<RefCell<Vec<u8>>>);

    impl Read for Capture {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl Write for Capture {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.1.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct Recorder {
        reply: Vec<u8>,
        dials: Vec<(String, u16, bool)>,
        sent: Rc<RefCell<Vec<u8>>>,
    }

    impl Dial for Recorder {
        fn dial(
            &mut self,
            host: &str,
            port: u16,
            tls: bool,
            _: f64,
        ) -> io::Result<Box<dyn transport::Connection>> {
            self.dials.push((host.into(), port, tls));
            Ok(Box::new(Capture(
                io::Cursor::new(self.reply.clone()),
                Rc::clone(&self.sent),
            )))
        }
    }

    fn recorder(reply: Vec<u8>) -> Recorder {
        Recorder {
            reply,
            dials: Vec::new(),
            sent: Rc::default(),
        }
    }

    #[test]
    fn default_and_environment_cannot_enable_core() {
        let client = CoreClient {
            config: CoreConfig::default(),
        };
        let mut dial = recorder(Vec::new());
        let error = client
            .list_clients_via(&mut dial, BEARER, 100, None)
            .unwrap_err();
        assert_eq!(error.category, CoreCategory::Disabled);
        assert_eq!(error.to_string(), "DISABLED");
        assert!(dial.dials.is_empty());
    }

    #[test]
    fn exact_request_projection_and_sensitive_representations() {
        let token = "next-page-é";
        let body = format!(r#"{{"items": [{}], "nextPageToken": "{token}"}}"#, item(""));
        let mut dial = recorder(json_reply("200 OK", &body));
        let continuation = CoreContinuation {
            value: "prev\"\u{1F600}".into(),
        };
        let page = enabled()
            .list_clients_via(&mut dial, BEARER, 7, Some(&continuation))
            .unwrap();
        assert_eq!(dial.dials, [("api.lodgeit.com".to_string(), 443, true)]);
        let sent = String::from_utf8(dial.sent.borrow().clone()).unwrap();
        let (head, payload) = sent.split_once("\r\n\r\n").unwrap();
        assert!(head.starts_with("POST /core/clients/v1-preview/list HTTP/1.1\r\n"));
        assert!(head.contains(&format!("\r\nAuthorization: Bearer {BEARER}")));
        assert!(head.contains("\r\nContent-Type: application/json\r\n"));
        assert_eq!(
            payload,
            r#"{"archived": false, "expansion": null, "paging": {"size": 7, "token": "prev\"\ud83d\ude00"}}"#
        );
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, ID);
        assert_eq!(page.continuation.as_ref().unwrap().value, token);
        let shown = format!("{page:?} {:?} {continuation:?}", page.items[0]);
        assert!(!shown.contains(ID) && !shown.contains("prev") && !shown.contains("next"));
    }

    #[test]
    fn only_the_exact_read_route_is_admitted() {
        let config = CoreConfig::new(true, 20.0, MIB).unwrap();
        assert!(config.check_url(CORE_URL).is_ok());
        for url in [
            "https://api.lodgeit.com/core/clients/v1-preview/list/",
            "https://API.lodgeit.com/core/clients/v1-preview/list",
            "http://api.lodgeit.com/core/clients/v1-preview/list",
            "https://api.lodgeit.com/core/clients/v1-preview/list?x=1",
        ] {
            assert_eq!(config.check_url(url).unwrap_err().class, DISALLOWED_TARGET);
        }
    }

    #[test]
    fn invalid_inputs_never_open_transport() {
        let long = CoreContinuation {
            value: "x".repeat(8193),
        };
        let empty = CoreContinuation {
            value: String::new(),
        };
        for (token, size, continuation) in [
            (BEARER, 0, None),
            (BEARER, 501, None),
            (BEARER, 100, Some(&long)),
            (BEARER, 100, Some(&empty)),
            ("a b", 100, None),
            ("", 100, None),
        ] {
            let mut dial = recorder(Vec::new());
            let error = enabled()
                .list_clients_via(&mut dial, token, size, continuation)
                .unwrap_err();
            assert_eq!(
                error.category,
                CoreCategory::InvalidInput,
                "{token:?} {size}"
            );
            assert!(dial.dials.is_empty());
        }
    }

    #[test]
    fn bad_success_is_rejected_without_context() {
        let other = "1f8fad5b-d9cb-469f-a165-70867728950e";
        for body in [
            "[]".to_string(),
            "{}".to_string(),
            r#"{"items": [], "extra": 1}"#.to_string(),
            r#"{"items": [], "items": []}"#.to_string(),
            r#"{"items": [], "nextPageToken": ""}"#.to_string(),
            r#"{"items": [], "nextPageToken": 1.5}"#.to_string(),
            r#"{"items": [], "nextPageToken": "\ud800"}"#.to_string(),
            format!(r#"{{"items": [{}, {}]}}"#, item(""), item("")),
            format!(r#"{{"items": [{}]}}"#, item(r#", "code": 7"#)),
            format!(r#"{{"items": [{}]}}"#, item(r#", "groups": []"#)),
            format!(r#"{{"items": [{}]}}"#, item(r#", "other": null"#)),
            format!(
                r#"{{"items": [{}]}}"#,
                item("").replace(ID, &ID.to_uppercase())
            ),
            format!(r#"{{"items": [{}]}}"#, item("").replace("false", "0")),
            format!(
                r#"{{"items": [{}]}}"#,
                item("").replace(", \"profile\": null", "")
            ),
            format!(
                r#"{{"items": [{}]}}"#,
                item(&format!(r#", "code": "{}""#, "c".repeat(1025)))
            ),
            "{\"items\": [], \"nextPageToken\": NaN}".to_string(),
        ] {
            let error = list(vec![json_reply("200 OK", &body)]).unwrap_err();
            assert_eq!(error.category, CoreCategory::Protocol, "{body}");
            assert_eq!(error.http_status, Some(200));
        }
        let page = list(vec![json_reply(
            "200 OK",
            &format!(
                r#"{{"items": [{}, {}]}}"#,
                item(r#", "code": "", "assignments": null"#),
                item("").replace(ID, other)
            ),
        )])
        .unwrap();
        assert_eq!(page.items[0].code.as_deref(), Some(""));
        assert_eq!(page.continuation, None);
    }

    #[test]
    fn http_error_bodies_are_discarded_and_never_retried() {
        for (status, category) in [
            ("401 X", CoreCategory::Authentication),
            ("403 X", CoreCategory::Authorisation),
            ("400 X", CoreCategory::RequestRejected),
            ("422 X", CoreCategory::RequestRejected),
            ("429 X", CoreCategory::Throttled),
            ("500 X", CoreCategory::Upstream),
            ("503 X", CoreCategory::Upstream),
            ("404 X", CoreCategory::Protocol),
            ("204 X", CoreCategory::Protocol),
        ] {
            let ok = json_reply("200 OK", r#"{"items": []}"#);
            let error = list(vec![json_reply(status, "private-provider-canary"), ok]).unwrap_err();
            assert_eq!(error.category, category, "{status}");
            assert_eq!(error.to_string(), category.as_str());
            assert_eq!(error.http_status, Some(status[..3].parse().unwrap()));
        }
        let error = list(vec![]).unwrap_err();
        assert_eq!(
            (error.category, error.http_status),
            (CoreCategory::Transport, None)
        );
    }

    #[test]
    fn response_ceiling_and_media_type_are_enforced() {
        let body = r#"{"items": []}"#;
        for headers in [
            "Content-Type: text/plain\r\n",
            "",
            "Content-Type: application/jsonx\r\n",
        ] {
            let error = list(vec![reply("200 OK", headers, body)]).unwrap_err();
            assert_eq!(error.category, CoreCategory::Protocol, "{headers}");
        }
        let page = list(vec![reply(
            "200 OK",
            "Content-Type:  Application/JSON ; charset=utf-8\r\n",
            body,
        )]);
        assert!(page.is_ok());
        let small = CoreClient {
            config: CoreConfig::new(true, 20.0, 4).unwrap(),
        };
        let error = small
            .list_clients_via(
                &mut Replies(vec![json_reply("200 OK", body)]),
                BEARER,
                1,
                None,
            )
            .unwrap_err();
        assert_eq!(error.category, CoreCategory::Transport);
    }

    #[test]
    fn synthetic_maximum_page_fits() {
        let items: Vec<String> = (0..500)
            .map(|i| item("").replace(&ID[..8], &format!("{i:08x}")))
            .collect();
        let body = format!(r#"{{"items": [{}]}}"#, items.join(", "));
        let page = enabled()
            .list_clients_via(
                &mut Replies(vec![json_reply("200 OK", &body)]),
                BEARER,
                500,
                None,
            )
            .unwrap();
        assert_eq!(page.items.len(), 500);
        let error = enabled()
            .list_clients_via(
                &mut Replies(vec![json_reply("200 OK", &body)]),
                BEARER,
                499,
                None,
            )
            .unwrap_err();
        assert_eq!(error.category, CoreCategory::Protocol);
    }

    #[test]
    fn config_safety_limits() {
        for (timeout, limit) in [
            (0.0, MIB),
            (-1.0, MIB),
            (60.000_001, MIB),
            (f64::NAN, MIB),
            (f64::INFINITY, MIB),
            (1e300, MIB),
            (20.0, 0),
            (20.0, MIB + 1),
        ] {
            let error = CoreConfig::new(true, timeout, limit).unwrap_err();
            assert_eq!(error.class, CONFIGURATION);
        }
        assert!(CoreConfig::new(false, 60.0, 1).is_ok());
    }
}
