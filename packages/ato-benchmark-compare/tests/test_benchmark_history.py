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


def test_unavailable_counterfactual_preserves_standalone_results(tmp_path):
    previous, current = tmp_path / "previous.csv", tmp_path / "current.csv"
    previous.write_text("Account,Amount\nSales,500000\nNew activity,0\nCosts,100000\n")
    current.write_text("Account,Amount\nSales,0\nNew activity,600000\nCosts,120000\n")
    old, new = tmp_path / "old.csv", tmp_path / "new.csv"
    mapping = "account,bucket,source,amount,note\nSales,turnover,reviewed,0,Fabricated\nNew activity,{bucket},reviewed,0,Fabricated\nCosts,cost_of_sales,reviewed,0,Fabricated\n"
    old.write_text(mapping.format(bucket="excluded"))
    new.write_text(mapping.format(bucket="turnover"))
    result = runpy.run_path(str(HISTORY))["history"](previous, current, old, new,
        previous_period="2025", current_period="2026", industry="Bakeries and hot bread shops")
    assert result["previous"] and result["current"]
    assert result["limitation"]
    assert all(row["comparison"] == "NOT_COMPARABLE" for row in result["ratio_bridge"])


def test_expense_signs_are_explicit_for_each_snapshot(tmp_path):
    from decimal import Decimal

    from atobenchmark.mapping import EXPENSE_BUCKETS, account_key, read_mapping
    from atobenchmark.pnl import read

    positive = ROOT / "examples/bakery-pnl.csv"
    mapping = ROOT / "examples/bakery-mapping.csv"
    reviewed = read_mapping(mapping)
    negative = tmp_path / "negative.csv"
    with negative.open("w", newline="") as stream:
        writer = csv.writer(stream)
        writer.writerow(["Account", "Amount"])
        for row in read(positive, None).rows:
            value = -row.amount if reviewed[account_key(row.account)].bucket in EXPENSE_BUCKETS else row.amount
            writer.writerow([row.account, value])
    result = runpy.run_path(str(HISTORY))["history"](positive, negative, mapping, mapping,
        previous_period="2025", current_period="2026", industry="Bakeries and hot bread shops",
        current_expenses_negative=True)
    cost = next(row for row in result["ratio_bridge"] if row["ratio"] == "cost_of_sales_to_turnover")
    assert cost["current"] == cost["previous"] and cost["current"] is not None
    assert Decimal(cost["movement_under_previous_mapping"]) == 0
    assert result["expense_signs"] == {"previous_negative": False, "current_negative": True}
