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
    let text = b"{\"12[{\":\"\\\\\\\"[123{\",\"a\":[null,true,false,-1.2e+30,{\"n\":2}]}";
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
