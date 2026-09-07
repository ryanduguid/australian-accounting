import pytest

from paydaysuper.csv_io import csv_safe


@pytest.mark.parametrize("prefix", ["", " ", "  ", " \t"])
@pytest.mark.parametrize("payload", ["=cmd|'/C calc'!A0", "@SUM(A1)", "+1E3", "-R1C1"])
def test_guard_quotes_formula_after_leading_whitespace(prefix: str, payload: str) -> None:
    value = prefix + payload
    assert csv_safe(value) == "'" + value


@pytest.mark.parametrize("value", ["", " ", "  Rent", "00123"])
def test_guard_preserves_ordinary_text(value: str) -> None:
    assert csv_safe(value) == value
