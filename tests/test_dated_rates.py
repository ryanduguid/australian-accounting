"""Pin dated figures that more than one engine keeps, and the GIC expiry warning.

The 2026-27 concessional cap sits in payday-super-checker's rates.json and in
australian-tax-calculators' metadata.py, and the maximum contributions base is
derived from it. Nothing compared the copies, so an update to one would leave
the other silently stale. Standard library only, and no component is imported,
as in tests/test_shared_blocks.py.
"""

from __future__ import annotations

import ast
import contextlib
import importlib.util
import io
import json
import unittest
from datetime import timedelta
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAYDAY_RATES = ROOT / "packages" / "payday-super-checker" / "paydaysuper" / "data" / "rates.json"
CALCULATOR_METADATA = ROOT / "packages" / "australian-tax-calculators" / "austaxcalc" / "metadata.py"


def _calculator_caps() -> dict[str, tuple[str, str]]:
    tree = ast.parse(CALCULATOR_METADATA.read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.AnnAssign) and getattr(node.target, "id", None) == "CONTRIBUTION_CAPS":
            return ast.literal_eval(node.value)
    raise AssertionError("CONTRIBUTION_CAPS not found in austaxcalc/metadata.py")


def _load_expiry_check():
    path = ROOT / ".github" / "ci" / "check_rate_expiry.py"
    spec = importlib.util.spec_from_file_location("check_rate_expiry", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class DatedRateTests(unittest.TestCase):
    def test_the_concessional_cap_copies_agree(self) -> None:
        payday = json.loads(PAYDAY_RATES.read_text(encoding="utf-8"))["financial_years"]
        calculators = _calculator_caps()
        # Every year payday-super-checker carries must also be in the calculators'
        # table, so a new year added to one copy cannot skip the comparison.
        self.assertLessEqual(set(payday), set(calculators))
        for year in sorted(payday):
            with self.subTest(year=year):
                self.assertEqual(payday[year]["concessional_cap"], calculators[year][0])

    def test_the_maximum_contributions_base_follows_the_cap(self) -> None:
        # SGAA s 10A(5): concessional cap x 100 / charge percentage, rounded
        # down to the nearest $10.
        payday = json.loads(PAYDAY_RATES.read_text(encoding="utf-8"))["financial_years"]
        for year, row in payday.items():
            with self.subTest(year=year):
                base = Decimal(row["concessional_cap"]) * 100 / Decimal(row["charge_percentage"])
                self.assertEqual(Decimal(row["max_contributions_base"]), (base // 10) * 10)

    def test_the_gic_warning_fires_inside_28_days_only(self) -> None:
        check = _load_expiry_check()
        end = check.last_known()
        for days_before, code in ((95, 0), (28, 0), (27, 1), (16, 1)):
            today = (end - timedelta(days=days_before)).isoformat()
            with self.subTest(days_before=days_before):
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    self.assertEqual(check.main(["--today", today]), code)
                if code:
                    self.assertIn("gic_rates.json", out.getvalue())


if __name__ == "__main__":
    unittest.main()
