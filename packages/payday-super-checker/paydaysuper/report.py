"""Verdicts, exposure figures, console summary and report.csv."""
from __future__ import annotations

import csv
import io
from datetime import date
from decimal import Decimal
from pathlib import Path
from typing import TextIO

from . import __version__
from .assess import (  # noqa: F401
    AT_RISK,
    EXPOSED,
    LATE,
    NO_RECEIPT_CAVEAT,
    ON_TIME,
    SKIPPED,
    TRANSITION_END,
    UNKNOWN,
    UNPAID,
    VERDICTS,
    Result,
    assess,
    financial_year,
)
from .atomic_io import atomic_text_output
from .csv_io import cents, csv_safe, money


def horizon_indeterminate(results: list[Result]) -> list[Result]:
    """Rows whose verdict could not be decided from the supplied deadline facts.

    This includes calendar-horizon gaps and an unevidenced item 4 extension.
    These drive the same non-zero exit code LATE does. A run that cannot tell
    whether a 9,000 shortfall exists has not found nothing."""
    return [r for r in results if r.horizon_verdicts is not None]


def remittance_only_unproven(results: list[Result]) -> bool:
    """True when no in-scope positive row has a usable fund-receipt date.

    Vendor imports never write ``fund_received_date``, so a fully remitted
    file is all ``AT_RISK`` and cannot produce ``ON_TIME``. Defined-benefit
    rows and nil amounts are not lateness-tested, so they do not count. A
    file with no assessable rows is not remittance-only.

    The test is ``receipt_established``, not the raw column: a receipt dated
    after the as-at date is discarded by the run that reads it, so the row
    still cannot produce ``ON_TIME`` and the file is still remittance-only.
    Reading the column instead let a mid-period run whose receipts all
    post-date ``--as-at`` exit 0 while its own per-row caveat said the
    statutory test of receipt by the fund was not met.
    """
    assessable = [
        r
        for r in results
        if r.verdict != SKIPPED and r.line.sg_amount > 0
    ]
    if not assessable:
        return False
    return not any(r.receipt_established for r in assessable)


def needs_attention(
    results: list[Result], *, remittance_only_confirmed: bool = False
) -> bool:
    """True where the run must not exit 0: real exposure, a line the
    supplied deadline facts could not decide, or a file that cannot
    produce ON_TIME because no fund-receipt date was supplied."""
    if any(r.verdict in EXPOSED for r in results) or bool(
        horizon_indeterminate(results)
    ):
        return True
    return remittance_only_unproven(results) and not remittance_only_confirmed


CSV_HEADER = [
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
    # Appended, never inserted: a positional consumer keeps its column
    # numbers. Blank on every row except the ones the calendar cannot settle,
    # where it holds conservative outer verdicts as "WORSE or BETTER". A
    # caveat may retain an intermediate third outcome for a partial receipt.
    # Without
    # it the CSV wrote UNKNOWN for a 9,000 contribution nobody can assess and
    # UNKNOWN for a nil row with nothing to assess, with the same blank
    # shortfall on both, so anyone reading the file rather than the console or
    # the exit code could not tell real exposure from nothing at all.
    "unassessable_between",
]


def _rounded_figures(r: Result) -> dict[str, Decimal | None]:
    """Round each component once, then build the totals from the rounded
    parts so the columns of a row always add up."""
    blank: dict[str, Decimal | None] = {
        k: None for k in ("shortfall", "nec", "up_low", "up_high", "low", "high")
    }
    if r.final_shortfall is None:
        return blank
    shortfall = cents(r.final_shortfall)
    raw_nec = r.nec
    uplift = r.uplift
    charge = (raw_nec, uplift, r.sgc_low, r.sgc_high)
    if all(value is None for value in charge):
        # An exposed row whose notional earnings period reached past the
        # published GIC quarters. The shortfall was established from the
        # deadline and the receipt facts, so it is reported; the figures that
        # compound a rate are left empty rather than estimated.
        return {**blank, "shortfall": shortfall}
    # The 4 stand or fall together. A partial set is a broken result, and
    # rendering it would print an estimate built from a missing component.
    assert raw_nec is not None and uplift is not None, (
        "exposed result has a partial SG charge estimate"
    )
    assert r.sgc_low is not None and r.sgc_high is not None, (
        "exposed result has a partial SG charge estimate"
    )
    nec = cents(raw_nec)
    up_low = cents(uplift["clean_history"]["vds_within_30d"])
    up_high = cents(uplift["prior_history"]["no_vds"])
    return {
        "shortfall": shortfall,
        "nec": nec,
        "up_low": up_low,
        "up_high": up_high,
        "low": shortfall + nec + up_low,
        "high": shortfall + nec + up_high,
    }


def _exposure_figures(r: Result) -> dict[str, Decimal | None]:
    """Return display figures for a result already classified as exposed.

    The shortfall is always there: an exposed verdict is reached from the
    deadline and the receipt facts. The 5 charge components are None together
    where the notional earnings period reached past the published GIC
    quarters, which is the one case an exposed row carries no estimate.
    """
    figures = _rounded_figures(r)
    assert figures["shortfall"] is not None, (
        "exposed result has no final shortfall, so its exposure figures are incomplete"
    )
    return figures


def _charge_assessed(r: Result) -> bool:
    """Whether this exposed row carries an SG charge estimate at all."""
    return r.sgc_high is not None


def _exposure_high(r: Result) -> Decimal:
    """Ordering key for the exposure list, highest estimate first.

    A row with no estimate sorts below every row that has one rather than
    dropping out of the list: its shortfall is still exposure, and its caveat
    is still owed to the reader.
    """
    return r.sgc_high if r.sgc_high is not None else Decimal("-1")


def _supported_due(r: Result) -> date:
    """Return the deadline required by every result listed in the console."""
    due = r.deadline.due
    assert due is not None, "listed result has no supported deadline"
    return due


def write_csv(
    results: list[Result],
    path: str | Path,
    as_at: date,
    law_date: str,
    assessment_date: date | None = None,
    source: str | Path | None = None,
    gic_provenance: str = "",
) -> None:
    # utf-8-sig keeps Excel from decoding non-ASCII employee references as cp1252.
    with atomic_text_output(path, encoding="utf-8-sig") as stream:
        _write_csv_rows(
            results, stream, as_at, law_date, assessment_date, source, gic_provenance,
        )


def render_csv(
    results: list[Result],
    as_at: date,
    law_date: str,
    assessment_date: date | None = None,
    source: str | Path | None = None,
    gic_provenance: str = "",
    *,
    include_employee_ids: bool = True,
) -> str:
    with io.StringIO(newline="") as stream:
        _write_csv_rows(
            results, stream, as_at, law_date, assessment_date, source, gic_provenance,
            include_employee_ids=include_employee_ids,
        )
        return stream.getvalue()


def _write_csv_rows(
    results: list[Result],
    f: TextIO,
    as_at: date,
    law_date: str,
    assessment_date: date | None = None,
    source: str | Path | None = None,
    gic_provenance: str = "",
    *,
    include_employee_ids: bool = True,
) -> None:
    writer = csv.writer(f)
    columns = [
        i for i, name in enumerate(CSV_HEADER)
        if include_employee_ids or name != "employee_id"
    ]

    def write_row(values: list) -> None:
        writer.writerow([values[i] for i in columns])

    write_row(CSV_HEADER)
    for r in results:
        figures = _rounded_figures(r)
        write_row(
            [
                r.line.row,
                csv_safe(r.line.employee_id),
                r.line.qe_day.isoformat(),
                r.deadline.pathway,
                r.deadline.due.isoformat() if r.deadline.due else "",
                r.verdict,
                "" if r.days_late is None else r.days_late,
                r.lateness_basis,
                money(r.line.sg_amount),
                money(figures["shortfall"]),
                money(figures["nec"]),
                money(figures["up_low"]),
                money(figures["up_high"]),
                money(figures["low"]),
                money(figures["high"]),
                " | ".join(r.caveats),
                " | ".join(r.notes),
                " or ".join(r.horizon_verdicts) if r.horizon_verdicts else "",
            ]
        )

    # Trailing note, full width so the file stays a clean table: a
    # one-field title row makes Power Query infer the wrong column count.
    note = [""] * len(CSV_HEADER)
    note[1 if include_employee_ids else 0] = "NOTE"
    assessment_text = (
        f"Assessment date {assessment_date.isoformat()}. "
        if assessment_date is not None
        else "No assessment date given, so contributions received late are assumed "
        "to have reached the fund before any assessment. "
    )
    # By name, not by position: the trailing note belongs in "notes", and
    # note[-1] silently moved it into whatever column was appended last.
    note[CSV_HEADER.index("notes")] = (
        f"payday-super-checker {__version__}"
        + (f", source {source}" if source else "")
        + f", as at {as_at.isoformat()}. {assessment_text}"
        + (f"{gic_provenance}. " if gic_provenance else "")
        + f"Legal content current at {law_date}. EXPERIMENTAL ESTIMATES: monetary "
        "components are displayed to cents with ROUND_HALF_UP, while TAA 1953 "
        "s 16B only rounds the Commissioner's final assessed SG charge down to "
        "the nearest 5 cents. The low estimate assumes a "
        "voluntary disclosure lodged within 30 days of the payday and a clean "
        "24-month history; the high estimate assumes neither. Estimates exclude "
        "choice loading, the maximum contributions base and post-assessment "
        "penalties. Educational tool, not advice: the ATO assesses the charge."
    )
    write_row(note)


def console_summary(
    results: list[Result],
    as_at: date,
    csv_path: str | Path,
    law_date: str,
    rates: dict,
    assessment_date: date | None = None,
    remittance_only_confirmed: bool = False,
) -> str:
    counts: dict[str, int] = {}
    for r in results:
        counts[r.verdict] = counts.get(r.verdict, 0) + 1

    lines = [
        f"payday-super-checker: {len(results)} contribution lines, as at {as_at.isoformat()}",
        "",
        "  " + "  ".join(f"{v}: {counts.get(v, 0)}" for v in VERDICTS),
        "",
    ]

    exposed = [r for r in results if r.verdict in EXPOSED]
    if exposed:
        exposed.sort(key=_exposure_high, reverse=True)
        lines.append("Lines with exposure (experimental estimates, largest first):")
        for r in exposed[:10]:
            figures = _exposure_figures(r)
            due = _supported_due(r)
            # Standard output is commonly retained by task runners and CI logs.
            # The report CSV is the private, row-level artefact; retain the source
            # row here so an operator can locate the result without leaking an
            # employee identifier into those logs. Days late is left unset where
            # the deadline runs past the calendar's coverage, so never print
            # "None days late" in that case.
            lateness = (
                f"{r.days_late} days late to {r.lateness_basis}"
                if r.days_late is not None
                else f"days late not pinned down, measured to {r.lateness_basis}"
            )
            lines.append(
                f"  row {r.line.row}  QE day {r.line.qe_day.isoformat()}"
                f"  due {due.isoformat()}  {r.verdict}, {lateness}"
            )
            shortfall_text = (
                f"super ${money(r.line.sg_amount)} (received, so the shortfall is nil)"
                if r.offset_s18d and figures["shortfall"] == 0
                else (
                    f"shortfall ${money(figures['shortfall'])} "
                    "(partially reduced by a late fund receipt under s 18D)"
                    if r.offset_s18d
                    else f"shortfall ${money(figures['shortfall'])}"
                )
            )
            at_most = "" if r.days_late is not None else "at most "
            if _charge_assessed(r):
                lines.append(
                    f"      {shortfall_text}  notional earnings {at_most}"
                    f"${money(figures['nec'])}  experimental SG charge estimate {at_most}"
                    f"${money(figures['low'])} - ${money(figures['high'])}"
                )
            else:
                lines.append(
                    f"      {shortfall_text}  notional earnings and SG charge estimate "
                    "not assessed"
                )
            for caveat in r.caveats:
                lines.append(f"      note: {caveat}")
        if len(exposed) > 10:
            lines.append(f"  ... and {len(exposed) - 10} more (see {csv_path})")

        def total(rows: list[Result], key: str) -> Decimal:
            return sum(
                (cents(_exposure_figures(r)[key]) for r in rows), Decimal("0")
            )

        # Every exposed row has a shortfall. Only the rows that carry a charge
        # estimate go into the charge totals, so a row the GIC table could not
        # reach is never added in as a nil estimate.
        estimated = [r for r in exposed if _charge_assessed(r)]
        lines += [
            "",
            f"  Total across {len(exposed)} line(s): shortfall "
            f"${money(total(exposed, 'shortfall'))}, notional earnings "
            f"${money(total(estimated, 'nec'))},",
            f"  experimental estimated SG charge ${money(total(estimated, 'low'))} - "
            f"${money(total(estimated, 'high'))}.",
        ]
        unassessed = len(exposed) - len(estimated)
        if unassessed:
            lines.append(
                f"  {unassessed} of those line(s) carry no charge estimate, so the "
                "notional earnings and SG charge totals cover the rest. See those "
                "rows' notes."
            )
        lines.append("")

    if remittance_only_unproven(results):
        if remittance_only_confirmed:
            lines.append(
                "Operator confirmed remittance-only review: no in-scope positive row "
                "has a fund-receipt date on or before the as-at date, so this file "
                "cannot produce ON_TIME. The confirmation is recorded; fill "
                "fund_received_date from the clearing house or fund before treating "
                "a verdict as final."
            )
        else:
            lines.append(
                "This file cannot produce ON_TIME: no in-scope positive row has a "
                "fund-receipt date on or before the as-at date. Fill "
                "fund_received_date from the clearing house or fund, then rerun. To "
                "accept remittance-only AT_RISK results after that gap is understood, "
                "pass --confirm-remittance-only. No payroll payment, lodgment or "
                "accounting decision is made by this tool."
            )
        lines.append("")

    at_risk = [r for r in results if r.verdict == AT_RISK]
    if at_risk:
        lines.append(
            f"{len(at_risk)} line(s) remitted by the deadline but with no fund-receipt "
            "date. The statutory timing test turns on receipt by the fund, not the "
            "day you paid, and clearing-house transit time is the employer's risk."
        )
        # An at-risk line is in neither the exposure listing nor the unflagged
        # count, so without this its caveats never reached the console at all
        # -- including the one saying 2 rows are identical and the payday is
        # counted twice, which is the data-quality warning most worth reading.
        #
        # The no-fund-receipt caveat is excluded because EVERY at-risk row
        # carries it and the header above already says it. Listed, it filled
        # the 10-row cap with rows whose only note repeated the header, and
        # truncated away the rows that had something of their own to say.
        flagged = [
            (r, [c for c in r.caveats if c != NO_RECEIPT_CAVEAT]) for r in at_risk
        ]
        flagged = [(r, others) for r, others in flagged if others]
        for r, others in flagged[:10]:
            due = _supported_due(r)
            lines.append(
                f"  row {r.line.row}  QE day {r.line.qe_day.isoformat()}  "
                f"due {due.isoformat()}"
            )
            for caveat in others:
                lines.append(f"      note: {caveat}")
        if len(flagged) > 10:
            lines.append(
                f"  ... and {len(flagged) - 10} more at-risk line(s) with notes "
                f"(see {csv_path})"
            )
        lines.append("")

    # A row left UNKNOWN because a material deadline fact is unresolved is
    # not a data-quality footnote. It may carry the whole shortfall, so it is
    # listed explicitly and drives a non-zero exit.
    indeterminate = horizon_indeterminate(results)
    if indeterminate:
        lines.append(
            f"{len(indeterminate)} line(s) cannot be assessed from the supplied "
            "deadline facts. Each one is between the two verdicts below and needs "
            "reconciliation before it can be treated as clear."
        )
        for r in indeterminate[:10]:
            horizon_verdicts = r.horizon_verdicts
            assert horizon_verdicts is not None, (
                "indeterminate result has no candidate verdicts"
            )
            worse, better = horizon_verdicts
            due = _supported_due(r)
            lines.append(
                f"  row {r.line.row}  QE day {r.line.qe_day.isoformat()}  "
                f"due {due.isoformat()}"
                f"  super ${money(r.line.sg_amount)}  {worse} or {better}"
            )
            for caveat in r.caveats:
                lines.append(f"      note: {caveat}")
        if len(indeterminate) > 10:
            lines.append(
                f"  ... and {len(indeterminate) - 10} more unassessable line(s) "
                f"(see {csv_path})"
            )
        lines.append("")

    unflagged = [
        r
        for r in results
        if r.verdict not in EXPOSED
        and r.verdict != AT_RISK
        and r.horizon_verdicts is None
        and r.caveats
    ]
    if unflagged:
        lines.append(
            f"{len(unflagged)} other line(s) carry data-quality notes; see the caveats "
            f"column in {csv_path}."
        )
        lines.append("")

    fy = rates.get("financial_years", {})
    qe_days = [r.line.qe_day for r in results]
    fy_labels = sorted({financial_year(d) for d in qe_days}) if qe_days else []
    years_to_report = fy_labels or [financial_year(as_at)]
    mcb_parts = []
    for fy_label in years_to_report:
        entry = fy.get(fy_label)
        mcb = (entry or {}).get("max_contributions_base")
        if mcb is None:
            mcb_parts.append(
                f"not on record for {fy_label}: add max_contributions_base for that year to "
                "paydaysuper/data/rates.json"
            )
        else:
            try:
                mcb_parts.append(f"${int(mcb):,} for {fy_label}, annual per employer")
            except (TypeError, ValueError):
                mcb_parts.append(
                    f"unreadable for {fy_label}: paydaysuper/data/rates.json holds "
                    f"max_contributions_base {mcb!r}, which is not a whole number of dollars"
                )
    mcb_text = "; ".join(mcb_parts)
    if len(fy_labels) > 1:
        mcb_text += f" (this file spans {', '.join(fy_labels)})"

    assessment_line = (
        f"  - Assessment date {assessment_date.isoformat()}: only contributions received "
        "before it clear the shortfall, and notional earnings stop the day before."
        if assessment_date is not None
        else "  - No assessment date given, so a late contribution that reached the fund "
        "is assumed to have done so before any assessment (s 18D)."
    )

    lines += [
        "Assumptions and limits:",
        f"  - Legal content current at {law_date}. LCR 2026/1, LCR 2026/2 and "
        "LCR 2026/3 were issued on 5 Aug 2026. LCR 2026/D1 remains a draft "
        "pending the appeal from Department of Education v Commissioner of "
        "Taxation [2026] FCA 898.",
        "  - The amount column must be the operator-determined super guarantee amount "
        "after applying the employee/payment boundaries in regulations 11 and 12, "
        "qualifying earnings and other applicable limits. Salary sacrifice and "
        "additional contributions must be filtered out. This tool does not make those "
        "classifications; LCR 2026/D1 also remains draft.",
        "  - Deadlines use the national business-day calendar in SGAA s 6(1): "
        "weekends plus holidays applying to the whole of any State, the ACT or the NT.",
        assessment_line,
        "  - Deadline alignment under s 18C(2) item 4 is applied only where an "
        "earlier row evidences an eligible contribution received by the fund, "
        "allocated to that QE day and on time. Include each employee's earlier "
        "paydays and reconcile the statutory allocation before treating an item 4 "
        "extension as settled.",
        "  - The low estimate assumes a voluntary disclosure lodged within 30 days of "
        "the payday and a clean 24-month history; the high estimate assumes neither. "
        "Choice loading, the late payment penalty and interest on an unpaid assessment "
        "are not included. The ATO assesses the charge.",
        "  - Exposure figures are EXPERIMENTAL ESTIMATES. This tool displays each "
        "component to cents with ROUND_HALF_UP so report columns add up. LCR 2026/3 "
        "confirms only that TAA 1953 s 16B reduces the Commissioner's final assessed "
        "SG charge to the nearest 5 cents; it does not authorise per-line cents "
        "rounding here.",
        f"  - Maximum contributions base ({mcb_text}) is "
        "not applied: it needs each employee's cumulative earnings for the year. "
        "High earners may show a larger shortfall here than the law requires.",
        "  - PCG 2026/1 sets the ATO's compliance approach for QE days to 30 Jun 2027. "
        "Fixing a late contribution promptly lowers ATO review risk; it does not "
        "remove the liability.",
        "  - This tool tests the SG charge only. Fund deeds, enterprise agreements and "
        "awards can require earlier payment.",
        "",
        f"Full detail written to {csv_path}",
        "",
        "Educational tool, not advice. Check anything material against the ATO's own "
        "guidance and calculators.",
    ]
    return "\n".join(lines)
