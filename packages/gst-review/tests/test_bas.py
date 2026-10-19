"""BAS to ledger reconciliation. Every line, period and figure is fabricated."""

from __future__ import annotations

from datetime import date, datetime
from decimal import Decimal

import pytest
from gstreview import LedgerLine, Period, __version__, reconcile_bas
from gstreview.bas import DEFAULT_BAS_CODES, LABELS, NOT_DECIDED

Q1 = Period("2026-Q1", date(2026, 7, 1), date(2026, 9, 30))
Q2 = Period("2026-Q2", date(2026, 10, 1), date(2026, 12, 31))
D = Decimal


def sale(id: str, gross: str, gst: str, code: str, invoiced: date,
         paid: date | None = None) -> LedgerLine:
    return LedgerLine(id, "sale", D(gross), D(gst), code, invoiced, paid)


def buy(id: str, gross: str, gst: str, code: str, invoiced: date,
        paid: date | None = None) -> LedgerLine:
    return LedgerLine(id, "purchase", D(gross), D(gst), code, invoiced, paid)


QUARTER = [
    sale("s1", "1100.40", "100.04", "GST on Income", date(2026, 7, 5), date(2026, 7, 20)),
    sale("s2", "300.00", "0.00", "GST Free Income", date(2026, 8, 1), date(2026, 8, 2)),
    sale("s3", "500.00", "0.00", "GST Free Exports", date(2026, 8, 3), date(2026, 8, 3)),
    sale("s4", "80.00", "0.00", "Input Taxed", date(2026, 8, 4), date(2026, 8, 4)),
    buy("p1", "220.00", "20.00", "GST on Expenses", date(2026, 7, 9), date(2026, 7, 9)),
    buy("p2", "3300.00", "300.00", "GST on Capital", date(2026, 9, 1), date(2026, 9, 2)),
    buy("p3", "45.00", "0.00", "GST Free Expenses", date(2026, 9, 3), date(2026, 9, 3)),
    buy("p4", "15.00", "0.00", "BAS Excluded", date(2026, 9, 4), date(2026, 9, 4)),
]
LODGED = {"2026-Q1": {"G1": D(1980), "G2": D(500), "G3": D(300), "G10": D(3300),
                      "G11": D(265), "1A": D(100), "1B": D(320)}}


def labels(result: dict, period: int = 0) -> dict[str, dict]:
    return result["periods"][period]["labels"]


def test_matching_quarter() -> None:
    result = reconcile_bas(QUARTER, [Q1], LODGED, basis="accruals")
    rows = labels(result)
    assert {label: row["status"] for label, row in rows.items()} == dict.fromkeys(
        LABELS, "matches")
    assert rows["G1"]["ledger"] == "1980.40"
    assert rows["G1"]["ledger_whole_dollars"] == "1980"
    assert rows["1A"]["ledger"] == "100.04"
    assert rows["1B"]["ledger"] == "320.00"
    assert rows["G1"]["lines"] == []
    assert result["periods"][0]["kept_out"] == ["p4"]
    assert result["not_lodged"] == result["no_ledger_lines"] == []
    assert result["line_issues"] == result["outside_periods"] == result["unpaid"] == []


def test_result_contract() -> None:
    result = reconcile_bas([], [Q1], {}, basis="cash")
    assert result["ok"] is True
    assert result["engine"] == "gst-review"
    assert result["engine_version"] == __version__
    assert result["review"] == "bas_reconciliation"
    assert result["basis"] == "cash"
    assert result["facts_used"]["bas_codes"] == DEFAULT_BAS_CODES
    assert result["facts_used"]["g1_includes_gst"] is True
    assert result["not_decided"] == NOT_DECIDED
    assert result["sources"][0].startswith("https://www.legislation.gov.au/C2004A00446/")
    assert result["periods"][0]["start"] == "2026-07-01"
    assert result["periods"][0]["lodged"] is False
    assert labels(result)["G1"] == {"ledger": "0", "ledger_whole_dollars": "0",
                                    "lodged": None, "difference": None, "lines": [],
                                    "status": "not_reported"}


def test_difference_lists_the_lines_behind_it() -> None:
    lodged = {"2026-Q1": {**LODGED["2026-Q1"], "1A": D(90), "G1": D(1980)}}
    rows = labels(reconcile_bas(QUARTER, [Q1], lodged, basis="accruals"))
    assert rows["1A"]["status"] == "differs"
    assert rows["1A"]["difference"] == "10"
    assert rows["1A"]["lines"] == ["s1"]
    assert rows["G1"]["status"] == "matches"


def test_rounding_drops_cents_before_comparing() -> None:
    lines = [sale("a", "109.99", "9.99", "GST", date(2026, 7, 1), date(2026, 7, 1))]
    rows = labels(reconcile_bas(lines, [Q1], {"2026-Q1": {"G1": D(109), "1A": D(10)}},
                                basis="cash"))
    assert rows["G1"]["status"] == "matches"
    assert rows["1A"]["status"] == "differs"
    assert rows["1A"]["difference"] == "-1"


def test_g1_without_gst_applies_to_every_g_label() -> None:
    rows = labels(reconcile_bas(QUARTER, [Q1], {}, basis="accruals", g1_includes_gst=False))
    assert rows["G1"]["ledger"] == "1880.36"
    assert rows["G10"]["ledger"] == "3000.00"
    assert rows["G11"]["ledger"] == "245.00"
    assert rows["1A"]["ledger"] == "100.04"


def test_cash_and_accruals_attribute_differently() -> None:
    lines = [
        sale("late", "110.00", "10.00", "GST", date(2026, 9, 28), date(2026, 10, 3)),
        sale("open", "220.00", "20.00", "GST", date(2026, 9, 29)),
        sale("early", "330.00", "30.00", "GST", date(2026, 10, 2), date(2026, 9, 30)),
    ]
    accruals = reconcile_bas(lines, [Q1, Q2], {}, basis="accruals")
    assert labels(accruals, 0)["1A"]["lines"] == []
    assert labels(accruals, 0)["1A"]["ledger"] == "60.00"
    assert accruals["periods"][1]["timing"] == ["early"]
    assert accruals["unpaid"] == []

    cash = reconcile_bas(lines, [Q1, Q2], {}, basis="cash")
    assert labels(cash, 0)["1A"]["ledger"] == "30.00"
    assert labels(cash, 1)["1A"]["ledger"] == "10.00"
    assert cash["periods"][0]["timing"] == ["late", "open"]
    assert cash["periods"][1]["timing"] == ["early"]
    assert cash["unpaid"] == ["open"]


def test_missing_periods_on_either_side() -> None:
    lines = [sale("a", "110.00", "10.00", "GST", date(2026, 7, 1))]
    result = reconcile_bas(lines, [Q1, Q2], {"2026-Q2": {"1A": D(0)}}, basis="accruals")
    assert result["not_lodged"] == ["2026-Q1"]
    assert result["no_ledger_lines"] == ["2026-Q2"]
    assert labels(result, 1)["1A"]["status"] == "matches"
    assert labels(result, 1)["G1"]["status"] == "not_reported"


def test_lines_outside_every_period() -> None:
    lines = [sale("old", "110.00", "10.00", "GST", date(2026, 6, 30))]
    result = reconcile_bas(lines, [Q1], {}, basis="accruals")
    assert result["outside_periods"] == ["old"]
    assert result["periods"][0]["timing"] == []


def test_credit_notes_reduce_and_a_negative_total_is_not_compared() -> None:
    lines = [
        sale("inv", "110.00", "10.00", "GST", date(2026, 7, 1)),
        sale("cn", "-220.00", "-20.00", "GST", date(2026, 7, 2)),
    ]
    rows = labels(reconcile_bas(lines, [Q1], {"2026-Q1": {"1A": D(0)}}, basis="accruals"))
    assert rows["1A"]["status"] == "negative_ledger"
    assert rows["1A"]["difference"] is None
    assert rows["1A"]["lines"] == ["inv", "cn"]


def test_lines_that_cannot_be_placed_are_kept_out_with_a_reason() -> None:
    lines = [
        sale("unknown", "10.00", "0.00", "ZZZ", date(2026, 7, 1)),
        sale("capital_sale", "110.00", "10.00", "CAP", date(2026, 7, 1)),
        buy("export_buy", "10.00", "0.00", "EXP", date(2026, 7, 1)),
        buy("gst_on_free", "11.00", "1.00", "FRE", date(2026, 7, 1)),
    ]
    result = reconcile_bas(lines, [Q1], {}, basis="accruals")
    assert [issue["id"] for issue in result["line_issues"]] == [
        "unknown", "capital_sale", "export_buy", "gst_on_free"]
    assert "not in the code table" in result["line_issues"][0]["issue"]
    assert "cannot be placed on a sale" in result["line_issues"][1]["issue"]
    assert "carries no GST" in result["line_issues"][3]["issue"]
    assert result["periods"][0]["kept_out"] == [
        "unknown", "capital_sale", "export_buy", "gst_on_free"]
    assert all(row["ledger"] == "0" for row in labels(result).values())


def test_caller_code_table_replaces_the_default() -> None:
    lines = [buy("a", "55.00", "5.00", "G11X", date(2026, 7, 1)),
             buy("b", "55.00", "5.00", "GST", date(2026, 7, 1))]
    result = reconcile_bas(lines, [Q1], {}, basis="accruals", bas_codes={"G11X": "taxable"})
    assert labels(result)["1B"]["ledger"] == "5.00"
    assert [issue["id"] for issue in result["line_issues"]] == ["b"]


GOOD = sale("a", "1.00", "0.00", "FRE", date(2026, 7, 1))


@pytest.mark.parametrize(
    ("lines", "periods", "lodged", "kwargs", "message"),
    [
        ([], [Q1], {}, {"basis": "invoice"}, "basis"),
        ([], [Q1], {}, {"basis": "cash", "g1_includes_gst": 1}, "g1_includes_gst"),
        ([], [Q1], {}, {"basis": "cash", "bas_codes": {"X": "zero"}}, "Unknown BAS categories"),
        ([], [Q1, Q1], {}, {"basis": "cash"}, "unique"),
        ([], [Period("", Q1.start, Q1.end)], {}, {"basis": "cash"}, "unique"),
        ([], [Period("x", Q1.end, Q1.start)], {}, {"basis": "cash"}, "start on or before"),
        ([], [Period("x", datetime(2026, 7, 1), Q1.end)], {}, {"basis": "cash"},  # type: ignore[arg-type]
         "start on or before"),
        ([], [Period("x", Q1.start, "2026-09-30")], {}, {"basis": "cash"},  # type: ignore[arg-type]
         "start on or before"),
        ([], [Q1, Period("x", date(2026, 9, 30), date(2026, 10, 31))], {}, {"basis": "cash"},
         "overlaps"),
        ([], [Q1], {"2027-Q1": {}}, {"basis": "cash"}, "unknown period"),
        ([], [Q1], {"2026-Q1": {"G4": D(1)}}, {"basis": "cash"}, "Unsupported BAS label"),
        ([], [Q1], {"2026-Q1": {"G1": D("1.50")}}, {"basis": "cash"}, "whole non-negative"),
        ([], [Q1], {"2026-Q1": {"G1": D(-1)}}, {"basis": "cash"}, "whole non-negative"),
        ([], [Q1], {"2026-Q1": {"G1": 1}}, {"basis": "cash"}, "whole non-negative"),
        ([GOOD, GOOD], [Q1], {}, {"basis": "cash"}, "unique"),
        ([LedgerLine("", "sale", D(1), D(0), "FRE", date(2026, 7, 1))], [Q1], {},
         {"basis": "cash"}, "unique"),
        ([LedgerLine("a", "income", D(1), D(0), "FRE", date(2026, 7, 1))], [Q1], {},
         {"basis": "cash"}, "side"),
        ([LedgerLine("a", "sale", D(1), D(0), None, date(2026, 7, 1))], [Q1], {},  # type: ignore[arg-type]
         {"basis": "cash"}, "GST code"),
        ([LedgerLine("a", "sale", D(1), D(0), "FRE", None)], [Q1], {},  # type: ignore[arg-type]
         {"basis": "cash"}, "invoice date"),
        ([LedgerLine("a", "sale", D(1), D(0), "FRE", date(2026, 7, 1), "2026-07-02")],  # type: ignore[arg-type]
         [Q1], {}, {"basis": "cash"}, "invoice date"),
        ([LedgerLine("a", "sale", D("1.001"), D(0), "FRE", date(2026, 7, 1))], [Q1], {},
         {"basis": "cash"}, "finite Decimals"),
        ([LedgerLine("a", "sale", D(1), 0, "FRE", date(2026, 7, 1))], [Q1], {},  # type: ignore[arg-type]
         {"basis": "cash"}, "finite Decimals"),
        ([LedgerLine("a", "sale", D(1), D(2), "GST", date(2026, 7, 1))], [Q1], {},
         {"basis": "cash"}, "share the sign"),
        ([LedgerLine("a", "sale", D(-11), D(1), "GST", date(2026, 7, 1))], [Q1], {},
         {"basis": "cash"}, "share the sign"),
    ],
)
def test_invalid_input_is_refused(lines: list[LedgerLine], periods: list[Period],
                                  lodged: dict, kwargs: dict, message: str) -> None:
    with pytest.raises(ValueError, match=message):
        reconcile_bas(lines, periods, lodged, **kwargs)
