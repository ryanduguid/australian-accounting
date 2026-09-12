from decimal import Decimal

import pytest
from div7aloan.money import MoneyError, parse_money


@pytest.mark.parametrize("raw", ["1.001", "1000000000000.01"])
def test_parse_money_rejects_values_outside_domain(raw: str) -> None:
    with pytest.raises(MoneyError):
        parse_money(raw, "amount")


@pytest.mark.parametrize("raw", ["1,2,3", "1,234.50", "(1,234.50)"])
def test_parse_money_refuses_every_grouped_or_bracketed_form(raw: str) -> None:
    # This engine reads a loan register field, not a spreadsheet cell: its input
    # is a plain decimal string, so a comma or a bracket is a malformed field
    # rather than an accounting presentation to decode. "1,2,3" in particular is
    # refused here and in every sibling engine, so no engine reads it as 123.
    with pytest.raises(MoneyError):
        parse_money(raw, "amount")


def test_parse_money_accepts_maximum_to_cents() -> None:
    assert parse_money("1000000000000.00", "amount") == Decimal(
        "1000000000000.00"
    )
