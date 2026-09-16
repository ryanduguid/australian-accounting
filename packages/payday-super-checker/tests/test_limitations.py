"""Guards binding LIMITATIONS.md to the behaviour it documents.

A published limitation that no longer matches the code is worse than no
register at all. Each test asserts the behaviour an entry describes, so
fixing the behaviour fails the test and forces the entry to be updated or
removed.
"""

import inspect
from datetime import date
from decimal import Decimal
from pathlib import Path

from paydaysuper import join, sgc
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
    notional earnings loop is the single caller of daily_rate()."""
    package = Path(__file__).resolve().parents[1] / "paydaysuper"
    callers = {
        path.name
        for path in package.glob("*.py")
        if "daily_rate(" in path.read_text(encoding="utf-8")
    }
    assert callers == {"rates.py", "sgc.py"}
    assert "daily_rate(" in inspect.getsource(sgc)


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
