"""Year of income labels and the values a YearOfIncome refuses."""
from __future__ import annotations

import pytest
from div7aloan.years import YearError, YearOfIncome, parse_year


@pytest.mark.parametrize("start", ["2026", 2026.0, True])
def test_a_year_must_start_on_a_whole_year(start):
    with pytest.raises(YearError, match="must be a whole year"):
        YearOfIncome(start)


@pytest.mark.parametrize("start", [1899, 2101])
def test_a_year_outside_the_plausible_range_is_refused(start):
    with pytest.raises(YearError, match=f"year of income {start}-.. is outside 1900-2100"):
        YearOfIncome(start)


def test_a_year_prints_as_its_label():
    assert str(parse_year("2026-27")) == "2026-27"  # nosec B101: pytest outcome assertion


def test_a_year_does_not_order_against_a_bare_number():
    with pytest.raises(TypeError):
        _ = YearOfIncome(2026) < 2027
