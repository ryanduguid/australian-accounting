from __future__ import annotations

import json

import pytest

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import (
    get_div7a_benchmark_rate,
    refuse_div7a,
    review_div7a_loan,
)


def _review(**overrides):
    arguments = {
        "year_of_income": "2026-27",
        "year_loan_made": "2025-26",
        "written_agreement": True,
        "terms_in_place_before_lodgment_day": True,
        "maximum_term_years": "7",
        "secured_by_registered_mortgage_over_real_property": False,
        "security_coverage_at_first_made": None,
        "interest_rate_for_years_after_year_loan_made": "0.0837",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": "1",
        "payments_applied_during_the_year": "108770.00",
        "loan_id": "synthetic-loan-1",
    }
    arguments.update(overrides)
    return review_div7a_loan(**arguments)


def test_known_benchmark_rate_keeps_engine_provenance() -> None:
    payload = get_div7a_benchmark_rate("2025-26", response_detail="full")

    assert payload["ok"] is True
    assert payload["engine"] == "div7a-loan-review"
    assert payload["engine_version"] == "0.1.0"
    assert payload["verdict"] == "KNOWN"
    assert payload["benchmark_rate"] == "0.0837"
    assert payload["provenance"]["rba_month"] == "2025-05"
    assert payload["statutory_trace"]


def test_unreviewed_benchmark_year_stays_unknown() -> None:
    payload = get_div7a_benchmark_rate("2027-28")

    assert payload["ok"] is True
    assert payload["verdict"] == "UNKNOWN"
    assert payload["benchmark_rate"] is None
    assert "does not extrapolate" in payload["reason"]


def test_review_reports_a_met_minimum_yearly_repayment() -> None:
    payload = _review(response_detail="full")

    assert payload["ok"] is True
    assert payload["gate"]["verdict"] == "COMPLYING"
    assert payload["minimum_yearly_repayment"]["verdict"] == "MYR_MET"
    assert payload["minimum_yearly_repayment"]["myr_required"] == "108770.00"
    assert payload["minimum_yearly_repayment"]["shortfall"] == "0.00"
    assert payload["minimum_yearly_repayment"]["statutory_trace"]


def test_summary_review_is_less_than_half_the_full_payload_without_losing_outcomes() -> None:
    summary = _review()
    full = _review(response_detail="full")

    assert len(json.dumps(summary)) < len(json.dumps(full)) / 2
    assert summary["response_detail"] == "summary"
    assert summary["engine"] == full["engine"]
    assert summary["engine_version"] == full["engine_version"]
    assert summary["law_content_date"] == full["law_content_date"]
    assert summary["gate"]["verdict"] == "COMPLYING"
    assert summary["minimum_yearly_repayment"]["verdict"] == "MYR_MET"
    assert summary["minimum_yearly_repayment"]["myr_required"] == "108770.00"
    assert summary["minimum_yearly_repayment"]["shortfall"] == "0.00"
    assert summary["source"]["verify_at"]
    assert "statutory_trace" not in summary["gate"]
    assert "benchmark_provenance" not in summary["minimum_yearly_repayment"]


def test_summary_rate_keeps_result_and_concise_source_while_full_keeps_audit_data() -> None:
    summary = get_div7a_benchmark_rate("2025-26")
    full = get_div7a_benchmark_rate("2025-26", response_detail="full")

    assert len(json.dumps(summary)) < len(json.dumps(full)) / 2
    assert summary["response_detail"] == "summary"
    assert summary["verdict"] == full["verdict"] == "KNOWN"
    assert summary["benchmark_rate"] == full["benchmark_rate"] == "0.0837"
    assert summary["source"]["verify_at"] == full["provenance"]["verify_at"]
    assert "statutory_trace" not in summary
    assert "provenance" not in summary
    assert full["statutory_trace"]
    assert full["provenance"]


def test_response_detail_rejects_values_other_than_summary_or_full() -> None:
    with pytest.raises(InputError, match="response_detail"):
        get_div7a_benchmark_rate("2025-26", response_detail="brief")


def test_review_reports_shortfall_as_exposure_not_a_determination() -> None:
    payload = _review(payments_applied_during_the_year="100000.00")
    myr = payload["minimum_yearly_repayment"]

    assert myr["verdict"] == "MYR_SHORT"
    assert myr["shortfall"] == "8770.00"
    assert myr["experimental_deemed_dividend_exposure"] == "8770.00"
    assert any("not an ATO assessment" in caveat for caveat in myr["caveats"])


def test_unknown_gate_fact_never_becomes_non_complying_or_a_number() -> None:
    payload = _review(written_agreement=None)

    assert payload["gate"]["verdict"] == "UNKNOWN"
    assert payload["minimum_yearly_repayment"]["verdict"] == "REFUSED"
    assert payload["minimum_yearly_repayment"]["myr_required"] is None


def test_legacy_div7a_tool_refuses_only_the_unsupported_scope() -> None:
    payload = refuse_div7a("Alice", "HoldingCo Pty Ltd", "50000.00")

    assert payload["ok"] is False
    assert payload["reviewed_engine"] is True
    assert payload["code"] == "ERR_POLICY_DIV7A_SCOPE_REFUSED"
    assert "unpaid present entitlements" in payload["reason"]
