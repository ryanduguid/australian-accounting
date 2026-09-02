import argparse
from decimal import Decimal

import pytest

from edwinnixon.cli import money_type, percentage_type, rate_type


@pytest.mark.parametrize(
    "raw", ["-0.01", "1.001", "1000000000000.01", "NaN", "Infinity"]
)
def test_money_type_rejects_values_outside_domain(raw: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError):
        money_type(raw)


def test_money_type_accepts_domain_boundary() -> None:
    assert money_type("1000000000000.00") == Decimal("1000000000000.00")


@pytest.mark.parametrize("raw", ["-0.01", "100.01", "NaN", "Infinity"])
def test_percentage_type_rejects_values_outside_domain(raw: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError):
        percentage_type(raw)


@pytest.mark.parametrize("raw", ["0", "1", "NaN", "Infinity"])
def test_rate_type_rejects_values_outside_domain(raw: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError):
        rate_type(raw)
