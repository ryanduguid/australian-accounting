// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `datetime.fromisoformat` against CPython 3.15: each row's expected value
//! is what CPython 3.15 gave for the same string (the date, clock, UTC offset
//! and `astimezone(timezone.utc).date()`, or the `ValueError` text).
use lodgeit_calculator_adapter::pydatetime::{date_fromisoformat, datetime_fromisoformat};

type Parsed = (
    &'static str,
    [i32; 4],
    Option<i64>,
    Option<Result<&'static str, &'static str>>,
);

/// What `parse` reads back: the owned form of `Parsed`.
type Owned = (
    String,
    [i32; 4],
    Option<i64>,
    Option<Result<String, String>>,
);

fn parse(points: &[u32]) -> Result<Owned, String> {
    let dt = datetime_fromisoformat(points)?;
    let utc = dt
        .offset_us
        .map(|_| dt.utc_date().map(|date| date.isoformat()));
    Ok((
        dt.date.isoformat(),
        [dt.hour, dt.minute, dt.second, dt.microsecond],
        dt.offset_us,
        utc,
    ))
}

#[test]
fn iso_strings_parse_as_cpython_does() {
    let rows: &[(&str, Result<Parsed, &str>)] = &[
        // BEGIN CPYTHON ROWS
        ("2026-10-24", Ok(("2026-10-24", [0, 0, 0, 0], None, None))),
        (
            "2026-10-24T01",
            Ok(("2026-10-24", [1, 0, 0, 0], None, None)),
        ),
        (
            "2026-10-24T0102",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        (
            "2026-10-24T01:02",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        (
            "2026-10-24T010203",
            Ok(("2026-10-24", [1, 2, 3, 0], None, None)),
        ),
        (
            "2026-10-24T01:02:03",
            Ok(("2026-10-24", [1, 2, 3, 0], None, None)),
        ),
        (
            "2026-10-24T01:02:03.4",
            Ok(("2026-10-24", [1, 2, 3, 400000], None, None)),
        ),
        (
            "2026-10-24T01:02:03.45678912",
            Ok(("2026-10-24", [1, 2, 3, 456789], None, None)),
        ),
        (
            "2026-10-24T01:02:03,5",
            Ok(("2026-10-24", [1, 2, 3, 500000], None, None)),
        ),
        (
            "2026-10-24T01:02:03.",
            Err("Invalid isoformat string: '2026-10-24T01:02:03.'"),
        ),
        (
            "2026-10-24T01:02:03.x",
            Err("Invalid isoformat string: '2026-10-24T01:02:03.x'"),
        ),
        (
            "2026-10-24T01:02:03Z",
            Ok(("2026-10-24", [1, 2, 3, 0], Some(0), Some(Ok("2026-10-24")))),
        ),
        (
            "2026-10-24T01:02:03z",
            Err("Invalid isoformat string: '2026-10-24T01:02:03z'"),
        ),
        (
            "2026-10-24T01:02:03-0530",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(-19800000000),
                Some(Ok("2026-10-24")),
            )),
        ),
        (
            "2026-10-24T01:02:03+05",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(18000000000),
                Some(Ok("2026-10-23")),
            )),
        ),
        (
            "2026-10-24T01:02:03+05:30:15",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(19815000000),
                Some(Ok("2026-10-23")),
            )),
        ),
        (
            "2026-10-24T01:02:03+05:30:15.123456",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(19815123456),
                Some(Ok("2026-10-23")),
            )),
        ),
        (
            "2026-10-24T01:02:03+05:30:15.1",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(19815100000),
                Some(Ok("2026-10-23")),
            )),
        ),
        (
            "2026-10-24T01:02:03+24:00",
            Err("hour must be in 0..23, not 24"),
        ),
        (
            "2026-10-24T01:02:03+23:60",
            Err("minute must be in 0..59, not 60"),
        ),
        (
            "2026-10-24T01:02:03+0x",
            Err("Invalid isoformat string: '2026-10-24T01:02:03+0x'"),
        ),
        (
            "2026-10-24T01:02:03+",
            Err("Invalid isoformat string: '2026-10-24T01:02:03+'"),
        ),
        ("2026-10-24T01:60", Err("minute must be in 0..59, not 60")),
        (
            "2026-10-24T01:02:60",
            Err("second must be in 0..59, not 60"),
        ),
        ("2026-10-24T25:00", Err("hour must be in 0..23, not 25")),
        (
            "2026-10-24T24:00",
            Ok(("2026-10-25", [0, 0, 0, 0], None, None)),
        ),
        (
            "2026-10-24T24:00:00.000001",
            Err("minute, second, and microsecond must be 0 when hour is 24"),
        ),
        (
            "2026-10-24T01:0",
            Err("Invalid isoformat string: '2026-10-24T01:0'"),
        ),
        (
            "2026-10-24T1",
            Err("Invalid isoformat string: '2026-10-24T1'"),
        ),
        (
            "2026-10-24T01:02:",
            Err("Invalid isoformat string: '2026-10-24T01:02:'"),
        ),
        (
            "2026-10-24T01:02:0",
            Err("Invalid isoformat string: '2026-10-24T01:02:0'"),
        ),
        (
            "2026-10-24 01:02",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        (
            "2026-10-24x01:02",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        (
            "2026-10-24\u{e9}01:02",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        (
            "2026-10-24T",
            Err("Invalid isoformat string: '2026-10-24T'"),
        ),
        (
            "20261024T010203",
            Ok(("2026-10-24", [1, 2, 3, 0], None, None)),
        ),
        (
            "20261024T01:02",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        ("2026W011", Ok(("2025-12-29", [0, 0, 0, 0], None, None))),
        ("2026W01", Ok(("2025-12-29", [0, 0, 0, 0], None, None))),
        ("2026-W01", Ok(("2025-12-29", [0, 0, 0, 0], None, None))),
        ("2026-W53-1", Ok(("2026-12-28", [0, 0, 0, 0], None, None))),
        ("2020-W53-7", Ok(("2021-01-03", [0, 0, 0, 0], None, None))),
        ("2026-W54-1", Err("Invalid isoformat string: '2026-W54-1'")),
        ("2026-W00-1", Err("Invalid isoformat string: '2026-W00-1'")),
        ("2026-W01-8", Err("Invalid isoformat string: '2026-W01-8'")),
        ("2026-W01-0", Err("Invalid isoformat string: '2026-W01-0'")),
        (
            "2026-W01-1T10:00",
            Ok(("2025-12-29", [10, 0, 0, 0], None, None)),
        ),
        (
            "2026-W011T10",
            Err("Invalid isoformat string: '2026-W011T10'"),
        ),
        ("2026-13-01", Err("month must be in 1..12, not 13")),
        ("2026-00-01", Err("month must be in 1..12, not 0")),
        (
            "2026-04-31",
            Err("day 31 must be in range 1..30 for month 4 in year 2026"),
        ),
        ("2024-02-29", Ok(("2024-02-29", [0, 0, 0, 0], None, None))),
        (
            "2023-02-29",
            Err("day 29 must be in range 1..28 for month 2 in year 2023"),
        ),
        ("0000-01-01", Err("year must be in 1..9999, not 0")),
        ("2026-1-01", Err("Invalid isoformat string: '2026-1-01'")),
        ("2026-10-2", Err("Invalid isoformat string: '2026-10-2'")),
        ("2026", Err("Invalid isoformat string: '2026'")),
        ("", Err("Invalid isoformat string: ''")),
        ("x", Err("Invalid isoformat string: 'x'")),
        (
            "2026-10-24T01:02:03.123456+00:00",
            Ok((
                "2026-10-24",
                [1, 2, 3, 123456],
                Some(0),
                Some(Ok("2026-10-24")),
            )),
        ),
        (
            "2026-10-24T01:02:03-23:59",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(-86340000000),
                Some(Ok("2026-10-25")),
            )),
        ),
        (
            "0001-01-01T00:00-01:00",
            Ok((
                "0001-01-01",
                [0, 0, 0, 0],
                Some(-3600000000),
                Some(Ok("0001-01-01")),
            )),
        ),
        (
            "9999-12-31T23:59+00:00",
            Ok((
                "9999-12-31",
                [23, 59, 0, 0],
                Some(0),
                Some(Ok("9999-12-31")),
            )),
        ),
        (
            "9999-12-31T23:00-01:00",
            Ok((
                "9999-12-31",
                [23, 0, 0, 0],
                Some(-3600000000),
                Some(Err("date value out of range")),
            )),
        ),
        (
            "2026-10-24T01:02:03+00:00 ",
            Err("Invalid isoformat string: '2026-10-24T01:02:03+00:00 '"),
        ),
        (
            "2026-10-24T01:02:03 ",
            Err("Invalid isoformat string: '2026-10-24T01:02:03 '"),
        ),
        (
            "2026-10-24T01:02:03+00:00x",
            Err("Invalid isoformat string: '2026-10-24T01:02:03+00:00x'"),
        ),
        (
            "2026-10-24T01:02:03.123456789+01:00",
            Ok((
                "2026-10-24",
                [1, 2, 3, 123456],
                Some(3600000000),
                Some(Ok("2026-10-24")),
            )),
        ),
        (
            "\u{e9}2026-10-24",
            Err("Invalid isoformat string: '\u{e9}2026-10-24'"),
        ),
        (
            "2026-10-24T01:02:03.1234567",
            Ok(("2026-10-24", [1, 2, 3, 123456], None, None)),
        ),
        (
            "2026-10-24T010203.5",
            Ok(("2026-10-24", [1, 2, 3, 500000], None, None)),
        ),
        (
            "2026-10-24T01:02:03.5Z",
            Ok((
                "2026-10-24",
                [1, 2, 3, 500000],
                Some(0),
                Some(Ok("2026-10-24")),
            )),
        ),
        (
            "2026-10-24T01:02:03+01:00:00.5",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(3600500000),
                Some(Ok("2026-10-24")),
            )),
        ),
        (
            "2026-10-24T01:02:03+0100",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(3600000000),
                Some(Ok("2026-10-24")),
            )),
        ),
        (
            "2026-10-24T01:02:03-01",
            Ok((
                "2026-10-24",
                [1, 2, 3, 0],
                Some(-3600000000),
                Some(Ok("2026-10-24")),
            )),
        ),
        ("0002-W01-2", Ok(("0002-01-01", [0, 0, 0, 0], None, None))),
        ("0400-W52-7", Ok(("0400-12-31", [0, 0, 0, 0], None, None))),
        ("0000-W01-1", Err("Invalid isoformat string: '0000-W01-1'")),
        ("2027-W01-1", Ok(("2027-01-04", [0, 0, 0, 0], None, None))),
        ("2026-01x01", Err("Invalid isoformat string: '2026-01x01'")),
        (
            "2026-10-24T01:02:03:4",
            Err("Invalid isoformat string: '2026-10-24T01:02:03:4'"),
        ),
        (
            "2026-10-24T01:02x03",
            Err("Invalid isoformat string: '2026-10-24T01:02x03'"),
        ),
        (
            "2026-10-24T01:02Z1",
            Err("Invalid isoformat string: '2026-10-24T01:02Z1'"),
        ),
        ("2026-W01-", Err("Invalid isoformat string: '2026-W01-'")),
        ("2026-W01-12", Ok(("2025-12-29", [12, 0, 0, 0], None, None))),
        ("2026W0110", Err("Invalid isoformat string: '2026W0110'")),
        ("2026W01123", Ok(("2025-12-29", [23, 0, 0, 0], None, None))),
        ("2026W01T10", Ok(("2025-12-29", [10, 0, 0, 0], None, None))),
        (
            "2026-10-24\u{20ac}01:02",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        (
            "2026-10-24\u{1f600}01:02",
            Ok(("2026-10-24", [1, 2, 0, 0], None, None)),
        ),
        (
            "2026-02-30T24:00",
            Err("day 30 must be in range 1..28 for month 2 in year 2026"),
        ),
        ("2026-13-01T24:00", Err("month must be in 1..12, not 13")),
        // END CPYTHON ROWS
    ];
    let mut wrong = Vec::new();
    for (text, want) in rows {
        let got = parse(&text.chars().map(u32::from).collect::<Vec<_>>());
        let want = want.map(|(date, clock, offset, utc)| {
            (
                date.to_string(),
                clock,
                offset,
                utc.map(|r| r.map(str::to_string).map_err(str::to_string)),
            )
        });
        if got != want.map_err(str::to_string) {
            wrong.push(format!("{text:?}: got {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_lone_surrogate_is_an_invalid_string_as_in_cpython() {
    // CPython 3.15: ValueError "Invalid isoformat string: '2026-10-24T01:02\udc80'".
    let mut points: Vec<u32> = "2026-10-24T01:02".chars().map(u32::from).collect();
    points.push(0xdc80);
    assert_eq!(
        datetime_fromisoformat(&points).unwrap_err(),
        "Invalid isoformat string: '2026-10-24T01:02\\udc80'"
    );
    let mut points: Vec<u32> = "2026-10-24".chars().map(u32::from).collect();
    points.push(0xdc80);
    assert_eq!(
        datetime_fromisoformat(&points).unwrap_err(),
        "Invalid isoformat string: '2026-10-24\\udc80'"
    );
}

#[test]
fn date_strings_parse_as_cpython_date_fromisoformat_does() {
    // CPython 3.15's date.fromisoformat(text).isoformat(), or the ValueError text.
    let rows: &[(&str, Result<&str, &str>)] = &[
        ("2026-10-24", Ok("2026-10-24")),
        ("2026W011", Ok("2025-12-29")),
        ("2026-W01x1", Err("Invalid isoformat string: '2026-W01x1'")),
        ("2026-01x01", Err("Invalid isoformat string: '2026-01x01'")),
        ("2026-1-1", Err("Invalid isoformat string: '2026-1-1'")),
        (
            "2026-10-24T01",
            Err("Invalid isoformat string: '2026-10-24T01'"),
        ),
        ("0000-01-01", Err("year must be in 1..9999, not 0")),
    ];
    for &(text, want) in rows {
        let got = date_fromisoformat(text).map(|date| date.isoformat());
        assert_eq!(
            got,
            want.map(str::to_string).map_err(str::to_string),
            "{text:?}"
        );
    }
}
