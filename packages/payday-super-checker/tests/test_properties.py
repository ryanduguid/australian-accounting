"""Seeded date-boundary and exact-money properties."""

from datetime import date, timedelta
from decimal import Decimal

from paydaysuper.calendar import load_calendar
from paydaysuper.sgc import exposure_range, uplift_scenarios

from _fastcheck import fastcheck_property

CALENDAR = load_calendar()
# Input domains live in tests/fastcheck/properties.cjs under each test's name.


@fastcheck_property(
    seed=0xDA7E,
    runs=100,
    examples=[
        dict(start=date(2026, 12, 24), count=7),
        dict(start=date(2027, 3, 25), count=7),
        dict(start=date(2026, 8, 1), count=0),
    ],
)
def test_business_day_addition_counts_only_days_after_the_start(
    start: date, count: int
) -> None:
    """Counting the start, a weekend or a confirmed holiday must break this."""
    result = CALENDAR.add_business_days(start, count)

    if count == 0:
        assert result == start
        return

    assert result > start
    assert CALENDAR.is_business_day(result)
    elapsed = (result - start).days
    counted = sum(
        CALENDAR.is_business_day(start + timedelta(days=offset))
        for offset in range(1, elapsed + 1)
    )
    before_result = sum(
        CALENDAR.is_business_day(start + timedelta(days=offset))
        for offset in range(1, elapsed)
    )
    assert counted == count
    assert before_result == count - 1


@fastcheck_property(
    seed=0x5AC,
    runs=100,
    examples=[
        dict(final_shortfall=Decimal("0"), nec=Decimal("0")),
        dict(final_shortfall=Decimal("1000"), nec=Decimal("100")),
    ],
)
def test_exposure_scenarios_conserve_the_shortfall_plus_nec_base(
    final_shortfall: Decimal, nec: Decimal
) -> None:
    """Changing an uplift rate or omitting NEC from its base must break this."""
    base = final_shortfall + nec
    expected_percentages = {
        "clean_history": {
            "vds_within_30d": Decimal("0"),
            "vds_31_60d": Decimal("5"),
            "vds_61_120d": Decimal("10"),
            "vds_after_120d": Decimal("25"),
            "no_vds": Decimal("40"),
        },
        "prior_history": {
            "vds_within_30d": Decimal("20"),
            "vds_31_60d": Decimal("25"),
            "vds_61_120d": Decimal("30"),
            "vds_after_120d": Decimal("45"),
            "no_vds": Decimal("60"),
        },
    }

    scenarios = uplift_scenarios(final_shortfall, nec)
    for history, row in expected_percentages.items():
        for scenario, percentage in row.items():
            assert scenarios[history][scenario] == base * percentage / Decimal("100")

    low, high = exposure_range(final_shortfall, nec)
    assert low == base
    assert high == base * Decimal("1.60")
