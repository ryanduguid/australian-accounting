from __future__ import annotations

from decimal import Decimal

import pytest
from wiptally.money import (
    AmountError,
    as_money,
    money,
    parse_amount,
    parse_bool,
    parse_money,
    parse_ratio,
    percent,
)


def test_parse_money_accepts_accounting_forms() -> None:
    assert parse_money("1,234.56") == Decimal("1234.56")
    assert parse_money("$(1,000.00)") == Decimal("-1000.00")
    assert parse_money("500.00 CR") == Decimal("-500.00")
    assert parse_money("500.00 DR") == Decimal("500.00")


def test_parse_money_refuses_nan_and_half_cent_dust() -> None:
    with pytest.raises(AmountError):
        parse_money("NaN")
    with pytest.raises(AmountError):
        parse_money("Infinity")
    with pytest.raises(AmountError):
        parse_money("0.001")


def test_parse_ratio_percent_and_unit_interval() -> None:
    assert parse_ratio("25%") == Decimal("0.25")
    assert parse_ratio("0.25") == Decimal("0.25")
    with pytest.raises(AmountError):
        parse_ratio("25")
    with pytest.raises(AmountError):
        parse_ratio("1.2")


def test_parse_bool() -> None:
    assert parse_bool("yes") is True
    assert parse_bool("NO") is False
    with pytest.raises(AmountError):
        parse_bool("maybe")


def test_display_does_not_round_a_verdict_across_a_boundary() -> None:
    ratio = Decimal("200000.00") / Decimal("700000.00")
    # 28.57% displayed, not 29%.
    assert percent(ratio) == "28.57%"
    assert money(Decimal("-0.001")) == "0.00"


def test_as_money_uses_half_up() -> None:
    assert as_money(Decimal("1.225")) == Decimal("1.23")
    assert as_money(Decimal("1.224")) == Decimal("1.22")


def test_parse_amount_never_returns_float() -> None:
    value = parse_amount("10.10")
    assert type(value) is Decimal
    assert type(value + value) is Decimal


@pytest.mark.parametrize("raw", ["1,2,3", "1,23", "12,34,567", "1,,000", "1 000", "(100) CR", "(100) DR", "-100 CR", "+100 DR", "$($100)"])
def test_malformed_grouping_and_conflicting_signs_are_refused(raw):
    with pytest.raises(AmountError):
        parse_amount(raw)


def test_the_ordinary_grouped_forms_still_parse() -> None:
    # The pair held identical across every engine that reads an accounting cell.
    assert parse_amount("1,234.50") == Decimal("1234.50")
    assert parse_amount("(1,234.50)") == Decimal("-1234.50")


@pytest.mark.parametrize("raw", [None, "", "   "])
def test_parse_amount_refuses_a_missing_amount(raw: str | None) -> None:
    with pytest.raises(AmountError, match="^cost: no amount given$"):
        parse_amount(raw, "cost")  # type: ignore[arg-type]


@pytest.mark.parametrize("raw", [None, "", "   "])
def test_parse_ratio_refuses_a_missing_ratio(raw: str | None) -> None:
    with pytest.raises(AmountError, match="^gst_rate: no ratio given$"):
        parse_ratio(raw, "gst_rate")  # type: ignore[arg-type]


@pytest.mark.parametrize("raw", ["-0.1", "-5%", "150%"])
def test_parse_ratio_refuses_a_ratio_outside_the_unit_interval(raw: str) -> None:
    with pytest.raises(AmountError, match=f"^gst_rate: '{raw}' is not between 0 and 1$"):
        parse_ratio(raw, "gst_rate")


def test_parse_bool_refuses_a_missing_flag() -> None:
    with pytest.raises(AmountError, match="^measurable: no yes/no value given$"):
        parse_bool(None, "measurable")  # type: ignore[arg-type]
