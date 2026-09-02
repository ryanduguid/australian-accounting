from __future__ import annotations

from decimal import Decimal

import pytest

from wiptally.money import AmountError, as_money, money, parse_amount, parse_bool, parse_money, parse_ratio, percent


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
