//! Verdicts, exposure figures, console summary and report.csv.

use std::cmp::Ordering;

use crate::assess::{AT_RISK, Assessed, NO_RECEIPT_CAVEAT, SKIPPED, VERDICTS, financial_year};
use crate::csv_io::{cents, csv_safe, money, zero};
use crate::csvwrite;
use crate::decimal::Decimal;
use crate::py::Date;
use crate::raw::Raw;
use crate::sgc::scenario;
use crate::{Failure, Result, VERSION};

/// Rows whose verdict the supplied deadline facts could not decide.
pub fn horizon_indeterminate(results: &[Assessed]) -> Vec<&Assessed> {
    results
        .iter()
        .filter(|r| r.horizon_verdicts.is_some())
        .collect()
}

/// True when no in-scope positive row has a usable fund-receipt date.
pub fn remittance_only_unproven(results: &[Assessed]) -> Result<bool> {
    let mut assessable = Vec::new();
    for r in results {
        if r.verdict != SKIPPED && r.line.sg_amount.gt(&zero())? {
            assessable.push(r);
        }
    }
    if assessable.is_empty() {
        return Ok(false);
    }
    Ok(!assessable.iter().any(|r| r.receipt_established))
}

/// True where the run must not exit 0.
pub fn needs_attention(results: &[Assessed], remittance_only_confirmed: bool) -> Result<bool> {
    if results.iter().any(Assessed::is_exposed) || !horizon_indeterminate(results).is_empty() {
        return Ok(true);
    }
    Ok(remittance_only_unproven(results)? && !remittance_only_confirmed)
}

pub const CSV_HEADER: [&str; 18] = [
    "row",
    "employee_id",
    "qe_day",
    "pathway",
    "due_date",
    "verdict",
    "days_late",
    "lateness_measured_to",
    "sg_amount",
    "final_shortfall",
    "notional_earnings",
    "uplift_best_case",
    "uplift_worst_case",
    "sgc_estimate_low",
    "sgc_estimate_high",
    "caveats",
    "notes",
    "unassessable_between",
];

/// Each component rounded once to cents, and totals built from the rounded parts.
#[derive(Clone, Debug, Default)]
pub struct Figures {
    pub shortfall: Option<Decimal>,
    pub nec: Option<Decimal>,
    pub up_low: Option<Decimal>,
    pub up_high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub high: Option<Decimal>,
}

impl Figures {
    pub fn get(&self, key: &str) -> Option<&Decimal> {
        match key {
            "shortfall" => self.shortfall.as_ref(),
            "nec" => self.nec.as_ref(),
            "up_low" => self.up_low.as_ref(),
            "up_high" => self.up_high.as_ref(),
            "low" => self.low.as_ref(),
            _ => self.high.as_ref(),
        }
    }
}

fn assertion(message: &str) -> Failure {
    Failure::Uncaught(format!("AssertionError: {message}"))
}

/// Round each component once, then build the totals from the rounded parts.
pub fn rounded_figures(r: &Assessed) -> Result<Figures> {
    let Some(final_shortfall) = &r.final_shortfall else {
        return Ok(Figures::default());
    };
    let shortfall = cents(Some(final_shortfall))?;
    if r.nec.is_none() && r.uplift.is_none() && r.sgc_low.is_none() && r.sgc_high.is_none() {
        return Ok(Figures {
            shortfall: Some(shortfall),
            ..Figures::default()
        });
    }
    let (Some(nec), Some(uplift), Some(_), Some(_)) = (&r.nec, &r.uplift, &r.sgc_low, &r.sgc_high)
    else {
        return Err(assertion("exposed result has a partial SG charge estimate"));
    };
    let nec = cents(Some(nec))?;
    let up_low = cents(Some(scenario(uplift, "clean_history", "vds_within_30d")))?;
    let up_high = cents(Some(scenario(uplift, "prior_history", "no_vds")))?;
    Ok(Figures {
        low: Some(shortfall.add(&nec)?.add(&up_low)?),
        high: Some(shortfall.add(&nec)?.add(&up_high)?),
        shortfall: Some(shortfall),
        nec: Some(nec),
        up_low: Some(up_low),
        up_high: Some(up_high),
    })
}

fn exposure_figures(r: &Assessed) -> Result<Figures> {
    let figures = rounded_figures(r)?;
    if figures.shortfall.is_none() {
        return Err(assertion(
            "exposed result has no final shortfall, so its exposure figures are incomplete",
        ));
    }
    Ok(figures)
}

fn supported_due(r: &Assessed) -> Result<Date> {
    r.deadline
        .due
        .ok_or_else(|| assertion("listed result has no supported deadline"))
}

/// What the trailing NOTE row says about the run.
pub struct NoteFacts<'a> {
    pub as_at: Date,
    pub law_date: &'a str,
    pub assessment_date: Option<Date>,
    pub source: Option<&'a str>,
    pub gic_provenance: &'a str,
}

/// The report CSV text, without the BOM the file carries.
pub fn render_csv(
    results: &[Assessed],
    facts: &NoteFacts,
    include_employee_ids: bool,
) -> Result<String> {
    let columns: Vec<usize> = (0..CSV_HEADER.len())
        .filter(|&i| include_employee_ids || CSV_HEADER[i] != "employee_id")
        .collect();
    let mut out = String::new();
    let mut write_row = |values: Vec<String>| {
        let picked: Vec<String> = columns.iter().map(|&i| values[i].clone()).collect();
        csvwrite::row(&picked, &mut out);
    };
    write_row(CSV_HEADER.iter().map(|s| s.to_string()).collect());
    for r in results {
        let figures = rounded_figures(r)?;
        write_row(vec![
            r.line.row.to_string(),
            csv_safe(&r.line.employee_id),
            r.line.qe_day.isoformat(),
            r.deadline.pathway.clone(),
            r.deadline.due.map(|d| d.isoformat()).unwrap_or_default(),
            r.verdict.clone(),
            r.days_late.map(|d| d.to_string()).unwrap_or_default(),
            r.lateness_basis.clone(),
            money(Some(&r.line.sg_amount))?,
            money(figures.shortfall.as_ref())?,
            money(figures.nec.as_ref())?,
            money(figures.up_low.as_ref())?,
            money(figures.up_high.as_ref())?,
            money(figures.low.as_ref())?,
            money(figures.high.as_ref())?,
            csv_safe(&r.caveats.join(" | ")),
            csv_safe(&r.notes.join(" | ")),
            r.horizon_verdicts
                .as_ref()
                .map(|(a, b)| format!("{a} or {b}"))
                .unwrap_or_default(),
        ]);
    }
    let mut note = vec![String::new(); CSV_HEADER.len()];
    note[if include_employee_ids { 1 } else { 0 }] = "NOTE".into();
    let assessment_text = match facts.assessment_date {
        Some(a) => format!("Assessment date {}. ", a.isoformat()),
        None => "No assessment date given, so contributions received late are assumed \
                 to have reached the fund before any assessment. "
            .into(),
    };
    let source = facts
        .source
        .filter(|s| !s.is_empty())
        .map(|s| format!(", source {s}"))
        .unwrap_or_default();
    let provenance = if facts.gic_provenance.is_empty() {
        String::new()
    } else {
        format!("{}. ", facts.gic_provenance)
    };
    let notes_at = CSV_HEADER
        .iter()
        .position(|c| *c == "notes")
        .expect("a notes column");
    note[notes_at] = format!(
        "payday-super-checker {VERSION}{source}, as at {}. {assessment_text}{provenance}\
         Legal content current at {}. EXPERIMENTAL ESTIMATES: monetary \
         components are displayed to cents with ROUND_HALF_UP, while TAA 1953 \
         s 16B only rounds the Commissioner's final assessed SG charge down to \
         the nearest 5 cents. The low estimate assumes a \
         voluntary disclosure lodged within 30 days of the payday and a clean \
         24-month history; the high estimate assumes neither. Estimates exclude \
         choice loading, the maximum contributions base and post-assessment \
         penalties. Educational tool, not advice: the ATO assesses the charge.",
        facts.as_at.isoformat(),
        facts.law_date
    );
    write_row(note);
    Ok(out)
}

/// `int(mcb)` with Python's arbitrary precision: TypeError and ValueError
/// give the unreadable text; an infinite float raises OverflowError uncaught.
fn cap_text(entry: Option<&Raw>, fy_label: &str) -> Result<String> {
    use num_traits::FromPrimitive;
    let mcb = entry.and_then(|e| e.get("max_contributions_base"));
    let whole: Option<num_bigint::BigInt> = match mcb {
        None | Some(Raw::None) => {
            return Ok(format!(
                "not on record for {fy_label}: add max_contributions_base for that year \
                 to data/rates.json"
            ));
        }
        Some(Raw::Int(digits)) => digits.parse().ok(),
        Some(Raw::Bool(b)) => Some(u8::from(*b).into()),
        Some(Raw::Str(s)) => crate::py::parse_int(s, 10).ok(),
        Some(Raw::Float(f)) if f.is_infinite() => {
            return Err(Failure::Uncaught(
                "OverflowError: cannot convert float infinity to integer".into(),
            ));
        }
        Some(Raw::Float(f)) => num_bigint::BigInt::from_f64(f.trunc()),
        Some(_) => None,
    };
    Ok(match whole {
        Some(n) => format!("${} for {fy_label}, annual per employer", grouped(&n)),
        None => format!(
            "unreadable for {fy_label}: data/rates.json holds \
             max_contributions_base {}, which is not a whole number of dollars",
            mcb.expect("present").py_repr()
        ),
    })
}

/// `f"{n:,}"`.
fn grouped(n: &num_bigint::BigInt) -> String {
    let negative = n.sign() == num_bigint::Sign::Minus;
    let digits = n.magnitude().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if negative { format!("-{out}") } else { out }
}

/// The run's options the console summary repeats.
pub struct SummaryFacts<'a> {
    pub as_at: Date,
    pub csv_path: &'a str,
    pub law_date: &'a str,
    pub rates: &'a Raw,
    pub assessment_date: Option<Date>,
    pub remittance_only_confirmed: bool,
}

fn exposure_high(r: &Assessed) -> Decimal {
    r.sgc_high.clone().unwrap_or_else(|| Decimal::from_int(-1))
}

pub fn console_summary(results: &[Assessed], facts: &SummaryFacts) -> Result<String> {
    let csv_path = facts.csv_path;
    let count = |v: &str| results.iter().filter(|r| r.verdict == v).count();
    let mut lines = vec![
        format!(
            "payday-super-checker: {} contribution lines, as at {}",
            results.len(),
            facts.as_at.isoformat()
        ),
        String::new(),
        format!(
            "  {}",
            VERDICTS
                .iter()
                .map(|v| format!("{v}: {}", count(v)))
                .collect::<Vec<_>>()
                .join("  ")
        ),
        String::new(),
    ];

    let mut exposed: Vec<&Assessed> = results.iter().filter(|r| r.is_exposed()).collect();
    if !exposed.is_empty() {
        let mut keyed: Vec<(Decimal, &Assessed)> =
            exposed.iter().map(|r| (exposure_high(r), *r)).collect();
        let mut failure = None;
        keyed.sort_by(|a, b| match b.0.compare(&a.0) {
            Ok(o) => o,
            Err(e) => {
                failure = Some(e);
                Ordering::Equal
            }
        });
        if let Some(e) = failure {
            return Err(e.into());
        }
        exposed = keyed.into_iter().map(|(_, r)| r).collect();
        lines.push("Lines with exposure (experimental estimates, largest first):".into());
        for r in exposed.iter().take(10) {
            let figures = exposure_figures(r)?;
            let due = supported_due(r)?;
            let lateness = match r.days_late {
                Some(d) => format!("{d} days late to {}", r.lateness_basis),
                None => format!(
                    "days late not pinned down, measured to {}",
                    r.lateness_basis
                ),
            };
            lines.push(format!(
                "  row {}  QE day {}  due {}  {}, {lateness}",
                r.line.row,
                r.line.qe_day.isoformat(),
                due.isoformat(),
                r.verdict
            ));
            let shortfall = figures.shortfall.as_ref();
            let shortfall_text = if r.offset_s18d && shortfall.is_some_and(Decimal::is_zero) {
                format!(
                    "super ${} (received, so the shortfall is nil)",
                    money(Some(&r.line.sg_amount))?
                )
            } else if r.offset_s18d {
                format!(
                    "shortfall ${} (partially reduced by a late fund receipt under s 18D)",
                    money(shortfall)?
                )
            } else {
                format!("shortfall ${}", money(shortfall)?)
            };
            let at_most = if r.days_late.is_some() {
                ""
            } else {
                "at most "
            };
            if r.sgc_high.is_some() {
                lines.push(format!(
                    "      {shortfall_text}  notional earnings {at_most}${}  \
                     experimental SG charge estimate {at_most}${} - ${}",
                    money(figures.nec.as_ref())?,
                    money(figures.low.as_ref())?,
                    money(figures.high.as_ref())?
                ));
            } else {
                lines.push(format!(
                    "      {shortfall_text}  notional earnings and SG charge estimate \
                     not assessed"
                ));
            }
            for caveat in &r.caveats {
                lines.push(format!("      note: {caveat}"));
            }
        }
        if exposed.len() > 10 {
            lines.push(format!(
                "  ... and {} more (see {csv_path})",
                exposed.len() - 10
            ));
        }
        let total = |rows: &[&Assessed], key: &str| -> Result<Decimal> {
            let mut sum = zero();
            for r in rows {
                sum = sum.add(&cents(exposure_figures(r)?.get(key))?)?;
            }
            Ok(sum)
        };
        let estimated: Vec<&Assessed> = exposed
            .iter()
            .copied()
            .filter(|r| r.sgc_high.is_some())
            .collect();
        lines.push(String::new());
        lines.push(format!(
            "  Total across {} line(s): shortfall ${}, notional earnings ${},",
            exposed.len(),
            money(Some(&total(&exposed, "shortfall")?))?,
            money(Some(&total(&estimated, "nec")?))?
        ));
        lines.push(format!(
            "  experimental estimated SG charge ${} - ${}.",
            money(Some(&total(&estimated, "low")?))?,
            money(Some(&total(&estimated, "high")?))?
        ));
        let unassessed = exposed.len() - estimated.len();
        if unassessed > 0 {
            lines.push(format!(
                "  {unassessed} of those line(s) carry no charge estimate, so the \
                 notional earnings and SG charge totals cover the rest. See those \
                 rows' notes."
            ));
        }
        lines.push(String::new());
    }

    if remittance_only_unproven(results)? {
        lines.push(if facts.remittance_only_confirmed {
            "Operator confirmed remittance-only review: no in-scope positive row \
             has a fund-receipt date on or before the as-at date, so this file \
             cannot produce ON_TIME. The confirmation is recorded; fill \
             fund_received_date from the clearing house or fund before treating \
             a verdict as final."
                .into()
        } else {
            "This file cannot produce ON_TIME: no in-scope positive row has a \
             fund-receipt date on or before the as-at date. Fill \
             fund_received_date from the clearing house or fund, then rerun. To \
             accept remittance-only AT_RISK results after that gap is understood, \
             pass --confirm-remittance-only. No payroll payment, lodgment or \
             accounting decision is made by this tool."
                .into()
        });
        lines.push(String::new());
    }

    let at_risk: Vec<&Assessed> = results.iter().filter(|r| r.verdict == AT_RISK).collect();
    if !at_risk.is_empty() {
        lines.push(format!(
            "{} line(s) remitted by the deadline but with no fund-receipt \
             date. The statutory timing test turns on receipt by the fund, not the \
             day you paid, and clearing-house transit time is the employer's risk.",
            at_risk.len()
        ));
        let flagged: Vec<(&Assessed, Vec<&String>)> = at_risk
            .iter()
            .map(|r| {
                (
                    *r,
                    r.caveats
                        .iter()
                        .filter(|c| *c != NO_RECEIPT_CAVEAT)
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, others)| !others.is_empty())
            .collect();
        for (r, others) in flagged.iter().take(10) {
            let due = supported_due(r)?;
            lines.push(format!(
                "  row {}  QE day {}  due {}",
                r.line.row,
                r.line.qe_day.isoformat(),
                due.isoformat()
            ));
            for caveat in others {
                lines.push(format!("      note: {caveat}"));
            }
        }
        if flagged.len() > 10 {
            lines.push(format!(
                "  ... and {} more at-risk line(s) with notes (see {csv_path})",
                flagged.len() - 10
            ));
        }
        lines.push(String::new());
    }

    let indeterminate = horizon_indeterminate(results);
    if !indeterminate.is_empty() {
        lines.push(format!(
            "{} line(s) cannot be assessed from the supplied \
             deadline facts. Each one is between the two verdicts below and needs \
             reconciliation before it can be treated as clear.",
            indeterminate.len()
        ));
        for r in indeterminate.iter().take(10) {
            let (worse, better) = r.horizon_verdicts.as_ref().expect("indeterminate");
            let due = supported_due(r)?;
            lines.push(format!(
                "  row {}  QE day {}  due {}  super ${}  {worse} or {better}",
                r.line.row,
                r.line.qe_day.isoformat(),
                due.isoformat(),
                money(Some(&r.line.sg_amount))?
            ));
            for caveat in &r.caveats {
                lines.push(format!("      note: {caveat}"));
            }
        }
        if indeterminate.len() > 10 {
            lines.push(format!(
                "  ... and {} more unassessable line(s) (see {csv_path})",
                indeterminate.len() - 10
            ));
        }
        lines.push(String::new());
    }

    let unflagged = results
        .iter()
        .filter(|r| {
            !r.is_exposed()
                && r.verdict != AT_RISK
                && r.horizon_verdicts.is_none()
                && !r.caveats.is_empty()
        })
        .count();
    if unflagged > 0 {
        lines.push(format!(
            "{unflagged} other line(s) carry data-quality notes; see the caveats \
             column in {csv_path}."
        ));
        lines.push(String::new());
    }

    let empty = Raw::Dict(Vec::new());
    let fy = facts.rates.get("financial_years").unwrap_or(&empty);
    let mut labels: Vec<String> = results
        .iter()
        .map(|r| financial_year(r.line.qe_day))
        .collect();
    labels.sort();
    labels.dedup();
    if labels.is_empty() {
        labels.push(financial_year(facts.as_at));
    }
    let mcb_text = labels
        .iter()
        .map(|label| cap_text(fy.get(label), label))
        .collect::<Result<Vec<_>>>()?
        .join("; ");
    let assessment_line = match facts.assessment_date {
        Some(a) => format!(
            "  - Assessment date {}: only contributions received \
             before it clear the shortfall, and notional earnings stop the day before.",
            a.isoformat()
        ),
        None => "  - No assessment date given, so a late contribution that reached the fund \
                 is assumed to have done so before any assessment (s 18D)."
            .into(),
    };
    lines.extend([
        "Assumptions and limits:".to_string(),
        format!(
            "  - Legal content current at {}. LCR 2026/1, LCR 2026/2 and \
             LCR 2026/3 were issued on 5 Aug 2026. LCR 2026/D1 remains a draft \
             pending the appeal from Department of Education v Commissioner of \
             Taxation [2026] FCA 898.",
            facts.law_date
        ),
        "  - The amount column must be the operator-determined super guarantee amount \
         after applying the employee/payment boundaries in regulations 11 and 12, \
         qualifying earnings and other applicable limits. Salary sacrifice and \
         additional contributions must be filtered out. This tool does not make those \
         classifications; LCR 2026/D1 also remains draft."
            .into(),
        "  - Deadlines use the national business-day calendar in SGAA s 6(1): \
         weekends plus holidays applying to the whole of any State, the ACT or the NT."
            .into(),
        assessment_line,
        "  - Deadline alignment under s 18C(2) item 4 is applied only where an \
         earlier row evidences an eligible contribution received by the fund, \
         allocated to that QE day and on time. Include each employee's earlier \
         paydays and reconcile the statutory allocation before treating an item 4 \
         extension as settled."
            .into(),
        "  - The low estimate assumes a voluntary disclosure lodged within 30 days of \
         the payday and a clean 24-month history; the high estimate assumes neither. \
         Choice loading, the late payment penalty and interest on an unpaid assessment \
         are not included. The ATO assesses the charge."
            .into(),
        "  - Exposure figures are EXPERIMENTAL ESTIMATES. This tool displays each \
         component to cents with ROUND_HALF_UP so report columns add up. LCR 2026/3 \
         confirms only that TAA 1953 s 16B reduces the Commissioner's final assessed \
         SG charge to the nearest 5 cents; it does not authorise per-line cents \
         rounding here."
            .into(),
        format!(
            "  - Maximum contributions base ({mcb_text}) is \
             not applied: it needs each employee's cumulative earnings for the year. \
             High earners may show a larger shortfall here than the law requires."
        ),
        "  - PCG 2026/1 sets the ATO's compliance approach for QE days to 30 Jun 2027. \
         Fixing a late contribution promptly lowers ATO review risk; it does not \
         remove the liability."
            .into(),
        "  - This tool tests the SG charge only. Fund deeds, enterprise agreements and \
         awards can require earlier payment."
            .into(),
        String::new(),
        format!("Full detail written to {csv_path}"),
        String::new(),
        "Educational tool, not advice. Check anything material against the ATO's own \
         guidance and calculators."
            .into(),
    ]);
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    //! The Python tests that build results by hand: the figure invariants and
    //! the console's at-risk block.

    use super::*;
    use crate::assess::LATE;
    use crate::assess::tests::{day, dec, result};
    use crate::sgc::uplift_scenarios;

    fn complete() -> Assessed {
        let mut r = result(day(2026, 7, 20), LATE);
        r.line.employee_id = "FABRICATED".into();
        r.line.qe_day = day(2026, 7, 9);
        r.line.sg_amount = dec("100");
        r.final_shortfall = Some(dec("100"));
        r.nec = Some(dec("1"));
        r.uplift = Some(uplift_scenarios(&dec("100"), &dec("1")).unwrap());
        r.sgc_low = Some(dec("101"));
        r.sgc_high = Some(dec("161.60"));
        r
    }

    fn shown(figures: &Figures) -> Vec<Option<String>> {
        ["shortfall", "nec", "up_low", "up_high", "low", "high"]
            .iter()
            .map(|k| figures.get(k).map(Decimal::to_sci_string))
            .collect()
    }

    fn summary(results: &[Assessed], law_date: &str) -> Result<String> {
        let rates = crate::rates::load_rates().unwrap();
        console_summary(
            results,
            &SummaryFacts {
                as_at: day(2026, 8, 10),
                csv_path: "report.csv",
                law_date,
                rates: &rates,
                assessment_date: None,
                remittance_only_confirmed: false,
            },
        )
    }

    #[test]
    fn report_invariants_hold_whatever_the_build() {
        let figures = shown(&rounded_figures(&complete()).unwrap());
        let want: Vec<Option<String>> = ["100.00", "1.00", "0.00", "60.60", "101.00", "161.60"]
            .iter()
            .map(|v| Some((*v).to_string()))
            .collect();
        assert_eq!(figures, want);

        let mut unavailable = complete();
        (
            unavailable.nec,
            unavailable.uplift,
            unavailable.sgc_low,
            unavailable.sgc_high,
        ) = (None, None, None, None);
        let figures = shown(&rounded_figures(&unavailable).unwrap());
        assert_eq!(figures[0].as_deref(), Some("100.00"));
        assert!(figures[1..].iter().all(Option::is_none));

        for blank in ["nec", "uplift", "sgc_low", "sgc_high"] {
            let mut partial = complete();
            match blank {
                "nec" => partial.nec = None,
                "uplift" => partial.uplift = None,
                "sgc_low" => partial.sgc_low = None,
                _ => partial.sgc_high = None,
            }
            let err = rounded_figures(&partial).unwrap_err();
            assert_eq!(
                err.traceback_line(),
                "AssertionError: exposed result has a partial SG charge estimate"
            );
        }

        let mut no_shortfall = complete();
        no_shortfall.final_shortfall = None;
        let err = summary(&[no_shortfall], "fixture").unwrap_err();
        assert_eq!(
            err.traceback_line(),
            "AssertionError: exposed result has no final shortfall, so its exposure figures \
             are incomplete"
        );
    }

    #[test]
    fn the_console_refuses_an_exposed_result_without_exposure_figures() {
        let mut r = result(day(2026, 7, 20), LATE);
        r.line.qe_day = day(2026, 7, 9);
        r.line.sg_amount = dec("100.00");
        let err = summary(&[r], "2026-08-15").unwrap_err();
        assert!(err.traceback_line().contains("exposure figures"));
    }

    fn at_risk(n: i64, caveats: &[String]) -> Assessed {
        let mut r = result(day(2026, 7, 20), AT_RISK);
        r.line.employee_id = format!("ARK{n:02}");
        r.line.qe_day = day(2026, 7, 9);
        r.line.sg_amount = dec("100.00");
        r.line.remitted = Some(day(2026, 7, 15));
        r.line.row = n + 1;
        r.caveats = std::iter::once(NO_RECEIPT_CAVEAT.to_string())
            .chain(caveats.iter().cloned())
            .collect();
        r
    }

    #[test]
    fn the_at_risk_block_names_every_row_and_every_caveat_it_prints() {
        let results: Vec<Assessed> = (1..13)
            .map(|n| {
                at_risk(
                    n,
                    &[format!("alpha marker {n}"), format!("beta marker {n}")],
                )
            })
            .collect();
        let text = summary(&results, "2026-08-02").unwrap();
        for n in 1..11 {
            assert!(text.contains(&format!(
                "  row {}  QE day 2026-07-09  due 2026-07-20",
                n + 1
            )));
            assert!(!text.contains(&format!("ARK{n:02}")));
            assert!(text.contains(&format!("      note: alpha marker {n}")));
            assert!(text.contains(&format!("      note: beta marker {n}")));
        }
        for n in [11, 12] {
            assert!(!text.contains(&format!("ARK{n}")));
            assert!(!text.contains(&format!("alpha marker {n}")));
            assert!(!text.contains(&format!("beta marker {n}")));
        }
        assert!(text.contains("... and 2 more at-risk line(s) with notes"));
        assert_eq!(text.matches("      note: ").count(), 20);
        assert_eq!(
            text.matches("QE day 2026-07-09  due 2026-07-20").count(),
            10
        );
    }

    #[test]
    fn the_at_risk_truncation_notice_counts_only_flagged_rows() {
        let mut results: Vec<Assessed> = (1..12)
            .map(|n| at_risk(n, &[format!("alpha marker {n}")]))
            .collect();
        results.extend((20..30).map(|n| at_risk(n, &[])));
        let text = summary(&results, "2026-08-02").unwrap();
        assert!(text.contains("21 line(s) remitted by the deadline"));
        assert!(text.contains("... and 1 more at-risk line(s) with notes"));
    }
}
