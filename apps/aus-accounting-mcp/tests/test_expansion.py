"""Synthetic checks for grouped assessment and local reference access."""

import asyncio
import hashlib

import pytest
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.server import mcp


def call(name, **arguments):
    return asyncio.run(mcp.call_tool(name, arguments)).structured_content


def contribution(day, received, first=False, employee="synthetic"):
    return dict(employee_id=employee, qe_day=day, received=received, sg_amount="120.00",
                first_to_fund=first, out_of_cycle=False, db_interest=False)


def test_group_alignment_and_employee_separation():
    first = contribution("2027-07-01", "2027-07-05", True)
    second = contribution("2027-07-08", "2027-07-20")
    result = call("review_payday_super_contributions", contributions=[first, second],
                  as_at="2027-08-01")
    assert result["assessment_scope"] == "contribution_group"
    assert result["results"][1]["due"] == "2027-07-29"
    assert result["results"][1]["pathway"] == "ITEM4_ALIGNED"
    assert result["results"][1]["input_row"] == 2
    assert not any("covers one contribution" in c for c in result["results"][1]["caveats"])
    second["employee_id"] = "different"
    result = call("review_payday_super_contributions", contributions=[first, second],
                  as_at="2027-08-01")
    assert result["results"][1]["verdict"] == "LATE"


@pytest.mark.parametrize("change", [dict(first_to_fund="false"), dict(sg_amount=120),
                                    dict(unknown=True), dict(employee_id=" ")])
def test_group_nested_validation(change):
    row = contribution("2027-07-08", None)
    row.update(change)
    with pytest.raises(ToolError):
        call("review_payday_super_contributions", contributions=[row], as_at="2027-08-01")


def test_group_preserves_partial_and_missing_receipt():
    partial = contribution("2027-07-08", "2027-07-09")
    partial["matched_amount"] = "20.00"
    missing = contribution("2027-07-08", None, employee="different")
    result = call("review_payday_super_contributions", contributions=[partial, missing],
                  as_at="2027-08-01")
    assert result["results"][0]["base_shortfall"] == "100.00"
    assert result["results"][1]["verdict"] != "ON_TIME"


def test_library_search_and_bounded_citation(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path))
    raw = b"# Synthetic reference\n<!-- PDF page 3 -->\n## GST\nInput tax credits example.\n"
    (tmp_path / "example.md").write_bytes(raw)
    result = call("search_accounting_library", query="input tax credits")
    hit = result["matches"][0]
    assert hit["path"] == "example.md"
    assert hit["sha256"] == hashlib.sha256(raw).hexdigest()
    assert hit["pdf_page"] == 3
    assert "Input tax credits" in hit["text"]
    read = call("read_accounting_library", path=hit["path"], start_line=4, line_count=1)
    assert read["start_line"] == read["end_line"] == 4
    assert read["text"] == "Input tax credits example."
    with pytest.raises(ToolError):
        call("read_accounting_library", path="../outside.md")
    with pytest.raises(ToolError):
        call("read_accounting_library", path=".env.md")


def test_library_is_opt_in_and_does_not_read_links(tmp_path, monkeypatch):
    monkeypatch.delenv("AUS_ACCOUNTING_LIBRARY_ROOT", raising=False)
    with pytest.raises(ToolError, match="AUS_ACCOUNTING_LIBRARY_ROOT"):
        call("search_accounting_library", query="GST")
    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path))
    (tmp_path / "safe.md").write_text("# Safe\nGST example", encoding="utf-8")
    (tmp_path / ".hidden.md").write_text("GST secret", encoding="utf-8")
    assert len(call("search_accounting_library", query="GST")["matches"]) == 1


@pytest.mark.parametrize("facts,key,expected", [
    ({"kind": "gst", "amount": "1100", "gst_inclusive": True}, "gst", "100.00"),
    ({"kind": "resident_tax", "taxable_income": "45000"}, "basic_income_tax", "4288.00"),
    ({"kind": "capital_gains", "other_gains": "100", "discount_gains": "1000",
      "current_losses": "200", "prior_losses": "300"}, "net_capital_gain", "300.00"),
    ({"kind": "fbt", "year_ended": 2026, "type_one_value": "16500",
      "type_two_value": "6000"}, "fbt_estimate", "21452.73"),
    ({"kind": "depreciation", "cost": "3000", "effective_life": "4", "days": 146,
      "taxable_use": "0.4", "method": "prime_cost"}, "deduction", "120.00"),
    ({"kind": "quarterly_sg", "quarter": 1, "ordinary_time_earnings": "80000",
      "qualifying_contributions": "2000"}, "additional_contribution", "5500.00"),
])
def test_calculator_transport(facts, key, expected):
    facts = {"scope_confirmed": True, **facts}
    if facts["kind"] != "fbt":
        facts["year"] = "2025-26"
    r = call("calculate_tax_worksheet", facts=facts)
    assert r["amounts"][key] == expected
    assert r["sources"] and r["scope"] and r["warnings"]


@pytest.mark.parametrize("change", [{"scope_confirmed": "true"}, {"scope_confirmed": 1},
    {"scope_confirmed": False}, {"amount": 1100}, {"year": "2027-28"}, {"extra": True}])
def test_calculators_refuse_unestablished_or_malformed_facts(change):
    facts = dict(kind="gst", amount="1100", gst_inclusive=True, scope_confirmed=True,
                 year="2025-26")
    facts.update(change)
    with pytest.raises(ToolError):
        call("calculate_tax_worksheet", facts=facts)


def test_library_pagination_unicode_and_external_link(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path))
    (tmp_path / "example.md").write_text("GST one\nGST two\nGST three", encoding="utf-8")
    first = call("search_accounting_library", query="GST", limit=2)
    second = call("search_accounting_library", query="GST", limit=2, offset=first["next_offset"])
    assert first["has_more"] and not second["has_more"]
    assert [r["start_line"] for r in first["matches"] + second["matches"]] == [1, 2, 3]
    (tmp_path / "invalid.md").write_bytes(b"\xff")
    assert call("search_accounting_library", query="GST")["skipped_files"] == 1
    try:
        (tmp_path / "link.md").symlink_to(tmp_path / "example.md")
    except OSError:
        pytest.skip("Creating symlinks requires OS permission; path guards remain tested.")
    with pytest.raises(ToolError, match="links"):
        call("read_accounting_library", path="link.md")


def test_library_scan_budget_counts_invalid_utf8(tmp_path, monkeypatch):
    from aus_accounting_mcp import library

    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path))
    monkeypatch.setattr(library, "MAX_LIBRARY_BYTES", 6)
    for name in ("a.md", "b.md"):
        (tmp_path / name).write_bytes(b"\xffabc")
    with pytest.raises(ToolError, match="64 MB"):
        call("search_accounting_library", query="GST")


def test_library_internal_excerpt_rejects_non_positive_start():
    from aus_accounting_mcp.errors import InputError
    from aus_accounting_mcp.library import _excerpt

    for start in (0, -1):
        with pytest.raises(InputError, match="at least 1"):
            _excerpt("fixture.md", ["first", "second"], "digest", start, 1)
