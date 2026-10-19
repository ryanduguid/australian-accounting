"""The GST review tools on fabricated lines, through the MCP boundary."""

import asyncio
import importlib.metadata

import pytest
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.adapters.gst import DISCLAIMER
from aus_accounting_mcp.money import MONEY_PATTERN
from aus_accounting_mcp.server import mcp


def call(name, **arguments):
    return asyncio.run(mcp.call_tool(name, arguments)).structured_content


def bank_line(line_id, entity, day, amount, **coding):
    return {"id": line_id, "entity": entity, "date": day, "amount": amount, **coding}


ALDER, BIRCH = "Synthetic Alder Pty Ltd", "Synthetic Birch Trust"
REGISTRATIONS = [
    {"entity": ALDER, "registered_from": "2024-07-01"},
    {"entity": BIRCH, "registered_from": None},
]
CODING = [
    bank_line("1", ALDER, "2026-08-03", "-110.00", account="Repairs", gst_code="GST",
              confirmed=True, supplier_registered=False),
    bank_line("2", ALDER, "2026-08-04", "-5000.00"),
    bank_line("3", BIRCH, "2026-08-05", "5000.00"),
    bank_line("4", BIRCH, "2026-08-06", "330.00", suggested_account="Rent",
              suggested_gst_code="GST"),
    bank_line("5", ALDER, "2026-08-07", "-20.00", account="Bank fees", gst_code="FRE",
              confirmed=True),
]


def ledger_line(line_id, side, gross, gst, invoiced, paid, code="GST"):
    return {"id": line_id, "side": side, "gross": gross, "gst": gst, "gst_code": code,
            "invoice_date": invoiced, "payment_date": paid}


LEDGER = [
    ledger_line("inv-101", "sale", "1100.00", "100.00", "2026-07-14", "2026-07-30"),
    ledger_line("inv-102", "sale", "550.00", "50.00", "2026-09-25", "2026-10-06"),
    ledger_line("bill-7", "purchase", "330.00", "30.00", "2026-08-02", "2026-08-02"),
]
Q1 = {"label": "2026-Q1", "start": "2026-07-01", "end": "2026-09-30",
      "lodged": {"G1": "1650", "1A": "150", "1B": "30"}}


def test_coding_review_returns_the_engine_states_with_its_provenance():
    result = call("review_gst_coding", lines=CODING, registrations=REGISTRATIONS)

    assert [(row["id"], row["state"]) for row in result["lines"]] == [
        ("1", "gst_differs_from_registration"),
        ("2", "between_entities"),
        ("3", "between_entities"),
        ("4", "gst_differs_from_registration"),
        ("5", "coded"),
    ]
    assert result["lines"][0]["sections"] == ["GST Act s 9-5(d)", "GST Act s 11-5(b)"]
    assert result["lines"][1]["counterpart"] == "3"
    assert result["engine"] == "gst-review"
    assert result["engine_version"] == importlib.metadata.version("gst-review")
    assert result["disclaimer"] == DISCLAIMER
    assert result["facts_used"]["registrations"][BIRCH] == {
        "registered_from": None, "registered_to": None}
    assert result["not_decided"] and result["warnings"] and result["sources"]


def test_coding_review_reads_a_caller_code_table_and_match_window():
    lines = [bank_line("1", ALDER, "2026-08-03", "-110.00", gst_code="G-IN"),
             bank_line("2", BIRCH, "2026-08-10", "110.00")]
    result = call("review_gst_coding", lines=lines, registrations=REGISTRATIONS,
                  match_window_days=7, gst_codes={"G-IN": "gst_free"})

    assert [row["state"] for row in result["lines"]] == ["between_entities"] * 2
    assert result["facts_used"]["gst_codes"] == {"G-IN": "gst_free"}


@pytest.mark.parametrize("lines,registrations,message", [
    (CODING, REGISTRATIONS[:1], "No registration facts for entity"),
    (CODING, [*REGISTRATIONS, REGISTRATIONS[0]], "appears more than once"),
    ([bank_line("1", ALDER, "2026-02-30", "1.00")], REGISTRATIONS, "is not a YYYY-MM-DD date"),
    ([bank_line("1", ALDER, "2026-08-03", "1.00", confirmed=True)], REGISTRATIONS,
     "confirmed without an account and GST code"),
    ([bank_line("1", ALDER, "2026-08-03", "1.00")] * 2, REGISTRATIONS, "must be unique"),
    ([bank_line("1", ALDER, "2026-08-03", "1.00")],
     [{"entity": ALDER, "registered_from": "2026-07-01", "registered_to": "2025-07-01"}],
     "ends before it starts"),
])
def test_coding_review_refuses_facts_the_engine_cannot_use(lines, registrations, message):
    with pytest.raises(ToolError, match=message):
        call("review_gst_coding", lines=lines, registrations=registrations)


@pytest.mark.parametrize("change", [
    {"lines": [bank_line("1", ALDER, "2026-08-03", "1,000.00")]},
    {"lines": [bank_line("1", ALDER, "3/08/2026", "1.00")]},
    {"lines": [bank_line("1", ALDER, "2026-08-03", "1.00", confirmed="true")]},
    {"lines": [bank_line("1", ALDER, "2026-08-03", "1.00", note="x")]},
    {"registrations": [{"entity": ALDER}]},
    {"gst_codes": {"GST": "standard"}},
    {"match_window_days": 32},
])
def test_coding_review_rejects_malformed_arguments(change):
    arguments = {"lines": CODING, "registrations": REGISTRATIONS, **change}
    with pytest.raises(ToolError):
        call("review_gst_coding", **arguments)


def test_reconciliation_on_the_cash_basis_names_the_timing_line():
    result = call("reconcile_bas", lines=LEDGER, periods=[Q1], basis="cash")
    period = result["periods"][0]

    assert period["labels"]["G1"] == {
        "ledger": "1100.00", "ledger_whole_dollars": "1100", "lodged": "1650",
        "difference": "-550", "lines": ["inv-101"], "status": "differs"}
    assert period["labels"]["1A"]["difference"] == "-50"
    assert period["labels"]["1B"]["status"] == "matches"
    assert period["labels"]["G10"]["status"] == "not_reported"
    assert period["timing"] == ["inv-102"]
    assert result["outside_periods"] == ["inv-102"]
    assert result["engine_version"] == importlib.metadata.version("gst-review")
    assert result["disclaimer"] == DISCLAIMER


def test_reconciliation_on_accruals_matches_and_lists_unlodged_periods():
    q2 = {"label": "2026-Q2", "start": "2026-10-01", "end": "2026-12-31"}
    unpaid = ledger_line("inv-103", "sale", "220.00", "20.00", "2026-11-02", None)
    result = call("reconcile_bas", lines=[*LEDGER, unpaid], periods=[Q1, q2],
                  basis="accruals")

    labels = result["periods"][0]["labels"]
    assert [labels[label]["status"] for label in ("G1", "1A", "1B")] == ["matches"] * 3
    assert result["not_lodged"] == ["2026-Q2"]
    assert result["periods"][1]["lodged"] is False


def test_reconciliation_excludes_gst_and_reads_a_caller_code_table():
    lines = [ledger_line("s-1", "sale", "1100.00", "100.00", "2026-07-14", "2026-07-14", "OUT"),
             ledger_line("s-2", "sale", "100.00", "0.00", "2026-07-15", "2026-07-15", "N-T")]
    period = {**Q1, "lodged": {"G1": "1000", "1A": "100"}}
    result = call("reconcile_bas", lines=lines, periods=[period], basis="cash",
                  g1_includes_gst=False, bas_codes={"OUT": "taxable", "N-T": "excluded"})

    assert result["periods"][0]["labels"]["G1"]["status"] == "matches"
    assert result["periods"][0]["kept_out"] == ["s-2"]
    assert "line_issues" not in result


def test_reconciliation_refuses_a_line_it_cannot_place():
    # Comparing without the line would report G1 as matching a total that omits it.
    unplaced = ledger_line("s-9", "sale", "110.00", "10.00", "2026-07-01", "2026-07-01", "FRE")
    with pytest.raises(ToolError, match="'s-9': GST of 10.00 on code 'FRE'"):
        call("reconcile_bas", lines=[unplaced], periods=[{**Q1, "lodged": {"G1": "0"}}],
             basis="cash")


@pytest.mark.parametrize("periods,message", [
    ([{**Q1, "lodged": {"G1": "1650.50"}}], "whole non-negative dollars"),
    ([{**Q1, "lodged": {"G1": "-1"}}], "whole non-negative dollars"),
    ([Q1, {**Q1, "label": "overlap"}], "overlaps"),
    ([{**Q1, "end": "2026-06-30"}], "start on or before its end"),
    ([{**Q1, "start": "2026-13-01"}], "is not a YYYY-MM-DD date"),
])
def test_reconciliation_refuses_periods_the_engine_cannot_use(periods, message):
    with pytest.raises(ToolError, match=message):
        call("reconcile_bas", lines=LEDGER, periods=periods, basis="cash")


@pytest.mark.parametrize("change", [
    {"basis": "hybrid"},
    {"g1_includes_gst": "false"},
    {"g1_includes_gst": 0},
    {"periods": [{**Q1, "lodged": {"G9": "1"}}]},
    {"periods": [{**Q1, "lodged": {"G1": "1e3"}}]},
    {"lines": [{**LEDGER[0], "side": "income"}]},
    {"bas_codes": {"GST": "standard"}},
])
def test_reconciliation_rejects_malformed_arguments(change):
    arguments = {"lines": LEDGER, "periods": [Q1], "basis": "cash", **change}
    with pytest.raises(ToolError):
        call("reconcile_bas", **arguments)


def test_lodged_figures_publish_the_plain_money_pattern():
    tool = next(tool for tool in asyncio.run(mcp.list_tools()) if tool.name == "reconcile_bas")
    lodged = tool.input_schema["$defs"]["BasPeriodInput"]["properties"]["lodged"]["anyOf"][0]

    assert lodged["additionalProperties"]["pattern"] == MONEY_PATTERN
