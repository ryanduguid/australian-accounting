"""CSV schedule, console summary, and practitioner review pack."""

from __future__ import annotations

import csv
import hashlib
from decimal import Decimal
from io import StringIO
from pathlib import Path
from typing import TextIO

from .atomic_io import atomic_text_writer, atomic_write_text
from .csvsafe import guard
from .model import ContractPosition, Schedule
from .money import money, percent, points
from .schedule import ScheduleError

OUTPUT_COLUMNS = (
    # The reporting date is written on every row so that it sits inside the
    # bytes the review pack hashes and rebuilds. A schedule cannot then be
    # signed off under a different period's header.
    "as_at",
    "contract_id",
    "customer",
    "description",
    "progress_method",
    "original_contract_sum",
    "approved_variations",
    "unapproved_variations_estimate",
    "variable_consideration_included",
    "variable_consideration_excluded",
    "transaction_price",
    "costs_incurred",
    "inefficiency_rework_wastage",
    "uninstalled_materials",
    "progress_cost",
    "estimated_cost_to_complete",
    "estimated_cost_at_completion",
    "progress_eac",
    "percent_complete",
    "revenue_to_date",
    "certified_billings",
    "uncertified_claims",
    "contract_asset",
    "contract_liability",
    "gross_profit_at_completion",
    "margin_at_completion",
    "prior_margin_at_completion",
    "profit_fade_points",
    "period_revenue",
    "retention_withheld",
    "retention_classification",
    "gst_on_certified_billings",
    "gst_on_retention",
    "committed_outstanding",
    "flags",
)

SCHEMA_VERSION = "wip-tally-schedule-v2"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _dec(value: Decimal | None) -> str:
    if value is None:
        return ""
    return format(value, "f")


def _one_line(value: str) -> str:
    """Collapse a ledger-derived value onto a single display line."""
    return " ".join(value.split())


def _md_cell(value: str) -> str:
    r"""Escape a ledger-derived value for one markdown table cell.

    Contract identifiers come from whatever the ledger holds, so the property
    to keep is a round trip: the identifier a reader lifts back out of the
    rendered cell is the identifier the ledger stored. A raw pipe loses that
    round trip in every conforming renderer, and an embedded newline ends the
    table outright.

    Backslashes are escaped first, and the order matters. Escaping only the
    pipe turns a ledger ``JOB\|A`` into ``JOB\\|A``, which round trips nowhere.
    Renderers differ only in how they wreck it, so do not reason from one of
    them: cmark-gfm, which GitHub renders with, keeps four columns but drops
    the backslash and shows ``JOB|A``, while a pair-consuming reader such as
    marked takes the pipe as live and shows ``JOB\`` with the remaining cells
    one heading to the right. Doubling the backslashes first leaves every pipe
    preceded by an odd-length backslash run, and ``JOB\|A`` comes back out of
    both.
    """
    return _one_line(value).replace("\\", "\\\\").replace("|", "\\|")


def _row(position: ContractPosition, as_at: str) -> list[str]:
    contract = position.contract
    return [
        guard(as_at),
        guard(contract.contract_id),
        guard(contract.customer),
        guard(contract.description),
        contract.progress_method,
        _dec(contract.original_contract_sum),
        _dec(contract.approved_variations),
        _dec(contract.unapproved_variations_estimate),
        _dec(position.variable_consideration_included),
        _dec(position.variable_consideration_excluded),
        _dec(position.transaction_price),
        _dec(contract.costs_incurred),
        _dec(contract.inefficiency_rework_wastage),
        _dec(contract.uninstalled_materials),
        _dec(position.progress_cost),
        _dec(contract.estimated_cost_to_complete),
        _dec(position.estimated_cost_at_completion),
        _dec(position.progress_eac),
        _dec(position.percent_complete),
        _dec(position.revenue_to_date),
        _dec(contract.certified_billings),
        _dec(contract.uncertified_claims),
        _dec(position.contract_asset),
        _dec(position.contract_liability),
        _dec(position.gross_profit_at_completion),
        _dec(position.margin_at_completion),
        _dec(position.prior_margin_at_completion),
        _dec(position.profit_fade_points),
        _dec(position.period_revenue),
        _dec(contract.retention_withheld),
        contract.retention_classification,
        _dec(position.gst_on_certified_billings),
        _dec(position.gst_on_retention),
        _dec(contract.committed_outstanding),
        "|".join(position.flags),
    ]


def _write_schedule_csv(handle: TextIO, schedule: Schedule) -> None:
    writer = csv.writer(handle, lineterminator="\n")
    writer.writerow(OUTPUT_COLUMNS)
    for position in schedule.positions:
        writer.writerow(_row(position, schedule.as_at))


def write_schedule_csv(path: Path, schedule: Schedule) -> None:
    with atomic_text_writer(path, encoding="utf-8", newline="") as handle:
        _write_schedule_csv(handle, schedule)


def render_console(schedule: Schedule) -> str:
    lines = [
        f"TheWIPTally: {len(schedule.positions)} contract(s), as at {schedule.as_at}",
        "",
        f"  Contract assets:      {money(schedule.total_contract_assets)}",
        f"  Contract liabilities: {money(schedule.total_contract_liabilities)}",
        f"  Revenue to date:      {money(schedule.total_revenue_to_date)}",
        "",
        "  Assets and liabilities are per contract. They are not netted.",
    ]
    review = schedule.review_rows
    if not review:
        lines.append("")
        lines.append("  No review flags. Practitioner sign-off is still required.")
        return "\n".join(lines) + "\n"

    lines.append("")
    lines.append(f"  Review flags on {len(review)} contract(s):")
    ranked = sorted(
        review,
        key=lambda row: row.profit_fade_points or Decimal("0"),
        reverse=True,
    )
    for position in ranked:
        fade = (
            points(position.profit_fade_points)
            if position.profit_fade_points is not None
            else "n/a"
        )
        margin = (
            percent(position.margin_at_completion)
            if position.margin_at_completion is not None
            else "n/a"
        )
        lines.append(
            f"    {_one_line(position.contract.contract_id)}  "
            f"margin {margin}  fade {fade}"
        )
        lines.append(f"      {', '.join(position.flags)}")
    return "\n".join(lines) + "\n"


def build_review_pack(
    schedule_path: Path,
    source_path: Path | None,
    schedule: Schedule,
) -> str:
    schedule_bytes = schedule_path.read_bytes()
    with StringIO(newline="") as expected:
        _write_schedule_csv(expected, schedule)
        expected_bytes = expected.getvalue().encode("utf-8")
    if schedule_bytes != expected_bytes:
        raise ScheduleError(
            f"{schedule_path} does not match the rebuilt schedule; the source "
            f"file, the mapping and --as-at must be the ones that produced it"
        )

    schedule_hash = hashlib.sha256(schedule_bytes).hexdigest()
    source_hash = sha256_file(source_path) if source_path is not None else "not supplied"
    source_name = source_path.name if source_path is not None else "not supplied"
    lines = [
        "# Construction WIP review pack",
        "",
        "Review aid only. Not accounting, tax or legal advice. Not a journal,",
        "not a lodgment, and not a determination under AASB 15 or AASB 137.",
        "",
        f"- Schema: `{SCHEMA_VERSION}`",
        f"- As at: {schedule.as_at}",
        f"- Source file: `{source_name}`",
        f"- Source SHA-256: `{source_hash}`",
        f"- Schedule file: `{schedule_path.name}`",
        f"- Schedule SHA-256: `{schedule_hash}`",
        f"- Contracts: {len(schedule.positions)}",
        f"- Contract assets (sum, not net): {money(schedule.total_contract_assets)}",
        f"- Contract liabilities (sum, not net): {money(schedule.total_contract_liabilities)}",
        "",
        "## Sign-off",
        "",
        "- [ ] Unit of account (AASB 15 para 17 combination) is accepted",
        "- [ ] Over-time vs point-in-time conclusion sits with the engagement lead",
        "- [ ] Cost to complete is current against programme and commitments",
        "- [ ] Constrained variable consideration is accepted or excluded",
        "- [ ] Retention classification under paras 105-108 is accepted",
        "- [ ] Onerous-contract flags have been run through AASB 137, not booked from this file",
        "- [ ] GST on certified claims, including retention, has been considered for the BAS",
        "- [ ] Tax does not follow this schedule; the ATO long-term construction position is separate",
        "",
        "## Portfolio",
        "",
        "| Side | Amount |",
        "| --- | ---: |",
        f"| Contract assets | {money(schedule.total_contract_assets)} |",
        f"| Contract liabilities | {money(schedule.total_contract_liabilities)} |",
        f"| Revenue to date | {money(schedule.total_revenue_to_date)} |",
        "",
        "Do not offset the two sides.",
        "",
        "## Review queue",
        "",
    ]
    review = schedule.review_rows
    if not review:
        lines.append("No review flags were raised. Sign-off is still required.")
        lines.append("")
    else:
        lines.append("| Contract | Margin | Fade | Flags |")
        lines.append("| --- | --- | --- | --- |")
        for position in review:
            margin = (
                percent(position.margin_at_completion)
                if position.margin_at_completion is not None
                else ""
            )
            fade = (
                points(position.profit_fade_points)
                if position.profit_fade_points is not None
                else ""
            )
            flags = ", ".join(position.flags)
            lines.append(
                f"| {_md_cell(position.contract.contract_id)} | {margin} "
                f"| {fade} | {flags} |"
            )
        lines.append("")
    lines.append("Keep this pack beside the schedule CSV. The CSV is the row-level evidence.")
    lines.append("")
    return "\n".join(lines)


def write_review_pack(path: Path, text: str) -> None:
    atomic_write_text(path, text, encoding="utf-8", newline="\n")
