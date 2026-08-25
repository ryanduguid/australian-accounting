import importlib
import json
from pathlib import Path

import pytest

from aus_accounting_mcp.server import (
    refuse_div7a,
    calc_payday_super_deadline,
    generate_synthetic_sbr_fixture,
    get_ato_benchmarks,
    list_ato_benchmark_industries,
)


def test_inlined_simulators_are_gone() -> None:
    for name in (
        "aus_accounting_mcp.engines.paydaysuper_sim",
        "aus_accounting_mcp.engines.benchmarks",
        "aus_accounting_mcp.engines.div7a",
    ):
        with pytest.raises(ModuleNotFoundError):
            importlib.import_module(name)


def test_payday_on_time_uses_payday_super_checker() -> None:
    payload = calc_payday_super_deadline(
        qe_day="2026-08-06",
        sg_amount="800.00",
        remitted="2026-08-07",
        received="2026-08-10",
        as_at="2026-08-21",
    )
    assert payload["ok"] is True
    assert payload["engine"] == "payday-super-checker"
    assert payload["law_content_date"] == "2026-08-15"
    assert payload["result"]["verdict"] == "ON_TIME"
    assert payload["result"]["experimental_sgc_high"] is None
    assert "clearing-house latency" in payload["disclaimer"]


def test_payday_without_fund_receipt_is_at_risk() -> None:
    payload = calc_payday_super_deadline(
        qe_day="2026-08-06",
        sg_amount="800.00",
        remitted="2026-08-07",
        as_at="2026-08-21",
    )
    assert payload["result"]["verdict"] == "AT_RISK"
    assert any("receipt by the fund" in c for c in payload["result"]["caveats"])


def test_payday_pre_regime_is_refused() -> None:
    with pytest.raises(ValueError, match="1 Jul 2026"):
        calc_payday_super_deadline(
            qe_day="2026-06-15",
            sg_amount="800.00",
            received="2026-06-20",
            as_at="2026-08-21",
        )


def test_payday_transition_period_cannot_be_confirmed_by_the_mcp() -> None:
    with pytest.raises(ValueError, match="this MCP cannot confirm"):
        calc_payday_super_deadline(
            qe_day="2026-07-09",
            sg_amount="800.00",
            received="2026-07-15",
            as_at="2026-08-10",
        )


def test_payday_rejects_non_decimal_amounts() -> None:
    with pytest.raises(ValueError, match="sg_amount"):
        calc_payday_super_deadline(
            qe_day="2026-08-06",
            sg_amount="nope",
            received="2026-08-10",
            as_at="2026-08-21",
        )


def test_ato_benchmarks_use_shipped_dataset() -> None:
    listed = list_ato_benchmark_industries(search="baker")
    assert listed["ok"] is True
    assert listed["engine"] == "ato-benchmark-compare"
    assert any(item["name"] == "Bakeries and hot bread shops" for item in listed["industries"])

    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        cost_of_sales="270000.00",
        cost_of_sales_labour="0",
        other_expense="437000.00",
        salary_wages="120000.00",
        contractor_commission="0",
        rent="40000.00",
        motor_vehicle="8000.00",
        associated_persons="0",
        w1="120000.00",
    )
    assert payload["ok"] is True
    assert payload["business_type"] == "Bakeries and hot bread shops"
    assert payload["complete_buckets"] is True
    assert payload["source"]["publisher"]
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    assert statuses["cost_of_sales_to_turnover"] == "within"


def test_ato_omitted_buckets_are_not_treated_as_zero() -> None:
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        cost_of_sales="270000.00",
    )
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    assert statuses["cost_of_sales_to_turnover"] == "within"
    assert statuses["rent_to_turnover"] == "not_supplied"
    assert statuses["total_expenses_to_turnover"] == "not_supplied"
    assert "rent" in payload["omitted_buckets"]
    assert payload["complete_buckets"] is False


def test_ato_partial_labour_picture_is_not_supplied() -> None:
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        salary_wages="120000.00",
    )
    statuses = {row["ratio"]: row["status"] for row in payload["ratios"]}
    assert statuses["labour_to_turnover"] == "not_supplied"


def test_ato_complete_labour_picture_is_evidenced() -> None:
    payload = get_ato_benchmarks(
        industry="Bakeries and hot bread shops",
        turnover="850000.00",
        salary_wages="120000.00",
        contractor_commission="0",
        cost_of_sales_labour="0",
    )
    rows = {row["ratio"]: row for row in payload["ratios"]}
    assert rows["labour_to_turnover"]["status"] != "not_supplied"
    assert rows["labour_to_turnover"]["value"] is not None


def test_ato_refuses_turnover_only() -> None:
    with pytest.raises(ValueError, match="no expense figures"):
        get_ato_benchmarks(industry="Bakeries and hot bread shops", turnover="850000.00")


def test_ato_unknown_industry_is_refused() -> None:
    with pytest.raises(ValueError, match="no ATO business type"):
        get_ato_benchmarks(
            industry="interstellar freight",
            turnover="100000.00",
            cost_of_sales="0",
        )


def test_div7a_is_refused() -> None:
    payload = refuse_div7a("Alice", "HoldingCo Pty Ltd", "50000.00")
    assert payload["ok"] is False
    assert payload["available"] is False
    assert payload["reviewed_engine"] is False
    assert payload["code"] == "ERR_POLICY_DIV7A_REFUSED"
    assert "payday-super-checker" in payload["reason"]


def test_synthetic_sbr_fixtures_are_labelled() -> None:
    ctr = generate_synthetic_sbr_fixture("CTR", revenue_or_sales="1000000.00")
    assert ctr["synthetic"] is True
    assert ctr["not_a_lodgment"] is True
    assert ctr["form_type"] == "CTR_AU_2025"
    assert ctr["income_statement"]["gross_profit"] == "600000.00"

    bas = generate_synthetic_sbr_fixture("BAS", revenue_or_sales="110000.00")
    assert bas["gst_labels"]["1A_gst_on_sales"] == "10000.00"


def test_client_snippets_use_uvx_from_github() -> None:
    root = Path(__file__).resolve().parents[1]
    expected_args = [
        "--from",
        "git+https://github.com/ryanduguid/au-tax-mcp-server",
        "aus-accounting-mcp",
    ]
    for name in ("cursor_mcp.json", "claude_desktop_config.json", "antigravity_config.json"):
        payload = json.loads((root / "clients" / name).read_text(encoding="utf-8"))
        server = payload["mcpServers"]["aus-accounting"]
        assert server["command"] == "uvx"
        assert server["args"] == expected_args
    readme = (root / "README.md").read_text(encoding="utf-8")
    disclaimer = (root / "DISCLAIMER.md").read_text(encoding="utf-8")
    assert "uvx --from git+https://github.com/ryanduguid/au-tax-mcp-server" in readme
    assert "DISCLAIMER.md" in readme
    assert "glama.ai/mcp/servers/ryanduguid/au-tax-mcp-server" in readme
    assert "not tax" in disclaimer.lower()
    assert "synthetic: true" in disclaimer
    citation = (root / "CITATION.cff").read_text(encoding="utf-8")
    assert "https://github.com/ryanduguid/au-tax-mcp-server" in citation
    glama = json.loads((root / "glama.json").read_text(encoding="utf-8"))
    assert glama["maintainers"] == ["ryanduguid"]
    pyproject = (root / "pyproject.toml").read_text(encoding="utf-8")
    # The engines stay pinned to an exact version, which is what the commit pins
    # used to buy. They cannot be pinned by URL: PyPI rejects a distribution
    # whose metadata carries a direct reference, so a git pin here would make
    # this package unpublishable and silently undo its own release process.
    assert "payday-super-checker==" in pyproject
    assert "ato-benchmark-compare==" in pyproject
    dependencies = pyproject.split("dependencies = [", 1)[1].split("]", 1)[0]
    assert "git+" not in dependencies
    assert "allow-direct-references" not in pyproject
