import csv
import runpy
from decimal import Decimal
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[1] / "examples/job_to_cash.py"


def test_profit_does_not_fund_cash_and_uncertified_claims_are_excluded(tmp_path):
    result = runpy.run_path(str(SCRIPT))["run"](tmp_path / "base")
    assert Decimal(result["gross_profit_at_completion"]) == 200000
    assert Decimal(result["revenue_to_date"]) == 500000
    assert Decimal(result["contract_asset"]) == 50000
    assert Decimal(result["cash_to_date"]) == -140000
    assert Decimal(result["retention_gross"]) == 49500
    assert Decimal(result["collectible_certified_balance"]) == 145500
    with (tmp_path / "base/cash-assumptions.csv").open() as stream:
        rows = list(csv.DictReader(stream))
    assert {row["id"] for row in rows} == {"CERTIFIED-RECEIVABLE", "RETENTION", "REMAINING-COST"}
    assert sum(Decimal(row["receipt"]) for row in rows) == 195000


def test_cost_change_flows_through_margin_and_cash(tmp_path):
    function = runpy.run_path(str(SCRIPT))["run"]
    base = function(tmp_path / "base")
    changed = function(tmp_path / "changed", extra_cost_to_complete=Decimal("100000"))
    assert Decimal(changed["gross_profit_at_completion"]) == Decimal(base["gross_profit_at_completion"]) - 100000
    assert Decimal(changed["future_cost_cash"]) == Decimal(base["future_cost_cash"]) + 110000
    with pytest.raises(FileExistsError):
        function(tmp_path / "base")
