"""Regression coverage for Payday Super Checker's hand-review-only calendar generator.

The generator is a Python tool beside the Rust engine; it writes the calendar the
engine compiles in, so its tests run from the repository root.
"""

from __future__ import annotations

import json
import subprocess
import sys
from datetime import date
from pathlib import Path

import holidays

ROOT = Path(__file__).resolve().parents[1] / "packages" / "payday-super-checker"
GENERATOR = ROOT / "tools" / "generate_calendar.py"


def generate() -> dict:
    completed = subprocess.run(
        [sys.executable, str(GENERATOR)],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(completed.stdout)


def test_raw_generator_is_provisional_until_the_reviewed_table_confirms_dates():
    """Generation is not primary-source review. The shipped table confirms
    Business Victoria's 25 September 2026 date, while raw output keeps it and
    the fixture-dependent 2027 candidate provisional."""
    generated = generate()
    raw_2026 = next(
        holiday for holiday in generated["non_business_days"] if holiday["date"] == "2026-09-25"
    )
    future = next(
        holiday
        for holiday in generated["non_business_days"]
        if holiday["jurisdictions"] == ["VIC"]
        and "Grand Final" in holiday["name"]
        and holiday["date"].startswith("2027-")
    )
    reviewed = json.loads((ROOT / "data" / "business_days.json").read_text(encoding="utf-8"))
    reviewed_2026 = next(
        holiday for holiday in reviewed["non_business_days"] if holiday["date"] == "2026-09-25"
    )

    assert raw_2026["name"] == "Friday before the AFL Grand Final"
    assert raw_2026["jurisdictions"] == ["VIC"]
    assert raw_2026["provisional"] is True
    assert reviewed_2026["provisional"] is False
    assert future["provisional"] is True


def test_generator_excludes_locally_substitutable_dates():
    """WA King's Birthday and Melbourne Cup Day do not apply throughout
    their respective State according to the official jurisdiction pages."""
    generated = generate()
    names = [entry["name"] for entry in generated["non_business_days"]]

    assert "Melbourne Cup Day" not in names
    assert not any(
        entry["jurisdictions"] == ["WA"] and entry["name"] == "King's Birthday"
        for entry in generated["non_business_days"]
    )
    assert generated["verified_until"] == "2026-07-01"
    assert generated["official_sources"]["checked"] is None


def test_generator_requests_public_holidays_only_and_emits_no_bank_holiday():
    """A bank holiday is not a public holiday for the whole of a State, so
    admitting one would extend a deadline and manufacture a false ON_TIME.

    The exclusion lives in the generator's category argument, not in
    NOT_WHOLE_OF_JURISDICTION and not in the shipped table's dates: the
    NSW/ACT August bank holiday falls on the first Monday in August, which NT
    Picnic Day already removes, so is_business_day can never observe it."""
    source = GENERATOR.read_text(encoding="utf-8")
    assert 'categories=("public",)' in source
    assert source.count("holidays.Australia(") == 1

    generated = generate()
    assert not [
        entry
        for entry in generated["non_business_days"]
        if "bank holiday" in entry["name"].lower()
    ]


def test_generator_provenance_reflects_the_environment_that_ran():
    """The pin in _comment and the generation date come from the installed
    holidays package and the clock, not from literals: a hardcoded pin
    survived the 0.101 -> 0.102 bump, and a hardcoded date would stamp any
    future regeneration with a false generation date."""
    before = date.today()
    generated = generate()
    after = date.today()

    assert f"holidays=={holidays.__version__}" in generated["_comment"]
    # Bracketed rather than compared to a single call so a run that crosses
    # midnight cannot fail spuriously.
    assert generated["generated"] in {before.isoformat(), after.isoformat()}
