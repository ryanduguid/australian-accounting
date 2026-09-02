from decimal import Decimal

import pytest

from div7aloan.money import MoneyError, parse_money


@pytest.mark.parametrize("raw", ["1.001", "1000000000000.01"])
def test_parse_money_rejects_values_outside_domain(raw: str) -> None:
    with pytest.raises(MoneyError):
        parse_money(raw, "amount")


def test_parse_money_accepts_maximum_to_cents() -> None:
    assert parse_money("1000000000000.00", "amount") == Decimal(
        "1000000000000.00"
    )
