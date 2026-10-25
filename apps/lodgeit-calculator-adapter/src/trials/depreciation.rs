//! `trials/depreciation.py`: accounting depreciation, a movement that has
//! to close, or a refusal.
//!
//! The provider's AASB 116 carrying amount is never compared with a
//! Division 40 decline in value; they answer different questions. What is
//! checked is the provider's own ledger identity over a range:
//!
//! ```text
//! closing_wdv = opening_wdv + cost_additions - range_dep
//! ```
//!
//! Where the window starts before the acquisition date, the provider emits
//! no `cost_additions` and the figures do not close. An unreconcilable
//! movement is `CONTRACT_FAILURE`; the addition that would close it is never
//! invented.
use super::{Comparison, Evaluation, Money, dict, get, shown};
use crate::client::{Client, Outcome, Status};
use crate::decimal::Decimal;
use crate::p;
use crate::pydatetime::date_fromisoformat;
use crate::pyjson::Json;
use crate::pyops::{Exc, R, S, decimal_signal, pts};

pub const RANGE_CALC_URI: &str = "urn:sbrm:calculator:depreciation:range";
pub const PERIOD_URI: &str = "urn:sbrm:period:depreciation:unscoped";
pub const DAY_COUNTS: [&str; 3] = ["actual/actual", "actual/365", "monthly"];

/// A fabricated asset. Accounting inputs only; nothing here is tax.
#[derive(Clone, Debug)]
pub struct Asset {
    pub case_id: String,
    pub cost: Decimal,
    pub acquisition_date: String,
    pub useful_life_years: i64,
    /// `"prime_cost"` by default.
    pub method: String,
}

impl Asset {
    /// `payload()`.
    pub fn payload(&self) -> Json {
        Json::obj([
            ("cost", Json::Dec(self.cost.clone())),
            ("acquisition_date", Json::str(&self.acquisition_date)),
            (
                "accounting_useful_life_years",
                Json::Int(self.useful_life_years.into()),
            ),
            ("accounting_method", Json::str(&self.method)),
        ])
    }
}

/// `range_request(asset, from_date, to_date, day_count)`.
pub fn range_request(asset: &Asset, from_date: &str, to_date: &str, day_count: &str) -> R<Json> {
    if !DAY_COUNTS.contains(&day_count) {
        return Err(Exc::msg(
            "ValueError",
            "day_count must be one of ('actual/actual', 'actual/365', 'monthly')",
        ));
    }
    Ok(Json::obj([
        ("basis", Json::str("accounting")),
        ("asset", asset.payload()),
        ("from_date", Json::str(from_date)),
        ("to_date", Json::str(to_date)),
        ("day_count", Json::str(day_count)),
    ]))
}

/// `reconcile(values, supplied_additions=..., tolerance=...)`: does opening
/// plus additions less depreciation equal closing? A supplied addition is the
/// caller's own evidence and is named in the reasons; none is derived from
/// the gap.
pub fn reconcile(
    values: &Money,
    supplied_additions: Option<&Decimal>,
    tolerance: &Decimal,
) -> R<(bool, Vec<S>)> {
    let values = &dict(values);
    let missing: Vec<&str> = ["opening_wdv", "closing_wdv", "range_dep"]
        .into_iter()
        .filter(|name| get(values, &pts(name)).is_none())
        .collect();
    if !missing.is_empty() {
        return Ok((
            false,
            vec![pts(&format!(
                "the response is missing {}, so the movement cannot be tested",
                missing.join(", ")
            ))],
        ));
    }
    let value = |name: &str| get(values, &pts(name)).expect("present");
    let (opening, closing, range_dep) = (
        value("opening_wdv"),
        value("closing_wdv"),
        value("range_dep"),
    );
    let nil = Decimal::parse_exact("0.00").expect("a literal");
    let additions = supplied_additions.unwrap_or(&nil);
    let expected_closing = opening
        .add(additions)
        .and_then(|sum| sum.sub(range_dep))
        .map_err(decimal_signal)?;
    let gap = closing.sub(&expected_closing).map_err(decimal_signal)?;
    let mut reasons = Vec::new();
    if let Some(supplied) = supplied_additions {
        reasons.push(p!(
            "cost additions of ",
            shown(supplied),
            " were supplied by the caller as separate evidence; the provider did not return them"
        ));
    }
    if gap.copy_abs().le(tolerance).map_err(decimal_signal)? {
        return Ok((true, reasons));
    }
    reasons.push(p!(
        "movement does not close: opening ",
        shown(opening),
        " plus additions ",
        shown(additions),
        " less depreciation ",
        shown(range_dep),
        " is ",
        shown(&expected_closing),
        ", against a closing balance of ",
        shown(closing),
        ", a gap of ",
        shown(&gap),
        ". No addition has been invented to close it."
    ));
    Ok((false, reasons))
}

/// `spans_acquisition(asset, from_date)`: the window starts before the asset
/// was acquired, the provider's documented pre-acquisition limitation. The
/// comparison is of the strings, as Python's is.
pub fn spans_acquisition(asset: &Asset, from_date: &str) -> bool {
    from_date < asset.acquisition_date.as_str()
}

/// `consecutive_windows(start, end, boundaries)`: split a window at the
/// boundaries, inclusive of both ends, so that consecutive ranges telescope.
pub fn consecutive_windows(
    start: &str,
    end: &str,
    boundaries: &[String],
) -> R<Vec<(String, String)>> {
    let mut windows = Vec::new();
    let mut current = start.to_owned();
    for boundary in boundaries {
        windows.push((current, boundary.clone()));
        let day = date_fromisoformat(boundary).map_err(|text| Exc::msg("ValueError", &text))?;
        current = day
            .add_days(1)
            .ok_or_else(|| Exc::msg("OverflowError", "date value out of range"))?
            .isoformat();
    }
    windows.push((current, end.to_owned()));
    Ok(windows)
}

/// `evaluate_range(asset, from_date, to_date, day_count, client,
/// supplied_additions=..., case_id=...)`.
pub fn evaluate_range(
    asset: &Asset,
    from_date: &str,
    to_date: &str,
    day_count: &str,
    client: Option<&mut Client<'_>>,
    supplied_additions: Option<&Decimal>,
    case_id: Option<&str>,
) -> R<Comparison> {
    let identifier = match case_id {
        Some(id) if !id.is_empty() => pts(id),
        _ => p!(asset.case_id, ":", from_date, "..", to_date, ":", day_count),
    };
    let Some(client) = client else {
        return Ok(Comparison {
            case_id: identifier,
            evaluation: Evaluation::NotRun,
            reasons: vec![pts("No client supplied; the provider was not called.")],
            ..Comparison::default()
        });
    };
    let body = range_request(asset, from_date, to_date, day_count)?;
    let outcome = client.invoke(&pts(RANGE_CALC_URI), &pts(PERIOD_URI), &body)?;
    evaluate_outcome(asset, from_date, identifier, &outcome, supplied_additions)
}

/// `evaluate_outcome(asset, from_date, case_id, outcome,
/// supplied_additions=...)`.
pub fn evaluate_outcome(
    asset: &Asset,
    from_date: &str,
    case_id: S,
    outcome: &Outcome,
    supplied_additions: Option<&Decimal>,
) -> R<Comparison> {
    let common = Comparison {
        case_id,
        upstream_status: Some(pts(outcome.status.as_str())),
        ..Comparison::default()
    };
    let with = |evaluation, reasons: Vec<S>| Comparison {
        evaluation,
        reasons,
        ..common.clone()
    };
    match outcome.status {
        Status::UpstreamNotFound => {
            return Ok(with(
                Evaluation::UnsupportedPeriod,
                outcome.findings.clone(),
            ));
        }
        Status::UpstreamUnavailable | Status::UpstreamThrottled | Status::RefusedToSend => {
            return Ok(with(
                Evaluation::UpstreamUnavailable,
                outcome.findings.clone(),
            ));
        }
        Status::UpstreamRefused => {
            return Ok(with(Evaluation::ScopeMismatch, outcome.findings.clone()));
        }
        Status::Computed => {}
        _ => return Ok(with(Evaluation::ContractFailure, outcome.findings.clone())),
    }
    let values: Money = dict(&outcome.values);
    let (closes, found) = reconcile(&values, supplied_additions, &super::cent())?;
    let mut reasons = outcome.findings.clone();
    reasons.extend(found);
    if closes {
        reasons.push(pts(
            "accounting carrying amounts only; this is not a Division 40 deduction and the tax \
             figure will not equal it",
        ));
        return Ok(Comparison {
            evaluation: Evaluation::Match,
            upstream: values,
            reasons,
            ..common
        });
    }
    if spans_acquisition(asset, from_date) {
        reasons.push(pts(
            "the window starts before the acquisition date, which is the provider's published \
             pre-acquisition limitation: the cost enters during the window and no cost_additions \
             field is emitted. Supply the addition as separate evidence or start the window on or \
             after the acquisition date.",
        ));
    }
    Ok(Comparison {
        evaluation: Evaluation::ContractFailure,
        upstream: values,
        reasons,
        ..common
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(text: &str) -> Decimal {
        Decimal::parse_exact(text).expect("a literal")
    }

    fn asset() -> Asset {
        Asset {
            case_id: "DEP-1-plant".to_owned(),
            cost: dec("120000.00"),
            acquisition_date: "2024-07-01".to_owned(),
            useful_life_years: 5,
            method: "prime_cost".to_owned(),
        }
    }

    fn range(opening: &str, closing: &str, depreciation: &str) -> Money {
        vec![
            (pts("opening_wdv"), dec(opening)),
            (pts("closing_wdv"), dec(closing)),
            (pts("range_dep"), dec(depreciation)),
        ]
    }

    fn computed(values: Money) -> Outcome {
        Outcome {
            status: Status::Computed,
            calculator: None,
            period: None,
            request_body: None,
            request_json: None,
            raw_response: Json::Null,
            http_status: Some(200),
            result: Json::Null,
            values,
            manifest: Json::Null,
            advisory: Json::Null,
            upstream_refusal_class: Json::Null,
            findings: Vec::new(),
            observed_at: String::new(),
            elapsed_ms: None,
            contract_snapshot: Json::Null,
            contract_sha256: String::new(),
            base_url: None,
        }
    }

    fn mentions(comparison: &Comparison, text: &str) -> bool {
        let text = pts(text);
        comparison.reasons.iter().any(|reason| {
            reason
                .windows(text.len())
                .any(|part| part == text.as_slice())
        })
    }

    #[test]
    fn the_request_requires_a_known_day_count() {
        let body =
            range_request(&asset(), "2024-07-01", "2025-06-30", "actual/actual").expect("a body");
        assert_eq!(body.at("basis"), &Json::str("accounting"));
        let refused = range_request(&asset(), "2024-07-01", "2025-06-30", "actual/360");
        assert_eq!(refused.expect_err("refused").class, "ValueError");
    }

    #[test]
    fn a_closing_movement_is_a_match() {
        let outcome = computed(range("120000.00", "96000.00", "24000.00"));
        let compared =
            evaluate_outcome(&asset(), "2024-07-01", pts("DEP-1"), &outcome, None).expect("ok");
        assert_eq!(compared.evaluation, Evaluation::Match);
        assert!(mentions(&compared, "Division 40"));
    }

    #[test]
    fn a_movement_that_does_not_close_is_refused_and_no_addition_is_invented() {
        let outcome = computed(range("0.00", "108000.00", "12000.00"));
        let compared =
            evaluate_outcome(&asset(), "2024-01-01", pts("DEP-1"), &outcome, None).expect("ok");
        assert_eq!(compared.evaluation, Evaluation::ContractFailure);
        assert!(mentions(&compared, "does not close"));
        assert!(mentions(&compared, "pre-acquisition limitation"));
        assert!(get(&compared.upstream, &pts("cost_additions")).is_none());

        let supplied = dec("120000.00");
        let compared = evaluate_outcome(
            &asset(),
            "2024-01-01",
            pts("DEP-1"),
            &outcome,
            Some(&supplied),
        )
        .expect("ok");
        assert_eq!(compared.evaluation, Evaluation::Match);
        assert!(mentions(&compared, "supplied by the caller"));
    }

    #[test]
    fn a_missing_figure_cannot_be_reconciled_around() {
        let (closes, reasons) = reconcile(
            &vec![(pts("opening_wdv"), dec("1.00"))],
            None,
            &super::super::cent(),
        )
        .expect("ok");
        assert!(!closes);
        assert_eq!(
            reasons,
            vec![pts(
                "the response is missing closing_wdv, range_dep, so the movement cannot be tested"
            )]
        );
    }

    #[test]
    fn the_pre_acquisition_window_is_recognised_before_the_call() {
        assert!(spans_acquisition(&asset(), "2024-01-01"));
        assert!(!spans_acquisition(&asset(), "2024-07-01"));
        assert!(!spans_acquisition(&asset(), "2025-01-01"));
    }

    #[test]
    fn consecutive_windows_telescope_and_cover_every_day() {
        let pairs = |list: &[(&str, &str)]| -> Vec<(String, String)> {
            list.iter()
                .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
                .collect()
        };
        assert_eq!(
            consecutive_windows("2024-07-01", "2025-06-30", &["2024-12-31".to_owned()]).unwrap(),
            pairs(&[("2024-07-01", "2024-12-31"), ("2025-01-01", "2025-06-30")])
        );
        assert_eq!(
            consecutive_windows("2024-01-01", "2024-12-31", &["2024-02-29".to_owned()]).unwrap(),
            pairs(&[("2024-01-01", "2024-02-29"), ("2024-03-01", "2024-12-31")])
        );
        let last = consecutive_windows("9999-01-01", "9999-12-31", &["9999-12-31".to_owned()]);
        assert_eq!(last.expect_err("past the last date").class, "OverflowError");
    }

    /// A repeated figure holds its last value, as in Python's dict.
    #[test]
    fn a_repeated_figure_is_read_as_its_last_value() {
        let mut values = range("120000.00", "96000.00", "24000.00");
        values.push((pts("closing_wdv"), dec("90000.00")));
        let compared = evaluate_outcome(
            &asset(),
            "2024-07-01",
            pts("DEP-1"),
            &computed(values),
            None,
        )
        .expect("ok");
        assert_eq!(compared.evaluation, Evaluation::ContractFailure);
        assert_eq!(compared.upstream.len(), 3);

        let mut values = range("120000.00", "90000.00", "24000.00");
        values.push((pts("closing_wdv"), dec("96000.00")));
        let compared = evaluate_outcome(
            &asset(),
            "2024-07-01",
            pts("DEP-1"),
            &computed(values),
            None,
        )
        .expect("ok");
        assert_eq!(compared.evaluation, Evaluation::Match);
    }
}
