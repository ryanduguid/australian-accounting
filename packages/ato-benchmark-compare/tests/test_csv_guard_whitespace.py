import pytest
from atobenchmark.csvsafe import guard


@pytest.mark.parametrize("prefix", ["", " ", "  ", " \t"])
@pytest.mark.parametrize("payload", ["=cmd|'/C calc'!A0", "@SUM(A1)", "+1E3", "-R1C1"])
def test_guard_quotes_formula_after_leading_whitespace(prefix: str, payload: str) -> None:
    value = prefix + payload
    assert guard(value) == "'" + value


@pytest.mark.parametrize("value", ["", " ", "  Rent", "00123"])
def test_guard_preserves_ordinary_text(value: str) -> None:
    assert guard(value) == value


@pytest.mark.parametrize("control", ["\t", "\r", "\n"])
def test_guard_quotes_a_leading_control_character(control: str) -> None:
    assert guard(control + "Rent") == "'" + control + "Rent"  # nosec B101: pytest outcome assertion
