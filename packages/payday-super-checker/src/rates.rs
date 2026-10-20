//! Dated legal rates: GIC quarters and financial-year super parameters.
//!
//! Every rate lives in data/*.json, compiled into the binary, recording where
//! it came from and when that was checked. GIC resets quarterly (TAA 1953
//! s 8AAD) and the SG parameters change each financial year.

use sha2::{Digest, Sha256};

use crate::decimal::Decimal;
use crate::py::Date;
use crate::raw::Raw;
use crate::{Failure, Result};

pub const GIC_RATES: &str = include_str!("../data/gic_rates.json");
pub const RATES: &str = include_str!("../data/rates.json");
const GIC_PATH: &str = "data/gic_rates.json";

/// A rate table a figure was computed from, and the digest of its decoded text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableSource {
    pub uri: String,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct GicQuarter {
    pub start: Date,
    pub end: Date,
    pub annual_pct: Decimal,
    pub seen: String,
}

/// `TAA 1953 s 8AAD` divides the annual rate by the days in the calendar year.
pub fn days_in_year(d: Date) -> i64 {
    let first = Date::new(i64::from(d.year), 1, 1).expect("a real year");
    let last = Date::new(i64::from(d.year), 12, 31).expect("a real year");
    last.days_since(first) + 1
}

fn rates_error(message: String) -> Failure {
    Failure::Value(message)
}

#[derive(Clone, Debug)]
pub struct GicTable {
    pub source: Option<TableSource>,
    quarters: Vec<GicQuarter>,
}

impl GicTable {
    pub fn new(mut quarters: Vec<GicQuarter>, source: Option<TableSource>) -> Result<GicTable> {
        quarters.sort_by_key(|q| q.start);
        if quarters.is_empty() {
            return Err(rates_error("GIC table is empty".into()));
        }
        for q in &quarters {
            if q.start > q.end {
                return Err(rates_error(format!(
                    "GIC interval {} to {} ends before it starts",
                    q.start.isoformat(),
                    q.end.isoformat()
                )));
            }
        }
        for pair in quarters.windows(2) {
            let (previous, current) = (&pair[0], &pair[1]);
            let gap = current.start.days_since(previous.end);
            if gap == 1 {
                continue;
            }
            let relation = if gap <= 0 { "overlap" } else { "gap" };
            return Err(rates_error(format!(
                "GIC intervals {} to {} and {} to {} {relation}; quarters must be contiguous",
                previous.start.isoformat(),
                previous.end.isoformat(),
                current.start.isoformat(),
                current.end.isoformat()
            )));
        }
        Ok(GicTable { source, quarters })
    }

    pub fn quarters(&self) -> &[GicQuarter] {
        &self.quarters
    }

    pub fn last_known(&self) -> Date {
        self.quarters.last().expect("never empty").end
    }

    fn latest(&self) -> &GicQuarter {
        self.quarters.last().expect("never empty")
    }

    /// One line naming the coverage, latest rate and check date.
    pub fn provenance(&self) -> String {
        let latest = self.latest();
        let checked = if latest.seen.is_empty() {
            String::new()
        } else {
            format!(", checked {}", latest.seen)
        };
        format!(
            "GIC table covers {} to {} (latest quarter {}% p.a.{checked})",
            self.quarters[0].start.isoformat(),
            latest.end.isoformat(),
            latest.annual_pct.to_sci_string()
        )
    }

    /// The daily GIC rate for one day; past the last quarter only on request.
    pub fn daily_rate(&self, d: Date, allow_stale: bool) -> Result<Decimal> {
        let divisor = Decimal::from_int(days_in_year(d));
        let hundred = Decimal::from_int(100);
        for q in &self.quarters {
            if q.start <= d && d <= q.end {
                return Ok(q.annual_pct.div(&hundred)?.div(&divisor)?);
            }
        }
        if d > self.last_known() {
            if !allow_stale {
                return Err(Failure::StaleGic(format!(
                    "{} is past the last GIC quarter on record ({}), and the ATO has published no \
                     rate for it. Update {GIC_PATH} from the ATO \
                     GIC rates page, or pass --allow-stale-gic to estimate those days \
                     at the last known rate ({}% p.a.), which the report will then say it did",
                    d.isoformat(),
                    self.last_known().isoformat(),
                    self.latest().annual_pct.to_sci_string()
                )));
            }
            return Ok(self.latest().annual_pct.div(&hundred)?.div(&divisor)?);
        }
        Err(rates_error(format!(
            "no GIC rate on record for {}",
            d.isoformat()
        )))
    }

    pub fn staleness(&self, d: Date) -> Option<String> {
        (d > self.last_known()).then(|| {
            format!(
                "GIC rate table ends {}; days after that use the last known rate ({}% p.a.): \
                 update {GIC_PATH} from the ATO GIC rates page",
                self.last_known().isoformat(),
                self.latest().annual_pct.to_sci_string()
            )
        })
    }
}

// A GIC rate above this is a typo, not a rate: the 2 hand-edit slips that print
// money are a dropped decimal point (1143 for 11.43) and a stray minus sign.
const RATE_CEILING: i64 = 100;

/// A hand-edited rate, refused when it is not a finite number in range.
fn rate(raw: &Raw, place: &str) -> Result<Decimal> {
    let shown = raw.py_repr();
    let value = Decimal::parse(&raw.py_str()).map_err(|_| {
        rates_error(format!(
            "{place} has annual_pct {shown}, which is not a number. Fix it and run again"
        ))
    })?;
    if !value.is_finite() {
        return Err(rates_error(format!(
            "{place} has annual_pct {shown}; a rate must be a finite number, and nan or \
             infinity would silently poison every figure in the report"
        )));
    }
    if value.lt(&Decimal::from_int(0))? {
        return Err(rates_error(format!(
            "{place} has annual_pct {shown}; a GIC rate cannot be negative. A minus sign \
             here prints negative notional earnings and inverts the SG charge estimate, so \
             the report reads as money owed back to the employer"
        )));
    }
    if Decimal::from_int(RATE_CEILING).lt(&value)? {
        return Err(rates_error(format!(
            "{place} has annual_pct {shown}, above the {RATE_CEILING}% a year this tool \
             will accept. The ATO general interest charge has never come near that, so \
             this is a typo, most likely a missing decimal point"
        )));
    }
    Ok(value)
}

fn rate_date(raw: &Raw, key: &str, place: &str) -> Result<Date> {
    Date::fromisoformat(&raw.py_str()).ok_or_else(|| {
        rates_error(format!(
            "{place} has {key} {}; write it as YYYY-MM-DD",
            raw.py_repr()
        ))
    })
}

fn json_document(text: &str, path: &str) -> Result<Raw> {
    crate::pyjson::loads(text)
        .map(|doc| Raw::from_json(&doc))
        .map_err(|e| rates_error(format!("{path} is not valid JSON: {e}")))
}

/// The bundled GIC table and the source it was read from.
pub fn load_gic() -> Result<GicTable> {
    parse_gic(GIC_RATES)
}

fn parse_gic(raw_text: &str) -> Result<GicTable> {
    let text = crate::py::universal_newlines(raw_text);
    let source = TableSource {
        uri: GIC_PATH.into(),
        sha256: hex(&Sha256::digest(text.as_bytes())),
    };
    let doc = json_document(&text, GIC_PATH)?;
    if !matches!(doc, Raw::Dict(_)) {
        return Err(rates_error(format!(
            "{GIC_PATH} must be a JSON object with a 'quarters' list; it holds a {} instead",
            doc.type_name()
        )));
    }
    let Some(entries) = doc.get("quarters") else {
        return Err(rates_error(format!(
            "{GIC_PATH} is missing the 'quarters' key"
        )));
    };
    let Raw::List(entries) = entries else {
        return Err(rates_error(format!(
            "{GIC_PATH}: 'quarters' must be a list of quarter objects"
        )));
    };
    let mut quarters = Vec::new();
    for (n, entry) in entries.iter().enumerate().map(|(i, e)| (i + 1, e)) {
        if !matches!(entry, Raw::Dict(_)) {
            return Err(rates_error(format!(
                "GIC quarter {n} in {GIC_PATH} is not an object"
            )));
        }
        let shown = |key: &str| entry.get(key).map_or("None".into(), Raw::py_str);
        let place = format!(
            "GIC quarter {n} in {GIC_PATH} ({} to {})",
            shown("from"),
            shown("to")
        );
        let mut fields = Vec::new();
        for key in ["from", "to", "annual_pct"] {
            let Some(value) = entry.get(key) else {
                return Err(rates_error(format!("{place} is missing '{key}'")));
            };
            fields.push(value);
        }
        quarters.push(GicQuarter {
            start: rate_date(fields[0], "from", &place)?,
            end: rate_date(fields[1], "to", &place)?,
            annual_pct: rate(fields[2], &place)?,
            seen: entry.get("seen").map(Raw::py_str).unwrap_or_default(),
        });
    }
    GicTable::new(quarters, Some(source))
}

/// The bundled financial-year parameters.
pub fn load_rates() -> Result<Raw> {
    parse_rates(RATES)
}

fn parse_rates(text: &str) -> Result<Raw> {
    const PATH: &str = "data/rates.json";
    let doc = json_document(&crate::py::universal_newlines(text), PATH)?;
    if !matches!(doc, Raw::Dict(_)) {
        return Err(rates_error(format!(
            "{PATH} must be a JSON object with a 'financial_years' map; it holds a {} instead",
            doc.type_name()
        )));
    }
    match doc.get("financial_years") {
        None => {}
        Some(years @ Raw::Dict(pairs)) => {
            // A repeated key keeps its first position and its last value.
            let mut seen: Vec<&String> = Vec::new();
            for (label, _) in pairs {
                if seen.contains(&label) {
                    continue;
                }
                seen.push(label);
                let entry = years.get(label).expect("a key of this map");
                if !matches!(entry, Raw::Dict(_)) {
                    return Err(rates_error(format!(
                        "{PATH}: financial year {} must hold an object of figures; it holds \
                         a {} instead",
                        crate::pytext::repr(label),
                        entry.type_name()
                    )));
                }
            }
        }
        Some(other) => {
            return Err(rates_error(format!(
                "{PATH}: 'financial_years' must be a map of labels like '2026-27' to their \
                 figures; it holds a {} instead",
                other.type_name()
            )));
        }
    }
    Ok(doc)
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_tables_load() {
        let gic = load_gic().unwrap();
        assert_eq!(gic.last_known().isoformat(), "2026-12-31");
        assert!(
            gic.provenance()
                .starts_with("GIC table covers 2026-04-01 to 2026-12-31")
        );
        assert!(load_rates().unwrap().get("financial_years").is_some());
    }

    /// Python 0.2.2's answers for hand-edited tables, its data path shortened to
    /// the crate's: a typo must stop the run rather than print money.
    #[test]
    fn malformed_tables_are_refused_as_python_refused_them() {
        for (text, expected) in [
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-06-30", "annual_pct": "11.43", "seen": "2026-07-01"}]}"#,
                r#"GIC interval 2026-07-01 to 2026-06-30 ends before it starts"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "11.43", "seen": "2026-07-01"}, {"from": "2026-10-02", "to": "2026-12-31", "annual_pct": "11.43", "seen": "2026-07-01"}]}"#,
                r#"GIC intervals 2026-07-01 to 2026-09-30 and 2026-10-02 to 2026-12-31 gap; quarters must be contiguous"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "eleven", "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct 'eleven', which is not a number. Fix it and run again"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": true, "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct True, which is not a number. Fix it and run again"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "NaN", "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct 'NaN'; a rate must be a finite number, and nan or infinity would silently poison every figure in the report"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": Infinity}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct inf; a rate must be a finite number, and nan or infinity would silently poison every figure in the report"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "-11.43", "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct '-11.43'; a GIC rate cannot be negative. A minus sign here prints negative notional earnings and inverts the SG charge estimate, so the report reads as money owed back to the employer"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "1143", "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct '1143', above the 100% a year this tool will accept. The ATO general interest charge has never come near that, so this is a typo, most likely a missing decimal point"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": 100.5, "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct 100.5, above the 100% a year this tool will accept. The ATO general interest charge has never come near that, so this is a typo, most likely a missing decimal point"#,
            ),
            (
                r#"{"quarter": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "11.43", "seen": "2026-07-01"}]}"#,
                r#"data/gic_rates.json is missing the 'quarters' key"#,
            ),
            (
                r#"[{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "11.43", "seen": "2026-07-01"}]"#,
                r#"data/gic_rates.json must be a JSON object with a 'quarters' list; it holds a list instead"#,
            ),
            (
                r#"{"quarters": {"a": {"from": "2026-07-01", "to": "2026-09-30", "annual_pct": "11.43", "seen": "2026-07-01"}}}"#,
                r#"data/gic_rates.json: 'quarters' must be a list of quarter objects"#,
            ),
            (
                r#"{"quarters": [,]}"#,
                r#"data/gic_rates.json is not valid JSON: Expecting value: line 1 column 15 (char 14)"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) is missing 'annual_pct'"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "2026-09-30", "annual_pct": null, "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 2026-09-30) has annual_pct None, which is not a number. Fix it and run again"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": "30/09/2026", "annual_pct": "11.43", "seen": "2026-07-01"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (2026-07-01 to 30/09/2026) has to '30/09/2026'; write it as YYYY-MM-DD"#,
            ),
            (
                r#"{"quarters": [{"from": "2026-07-01", "to": 20260930, "annual_pct": "11.43", "seen": "2026-07-01"}]}"#,
                r#"OK"#,
            ),
            (r#"{"quarters": []}"#, r#"GIC table is empty"#),
            (
                r#"{"quarters": ["2026-07-01"]}"#,
                r#"GIC quarter 1 in data/gic_rates.json is not an object"#,
            ),
            (
                r#"{"quarters": [{"to": "2026-09-30", "annual_pct": "1"}]}"#,
                r#"GIC quarter 1 in data/gic_rates.json (None to 2026-09-30) is missing 'from'"#,
            ),
        ] {
            let got = match parse_gic(text) {
                Ok(_) => "OK".to_string(),
                Err(f) => f.message(),
            };
            assert_eq!(got, expected, "{text}");
        }
        for (text, expected) in [
            (
                r#"[1]"#,
                r#"data/rates.json must be a JSON object with a 'financial_years' map; it holds a list instead"#,
            ),
            (
                r#"{"financial_years": [1]}"#,
                r#"data/rates.json: 'financial_years' must be a map of labels like '2026-27' to their figures; it holds a list instead"#,
            ),
            (
                r#"{"financial_years": {"2026-27": {}, "2027-28": 5}}"#,
                r#"data/rates.json: financial year '2027-28' must hold an object of figures; it holds a int instead"#,
            ),
            (
                r#"{"financial_years": {"2026-27": 5, "2027-28": {}, "2026-27": {}}}"#,
                r#"OK"#,
            ),
            (
                r#"{"#,
                r#"data/rates.json is not valid JSON: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"#,
            ),
            (r#"{}"#, r#"OK"#),
        ] {
            let got = match parse_rates(text) {
                Ok(_) => "OK".to_string(),
                Err(f) => f.message(),
            };
            assert_eq!(got, expected, "{text}");
        }
    }

    #[test]
    fn staleness_warns_past_the_shipped_table() {
        let gic = load_gic().unwrap();
        let last = gic.last_known();
        assert_eq!(gic.staleness(last), None);
        assert!(
            gic.staleness(last.add_days(1).unwrap())
                .unwrap()
                .contains("GIC rate table ends")
        );
    }

    #[test]
    fn a_crlf_checkout_and_an_lf_checkout_agree() {
        let lf = parse_gic(GIC_RATES).unwrap().source.unwrap();
        let crlf = parse_gic(&GIC_RATES.replace("\r\n", "\n").replace('\n', "\r\n"))
            .unwrap()
            .source
            .unwrap();
        assert_eq!(lf, crlf);
    }
}
