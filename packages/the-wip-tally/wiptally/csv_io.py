"""Read the contract CSV the schedule consumes."""

from __future__ import annotations

import csv
import json
from collections.abc import Iterable, Iterator
from decimal import Decimal
from pathlib import Path

from .model import (
    PROGRESS_COST_TO_COST,
    PROGRESS_METHODS,
    RETENTION_CLASSES,
    RETENTION_REVIEW,
    ContractInput,
)
from .money import AmountError, parse_bool, parse_money, parse_ratio

CANONICAL_FIELDS = (
    "contract_id",
    "customer",
    "description",
    "original_contract_sum",
    "approved_variations",
    "unapproved_variations_estimate",
    "constraint_include_ratio",
    "costs_incurred",
    "inefficiency_rework_wastage",
    "uninstalled_materials",
    "estimated_cost_to_complete",
    "certified_billings",
    "uncertified_claims",
    "retention_withheld",
    "retention_classification",
    "committed_outstanding",
    "outcome_reasonably_measurable",
    "recoverable_costs",
    "progress_method",
    "output_percent",
    "prior_transaction_price",
    "prior_estimated_cost_at_completion",
    "prior_costs_incurred",
    "prior_estimated_cost_to_complete",
    "prior_revenue_to_date",
    "gst_rate",
    "assets_used_carrying",
)

REQUIRED_FIELDS = (
    "contract_id",
    "original_contract_sum",
    "costs_incurred",
    "estimated_cost_to_complete",
    "certified_billings",
)


class CsvError(ValueError):
    """Raised when the contract file cannot be read as a schedule input."""


def load_mapping(path: Path | None) -> dict[str, str]:
    """Return canonical_field -> source_heading, defaulting to identity."""
    mapping = {field: field for field in CANONICAL_FIELDS}
    if path is None:
        return mapping
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise CsvError(f"cannot read mapping file {path}: {exc}") from exc
    except UnicodeDecodeError as exc:
        raise CsvError(f"mapping file {path} is not UTF-8: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise CsvError(f"mapping file {path} is not JSON: {exc}") from exc
    if not isinstance(payload, dict):
        raise CsvError(f"mapping file {path} must be a JSON object")
    for key, value in payload.items():
        if key not in mapping:
            raise CsvError(f"mapping file names unknown field {key!r}")
        if not isinstance(value, str) or not value.strip():
            raise CsvError(f"mapping for {key} must be a non-empty column heading")
        mapping[key] = value.strip()
    return mapping


def _heading_index(header: list[str]) -> dict[str, int]:
    index: dict[str, int] = {}
    seen: dict[str, str] = {}
    for position, raw in enumerate(header):
        name = raw.strip()
        if not name:
            continue
        key = name.casefold()
        if key in seen:
            raise CsvError(f"duplicated column heading {name!r}")
        seen[key] = name
        index[key] = position
    return index


def _cell(row: list[str], index: dict[str, int], heading: str) -> str | None:
    position = index.get(heading.casefold())
    if position is None:
        return None
    if position >= len(row):
        return None
    value = row[position].strip()
    return value if value else None


def _money_or_zero(
    row: list[str], index: dict[str, int], heading: str, where: str
) -> Decimal:
    raw = _cell(row, index, heading)
    if raw is None:
        return Decimal("0.00")
    return parse_money(raw, where)


def _money_or_none(
    row: list[str], index: dict[str, int], heading: str, where: str
) -> Decimal | None:
    raw = _cell(row, index, heading)
    if raw is None:
        return None
    return parse_money(raw, where)


def _readable_rows(reader: Iterable[list[str]], path: Path) -> Iterator[list[str]]:
    """Yield CSV rows, naming the file when the bytes are not readable as CSV.

    A ledger export saved in the Windows ANSI codepage, or one carrying a field
    larger than the csv module will take, is a data error like any other. It
    reports as one line of `error:`, not as a stack trace.
    """
    try:
        yield from reader
    except UnicodeDecodeError as exc:
        raise CsvError(
            f"cannot read {path}: it is not UTF-8 at byte {exc.start} "
            f"({exc.reason}); re-export it as UTF-8"
        ) from exc
    except csv.Error as exc:
        raise CsvError(f"cannot read {path} as CSV: {exc}") from exc


def read_contracts(path: Path, mapping: dict[str, str]) -> list[ContractInput]:
    try:
        handle = path.open("r", encoding="utf-8-sig", newline="")
    except OSError as exc:
        raise CsvError(f"cannot read {path}: {exc}") from exc

    with handle:
        reader = _readable_rows(csv.reader(handle), path)
        try:
            header = next(reader)
        except StopIteration as exc:
            raise CsvError(f"{path} is empty") from exc
        index = _heading_index(header)
        missing_required = [
            field
            for field in REQUIRED_FIELDS
            if mapping[field].casefold() not in index
        ]
        if missing_required:
            needed = ", ".join(mapping[field] for field in missing_required)
            raise CsvError(f"{path} is missing required column(s): {needed}")

        contracts: list[ContractInput] = []
        seen_ids: dict[str, int] = {}
        errors: list[str] = []
        for line_number, row in enumerate(reader, start=2):
            if not row or all(not cell.strip() for cell in row):
                continue
            try:
                if len(row) != len(header):
                    raise CsvError(
                        f"row {line_number}: has {len(row)} field(s) but the "
                        f"header has {len(header)}; a short row would read its "
                        f"missing trailing columns as absent"
                    )
                contract = _parse_row(row, index, mapping, line_number)
            except (AmountError, CsvError) as exc:
                errors.append(str(exc))
                if len(errors) >= 20:
                    break
                continue
            previous = seen_ids.get(contract.contract_id)
            if previous is not None:
                errors.append(
                    f"row {line_number}: contract_id {contract.contract_id!r} "
                    f"already used on row {previous}"
                )
                continue
            seen_ids[contract.contract_id] = line_number
            contracts.append(contract)

    if errors:
        listed = "\n  ".join(errors[:20])
        more = "" if len(errors) <= 20 else "\n  ... and more"
        raise CsvError(f"{path} has unreadable cells:\n  {listed}{more}")
    if not contracts:
        raise CsvError(f"{path} has a header but no contract rows")
    return contracts


def _parse_row(
    row: list[str],
    index: dict[str, int],
    mapping: dict[str, str],
    line_number: int,
) -> ContractInput:
    def heading(field: str) -> str:
        return mapping[field]

    def cell(field: str) -> str | None:
        return _cell(row, index, heading(field))

    contract_id = cell("contract_id")
    if not contract_id:
        raise CsvError(f"row {line_number}: contract_id is blank")

    def money_req(field: str) -> Decimal:
        raw = cell(field)
        if raw is None:
            raise CsvError(f"row {line_number} ({contract_id}): {field} is blank")
        return parse_money(raw, f"row {line_number} ({contract_id}): {field}")

    def money_zero(field: str) -> Decimal:
        return _money_or_zero(
            row, index, heading(field), f"row {line_number} ({contract_id}): {field}"
        )

    def money_none(field: str) -> Decimal | None:
        return _money_or_none(
            row, index, heading(field), f"row {line_number} ({contract_id}): {field}"
        )

    constraint_raw = cell("constraint_include_ratio")
    constraint = (
        parse_ratio(
            constraint_raw, f"row {line_number} ({contract_id}): constraint_include_ratio"
        )
        if constraint_raw is not None
        else Decimal("0")
    )

    measurable_raw = cell("outcome_reasonably_measurable")
    measurable = (
        parse_bool(
            measurable_raw,
            f"row {line_number} ({contract_id}): outcome_reasonably_measurable",
        )
        if measurable_raw is not None
        else True
    )

    method_raw = (cell("progress_method") or PROGRESS_COST_TO_COST).strip().casefold()
    if method_raw not in PROGRESS_METHODS:
        raise CsvError(
            f"row {line_number} ({contract_id}): progress_method {method_raw!r} "
            f"is not one of {sorted(PROGRESS_METHODS)}"
        )

    output_raw = cell("output_percent")
    output_percent = (
        parse_ratio(output_raw, f"row {line_number} ({contract_id}): output_percent")
        if output_raw is not None
        else None
    )

    retention_raw = (cell("retention_classification") or RETENTION_REVIEW).strip().casefold()
    if retention_raw not in RETENTION_CLASSES:
        raise CsvError(
            f"row {line_number} ({contract_id}): retention_classification "
            f"{retention_raw!r} is not one of {sorted(RETENTION_CLASSES)}"
        )

    gst_raw = cell("gst_rate")
    gst_rate = (
        parse_ratio(gst_raw, f"row {line_number} ({contract_id}): gst_rate")
        if gst_raw is not None
        else Decimal("0.10")
    )

    return ContractInput(
        contract_id=contract_id,
        line_number=line_number,
        customer=cell("customer") or "",
        description=cell("description") or "",
        original_contract_sum=money_req("original_contract_sum"),
        approved_variations=money_zero("approved_variations"),
        unapproved_variations_estimate=money_zero("unapproved_variations_estimate"),
        constraint_include_ratio=constraint,
        costs_incurred=money_req("costs_incurred"),
        inefficiency_rework_wastage=money_zero("inefficiency_rework_wastage"),
        uninstalled_materials=money_zero("uninstalled_materials"),
        estimated_cost_to_complete=money_req("estimated_cost_to_complete"),
        certified_billings=money_req("certified_billings"),
        uncertified_claims=money_zero("uncertified_claims"),
        retention_withheld=money_zero("retention_withheld"),
        retention_classification=retention_raw,
        committed_outstanding=money_zero("committed_outstanding"),
        outcome_reasonably_measurable=measurable,
        recoverable_costs=money_none("recoverable_costs"),
        progress_method=method_raw,
        output_percent=output_percent,
        prior_transaction_price=money_none("prior_transaction_price"),
        prior_estimated_cost_at_completion=money_none("prior_estimated_cost_at_completion"),
        prior_costs_incurred=money_none("prior_costs_incurred"),
        prior_estimated_cost_to_complete=money_none("prior_estimated_cost_to_complete"),
        prior_revenue_to_date=money_none("prior_revenue_to_date"),
        gst_rate=gst_rate,
        assets_used_carrying=money_none("assets_used_carrying"),
    )
