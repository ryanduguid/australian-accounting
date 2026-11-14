// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Stateless Streamable HTTP for the native server's 6 hosted tools. Everything
//! but the Workers glue in `worker.rs` is plain Rust, tested natively.

use aus_accounting_mcp_core::contracts;
use base64::Engine;
use serde_json::{Value, json};

pub const PATH: &str = "/mcp";
pub const BODY_LIMIT: usize = 16 * 1024;
pub const RELEASE: &str = concat!("aus-accounting-connector/v", env!("CARGO_PKG_VERSION"));
pub const COMMIT: &str = env!("CONNECTOR_COMMIT");
/// The first day every tool call is refused (build.rs).
pub const SERVE_UNTIL: &str = env!("CONNECTOR_SERVE_UNTIL");
pub const MODERN: &str = "2026-07-28";
const LEGACY: [&str; 2] = ["2025-11-25", "2025-06-18"];
const VERSION_KEY: &str = "io.modelcontextprotocol/protocolVersion";
const CAPABILITIES_KEY: &str = "io.modelcontextprotocol/clientCapabilities";
const ORIGINS: [&str; 4] = [
    "https://claude.ai",
    "https://claude.com",
    "https://chatgpt.com",
    "https://duguid.com.au",
];
const LOCAL: &str =
    "https://github.com/ryanduguid/australian-accounting/tree/main/apps/aus-accounting-mcp";
const TTL_MS: u64 = 3_600_000;

pub struct Reply {
    pub status: u16,
    pub headers: Vec<(&'static str, String)>,
    pub body: Option<String>,
}

impl Reply {
    fn new(status: u16, body: Option<String>) -> Self {
        let mut headers = vec![
            ("cache-control", "no-store".to_owned()),
            ("x-content-type-options", "nosniff".to_owned()),
            ("vary", "Origin".to_owned()),
            ("x-connector-release", release()),
        ];
        if body.is_some() {
            headers.push(("content-type", "application/json".to_owned()));
        }
        Self {
            status,
            headers,
            body,
        }
    }
    fn json(status: u16, value: &Value) -> Self {
        Self::new(status, Some(value.to_string()))
    }
    fn error(status: u16, id: &Value, code: i64, message: &str, data: Option<Value>) -> Self {
        let mut error = json!({"code":code,"message":message});
        if let Some(data) = data {
            error["data"] = data;
        }
        Self::json(status, &json!({"jsonrpc":"2.0","id":id,"error":error}))
    }
    fn header(mut self, name: &'static str, value: &str) -> Self {
        self.headers.push((name, value.to_owned()));
        self
    }
}

/// The UTC date of a Unix time in milliseconds, as YYYY-MM-DD.
pub fn today(millis: u64) -> String {
    chrono::DateTime::from_timestamp_millis(millis as i64)
        .expect("the platform clock is a valid time")
        .format("%Y-%m-%d")
        .to_string()
}

pub fn release() -> String {
    format!("{RELEASE}; commit={COMMIT}; serve-until={SERVE_UNTIL}")
}

fn server_info() -> Value {
    json!({"name":"aus-accounting-connector","title":"Aus Accounting MCP hosted calculators",
        "version":format!("{}+{COMMIT}", env!("CARGO_PKG_VERSION"))})
}

/// The `_meta` of every 2026-07-28 result, which names the server producing it.
fn meta() -> Value {
    json!({"io.modelcontextprotocol/serverInfo":server_info()})
}

/// The `fields` of `value` that it has, in that order.
fn ordered(value: &Value, fields: &[&str]) -> Value {
    let present = fields
        .iter()
        .filter_map(|field| Some(((*field).to_owned(), value.get(*field)?.clone())));
    Value::Object(present.collect())
}

/// The catalogue's tools, each with its fields in the order the native server
/// serialises them.
fn tools() -> Value {
    const TOOL: [&str; 6] = [
        "name",
        "title",
        "description",
        "inputSchema",
        "outputSchema",
        "annotations",
    ];
    const ANNOTATIONS: [&str; 5] = [
        "title",
        "readOnlyHint",
        "destructiveHint",
        "idempotentHint",
        "openWorldHint",
    ];
    let catalogue = contracts::CATALOGUE["tools"]
        .as_array()
        .into_iter()
        .flatten();
    catalogue
        .map(|tool| {
            let mut entry = ordered(tool, &TOOL);
            if let Some(annotations) = tool.get("annotations") {
                entry["annotations"] = ordered(annotations, &ANNOTATIONS);
            }
            entry
        })
        .collect()
}

/// Header lookup by case-insensitive name, as the platform's `Headers` gives it.
pub trait Headers {
    fn get(&self, name: &str) -> Option<String>;
}

/// The checks made before the body is read. `Err` is the whole reply.
pub fn admit(method: &str, path: &str, headers: &impl Headers) -> Result<(), Reply> {
    if path != PATH {
        return Err(Reply::new(404, None));
    }
    let origin = headers.get("origin");
    let allowed = origin.as_deref().is_none_or(|o| ORIGINS.contains(&o));
    if method == "OPTIONS" {
        return Err(match origin {
            Some(origin) if allowed => Reply::new(204, None)
                .header("access-control-allow-origin", &origin)
                .header("access-control-allow-methods", "POST")
                .header(
                    "access-control-allow-headers",
                    "Content-Type, Accept, MCP-Protocol-Version, Mcp-Method, Mcp-Name",
                ),
            _ => Reply::new(403, None),
        });
    }
    if method != "POST" {
        return Err(Reply::new(405, None).header("allow", "POST"));
    }
    if !allowed {
        return Err(Reply::new(403, None));
    }
    let length = headers.get("content-length");
    if length.is_some_and(|n| n.trim().parse::<u64>().is_ok_and(|n| n > BODY_LIMIT as u64)) {
        return Err(Reply::new(413, None));
    }
    let media = headers.get("content-type").unwrap_or_default();
    let media = media.split(';').next().unwrap_or_default().trim();
    if !media.eq_ignore_ascii_case("application/json") {
        return Err(Reply::new(415, None));
    }
    Ok(())
}

/// The reply to an admitted body of at most `BODY_LIMIT` bytes, on `today`
/// (UTC, YYYY-MM-DD).
pub fn respond(body: &[u8], headers: &impl Headers, today: &str) -> Reply {
    let reply = rpc(body, headers, today);
    match headers.get("origin") {
        Some(origin) if reply.status != 403 => reply.header("access-control-allow-origin", &origin),
        _ => reply,
    }
}

fn rpc(body: &[u8], headers: &impl Headers, today: &str) -> Reply {
    let null = Value::Null;
    let Ok(message) = serde_json::from_slice::<Value>(body) else {
        return Reply::error(400, &null, -32700, "Parse error", None);
    };
    let Some(object) = message.as_object() else {
        let text = if message.is_array() {
            "Batch requests are not supported"
        } else {
            "Invalid Request"
        };
        return Reply::error(400, &null, -32600, text, None);
    };
    let id = object.get("id");
    let (Some("2.0"), Some(method)) = (
        object.get("jsonrpc").and_then(Value::as_str),
        object.get("method").and_then(Value::as_str),
    ) else {
        return Reply::error(400, id.unwrap_or(&null), -32600, "Invalid Request", None);
    };
    let Some(id) = id else {
        // A notification, which this stateless server accepts and ignores.
        return Reply::new(202, None);
    };
    if !(id.is_string() || id.is_number()) {
        return Reply::error(400, &null, -32600, "Invalid Request", None);
    }
    let params = object.get("params").unwrap_or(&null);
    if !(params.is_object() || params.is_null()) {
        return Reply::error(400, id, -32600, "Invalid Request", None);
    }
    let header = headers.get("mcp-protocol-version");
    match params["_meta"][VERSION_KEY].as_str() {
        Some(version) => modern(id, method, params, version, header, headers, today),
        None => legacy(id, method, params, header.as_deref(), today),
    }
}

fn mismatch(id: &Value, name: &str, header: Option<&str>, body: &str) -> Reply {
    let message = match header {
        Some(value) => {
            format!(
                "Header mismatch: {name} header value '{value}' does not match body value '{body}'"
            )
        }
        None => format!("Header mismatch: the {name} header is missing"),
    };
    Reply::error(400, id, -32020, &message, None)
}

fn unsupported(id: &Value, requested: &str) -> Reply {
    let mut supported = vec![MODERN];
    supported.extend(LEGACY);
    Reply::error(
        400,
        id,
        -32022,
        "Unsupported protocol version",
        Some(json!({"supported":supported,"requested":requested})),
    )
}

/// An `Mcp-Name` value, decoded from the `=?base64?...?=` sentinel form.
fn decoded(value: &str) -> Option<String> {
    match value
        .strip_prefix("=?base64?")
        .and_then(|rest| rest.strip_suffix("?="))
    {
        Some(encoded) => base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok()),
        None => Some(value.to_owned()),
    }
}

fn modern(
    id: &Value,
    method: &str,
    params: &Value,
    version: &str,
    header: Option<String>,
    headers: &impl Headers,
    today: &str,
) -> Reply {
    if header.as_deref() != Some(version) {
        return mismatch(id, "MCP-Protocol-Version", header.as_deref(), version);
    }
    if version != MODERN {
        return unsupported(id, version);
    }
    if params["_meta"].get(CAPABILITIES_KEY).is_none() {
        return Reply::error(
            400,
            id,
            -32602,
            &format!("params._meta is missing the required envelope key(s): {CAPABILITIES_KEY}"),
            None,
        );
    }
    let sent = headers.get("mcp-method");
    if sent.as_deref() != Some(method) {
        return mismatch(id, "Mcp-Method", sent.as_deref(), method);
    }
    let result = match method {
        "server/discover" => json!({"resultType":"complete","supportedVersions":[MODERN],
            "capabilities":{"tools":{}},"_meta":meta(),"ttlMs":TTL_MS,"cacheScope":"public"}),
        "tools/list" => json!({"resultType":"complete","_meta":meta(),"ttlMs":TTL_MS,
            "cacheScope":"public","tools":tools()}),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            let sent = headers.get("mcp-name");
            if sent.as_deref().and_then(decoded).as_deref() != Some(name) {
                return mismatch(id, "Mcp-Name", sent.as_deref(), name);
            }
            return call(id, params, today, true);
        }
        _ => {
            return Reply::error(404, id, -32601, "Method not found", Some(json!(method)));
        }
    };
    Reply::json(200, &json!({"jsonrpc":"2.0","id":id,"result":result}))
}

/// The `initialize` era (2025-06-18 and 2025-11-25), without sessions.
fn legacy(id: &Value, method: &str, params: &Value, header: Option<&str>, today: &str) -> Reply {
    if method == "initialize" {
        let requested = params["protocolVersion"].as_str().unwrap_or_default();
        let version = LEGACY
            .iter()
            .find(|v| **v == requested)
            .unwrap_or(&LEGACY[0]);
        let result = json!({"protocolVersion":version,"capabilities":{"tools":{"listChanged":false}},
            "serverInfo":server_info()});
        return Reply::json(200, &json!({"jsonrpc":"2.0","id":id,"result":result}));
    }
    match header {
        Some(version) if LEGACY.contains(&version) => {}
        Some(MODERN) => return mismatch(id, "MCP-Protocol-Version", Some(MODERN), "absent"),
        // A request without the header is 2025-03-26, which is not served.
        other => return unsupported(id, other.unwrap_or("2025-03-26")),
    }
    let result = match method {
        "ping" => json!({}),
        "tools/list" => json!({"tools":tools()}),
        "tools/call" => return call(id, params, today, false),
        // Before 2026-07-28 a 404 tells the client its session has ended.
        _ => return Reply::error(200, id, -32601, "Method not found", Some(json!(method))),
    };
    Reply::json(200, &json!({"jsonrpc":"2.0","id":id,"result":result}))
}

/// A `tools/call` result: the core's, inside `resultType` and `_meta` when `modern`.
fn call(id: &Value, params: &Value, today: &str, modern: bool) -> Reply {
    let name = params["name"].as_str().unwrap_or_default();
    if contracts::tool(name).is_none() {
        let message = format!(
            "{name:?} is not one of the hosted connector's tools. Run the local Aus Accounting MCP server for it: {LOCAL}"
        );
        return Reply::error(200, id, -32602, &message, None);
    }
    let result = if today >= SERVE_UNTIL {
        contracts::error(format!(
            "This hosted connector release stopped answering tool calls on {SERVE_UNTIL}, when the tables it was built with could be out of date. Run the local Aus Accounting MCP server instead: {LOCAL}"
        ))
    } else {
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
        aus_accounting_mcp_core::call(name, &arguments)
    };
    // The core's result ends with `resultType`, which the native server sends
    // first in 2026-07-28 and not at all before it.
    let Value::Object(mut fields) = result else {
        unreachable!("the core's results are objects")
    };
    fields.shift_remove("resultType");
    let result = if modern {
        let mut wrapped =
            serde_json::Map::from_iter([("resultType".to_owned(), json!("complete"))]);
        wrapped.extend(fields);
        wrapped.insert("_meta".to_owned(), meta());
        wrapped
    } else {
        fields
    };
    Reply::json(200, &json!({"jsonrpc":"2.0","id":id,"result":result}))
}

#[cfg(target_arch = "wasm32")]
mod worker;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Map(HashMap<&'static str, String>);
    impl Headers for Map {
        fn get(&self, name: &str) -> Option<String> {
            self.0.get(name).cloned()
        }
    }

    fn call_on(today: &str, name: &str, mcp_name: &str) -> Value {
        let body = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,
            "arguments":{},"_meta":{VERSION_KEY:MODERN,CAPABILITIES_KEY:{}}}});
        let headers = Map(HashMap::from([
            ("mcp-protocol-version", MODERN.to_owned()),
            ("mcp-method", "tools/call".to_owned()),
            ("mcp-name", mcp_name.to_owned()),
        ]));
        let reply = respond(body.to_string().as_bytes(), &headers, today);
        serde_json::from_str(&reply.body.unwrap()).unwrap()
    }

    #[test]
    fn the_request_date_is_the_utc_calendar_day() {
        assert_eq!(today(0), "1970-01-01");
        // 2027-06-29T23:59:59.999Z, then the next millisecond.
        assert_eq!(today(1_814_313_599_999), "2027-06-29");
        assert_eq!(today(1_814_313_600_000), "2027-06-30");
    }

    #[test]
    fn every_tool_call_is_refused_from_serve_until() {
        let day = chrono::NaiveDate::parse_from_str(SERVE_UNTIL, "%Y-%m-%d").unwrap();
        let before = day.pred_opt().unwrap().to_string();
        for tool in contracts::CATALOGUE["tools"].as_array().unwrap() {
            let name = tool["name"].as_str().unwrap();
            let served = call_on(&before, name, name);
            assert!(
                !served["result"]["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains("stopped answering"),
                "{name}"
            );
            for today in [SERVE_UNTIL.to_owned(), "2999-01-01".to_owned()] {
                let refused = call_on(&today, name, name);
                assert_eq!(refused["result"]["isError"], true, "{name}");
                assert!(
                    refused["result"]["content"][0]["text"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("stopped answering tool calls on {SERVE_UNTIL}")),
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn an_encoded_mcp_name_is_decoded_before_it_is_compared() {
        // base64 of "refuse_div7a".
        let reply = call_on("2000-01-01", "refuse_div7a", "=?base64?cmVmdXNlX2Rpdjdh?=");
        assert_eq!(
            reply["result"]["structuredContent"]["code"],
            "ERR_POLICY_DIV7A_SCOPE_REFUSED"
        );
        for wrong in ["=?base64?cmVmdXNl?=", "=?base64?!!?=", "refuse_div7"] {
            assert_eq!(
                call_on("2000-01-01", "refuse_div7a", wrong)["error"]["code"],
                -32020,
                "{wrong}"
            );
        }
    }
}
