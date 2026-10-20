//! One command for the Rust engines in this repository. `au-accounting <engine> ...`
//! hands the remaining arguments to that engine's library exactly as the engine's
//! own command does: JSON stdin commands, `--api-help` and the engine's CLI.

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::ffi::OsString;
use std::io::{Read, Write};

/// (exit code, stdout, stderr), as every engine's CLI returns them.
type Outcome = (i32, String, String);

struct Engine {
    name: &'static str,
    /// The engine's standalone command, also accepted as the subcommand.
    command: &'static str,
    summary: &'static str,
    run: fn(&[OsString]) -> Outcome,
    /// The engine's own stream encoding (`true` for stderr).
    encode: fn(&str, bool) -> Vec<u8>,
    /// Exit 120 with Python's message when a stream cannot be written, on Unix,
    /// as the engine's own command does; otherwise exit 1.
    python_flush: bool,
}

const ENGINES: &[Engine] = &[
    Engine {
        name: "benchmarks",
        command: "ato-benchmark-compare",
        summary: "P&L figures against the ATO small business benchmarks",
        run: benchmarks,
        encode: |text, _| ato_benchmark_compare::cli::output_bytes(text),
        python_flush: true,
    },
    Engine {
        name: "div7a",
        command: "div7a-loan-review",
        summary: "Division 7A complying loans and minimum yearly repayments",
        run: div7a,
        encode: |text, _| div7a_loan_review::cli::output_bytes(text),
        python_flush: true,
    },
    Engine {
        name: "exchequer",
        command: "the-exchequer-tally",
        summary: "Corporate tax, franking ledger and Division 203 review aids",
        run: exchequer,
        encode: the_exchequer_tally::cli::output_bytes,
        python_flush: false,
    },
    Engine {
        name: "payday-super",
        command: "payday-super-check",
        summary: "Payday super deadlines and SG charge estimates",
        run: payday_super,
        encode: |text, _| payday_super_checker::cli::output_bytes(text),
        python_flush: true,
    },
    Engine {
        name: "solomons",
        command: "solomons-sword",
        summary: "Trust allocation, s 100A risk zoning and s 99B review aids",
        run: solomons,
        encode: |text, _| solomons_sword::cli::output_bytes(text),
        python_flush: true,
    },
    Engine {
        name: "tax",
        command: "australian-tax-calculators",
        summary: "Bounded Australian tax calculation worksheets",
        run: tax,
        encode: |text, _| {
            if cfg!(windows) {
                text.replace('\n', "\r\n").into_bytes()
            } else {
                text.as_bytes().to_vec()
            }
        },
        python_flush: false,
    },
    Engine {
        name: "wip",
        command: "wip-tally",
        summary: "AASB 15 construction WIP schedule and review pack",
        run: wip,
        encode: |text, _| the_wip_tally::cli::output_bytes(text),
        python_flush: true,
    },
];

fn benchmarks(args: &[OsString]) -> Outcome {
    use ato_benchmark_compare::{api, cli};
    let args: Vec<String> = args.iter().map(|a| cli::decode_os(a)).collect();
    dispatch(
        &args,
        &api::COMMANDS,
        api::evaluate,
        FULL_JSON,
        "See README.md for their schemas.",
        cli::run,
    )
}

fn div7a(args: &[OsString]) -> Outcome {
    use div7a_loan_review::{api, cli};
    let args: Vec<String> = args.iter().map(|a| cli::decode_os(a)).collect();
    dispatch(
        &args,
        &api::COMMANDS,
        api::evaluate,
        FULL_JSON,
        "See README.md for their schemas.",
        cli::run,
    )
}

fn exchequer(args: &[OsString]) -> Outcome {
    use the_exchequer_tally::{api, cli};
    let args: Vec<String> = args.iter().map(|a| cli::decode_os(a)).collect();
    let note = "Money, rates and percentages are decimal strings. See README.md for schemas.";
    let json = Json {
        preserve_order: false,
        arbitrary_precision: false,
    };
    dispatch(&args, api::COMMANDS, api::evaluate, json, note, cli::run)
}

fn payday_super(args: &[OsString]) -> Outcome {
    use payday_super_checker::{api, cli};
    let args: Vec<String> = args.iter().map(|a| cli::decode_os(a)).collect();
    dispatch(
        &args,
        &api::COMMANDS,
        api::evaluate,
        FULL_JSON,
        "See README.md for their schemas.",
        cli::run,
    )
}

fn solomons(args: &[OsString]) -> Outcome {
    use solomons_sword::{api, cli};
    let args: Vec<String> = args.iter().map(|a| cli::decode_os(a)).collect();
    let note =
        "Amounts are decimal strings and facts are true, false or null. See README.md for schemas.";
    let json = Json {
        preserve_order: false,
        arbitrary_precision: false,
    };
    dispatch(&args, api::COMMANDS, api::evaluate, json, note, cli::run)
}

fn tax(args: &[OsString]) -> Outcome {
    use australian_tax_calculators::api;
    let args: Vec<String> = args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let commands = api::commands();
    // The engine has no CLI beyond its JSON commands; this is its own usage error.
    let usage = |_: &[String]| -> Outcome {
        let commands = api::commands().join(", ");
        let text = format!(
            "usage: australian-tax-calculators COMMAND < input.json\ncommands: {commands}\n"
        );
        (2, String::new(), text)
    };
    let note = "Amounts are decimal strings. See README.md for schemas.";
    dispatch(&args, &commands, api::evaluate, FULL_JSON, note, usage)
}

fn wip(args: &[OsString]) -> Outcome {
    use the_wip_tally::{api, cli};
    let args: Vec<String> = args.iter().map(|a| cli::decode_os(a)).collect();
    let note = "Amounts are decimal strings. See README.md for schemas.";
    let json = Json {
        preserve_order: false,
        arbitrary_precision: true,
    };
    dispatch(&args, api::COMMANDS, api::evaluate, json, note, cli::run)
}

/// The serde_json features an engine enables when it is built on its own. This
/// binary builds every engine against one serde_json, with the `preserve_order`
/// and `arbitrary_precision` features some engines enable, so the JSON boundary
/// gives the other engines what their own command would: keys in sorted order,
/// and numbers as the default parser reads them.
#[derive(Clone, Copy)]
struct Json {
    preserve_order: bool,
    arbitrary_precision: bool,
}

const FULL_JSON: Json = Json {
    preserve_order: true,
    arbitrary_precision: true,
};

/// The routing every engine's own `main` applies: one JSON stdin command name,
/// `--api-help`, or anything else to the engine's CLI.
fn dispatch(
    args: &[String],
    commands: &[&str],
    evaluate: fn(&str, &Value) -> Result<Value, String>,
    json: Json,
    note: &str,
    cli: impl Fn(&[String]) -> Outcome,
) -> Outcome {
    match args {
        [command] if commands.contains(&command.as_str()) => api(command, evaluate, json),
        [flag] if flag == "--api-help" => (
            0,
            format!("JSON stdin commands: {}\n{note}\n", commands.join(", ")),
            String::new(),
        ),
        _ => cli(args),
    }
}

/// One JSON stdin command, bounded at 8 MiB as each engine bounds it.
fn api(command: &str, evaluate: fn(&str, &Value) -> Result<Value, String>, json: Json) -> Outcome {
    let mut bytes = Vec::new();
    if let Err(e) = std::io::stdin().take((8 << 20) + 1).read_to_end(&mut bytes) {
        return (2, String::new(), format!("error: {e}\n"));
    }
    let result = if bytes.len() > 8 << 20 {
        Err("JSON input exceeds 8 MiB".into())
    } else {
        parse_json(&bytes, json)
            .map_err(|e| format!("invalid JSON: {e}"))
            .and_then(|v| evaluate(command, &v))
            .and_then(|v| adapt(v, json))
    };
    match result {
        Ok(v) => (0, format!("{v}\n"), String::new()),
        Err(m) => (2, String::new(), format!("error: {m}\n")),
    }
}

/// Preserve actual object types when an engine does not enable arbitrary
/// precision. serde_json otherwise treats a literal object whose first key is
/// `$serde_json::private::Number` as its internal number representation.
fn parse_json(bytes: &[u8], json: Json) -> Result<Value, String> {
    let value = if json.arbitrary_precision {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())?
    } else {
        let tokens = json_tokens(bytes);
        let mut tokens = tokens.into_iter();
        let mut parser = serde_json::Deserializer::from_slice(bytes);
        let value = PlainValue {
            tokens: &mut tokens,
        }
        .deserialize(&mut parser)
        .map_err(|e| e.to_string())?;
        parser.end().map_err(|e| e.to_string())?;
        value
    };
    adapt(value, json)
}

#[derive(Debug, PartialEq)]
enum Token {
    Object,
    Array,
    Number,
}

/// Only the token kinds that serde's visitor cannot distinguish. Strings,
/// including object keys and escaped quotes, contribute no tokens. Serde still
/// owns all syntax, UTF-8, number and recursion validation.
fn json_tokens(bytes: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'{' => tokens.push(Token::Object),
            b'[' => tokens.push(Token::Array),
            b'-' | b'0'..=b'9' => {
                tokens.push(Token::Number);
                while i + 1 < bytes.len()
                    && matches!(bytes[i + 1], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    tokens
}

struct PlainValue<'a> {
    tokens: &'a mut std::vec::IntoIter<Token>,
}

impl<'de> DeserializeSeed<'de> for PlainValue<'_> {
    type Value = Value;

    fn deserialize<D: de::Deserializer<'de>>(self, parser: D) -> Result<Value, D::Error> {
        parser.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for PlainValue<'_> {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("any valid JSON value")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        self.number(value.into())
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        self.number(value.into())
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        self.number(
            serde_json::Number::from_f64(value).ok_or_else(|| E::custom("number out of range"))?,
        )
    }

    fn visit_seq<A: SeqAccess<'de>>(mut self, mut seq: A) -> Result<Value, A::Error> {
        self.expect(Token::Array)?;
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(PlainValue {
            tokens: self.tokens,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        match self.tokens.next() {
            Some(Token::Number) => {
                let number =
                    serde_json::Number::deserialize(de::value::MapAccessDeserializer::new(map))?;
                plain_number(&number)
                    .map(Value::Number)
                    .map_err(de::Error::custom)
            }
            Some(Token::Object) => {
                let mut values = serde_json::Map::new();
                // Validate every value before insertion, including overwritten
                // duplicate keys, as the standalone parser does.
                while let Some(key) = map.next_key::<String>()? {
                    let value = map.next_value_seed(PlainValue {
                        tokens: self.tokens,
                    })?;
                    values.insert(key, value);
                }
                Ok(Value::Object(values))
            }
            _ => Err(de::Error::custom("unexpected JSON token")),
        }
    }
}

impl PlainValue<'_> {
    fn expect<E: de::Error>(&mut self, expected: Token) -> Result<(), E> {
        if self.tokens.next() == Some(expected) {
            Ok(())
        } else {
            Err(E::custom("unexpected JSON token"))
        }
    }

    fn number<E: de::Error>(mut self, number: serde_json::Number) -> Result<Value, E> {
        self.expect(Token::Number)?;
        Ok(Value::Number(number))
    }
}

/// `value` as an engine built with `json`'s features would hold it.
fn adapt(value: Value, json: Json) -> Result<Value, String> {
    Ok(match value {
        Value::Number(n) if !json.arbitrary_precision => Value::Number(plain_number(&n)?),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|v| adapt(v, json))
                .collect::<Result<_, _>>()?,
        ),
        Value::Object(map) => {
            let mut entries = map
                .into_iter()
                .map(|(k, v)| adapt(v, json).map(|v| (k, v)))
                .collect::<Result<Vec<_>, _>>()?;
            if !json.preserve_order {
                entries.sort_by(|a, b| a.0.cmp(&b.0));
            }
            Value::Object(entries.into_iter().collect())
        }
        other => other,
    })
}

/// The number serde_json's default parser reads from `n`'s text: an integer
/// when it fits u64 or i64 (`-0` is a float), otherwise the nearest finite f64.
fn plain_number(n: &serde_json::Number) -> Result<serde_json::Number, String> {
    let text = n.to_string();
    if !text.bytes().any(|b| b".eE".contains(&b)) {
        if let Ok(u) = text.parse::<u64>() {
            return Ok(u.into());
        }
        match text.parse::<i64>() {
            Ok(i) if i != 0 => return Ok(i.into()),
            _ => {}
        }
    }
    text.parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
        .ok_or_else(|| "number out of range".into())
}

fn help() -> String {
    let mut text = String::from(
        "usage: au-accounting ENGINE [ARGS...]\n\n\
         Runs one of the Rust engines in ryanduguid/australian-accounting. ARGS are\n\
         the engine's own; `au-accounting ENGINE --help` shows its commands and\n\
         `au-accounting ENGINE --api-help` its JSON stdin commands.\n\nengines:\n",
    );
    for engine in ENGINES {
        text.push_str(&format!("  {:<13} {}\n", engine.name, engine.summary));
    }
    text.push_str("\nEach engine's own command name also works, for example div7a-loan-review.\n");
    text
}

fn main() {
    // argv[0] is skipped, and the arguments only select an engine and become its
    // input, as each engine's own main() takes them.
    let mut args = std::env::args_os().skip(1); // nosemgrep: rust.lang.security.args-os.args-os
    let first = args.next();
    let rest: Vec<OsString> = args.collect();
    let selected = first
        .as_ref()
        .and_then(|name| ENGINES.iter().find(|e| name == e.name || name == e.command));
    let Some(engine) = selected else {
        let (code, out, err) = match first.as_ref().map(|f| f.to_string_lossy()) {
            Some(f) if f == "-h" || f == "--help" => (0, help(), String::new()),
            Some(f) if f == "-V" || f == "--version" => (
                0,
                format!("au-accounting {}\n", env!("CARGO_PKG_VERSION")),
                String::new(),
            ),
            Some(f) => (
                2,
                String::new(),
                format!("au-accounting: unknown engine '{f}'\n\n{}", help()),
            ),
            None => (2, String::new(), help()),
        };
        print!("{out}");
        eprint!("{err}");
        std::process::exit(code);
    };
    let (code, out, err) = (engine.run)(&rest);
    let mut stdout = std::io::stdout().lock();
    let out_result = stdout
        .write_all(&(engine.encode)(&out, false))
        .and_then(|()| stdout.flush());
    let mut stderr = std::io::stderr().lock();
    let err_result = stderr
        .write_all(&(engine.encode)(&err, true))
        .and_then(|()| stderr.flush());
    std::process::exit(finish(engine.python_flush, code, out_result, err_result));
}

/// The exit status once both streams are written, as the engines' own commands
/// report it: Python reported a stream it could not flush at shutdown, on stderr,
/// and exited 120; elsewhere a write failure exits 1.
fn finish(
    python_flush: bool,
    code: i32,
    out: std::io::Result<()>,
    err: std::io::Result<()>,
) -> i32 {
    let python = python_flush && cfg!(unix);
    if let Err(e) = &out {
        if python {
            let _ = std::io::stderr().write_all(unflushed("stdout", e).as_bytes());
            return 120;
        }
        return 1;
    }
    if err.is_err() {
        return if python { 120 } else { 1 };
    }
    code
}

/// "Exception ignored while flushing sys.stdout:" and the OSError, as Python printed them.
fn unflushed(stream: &str, e: &std::io::Error) -> String {
    let code = e.raw_os_error().unwrap_or(0);
    let class = match e.kind() {
        std::io::ErrorKind::BrokenPipe => "BrokenPipeError",
        std::io::ErrorKind::ConnectionReset => "ConnectionResetError",
        std::io::ErrorKind::WouldBlock => "BlockingIOError",
        _ => "OSError",
    };
    let text = e.to_string();
    let text = text
        .rfind(" (os error ")
        .map_or(text.as_str(), |i| &text[..i]);
    format!("Exception ignored while flushing sys.{stream}:\n{class}: [Errno {code}] {text}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_JSON: Json = Json {
        preserve_order: false,
        arbitrary_precision: false,
    };

    fn adapted(text: &str, json: Json) -> Result<String, String> {
        parse_json(text.as_bytes(), json).map(|v| v.to_string())
    }

    #[test]
    fn numbers_read_as_the_default_parser_reads_them() {
        for (text, want) in [
            ("0", "0"),
            ("-0", "-0.0"),
            ("1.0", "1.0"),
            ("1e2", "100.0"),
            ("18446744073709551615", "18446744073709551615"),
            ("18446744073709551616", "1.8446744073709552e+19"),
            ("-9223372036854775808", "-9223372036854775808"),
            ("-9223372036854775809", "-9.223372036854776e+18"),
            ("1e-400", "0.0"),
        ] {
            assert_eq!(adapted(text, DEFAULT_JSON).as_deref(), Ok(want), "{text}");
        }
        assert!(
            adapted("[1e309]", DEFAULT_JSON)
                .unwrap_err()
                .starts_with("number out of range")
        );
        assert_eq!(adapted("1e309", FULL_JSON).as_deref(), Ok("1e+309"));
    }

    #[test]
    fn keys_sort_unless_the_engine_preserves_order() {
        let text = r#"{"b":{"d":1,"c":[{"f":2,"e":3}]},"a":4}"#;
        let sorted = r#"{"a":4,"b":{"c":[{"e":3,"f":2}],"d":1}}"#;
        assert_eq!(adapted(text, DEFAULT_JSON).as_deref(), Ok(sorted));
        assert_eq!(adapted(text, FULL_JSON).as_deref(), Ok(text));
    }

    #[test]
    fn reserved_number_keys_remain_objects_without_arbitrary_precision() {
        for text in [
            r#"{"$serde_json::private::Number":"2026"}"#,
            r#"{"financial_year":{"$serde_json::private::Number":"2026"}}"#,
            r#"[{"$serde_json::private::Number":42}]"#,
            r#"{"a":{"$serde_json::private::Number":"bad","z":1},"b":true}"#,
            r#"{"$serde_json::private::Number":null}"#,
            r#"{"$serde_json::private::Number":{"a":1}}"#,
        ] {
            let value = parse_json(text.as_bytes(), DEFAULT_JSON).unwrap();
            let result = value.to_string();
            assert!(result.contains("$serde_json::private::Number"), "{text}");
        }
        let input = parse_json(
            br#"{"financial_year":{"$serde_json::private::Number":"2026"}}"#,
            DEFAULT_JSON,
        )
        .unwrap();
        assert_eq!(
            the_exchequer_tally::api::evaluate("turnover-threshold", &input),
            Err("financial_year must be an integer".into())
        );
        // Engines which enable arbitrary_precision retain their own semantics.
        assert_eq!(
            adapted(r#"{"$serde_json::private::Number":"2026"}"#, FULL_JSON).as_deref(),
            Ok("2026")
        );
    }

    #[test]
    fn token_kinds_ignore_keys_strings_and_escapes() {
        let text = br#"{"12[{":"\\\"[123{","a":[null,true,false,-1.2e+30,{"n":2}]}"#;
        assert_eq!(
            json_tokens(text),
            vec![
                Token::Object,
                Token::Array,
                Token::Number,
                Token::Object,
                Token::Number
            ]
        );
        assert!(parse_json(text, DEFAULT_JSON).is_ok());
    }

    #[test]
    fn every_duplicate_value_is_validated_and_last_value_wins() {
        assert_eq!(
            adapted(r#"{"a":1,"a":2}"#, DEFAULT_JSON).as_deref(),
            Ok(r#"{"a":2}"#)
        );
        for text in [r#"{"a":1e400,"a":2}"#, r#"{"a":[1e400],"a":null}"#] {
            assert!(parse_json(text.as_bytes(), DEFAULT_JSON).is_err(), "{text}");
        }
    }

    #[test]
    fn serde_keeps_syntax_utf8_and_recursion_validation() {
        for text in [
            b"{\"a\":1,}".as_slice(),
            b"[1 2]",
            b"null false",
            b"\"\xff\"",
            br#"{"a":"\uD800"}"#,
        ] {
            let own = serde_json::from_slice::<Value>(text)
                .unwrap_err()
                .to_string();
            assert_eq!(parse_json(text, DEFAULT_JSON).unwrap_err(), own);
        }
        let text = format!("{}0{}", "[".repeat(127), "]".repeat(127));
        assert!(parse_json(text.as_bytes(), DEFAULT_JSON).is_ok());
        let text = format!("{}0{}", "[".repeat(128), "]".repeat(128));
        assert_eq!(
            parse_json(text.as_bytes(), DEFAULT_JSON).unwrap_err(),
            serde_json::from_str::<Value>(&text)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn every_engine_and_command_name_routes_to_one_engine() {
        let mut names: Vec<&str> = ENGINES.iter().flat_map(|e| [e.name, e.command]).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 2 * ENGINES.len());
        assert!(help().lines().all(|line| line.len() <= 80));
    }
}
