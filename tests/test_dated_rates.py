"""Pin dated figures that more than one copy keeps, and the GIC expiry warning.

The 2026-27 concessional cap sits in payday-super-checker's rates.json and in
australian-tax-calculators' metadata.py, and the maximum contributions base is
derived from it. Nothing compared the copies, so an update to one would leave
the other silently stale. The engines' copies of figures that
packages/au-tax-rates-data records, with their ATO sources, are held to those
records too. Standard library only, and no component is imported, as in
tests/test_shared_blocks.py.
"""

from __future__ import annotations

import ast
import contextlib
import csv
import importlib.util
import io
import json
import unittest
from datetime import timedelta
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ROOT / "packages"
PAYDAY_RATES = PACKAGES / "payday-super-checker" / "paydaysuper" / "data" / "rates.json"
PAYDAY_GIC = PACKAGES / "payday-super-checker" / "paydaysuper" / "data" / "gic_rates.json"
CALCULATOR_METADATA = PACKAGES / "australian-tax-calculators" / "austaxcalc" / "metadata.py"
CALCULATIONS = PACKAGES / "australian-tax-calculators" / "austaxcalc" / "calculations.py"
DIV7A_RATES = PACKAGES / "div7a-loan-review" / "div7aloan" / "data" / "benchmark_rates.csv"
DATASET = PACKAGES / "au-tax-rates-data" / "data"

# How many engine copies of dataset figures _dataset_pairs() compares. An engine
# table that starts holding a dataset figure needs its own entry there.
DATASET_PAIRS = 25


def _metadata_literal(name: str):
    tree = ast.parse(CALCULATOR_METADATA.read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.AnnAssign) and getattr(node.target, "id", None) == name:
            return ast.literal_eval(node.value)
    raise AssertionError(f"{name} not found in austaxcalc/metadata.py")


def _calculator_caps() -> dict[str, tuple[str, str]]:
    return _metadata_literal("CONTRIBUTION_CAPS")


def _fbt_rate() -> str:
    """The rate fbt() reports in its working, read from calculations.py."""
    tree = ast.parse(CALCULATIONS.read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef) and node.name == "fbt":
            for inner in ast.walk(node):
                if isinstance(inner, ast.Dict):
                    for key, value in zip(inner.keys, inner.values):
                        if isinstance(key, ast.Constant) and key.value == "fbt_rate":
                            return ast.literal_eval(value)
    raise AssertionError("fbt() reports no fbt_rate in austaxcalc/calculations.py")


def _record(name: str) -> dict:
    return json.loads((DATASET / f"{name}.json").read_text(encoding="utf-8"))


def _value(name: str) -> Decimal:
    return Decimal(str(_record(name)["value"]))


def _dataset_pairs() -> list[tuple[str, Decimal, Decimal | None]]:
    """(figure, dataset value, engine copy in the dataset's unit) for each copy.

    Percentages compare as per cent: the engines store fractions (0.0877) or
    per cent strings ("11.43"), the dataset per cent numbers (8.77).
    """
    pairs: list[tuple[str, Decimal, Decimal | None]] = []
    scales = _metadata_literal("RESIDENT_TAX_SCALES")
    for year in ("2025-26", "2026-27"):
        taxed = [row for row in _record(f"resident-tax-rates-{year}")["value"] if row["marginal_rate"]]
        for row, (low, _high, rate) in zip(taxed, scales[year]):
            where = f"{year} resident scale over {low} (austaxcalc RESIDENT_TAX_SCALES)"
            pairs.append((f"{where} threshold", Decimal(row["from"]) - 1, Decimal(low)))
            pairs.append((f"{where} rate", Decimal(str(row["marginal_rate"])), Decimal(rate) * 100))
    payday = json.loads(PAYDAY_RATES.read_text(encoding="utf-8"))["financial_years"]["2026-27"]
    caps = _calculator_caps()["2026-27"]
    pairs += [
        ("2026-27 super guarantee rate (paydaysuper rates.json)",
         _value("super-guarantee-rate-2026-27"), Decimal(payday["charge_percentage"])),
        ("2026-27 concessional cap (paydaysuper rates.json)",
         _value("concessional-cap-2026-27"), Decimal(payday["concessional_cap"])),
        ("2026-27 concessional cap (austaxcalc CONTRIBUTION_CAPS)",
         _value("concessional-cap-2026-27"), Decimal(caps[0])),
        # ITAA 1997 s 292-85(2): four times the concessional cap, as austaxcalc derives it.
        ("2026-27 non-concessional cap (austaxcalc, 4 x CONTRIBUTION_CAPS)",
         _value("non-concessional-cap-2026-27"), Decimal(caps[0]) * 4),
        ("FBT rate (austaxcalc fbt())", _value("fbt-rate"), Decimal(_fbt_rate()) * 100),
    ]
    lines = DIV7A_RATES.read_text(encoding="utf-8").splitlines()
    benchmark = {row["year_of_income"]: row["rate"]
                 for row in csv.DictReader(line for line in lines if not line.startswith("#"))}
    pairs.append(("2026-27 Division 7A benchmark rate (div7aloan benchmark_rates.csv)",
                  _value("div7a-benchmark-rate-2026-27"), Decimal(benchmark["2026-27"]) * 100))
    quarters = {(q["from"], q["to"]): q["annual_pct"]
                for q in json.loads(PAYDAY_GIC.read_text(encoding="utf-8"))["quarters"]}
    for name in ("gic-annual-rate-apr-jun-2026", "gic-annual-rate-jul-sep-2026",
                 "gic-annual-rate-oct-dec-2026"):
        period = _record(name)["period"]
        copy = quarters.get((period["effective_from"], period["effective_to"]))
        pairs.append((f"GIC from {period['effective_from']} (paydaysuper gic_rates.json)",
                      _value(name), None if copy is None else Decimal(copy)))
    return pairs


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


class DatasetAgreementTests(unittest.TestCase):
    def test_every_engine_copy_equals_the_dataset_record(self) -> None:
        for figure, dataset, engine in _dataset_pairs():
            with self.subTest(figure=figure):
                self.assertIsNotNone(engine, f"{figure}: no engine copy for the record's period")
                self.assertEqual(engine, dataset, figure)

    def test_no_listed_copy_drops_out_of_the_comparison(self) -> None:
        # A copy removed from _dataset_pairs(), or a tax bracket added on one side
        # only, changes these counts instead of passing quietly.
        self.assertEqual(len(_dataset_pairs()), DATASET_PAIRS)
        scales = _metadata_literal("RESIDENT_TAX_SCALES")
        for year in ("2025-26", "2026-27"):
            taxed = [row for row in _record(f"resident-tax-rates-{year}")["value"] if row["marginal_rate"]]
            with self.subTest(year=year):
                self.assertEqual(len(taxed), len(scales[year]))


if __name__ == "__main__":
    unittest.main()
