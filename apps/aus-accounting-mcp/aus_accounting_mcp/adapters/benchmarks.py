"""Adapter over ato-benchmark-compare.

No ATO ratios are hardcoded here. Figures are bucket totals; the engine
applies QC 37143 turnover and labour rules and the shipped dataset.
Omitted buckets are not treated as evidenced zeros, in the structured figures
or in the engine's own notes and checks. The turnover rule reads other_income to
choose the ratio denominator, so no ratio is reported without it.

The engine needs a figure for every bucket, so an omitted bucket reaches it as a
zero that is not evidence. Deciding which outputs that zero reached is the
engine's own job: ``to_evidenced_dict`` takes the set of fields the operator
actually supplied and withholds every figure, ratio, note and check whose stated
value depends on one that was not. This module supplies that set and adds the
facade's own fields; it does not re-derive which outputs are established.
"""

from __future__ import annotations

from decimal import Decimal
from typing import Any

from atobenchmark import __version__ as BENCHMARK_VERSION
from atobenchmark.dataset import DatasetError, load
from atobenchmark.mapping import BUCKETS
from atobenchmark.ratios import RatioError, compute
from atobenchmark.report import compare, to_evidenced_dict

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.money import parse_amount, parse_optional_amount

EXPENSE_FIELDS = (
    "cost_of_sales",
    "cost_of_sales_labour",
    "salary_wages",
    "contractor_commission",
    "associated_persons",
    "rent",
    "motor_vehicle",
    "other_expense",
)


def list_industries(
    search: str | None = None,
    year: str | None = None,
    *,
    limit: int | None = None,
    offset: int = 0,
) -> dict[str, Any]:
    # Direct Python callers receive the same validation as MCP callers.
    if limit is not None and (type(limit) is not int or not 1 <= limit <= 100):
        raise InputError("limit must be an integer from 1 to 100, or null for all matches.")
    if type(offset) is not int or offset < 0:
        raise InputError("offset must be a non-negative integer; start at 0.")
    try:
        data = load(year)
    except DatasetError as exc:
        raise InputError(str(exc)) from exc
    matches = data.search(search) if search else list(data.business_types)
    page = matches[offset : None if limit is None else offset + limit]
    next_offset = offset + len(page)
    has_more = next_offset < len(matches)
    return {
        "ok": True,
        "engine": "ato-benchmark-compare",
        "engine_version": BENCHMARK_VERSION,
        "benchmark_year": data.year,
        "count": len(page),
        "total_count": len(matches),
        "offset": offset,
        "has_more": has_more,
        "next_offset": next_offset if has_more else None,
        "total_business_types": len(data.business_types),
        "industries": [{"name": bt.name, "key_ratio": bt.key_ratio} for bt in page],
        "source": dict(data.source),
    }


def compare_figures(
    *,
    industry: str,
    turnover: str,
    year: str | None = None,
    other_income: str | None = None,
    cost_of_sales: str | None = None,
    cost_of_sales_labour: str | None = None,
    salary_wages: str | None = None,
    contractor_commission: str | None = None,
    associated_persons: str | None = None,
    rent: str | None = None,
    motor_vehicle: str | None = None,
    other_expense: str | None = None,
    w1: str | None = None,
) -> dict[str, Any]:
    """Compare operator-supplied bucket totals against the ATO dataset."""
    supplied: dict[str, Decimal] = {"turnover": parse_amount(turnover, "turnover")}
    optional = {
        "other_income": other_income,
        "cost_of_sales": cost_of_sales,
        "cost_of_sales_labour": cost_of_sales_labour,
        "salary_wages": salary_wages,
        "contractor_commission": contractor_commission,
        "associated_persons": associated_persons,
        "rent": rent,
        "motor_vehicle": motor_vehicle,
        "other_expense": other_expense,
    }
    for field, raw in optional.items():
        amount = parse_optional_amount(raw, field)
        if amount is not None:
            supplied[field] = amount
    w1_amount = parse_optional_amount(w1, "w1")

    if not any(field in supplied for field in EXPENSE_FIELDS):
        raise InputError(
            "no expense figures were supplied, so no ATO ratio can be compared. "
            "Pass at least one expense bucket as a decimal string; use 0 only when "
            "the operator established that the amount is zero."
        )

    totals = {name: Decimal("0") for name in BUCKETS}
    totals.update(supplied)

    try:
        data = load(year)
        business_type = data.get(industry)
        figures = compute(totals, w1_amount)
        comparison = compare(data, business_type, figures)
    except (DatasetError, RatioError) as exc:
        raise InputError(str(exc)) from exc

    # The fields the operator established, in the engine's own vocabulary. "w1"
    # is an activity statement label rather than a bucket, and the engine takes
    # it in this same collection.
    supplied_fields = set(supplied)
    if w1_amount is not None:
        supplied_fields.add("w1")

    payload = to_evidenced_dict(comparison, supplied_fields)
    payload.update(
        {
            "ok": True,
            "engine": "ato-benchmark-compare",
            "engine_version": BENCHMARK_VERSION,
        }
    )
    return payload
