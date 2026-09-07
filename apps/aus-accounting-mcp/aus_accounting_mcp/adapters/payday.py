"""Adapter over payday-super-checker.

This module does not compute deadlines, holidays or SG charge. It translates
MCP arguments into ``paydaysuper.report.assess`` and serialises the result.
Clearing-house latency is never invented. as_at is required. Transition
allocation cannot be confirmed through this facade.
"""

from __future__ import annotations

import re
from datetime import date
from decimal import Decimal
from typing import Any

from paydaysuper import LAW_CONTENT_DATE, __version__ as PAYDAY_VERSION
from paydaysuper.calendar import load_calendar
from paydaysuper.csv_io import CsvError, parse_date_text
from paydaysuper.deadlines import ContribLine, PreRegimeError
from paydaysuper.rates import load_gic
from paydaysuper.report import Result, assess

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.money import parse_amount

#: A purely numeric slash or dash date, captured to its first two components.
#: The engine reads these day first, as the Australian calendar is written, and
#: the guard below refuses the ones where that reading cannot be checked.
NUMERIC_DATE = re.compile(r"^(\d{1,2})[/-](\d{1,2})[/-]\d{2,4}$")

DISCLAIMER = (
    "Experimental review aid. Not a compliance determination, an ATO assessment "
    "or professional advice. payday-super-checker refuses or marks UNKNOWN where "
    "the supplied facts do not establish the statutory test (SGAA 1992 s 18C). "
    "This MCP does not model clearing-house latency. Fund receipt must be supplied "
    "before a contribution can be ON_TIME."
)


def _read_date(text: str, field: str) -> date:
    """Read a date the way the engine reads one from a contributions CSV.

    The dates an operator holds come out of a payroll or clearing-house export,
    and payday-super-checker already accepts the shapes those arrive in: ISO,
    day-first numeric, "9 Jul 2026", and a zone-less date-time whose time
    component the law ignores. Parsing is the engine's, through its own public
    parse_date_text, so this adapter adds no date format of its own and cannot
    drift from what the CSV path accepts.

    Two refusals sit at this boundary rather than in the engine.

    A stamp carrying Z or a UTC offset is the engine's refusal, raised as a
    CsvError and re-raised here: its written day belongs to that zone, and a
    UTC evening is already the next day in Australia, so keeping the written day
    could pass a receipt that was really a day later.

    A purely numeric date is refused here only where the two readings give
    different days. The engine reads 01/02/2027 day first, as 1 February, which
    is right for the Australian export it was written for. Through an MCP tool
    the same text can as easily be a caller writing 2 January, and nothing in
    the string says which. That is a month-sized error in a date that decides a
    lateness verdict, so it fails closed and asks for ISO rather than guessing.

    Two things settle the reading and are accepted. A component above 12 can
    only be the day, so 13/07/2027 is the 13th. And equal components mean both
    readings land on the same date, so 12/12/2027 is the 12th of December
    whichever way it is read, and refusing it would be refusing a date nobody
    could misread.
    """
    numeric = NUMERIC_DATE.match(text)
    if numeric:
        first, second = int(numeric.group(1)), int(numeric.group(2))
        if first != second and first <= 12 and second <= 12:
            raise InputError(
                f"{field}: {text!r} is ambiguous. A numeric date like this is read day "
                f"first, so it means day {first} of month {second}, but nothing in the text "
                f"rules out day {second} of month {first}, and the difference decides the "
                "verdict. Supply it as YYYY-MM-DD."
            )
    try:
        parsed = parse_date_text(text)
    except CsvError as exc:
        raise InputError(f"{field}: {exc}") from exc
    if parsed is None:
        raise InputError(
            f"{field}: {text!r} is not a date this tool reads. Use YYYY-MM-DD; "
            "a day-first numeric date, a form like '9 Jul 2026', and a date-time "
            "with no timezone marker are also read."
        )
    return parsed


def _required_date(value: str, field: str) -> date:
    text = str(value).strip()
    if not text:
        raise InputError(f"{field} is required (YYYY-MM-DD)")
    return _read_date(text, field)


def _optional_date(value: str | None, field: str) -> date | None:
    if value is None:
        return None
    text = str(value).strip()
    if not text:
        return None
    return _read_date(text, field)


def _money(value: Decimal | None) -> str | None:
    if value is None:
        return None
    return str(value)


def _serialise(result: Result) -> dict[str, Any]:
    due = result.deadline.due
    uplift = None
    if result.uplift is not None:
        uplift = {
            name: {k: str(v) for k, v in scenario.items()}
            for name, scenario in result.uplift.items()
        }
    return {
        "employee_id": result.line.employee_id,
        "qe_day": result.line.qe_day.isoformat(),
        "sg_amount": str(result.line.sg_amount),
        "remitted": None if result.line.remitted is None else result.line.remitted.isoformat(),
        "received": None if result.line.received is None else result.line.received.isoformat(),
        "due": None if due is None else due.isoformat(),
        "pathway": result.deadline.pathway,
        "verdict": result.verdict,
        "days_late": result.days_late,
        "lateness_basis": result.lateness_basis or None,
        "base_shortfall": _money(result.base_shortfall),
        "final_shortfall": _money(result.final_shortfall),
        "notional_earnings": _money(result.nec),
        "experimental_sgc_low": _money(result.sgc_low),
        "experimental_sgc_high": _money(result.sgc_high),
        "uplift": uplift,
        "notes": list(result.notes),
        "caveats": list(result.caveats),
        "horizon_verdicts": (
            None if result.horizon_verdicts is None else list(result.horizon_verdicts)
        ),
    }


def review_contribution(
    *,
    qe_day: str,
    sg_amount: str,
    as_at: str,
    remitted: str | None = None,
    received: str | None = None,
    employee_id: str = "mcp-1",
    first_to_fund: bool = False,
    out_of_cycle: bool = False,
    next_standard_qe_day: str | None = None,
    db_interest: bool = False,
) -> dict[str, Any]:
    """Review one contribution against payday-super-checker."""
    line = ContribLine(
        employee_id=employee_id,
        qe_day=_required_date(qe_day, "qe_day"),
        sg_amount=parse_amount(sg_amount, "sg_amount"),
        remitted=_optional_date(remitted, "remitted"),
        received=_optional_date(received, "received"),
        first_to_fund=first_to_fund,
        out_of_cycle=out_of_cycle,
        next_standard_qe_day=_optional_date(next_standard_qe_day, "next_standard_qe_day"),
        db_interest=db_interest,
        row=1,
    )
    as_at_day = _required_date(as_at, "as_at")
    try:
        results = assess(
            [line],
            load_calendar(),
            load_gic(),
            as_at_day,
            transition_allocation_confirmed=False,
        )
    except PreRegimeError as exc:
        raise InputError(str(exc)) from exc
    except ValueError as exc:
        message = str(exc).replace(
            "--confirm-transition-allocation",
            "a human reconciliation of June-quarter balances; this MCP cannot confirm that",
        )
        raise InputError(message) from exc
    result = results[0]
    return {
        "ok": True,
        "engine": "payday-super-checker",
        "engine_version": PAYDAY_VERSION,
        "law_content_date": LAW_CONTENT_DATE,
        "as_at": as_at_day.isoformat(),
        "disclaimer": DISCLAIMER,
        "result": _serialise(result),
    }
