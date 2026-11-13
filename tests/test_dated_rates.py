# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Pin dated figures that more than one copy keeps, and the table expiry warnings.

The 2026-27 concessional cap sits in payday-super-checker's rates.json and in
australian-tax-calculators' metadata.rs, and the maximum contributions base is
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
import re
import tempfile
import unittest
from datetime import date, timedelta
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ROOT / "packages"
PAYDAY_RATES = PACKAGES / "payday-super-checker" / "data" / "rates.json"
PAYDAY_GIC = PACKAGES / "payday-super-checker" / "data" / "gic_rates.json"
PAYDAY_CALENDAR = PACKAGES / "payday-super-checker" / "data" / "business_days.json"
LODGMENT_HOLIDAYS = PACKAGES / "lodgment-calendar" / "lodgmentcalendar" / "holidays.py"
CALCULATOR_METADATA = PACKAGES / "australian-tax-calculators" / "src" / "metadata.rs"
CALCULATIONS = PACKAGES / "australian-tax-calculators" / "src" / "engine.rs"
DIV7A_RATES = PACKAGES / "div7a-loan-review" / "data" / "benchmark_rates.csv"
DATASET = PACKAGES / "au-tax-rates-data" / "data"

# How many engine copies of dataset figures _dataset_pairs() compares. An engine
# table that starts holding a dataset figure needs its own entry there.
DATASET_PAIRS = 54


def _metadata_literal(name: str):
    """A `pub const` table in metadata.rs as the dict the Python engine kept."""
    text = CALCULATOR_METADATA.read_text(encoding="utf-8")
    match = re.search(rf"^pub const {name}: [^=]*=\s*&?(\[.*?\]);$", text, re.S | re.M)
    if match is None:
        raise AssertionError(f"{name} not found in src/metadata.rs")
    literal = re.sub(r"Some\((\d+)\)", r"\1", match.group(1).replace("&[", "["))
    return dict(ast.literal_eval(literal))


def _metadata_string(name: str) -> str:
    """A `pub const` string in metadata.rs."""
    text = CALCULATOR_METADATA.read_text(encoding="utf-8")
    match = re.search(rf'^pub const {name}: &str = "([^"]*)";$', text, re.M)
    if match is None:
        raise AssertionError(f"{name} not found in src/metadata.rs")
    return match.group(1)


def _calculator_caps() -> dict[str, tuple[str, str]]:
    return _metadata_literal("CONTRIBUTION_CAPS")


def _fbt_working(name: str) -> str:
    """A rate fbt() reports in its working, read from engine.rs."""
    match = re.search(rf'rate\("{name}", "([^"]+)"\)', CALCULATIONS.read_text(encoding="utf-8"))
    if match is None:
        raise AssertionError(f"fbt() reports no {name} in src/engine.rs")
    return match.group(1)


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
            where = f"{year} resident scale over {low} (metadata.rs RESIDENT_TAX_SCALES)"
            pairs.append((f"{where} threshold", Decimal(row["from"]) - 1, Decimal(low)))
            pairs.append((f"{where} rate", Decimal(str(row["marginal_rate"])), Decimal(rate) * 100))
    payday = json.loads(PAYDAY_RATES.read_text(encoding="utf-8"))["financial_years"]["2026-27"]
    caps = _calculator_caps()
    pairs += [
        ("2026-27 super guarantee rate (paydaysuper rates.json)",
         _value("super-guarantee-rate-2026-27"), Decimal(payday["charge_percentage"])),
        ("2026-27 concessional cap (paydaysuper rates.json)",
         _value("concessional-cap-2026-27"), Decimal(payday["concessional_cap"])),
        ("2026-27 transfer balance cap (metadata.rs CONTRIBUTION_CAPS)",
         _value("transfer-balance-cap-2026-27"), Decimal(caps["2026-27"][1])),
        ("FBT rate (engine.rs fbt())", _value("fbt-rate"), Decimal(_fbt_working("fbt_rate")) * 100),
        ("FBT type 1 gross-up rate (engine.rs fbt())",
         _value("fbt-type-1-gross-up-rate"), Decimal(_fbt_working("type_one"))),
        ("FBT type 2 gross-up rate (engine.rs fbt())",
         _value("fbt-type-2-gross-up-rate"), Decimal(_fbt_working("type_two"))),
        ("carry-forward concessional balance limit (metadata.rs CARRY_FORWARD_BALANCE_LIMIT)",
         _value("carry-forward-concessional-balance-limit"),
         Decimal(_metadata_string("CARRY_FORWARD_BALANCE_LIMIT"))),
    ]
    for year in ("2025-26", "2026-27"):
        pairs += [
            (f"{year} concessional cap (metadata.rs CONTRIBUTION_CAPS)",
             _value(f"concessional-cap-{year}"), Decimal(caps[year][0])),
            # ITAA 1997 s 292-85(2): four times the concessional cap, as the calculators derive it.
            (f"{year} non-concessional cap (metadata.rs, 4 x CONTRIBUTION_CAPS)",
             _value(f"non-concessional-cap-{year}"), Decimal(caps[year][0]) * 4),
        ]
    medicare = _metadata_literal("MEDICARE_LOW_INCOME_THRESHOLDS")["2025-26"]
    tables = {
        "STUDY_LOAN_REPAYMENT_THRESHOLDS 2026-27":
            dict(_metadata_literal("STUDY_LOAN_REPAYMENT_THRESHOLDS")["2026-27"]),
        "STUDY_LOAN_REPAYMENT_RATES": _metadata_literal("STUDY_LOAN_REPAYMENT_RATES"),
        "MEDICARE_LOW_INCOME_THRESHOLDS 2025-26": {"threshold": medicare[0], "phase_in": medicare[1]},
        "MEDICARE_LEVY_RATES": _metadata_literal("MEDICARE_LEVY_RATES"),
        "LOW_INCOME_TAX_OFFSET": _metadata_literal("LOW_INCOME_TAX_OFFSET"),
        "DIVISION_293": _metadata_literal("DIVISION_293"),
        "DIVISION_296": _metadata_literal("DIVISION_296"),
        "TERMINATION_PAYMENT": _metadata_literal("TERMINATION_PAYMENT"),
        "FBT_BENCHMARK_INTEREST_RATES": _metadata_literal("FBT_BENCHMARK_INTEREST_RATES"),
    }
    for name, table, key in (
        ("study-loan-repayment-threshold-2026-27", "STUDY_LOAN_REPAYMENT_THRESHOLDS 2026-27", "minimum"),
        ("study-loan-repayment-upper-threshold-2026-27", "STUDY_LOAN_REPAYMENT_THRESHOLDS 2026-27", "step"),
        ("study-loan-repayment-base-amount-2026-27", "STUDY_LOAN_REPAYMENT_THRESHOLDS 2026-27", "printed_base"),
        ("study-loan-repayment-flat-threshold-2026-27", "STUDY_LOAN_REPAYMENT_THRESHOLDS 2026-27", "flat_from"),
        ("study-loan-repayment-lower-rate-2026-27", "STUDY_LOAN_REPAYMENT_RATES", "lower"),
        ("study-loan-repayment-upper-rate-2026-27", "STUDY_LOAN_REPAYMENT_RATES", "upper"),
        ("study-loan-repayment-flat-rate-2026-27", "STUDY_LOAN_REPAYMENT_RATES", "flat"),
        ("medicare-low-income-lower-threshold-2025-26", "MEDICARE_LOW_INCOME_THRESHOLDS 2025-26", "threshold"),
        ("medicare-low-income-upper-threshold-2025-26", "MEDICARE_LOW_INCOME_THRESHOLDS 2025-26", "phase_in"),
        ("medicare-levy-rate", "MEDICARE_LEVY_RATES", "levy"),
        ("lito-maximum", "LOW_INCOME_TAX_OFFSET", "maximum"),
        ("lito-income-limit", "LOW_INCOME_TAX_OFFSET", "cut_out"),
        ("division-293-threshold", "DIVISION_293", "threshold"),
        ("division-293-tax-rate", "DIVISION_293", "rate"),
        ("division-296-large-super-balance-threshold-2026-27", "DIVISION_296", "large_threshold"),
        ("division-296-very-large-super-balance-threshold-2026-27", "DIVISION_296", "very_large_threshold"),
        ("division-296-tax-rate", "DIVISION_296", "rate"),
        ("division-296-very-large-balance-tax-rate", "DIVISION_296", "very_large_rate"),
        ("etp-cap-life-benefit-2026-27", "TERMINATION_PAYMENT", "etp_cap"),
        ("etp-whole-of-income-cap-2026-27", "TERMINATION_PAYMENT", "whole_of_income_cap"),
        ("genuine-redundancy-base-limit-2026-27", "TERMINATION_PAYMENT", "base_amount"),
        ("genuine-redundancy-service-amount-2026-27", "TERMINATION_PAYMENT", "service_amount"),
        ("fbt-benchmark-interest-rate-fbt-year-2027", "FBT_BENCHMARK_INTEREST_RATES", "2027"),
    ):
        copy = Decimal(tables[table][key])
        if _record(name)["unit"] == "percent":
            copy *= 100
        pairs.append((f"{name} (metadata.rs {table} {key})", _value(name), copy))
    lines = DIV7A_RATES.read_text(encoding="utf-8").splitlines()
    benchmark = {row["year_of_income"]: row["rate"]
                 for row in csv.DictReader(line for line in lines if not line.startswith("#"))}
    pairs.append(("2026-27 Division 7A benchmark rate (div7a-loan-review benchmark_rates.csv)",
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


# The weekdays the ATO's lodgment program lists that the payday table counts as
# business days, because SGAA 1992 s 6(1) needs a public holiday for the whole of
# a State or Territory. The payday README and its 15 August 2026 source review
# record each reason.
LODGMENT_ONLY = {
    date(2026, 9, 28): "King's Birthday: some regional WA areas keep another date",
    date(2026, 11, 3): "Melbourne Cup: non-metro Victorian councils may arrange another day",
    date(2026, 12, 24): "Christmas Eve: a part-day holiday in QLD, NT and SA",
    date(2026, 12, 31): "New Year's Eve: a part-day holiday in NT and SA",
}


def _lodgment_holidays() -> set[date]:
    tree = ast.parse(LODGMENT_HOLIDAYS.read_text(encoding="utf-8"))
    table = next(node.value for node in tree.body if isinstance(node, ast.AnnAssign)
                 and isinstance(node.target, ast.Name) and node.target.id == "HOLIDAYS")
    return {date(*(part.value for part in key.args)) for key in table.keys}


def _payday_holidays() -> set[date]:
    rows = json.loads(PAYDAY_CALENDAR.read_text(encoding="utf-8"))["non_business_days"]
    return {date.fromisoformat(row["date"]) for row in rows if not row["provisional"]}


class HolidayTableTests(unittest.TestCase):
    def test_the_payday_and_lodgment_tables_differ_only_on_the_recorded_days(self) -> None:
        lodgment = _lodgment_holidays()
        first, last = min(lodgment), max(lodgment)
        payday = {day for day in _payday_holidays()
                  if first <= day <= last and day.weekday() < 5}
        self.assertEqual(lodgment - payday, set(LODGMENT_ONLY))
        self.assertEqual(payday - lodgment, set())


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
                    self.assertEqual(check.main(["--today", today, "--table", "gic"]), code)
                if code:
                    self.assertIn("gic_rates.json", out.getvalue())

    def test_the_business_day_warning_fires_inside_28_days_only(self) -> None:
        check = _load_expiry_check()
        end = check.business_days_end()
        for days_before, code in ((95, 0), (28, 0), (27, 1), (16, 1)):
            today = (end - timedelta(days=days_before)).isoformat()
            with self.subTest(days_before=days_before):
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    self.assertEqual(check.main(["--today", today, "--table", "business-days"]),
                                     code)
                if code:
                    self.assertIn("business_days.json", out.getvalue())

    def test_the_business_day_end_is_verified_until(self) -> None:
        # verified_until, not the last listed holiday: later rows are provisional and
        # extend no deadline.
        check = _load_expiry_check()
        with tempfile.TemporaryDirectory() as directory:
            table = Path(directory) / "business_days.json"
            table.write_text(json.dumps({
                "verified_until": "2030-06-30",
                "non_business_days": [{"date": "2030-12-25", "provisional": True}],
            }), encoding="utf-8")
            self.assertEqual(check.business_days_end(table).isoformat(), "2030-06-30")

    def test_the_holiday_warning_fires_inside_28_days_only(self) -> None:
        check = _load_expiry_check()
        end = check.holidays_end()
        for days_before, code in ((95, 0), (28, 0), (27, 1), (16, 1)):
            today = (end - timedelta(days=days_before)).isoformat()
            with self.subTest(days_before=days_before):
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    self.assertEqual(check.main(["--today", today, "--table", "holidays"]), code)
                if code:
                    self.assertIn("holidays.py", out.getvalue())

    def test_the_holiday_end_is_the_tables_covers_to(self) -> None:
        # COVERS_TO, not the last listed holiday: roll() settles no weekday past it.
        check = _load_expiry_check()
        with tempfile.TemporaryDirectory() as directory:
            table = Path(directory) / "holidays.py"
            table.write_text("HOLIDAYS = {date(2030, 1, 1): 'x'}\n"
                             "COVERS_TO = date(2030, 6, 30)\n", encoding="utf-8")
            self.assertEqual(check.holidays_end(table).isoformat(), "2030-06-30")
            table.write_text("COVERS_TO = max(HOLIDAYS)\n", encoding="utf-8")
            with self.assertRaises(ValueError):
                check.holidays_end(table)

    def test_every_table_is_checked_by_default(self) -> None:
        # A table that starts expiring fails the scheduled run, which passes no --table.
        check = _load_expiry_check()
        today = (check.holidays_end() - timedelta(days=1)).isoformat()
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(check.main(["--today", today]), 1)
        self.assertIn("holidays.py", out.getvalue())


ACT_SHORT_NAMES = {"Income Tax Assessment Act 1997": "ITAA 1997"}


class LegislationCitationTests(unittest.TestCase):
    """CI has no corpus, so citations are held to the list built from one."""

    package = ROOT / "packages" / "au-tax-rates-data"

    def setUp(self) -> None:
        self.records = [
            json.loads(path.read_text(encoding="utf-8"))
            for path in sorted((self.package / "data").glob("*.json"))
        ]
        self.listing = json.loads(
            (self.package / "legislation-provisions.json").read_text(encoding="utf-8")
        )

    def test_every_citation_resolves_and_every_listed_provision_is_cited(self) -> None:
        listed = {(p["title_id"], p["provision"]) for p in self.listing["provisions"]}
        cited = {
            (c["title_id"], c["provision"])
            for record in self.records
            for c in record.get("legislation", ())
        }
        self.assertTrue(cited)
        self.assertEqual(cited, listed)

    def test_the_list_records_where_each_provision_came_from(self) -> None:
        self.assertRegex(self.listing["corpus"]["retrieved"], r"^\d{4}-\d{2}-\d{2}$")
        for provision in self.listing["provisions"]:
            with self.subTest(provision=provision["provision"]):
                self.assertRegex(provision["sections_sha256"], r"^[0-9a-f]{64}$")
                self.assertRegex(provision["compilation_date"], r"^\d{4}-\d{2}-\d{2}$")
                self.assertEqual(provision["row_id"].split(":")[0], provision["title_id"])
                self.assertEqual(
                    provision["heading"].replace("\u2011", "-").split()[0],
                    provision["provision"],
                )

    def test_each_citation_is_one_its_own_notes_make(self) -> None:
        acts = {p["title_id"]: p["act"] for p in self.listing["provisions"]}
        for record in self.records:
            for citation in record.get("legislation", ()):
                with self.subTest(record=record["id"], provision=citation["provision"]):
                    act = acts[citation["title_id"]]
                    self.assertTrue(
                        act in record["notes"] or ACT_SHORT_NAMES.get(act, act) in record["notes"],
                        act,
                    )
                    self.assertRegex(
                        record["notes"],
                        r"(?:\bss?|\bsections?|\bregulations?) "
                        + re.escape(citation["provision"])
                        + r"(?![\w-]|\.\d)",
                    )

    def test_a_quoted_income_year_is_the_records_period(self) -> None:
        checked = 0
        for record in self.records:
            quoted = re.search(r"(\d{4})[\u2013-](\d{2}) income year", record["quote"])
            year = record["period"].get("income_year")
            if quoted and year:
                checked += 1
                with self.subTest(record=record["id"]):
                    self.assertEqual(f"{quoted[1]}-{quoted[2]}", year)
        self.assertGreaterEqual(checked, 2)

    def test_an_application_provision_quotes_the_notes(self) -> None:
        for record in self.records:
            for citation in record.get("legislation", ()):
                if "application" in citation:
                    with self.subTest(record=record["id"]):
                        self.assertIn(citation["application"], record["notes"])


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
