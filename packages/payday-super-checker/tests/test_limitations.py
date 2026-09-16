"""Guards binding LIMITATIONS.md to the behaviour it documents.

A published limitation that no longer matches the code is worse than no
register at all. Each test asserts the behaviour an entry describes, so
fixing the behaviour fails the test and forces the entry to be updated or
removed.
"""

import ast
import inspect
from datetime import date
from decimal import Decimal
from pathlib import Path

from paydaysuper import join, sgc

# The package re-exports an `assess` function that shadows the module, so the
# verdict vocabulary is imported by name rather than through the package.
from paydaysuper.assess import EXPOSED, LATE, UNPAID, VERDICTS
from paydaysuper.rates import GicQuarter, GicTable

LIMITATIONS = Path(__file__).resolve().parents[1] / "LIMITATIONS.md"


def _register() -> str:
    return LIMITATIONS.read_text(encoding="utf-8")


def test_register_lists_every_documented_entry():
    register = _register()
    for entry in ("PSC-1", "PSC-2"):
        assert entry in register
    # Each entry must keep the clause that distinguishes a register from a
    # disclaimer: what a reader may still rely on.
    assert register.count("**What stays correct.**") == 2


def test_psc_1_gic_table_still_extrapolates_past_its_last_quarter():
    """PSC-1: a day past the table takes the last known quarter's rate."""
    table = GicTable([GicQuarter(date(2026, 7, 1), date(2026, 9, 30), Decimal("11.43"))])
    past_the_end = date(2026, 12, 31)
    assert past_the_end > table.last_known

    carried = table.daily_rate(past_the_end)
    last_known_rate = table.daily_rate(date(2026, 9, 30))
    # Same annual rate; the divisor is the same calendar year here.
    assert carried == last_known_rate

    stale = table.staleness(past_the_end)
    assert stale is not None
    assert "2026-09-30" in stale
    assert "11.43" in stale
    assert table.staleness(date(2026, 9, 30)) is None


def test_psc_1_only_the_exposure_estimate_reads_the_gic_rate():
    """PSC-1 claims verdicts are unaffected. That holds only while the
    notional earnings loop is the single caller of daily_rate().

    Counts call expressions with ast rather than matching text: the method
    definition in rates.py is not a call, and a second call added inside an
    already-listed module would not change a set of filenames.
    """
    package = Path(__file__).resolve().parents[1] / "paydaysuper"
    call_sites = [
        f"{path.name}:{node.lineno}"
        for path in sorted(package.glob("*.py"))
        for node in ast.walk(ast.parse(path.read_text(encoding="utf-8")))
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and node.func.attr == "daily_rate"
    ]
    assert len(call_sites) == 1, call_sites
    assert call_sites[0].startswith("sgc.py:")


def test_psc_1_names_every_verdict_and_the_two_that_carry_the_estimate():
    """PSC-1 vouches for the verdict, so it must cover the whole vocabulary,
    and must not imply a sound verdict makes the row's figures sound."""
    register = _register()
    for verdict in VERDICTS:
        assert f"`{verdict}`" in register
    assert len(VERDICTS) == 6
    assert EXPOSED == (LATE, UNPAID)
    for exposed in EXPOSED:
        assert f"`{exposed}`" in register


def test_psc_1_uplift_is_a_range_not_a_flat_rate():
    """PSC-1 said the uplift was 60%. It spans 0% to 60%."""
    scenarios = sgc.uplift_scenarios(Decimal("1000"), Decimal("100"))
    assert scenarios["clean_history"]["vds_within_30d"] == Decimal("0")
    assert scenarios["prior_history"]["no_vds"] > Decimal("0")
    register = _register()
    assert "0%" in register and "60%" in register


def test_psc_1_both_exposure_totals_move_with_the_notional_earnings():
    """PSC-1 first said the 0% low estimate was unmoved. It is not:
    exposure_range adds notional earnings into both totals before the uplift,
    so only the low estimate's uplift COMPONENT is unmoved."""
    shortfall = Decimal("1000")
    low_a, high_a = sgc.exposure_range(shortfall, Decimal("100"))
    low_b, high_b = sgc.exposure_range(shortfall, Decimal("120"))

    # Exact totals, not merely "they differ": subtracting the notional
    # earnings, or dropping the high uplift, would satisfy an inequality
    # while contradicting the published explanation.
    assert (low_a, high_a) == (Decimal("1100"), Decimal("1760"))
    assert (low_b, high_b) == (Decimal("1120"), Decimal("1792"))

    # Both totals rise by the full increase in notional earnings, plus the
    # uplift charged on it where the percentage is non-zero.
    assert low_b - low_a == Decimal("20")
    assert high_b - high_a == Decimal("32")

    # The component that does not move: 0% of a larger base is still nil,
    # while the high estimate's uplift component grows with the base.
    scenarios_a = sgc.uplift_scenarios(shortfall, Decimal("100"))
    scenarios_b = sgc.uplift_scenarios(shortfall, Decimal("120"))
    assert scenarios_a["clean_history"]["vds_within_30d"] == Decimal("0")
    assert scenarios_b["clean_history"]["vds_within_30d"] == Decimal("0")
    assert scenarios_b["prior_history"]["no_vds"] > scenarios_a["prior_history"]["no_vds"]

    register = _register()
    assert "Both exposure totals move" in register


def test_psc_2_join_declares_each_degraded_match():
    """PSC-2: four column conditions each raise a structural warning."""
    source = inspect.getsource(join)
    for declared in (
        "matched on employee name because one of the files has no id column",
        "the payroll file has no pay period end column",
        "the super file has only one of the pay period start/end columns",
        "the super file has no pay period columns at all",
    ):
        assert declared in source
    # The fallback itself, not just its warning.
    assert 'key_mode = "id" if both_have_ids else "name"' in source


def test_psc_2_register_names_the_same_four_conditions():
    register = _register()
    for condition in (
        "employee_id",
        "pay-period-end column",
        "only one of pay-period start or end",
        "no pay-period columns at all",
    ):
        assert condition in register
