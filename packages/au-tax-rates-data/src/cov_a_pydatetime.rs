// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `datetime.fromisoformat` against the answers CPython 3.15.0 gave
//! (written by a script that ran each input through it): ISO week dates, compact
//! and separated clocks, fractions, `Z` and offsets, surrogate and multibyte
//! separators and the hour-24 rollover.

use super::*;

fn render(dt: &DateTime) -> String {
    format!(
        "{} {:02}:{:02}:{:02}.{:06} {:?}",
        dt.date.isoformat(),
        dt.hour,
        dt.minute,
        dt.second,
        dt.microsecond,
        dt.offset_us
    )
}

/// Inputs as text; U+FFFE stands for the lone surrogate U+D800, which a Rust
/// string cannot hold.
const FROMISOFORMAT_CASES: &[(&str, Result<&str, &str>)] = &[
    ("2020-W53-7", Ok("2021-01-03 00:00:00.000000 None")),
    ("2020-W01x1", Err("Invalid isoformat string: '2020-W01x1'")),
    (
        "2020-W01x1T00:00",
        Err("Invalid isoformat string: '2020-W01x1T00:00'"),
    ),
    ("2020-12-31T24:00", Ok("2021-01-01 00:00:00.000000 None")),
    (
        "2021-02-30T24:00",
        Err("day 30 must be in range 1..28 for month 2 in year 2021"),
    ),
    ("2021-13-01T24:00", Err("month must be in 1..12, not 13")),
    (
        "1999-12-31T24:00:00+01:00",
        Ok("2000-01-01 00:00:00.000000 Some(3600000000)"),
    ),
    ("2015-W53-7", Ok("2016-01-03 00:00:00.000000 None")),
    ("2004-W53-7", Ok("2005-01-02 00:00:00.000000 None")),
    ("0001-W01-1", Ok("0001-01-01 00:00:00.000000 None")),
    ("9999-W52-7", Err("year must be in 1..9999, not 10000")),
    ("2026-W01-1", Ok("2025-12-29 00:00:00.000000 None")),
    ("0000-W01-1", Err("Invalid isoformat string: '0000-W01-1'")),
    ("2021-W53-1", Err("Invalid isoformat string: '2021-W53-1'")),
    ("2021-W00-1", Err("Invalid isoformat string: '2021-W00-1'")),
    ("2021-W01-8", Err("Invalid isoformat string: '2021-W01-8'")),
    ("2021-W01-0", Err("Invalid isoformat string: '2021-W01-0'")),
    ("2021-W01-1", Ok("2021-01-04 00:00:00.000000 None")),
    ("2021-W011", Err("Invalid isoformat string: '2021-W011'")),
    ("2021-W01", Ok("2021-01-04 00:00:00.000000 None")),
    ("2021W01", Ok("2021-01-04 00:00:00.000000 None")),
    ("2021W011", Ok("2021-01-04 00:00:00.000000 None")),
    ("2021W01-1", Err("Invalid isoformat string: '2021W01-1'")),
    ("2021-1024", Err("Invalid isoformat string: '2021-1024'")),
    ("202101-24", Err("Invalid isoformat string: '202101-24'")),
    (
        "2021-01-01T1",
        Err("Invalid isoformat string: '2021-01-01T1'"),
    ),
    (
        "2021-01-01T10.5",
        Err("Invalid isoformat string: '2021-01-01T10.5'"),
    ),
    (
        "2021-01-01T10:00:00:00",
        Err("Invalid isoformat string: '2021-01-01T10:00:00:00'"),
    ),
    ("2021-01-01T100000", Ok("2021-01-01 10:00:00.000000 None")),
    (
        "2021-01-01T10:00x00",
        Err("Invalid isoformat string: '2021-01-01T10:00x00'"),
    ),
    (
        "2021-01-01T10:00:00.x",
        Err("Invalid isoformat string: '2021-01-01T10:00:00.x'"),
    ),
    (
        "2021-01-01T10:00:00.5",
        Ok("2021-01-01 10:00:00.500000 None"),
    ),
    (
        "2021-01-01T10:00:00.1234567",
        Ok("2021-01-01 10:00:00.123456 None"),
    ),
    (
        "2021-01-01T10:00:00,25",
        Ok("2021-01-01 10:00:00.250000 None"),
    ),
    ("2021-01-01T10:60", Err("minute must be in 0..59, not 60")),
    (
        "2021-01-01T10:00:60",
        Err("second must be in 0..59, not 60"),
    ),
    (
        "2021-01-01T10:00+10:60",
        Err("minute must be in 0..59, not 60"),
    ),
    (
        "2021-01-01T10:00+10:00:60",
        Err("second must be in 0..59, not 60"),
    ),
    (
        "2021-01-01Tx",
        Err("Invalid isoformat string: '2021-01-01Tx'"),
    ),
    (
        "2021-01-01T10a",
        Err("Invalid isoformat string: '2021-01-01T10a'"),
    ),
    (
        "2021-01-01T10:00Z",
        Ok("2021-01-01 10:00:00.000000 Some(0)"),
    ),
    (
        "2021-01-01T10:00Zx",
        Err("Invalid isoformat string: '2021-01-01T10:00Zx'"),
    ),
    (
        "2021-01-01T10:00+10:00x",
        Err("Invalid isoformat string: '2021-01-01T10:00+10:00x'"),
    ),
    (
        "2021-01-01T10:00+1",
        Err("Invalid isoformat string: '2021-01-01T10:00+1'"),
    ),
    (
        "2021-01-01T10:00-05:30",
        Ok("2021-01-01 10:00:00.000000 Some(-19800000000)"),
    ),
    ("20210101T10", Ok("2021-01-01 10:00:00.000000 None")),
    ("2021-W01-1T10", Ok("2021-01-04 10:00:00.000000 None")),
    ("2021W011T10", Ok("2021-01-04 10:00:00.000000 None")),
    ("2021W0110", Err("Invalid isoformat string: '2021W0110'")),
    ("2021W01T10", Ok("2021-01-04 10:00:00.000000 None")),
    (
        "2021-W0110:00",
        Err("Invalid isoformat string: '2021-W0110:00'"),
    ),
    ("2021-01-0", Err("Invalid isoformat string: '2021-01-0'")),
    (
        "2021-01-012",
        Err("Invalid isoformat string: '2021-01-012'"),
    ),
    (
        "2021-01-01T",
        Err("Invalid isoformat string: '2021-01-01T'"),
    ),
    ("2021-1", Err("Invalid isoformat string: '2021-1'")),
    ("20210101T1000Z", Ok("2021-01-01 10:00:00.000000 Some(0)")),
    (
        "2021-01-01\u{fffe}10:00",
        Ok("2021-01-01 10:00:00.000000 None"),
    ),
    (
        "2021-01\u{fffe}10",
        Err("Invalid isoformat string: '2021-01\u{5c}ud80010'"),
    ),
    ("2021W01\u{fffe}10", Ok("2021-01-04 10:00:00.000000 None")),
    (
        "2021-01-01T10:00\u{fffe}",
        Err("Invalid isoformat string: '2021-01-01T10:00\u{5c}ud800'"),
    ),
    (
        "2021-01-01\u{e9}10:00",
        Ok("2021-01-01 10:00:00.000000 None"),
    ),
    (
        "2021-01-01\u{3000}10:00",
        Ok("2021-01-01 10:00:00.000000 None"),
    ),
    (
        "2021-01-01\u{1f600}10:00",
        Ok("2021-01-01 10:00:00.000000 None"),
    ),
    ("2021-12-31T24:00", Ok("2022-01-01 00:00:00.000000 None")),
    ("2021-01-31T24:00", Ok("2021-02-01 00:00:00.000000 None")),
    (
        "2021-02-28T24:00:00.000001",
        Err("minute, second, and microsecond must be 0 when hour is 24"),
    ),
    ("1991-W01-2", Ok("1991-01-01 00:00:00.000000 None")),
    ("2021-W01-", Err("Invalid isoformat string: '2021-W01-'")),
    ("2021-W01-12", Ok("2021-01-04 12:00:00.000000 None")),
    ("2021W01123", Ok("2021-01-04 23:00:00.000000 None")),
    ("2021W011234", Err("hour must be in 0..23, not 34")),
    ("2021-01-30T24:00", Ok("2021-01-31 00:00:00.000000 None")),
    ("2021-W01x1", Err("Invalid isoformat string: '2021-W01x1'")),
    ("2021-W01T1", Err("Invalid isoformat string: '2021-W01T1'")),
    (
        "2021-W01-1x",
        Err("Invalid isoformat string: '2021-W01-1x'"),
    ),
    (
        "2021-01-01T10:00:00.123456+05:00:30.5",
        Ok("2021-01-01 10:00:00.123456 Some(18030500000)"),
    ),
    (
        "2021-01-01T10:00:00+05:00:30.1234567",
        Ok("2021-01-01 10:00:00.000000 Some(18030123456)"),
    ),
];

#[test]
fn fromisoformat_answers_as_cpython_does() {
    for (input, expected) in FROMISOFORMAT_CASES {
        let points: Vec<u32> = input
            .chars()
            .map(|c| {
                if c == '\u{fffe}' {
                    0xd800
                } else {
                    u32::from(c)
                }
            })
            .collect();
        let got = datetime_fromisoformat(&points);
        let got = got.as_ref().map(render).map_err(String::as_str);
        let got = got.as_ref().map(String::as_str).map_err(|e| *e);
        assert_eq!(got, *expected, "input {input:x?}");
    }
}

/// `date.fromisoformat` of a week date whose weekday follows another
/// character than `-` (CPython 3.15.0: `Invalid isoformat string`).
#[test]
fn a_week_date_needs_its_hyphen_before_the_weekday() {
    assert_eq!(
        date_fromisoformat("2021-W01x1").map(|d| d.isoformat()),
        Err(String::from("Invalid isoformat string: '2021-W01x1'"))
    );
}
