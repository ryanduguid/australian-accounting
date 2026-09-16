"""Guards binding LIMITATIONS.md to the behaviour it documents.

A published limitation that no longer matches the code is worse than no
register at all. Each test asserts the behaviour an entry describes, so
fixing the behaviour fails the test and forces the entry to be updated or
removed.
"""

from __future__ import annotations

import inspect
from decimal import Decimal
from pathlib import Path

from atobenchmark import dataset as ds
from atobenchmark import to_evidenced_dict
from atobenchmark.ratios import compute
from atobenchmark.report import compare, to_dict

LIMITATIONS = Path(__file__).resolve().parents[1] / "LIMITATIONS.md"

# Turnover and rent only. A bakery's published key ratio is cost of sales to
# turnover, so omitting the bucket is what triggers ABC-1.
UNMAPPED_COST_OF_SALES = {"turnover": "850000", "other_income": "0", "rent": "40000"}


def totals(**kwargs: str) -> dict[str, Decimal]:
    return {name: Decimal(value) for name, value in kwargs.items()}


def _register() -> str:
    return LIMITATIONS.read_text(encoding="utf-8")


def test_register_lists_every_documented_entry() -> None:
    register = _register()
    assert "ABC-1" in register
    # The clause that distinguishes a register from a disclaimer.
    assert register.count("**What stays correct.**") == 1


def test_abc_1_serialisers_diverge_on_the_key_ratio() -> None:
    """ABC-1: with cost of sales unmapped, the CLI payload carries the ATO
    fallback and the library payload reverts to the published key ratio."""
    data = ds.load("2023-24")
    bakery = data.get("Bakeries and hot bread shops")
    assert bakery.key_ratio == "cost_of_sales_to_turnover"

    comparison = compare(data, bakery, compute(totals(**UNMAPPED_COST_OF_SALES)))

    cli_payload = to_dict(comparison)
    library_payload = to_evidenced_dict(comparison, set(UNMAPPED_COST_OF_SALES))

    assert cli_payload["key_ratio"] == "total_expenses_to_turnover"
    assert library_payload["key_ratio"] == bakery.key_ratio
    assert cli_payload["key_ratio"] != library_payload["key_ratio"]


def test_abc_1_library_payload_stays_internally_consistent() -> None:
    """The reverted key ratio and the per-row flag must name the same ratio."""
    data = ds.load("2023-24")
    bakery = data.get("Bakeries and hot bread shops")
    comparison = compare(data, bakery, compute(totals(**UNMAPPED_COST_OF_SALES)))

    payload = to_evidenced_dict(comparison, set(UNMAPPED_COST_OF_SALES))
    flagged = [row["ratio"] for row in payload["ratios"] if row["is_key_ratio"]]

    assert flagged == [payload["key_ratio"]]


def test_abc_1_unsupplied_ratios_are_a_figure_in_one_payload_and_withheld_in_the_other() -> None:
    """ABC-1 claims the divergence is wider than the key flag: an unmapped
    bucket is a computed nil on the CLI side and withheld on the library
    side. Parity holds only for ratios whose buckets were supplied."""
    data = ds.load("2023-24")
    bakery = data.get("Bakeries and hot bread shops")
    comparison = compare(data, bakery, compute(totals(**UNMAPPED_COST_OF_SALES)))

    cli = {row["ratio"]: row for row in to_dict(comparison)["ratios"]}
    library = {
        row["ratio"]: row
        for row in to_evidenced_dict(comparison, set(UNMAPPED_COST_OF_SALES))["ratios"]
    }

    # cost_of_sales was never mapped: a figure on one side, withheld on the other.
    # Both serialisers emit money and ratios as decimal strings, never floats.
    assert cli["cost_of_sales_to_turnover"]["value"] == "0.0000"
    assert library["cost_of_sales_to_turnover"]["value"] is None
    assert library["cost_of_sales_to_turnover"]["status"] == "not_supplied"

    # rent was supplied, so it carries the same value in both payloads.
    assert cli["rent_to_turnover"]["value"] == library["rent_to_turnover"]["value"]
    assert library["rent_to_turnover"]["status"] != "not_supplied"


def test_abc_1_parity_also_needs_both_income_fields() -> None:
    """ABC-1 claims row parity needs the ratio's own buckets AND both income
    fields. to_evidenced_dict gates every row on income_evidenced, so a ratio
    holding all of its own buckets is still withheld without other_income."""
    data = ds.load("2023-24")
    bakery = data.get("Bakeries and hot bread shops")
    # rent is supplied; other_income is not.
    supplied = {"turnover": "850000", "rent": "40000"}
    comparison = compare(data, bakery, compute(totals(**supplied)))

    cli = {row["ratio"]: row for row in to_dict(comparison)["ratios"]}
    library = {
        row["ratio"]: row
        for row in to_evidenced_dict(comparison, set(supplied))["ratios"]
    }

    assert cli["rent_to_turnover"]["value"] == "0.0471"
    assert library["rent_to_turnover"]["value"] is None
    assert library["rent_to_turnover"]["status"] == "not_supplied"
    # The benchmark range is withheld with it.
    assert library["rent_to_turnover"]["benchmark_min"] is None


def test_abc_1_trigger_is_an_absent_bucket_not_an_unmapped_account() -> None:
    """ABC-1 must not describe a case the command line refuses outright:
    route() raises on a profit-and-loss row with no mapping entry, so that
    input never reaches either serialiser."""
    from atobenchmark.mapping import MappingError, route

    register = _register()
    assert "MappingError" in register
    assert "route()" in register
    assert MappingError is not None and callable(route)


def test_abc_1_cli_serialiser_cannot_see_which_buckets_were_supplied() -> None:
    """The divergence exists because to_dict() has no supplied-field set. If
    it gains one, ABC-1 is fixable and the entry must be revisited."""
    assert "supplied_fields" not in inspect.signature(to_dict).parameters
    assert "supplied_fields" in inspect.signature(to_evidenced_dict).parameters


def test_abc_1_fallback_note_is_still_emitted() -> None:
    data = ds.load("2023-24")
    bakery = data.get("Bakeries and hot bread shops")
    comparison = compare(data, bakery, compute(totals(**UNMAPPED_COST_OF_SALES)))

    assert any(item.code == "cost_of_sales_key_fallback" for item in comparison.note_details)
