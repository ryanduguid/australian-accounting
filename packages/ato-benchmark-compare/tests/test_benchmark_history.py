import csv
import runpy
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
HISTORY = ROOT / "examples/benchmark_history.py"


def test_reclassification_is_separate_from_unchanged_business(tmp_path):
    source = ROOT / "examples/bakery-pnl.csv"
    original = ROOT / "examples/bakery-mapping.csv"
    with original.open() as stream:
        reader = csv.DictReader(stream)
        fields = reader.fieldnames
        rows = list(reader)
    changed = [row for row in rows if row["bucket"] == "salary_wages"]
    assert changed
    for row in changed:
        row["bucket"] = "other_expense"
        row["note"] = "Fabricated reclassification to demonstrate comparison basis"
    mapping = tmp_path / "changed.csv"
    with mapping.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fields)
        writer.writeheader()
        writer.writerows(rows)
    result = runpy.run_path(str(HISTORY))["history"](source, source, original, mapping,
        previous_period="FY2025", current_period="FY2026", industry="Bakeries and hot bread shops")
    assert result["mapping_changes"]
    for row in result["ratio_bridge"]:
        if row["comparison"] == "DECOMPOSED":
            assert float(row["movement_under_previous_mapping"]) == 0
    labour = next(row for row in result["ratio_bridge"] if row["ratio"] == "labour_to_turnover")
    # An omitted labour bucket is unknown, never an invented zero improvement.
    assert labour["comparison"] == "NOT_COMPARABLE"
    assert labour["current"] is None


def test_unsupported_industry_gets_no_invented_range():
    from atobenchmark.dataset import DatasetError
    with pytest.raises(DatasetError):
        runpy.run_path(str(HISTORY))["history"](
            ROOT / "examples/bakery-pnl.csv", ROOT / "examples/bakery-pnl.csv",
            ROOT / "examples/bakery-mapping.csv", ROOT / "examples/bakery-mapping.csv",
            previous_period="2025", current_period="2026", industry="Invented unsupported industry")
