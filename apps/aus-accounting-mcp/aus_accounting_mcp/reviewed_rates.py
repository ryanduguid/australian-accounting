# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Look up a reviewed ATO rate for a date, or refuse. Never a nearest-year figure.

`rate_records/` is a copy of `packages/au-tax-rates-data/data`, which no published engine
carries; that package's validator reviews the records and a root test holds the copy equal.
Each record is a point-in-time copy of an ATO page, never a statement of current law.
"""

from __future__ import annotations

import json
from datetime import date
from functools import cache
from pathlib import Path
from typing import Any

from .errors import InputError

RECORDS = Path(__file__).parent / "rate_records"
STATEMENT = (
    "A point-in-time copy of an ATO page, captured at retrieved_at, not a live lookup or a "
    "confirmation of current law. A record with no effective_to says only that the figure "
    "applied when it was retrieved. Retain source_url, quote and notes, and confirm the "
    "figure at the source before relying on it."
)
AFTER_LAST_READ = (
    "This record was last read on {read} and does not show that the figure still applied "
    "on {as_at}."
)


@cache
def records() -> dict[str, dict[str, Any]]:
    """Every reviewed record by id."""
    loaded = (json.loads(path.read_text(encoding="utf-8")) for path in RECORDS.glob("*.json"))
    return {record["id"]: record for record in sorted(loaded, key=lambda record: record["id"])}


def covered_years() -> list[str]:
    """The income years the records name, which bound every lookup."""
    return sorted(
        {r["period"]["income_year"] for r in records().values() if "income_year" in r["period"]}
    )


def _year_start(label: str) -> date:
    return date(int(label[:4]), 7, 1)


def _year_end(label: str) -> date:
    return date(int(label[:4]) + 1, 6, 30)


def _covers(record: dict[str, Any], day: date) -> bool | None:
    """Whether the record's own period holds day; None when it records no period."""
    period = record["period"]
    if "income_year" in period:
        return _year_start(period["income_year"]) <= day <= _year_end(period["income_year"])
    if "effective_from" not in period:
        return None
    # An open-ended record is bounded by covered_years before it reaches this test.
    end = date.fromisoformat(period.get("effective_to", date.max.isoformat()))
    return date.fromisoformat(period["effective_from"]) <= day <= end


def _refused(
    record_id: str,
    day: date,
    code: str,
    reason: str,
    *,
    record: dict[str, Any] | None = None,
    covering_ids: list[str] | None = None,
    known_ids: list[str] | None = None,
) -> dict[str, Any]:
    return {
        **_result(record_id, day, "REFUSED"),
        "refusal": code,
        "reason": reason + " No figure is returned.",
        "record_period": None if record is None else record["period"],
        "covering_ids": covering_ids or [],
        "known_ids": known_ids or [],
    }


def _result(record_id: str, day: date, status: str) -> dict[str, Any]:
    return {
        "ok": True,
        "status": status,
        "record_id": record_id,
        "as_at": day.isoformat(),
        "covered_years": covered_years(),
        "record": None,
        "refusal": None,
        "reason": None,
        "record_period": None,
        "covering_ids": [],
        "known_ids": [],
        "after_last_read": False,
        "after_last_read_note": None,
        "currency": "NOT_ESTABLISHED",
        "statement": STATEMENT,
    }


def reviewed_rate(record_id: str, as_at: str) -> dict[str, Any]:
    """The named record if as_at falls in the covered years and its period, else a refusal."""
    try:
        day = date.fromisoformat(as_at)
    except ValueError as exc:
        raise InputError(f"as_at must be a YYYY-MM-DD date; got {as_at!r}") from exc
    record = records().get(record_id)
    if record is None:
        return _refused(
            record_id, day, "UNKNOWN_ID", f"No reviewed record has the id {record_id!r}.",
            known_ids=list(records()),
        )
    years = covered_years()
    if not _year_start(years[0]) <= day <= _year_end(years[-1]):
        return _refused(
            record_id, day, "OUTSIDE_COVERED_YEARS",
            f"{day.isoformat()} is outside the reviewed records, which cover "
            f"{' and '.join(years)} only.",
            record=record,
        )
    covers = _covers(record, day)
    if covers is None:
        return _refused(
            record_id, day, "NO_RECORDED_PERIOD",
            f"{record_id} records no effective period, only the date its page was read, so "
            "it cannot be placed at any date.",
            record=record,
        )
    if not covers:
        return _refused(
            record_id, day, "OUTSIDE_RECORD_PERIOD",
            f"{day.isoformat()} is outside the period {record_id} holds.",
            record=record,
            covering_ids=[
                other["id"] for other in records().values()
                if other["label"] == record["label"] and _covers(other, day)
            ],
        )
    found = {**_result(record_id, day, "FOUND"), "record": record}
    period, read = record["period"], date.fromisoformat(record["retrieved_at"][:10])
    if day > read and "effective_from" in period and "effective_to" not in period:
        found["after_last_read"] = True
        found["after_last_read_note"] = AFTER_LAST_READ.format(read=read, as_at=day)
    return found
