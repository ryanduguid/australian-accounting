# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

from __future__ import annotations

import asyncio
import json

import pytest

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import (
    get_div7a_benchmark_rate,
    mcp,
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
    assert payload["engine_version"] == "0.3.2"
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
    payload = refuse_div7a()

    assert payload["ok"] is False
    assert payload["reviewed_engine"] is True
    assert payload["code"] == "ERR_POLICY_DIV7A_SCOPE_REFUSED"
    assert "unpaid present entitlements" in payload["reason"]


def test_a_year_label_the_engine_cannot_read_is_an_input_error() -> None:
    from aus_accounting_mcp.adapters.div7a import get_benchmark_rate

    with pytest.raises(InputError, match="2026-27"):
        get_benchmark_rate("2026-2027")


ABOVE_MAXIMUM = "REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM"
ABOVE_LOAN_TERM = "REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM"
SECURED = {
    "secured_by_registered_mortgage_over_real_property": True,
    "security_coverage_at_first_made": "1.10",
}


@pytest.mark.parametrize(
    ("overrides", "codes"),
    [
        ({"remaining_term_years": "7"}, []),
        ({"remaining_term_years": "8"}, [ABOVE_MAXIMUM]),
        ({"maximum_term_years": "5", "remaining_term_years": "5"}, []),
        ({"maximum_term_years": "5", "remaining_term_years": "6"}, [ABOVE_LOAN_TERM]),
        ({"maximum_term_years": "5", "remaining_term_years": "5.5"}, [ABOVE_LOAN_TERM]),
        ({"maximum_term_years": "5.5", "remaining_term_years": "6"}, []),
        ({"maximum_term_years": "5", "remaining_term_years": "8"}, [ABOVE_MAXIMUM]),
        ({**SECURED, "maximum_term_years": "25", "remaining_term_years": "25"}, []),
        ({**SECURED, "maximum_term_years": "25", "remaining_term_years": "26"}, [ABOVE_MAXIMUM]),
        ({**SECURED, "maximum_term_years": "20", "remaining_term_years": "21"}, [ABOVE_LOAN_TERM]),
        (
            {
                "secured_by_registered_mortgage_over_real_property": None,
                "remaining_term_years": "8",
            },
            [ABOVE_LOAN_TERM],
        ),
        ({"written_agreement": False, "remaining_term_years": "8"}, ["REFUSED_GATE_NOT_COMPLYING"]),
        ({"year_loan_made": None, "remaining_term_years": "8"}, ["REFUSED_GATE_UNKNOWN"]),
        (
            {"payments_applied_during_the_year": None, "remaining_term_years": "8"},
            [ABOVE_MAXIMUM],
        ),
        (
            {
                "year_loan_made": "2026-27",
                "interest_rate_for_years_after_year_loan_made": "0.0900",
                "remaining_term_years": "8",
            },
            ["REFUSED_YEAR_IS_YEAR_OF_LOAN", ABOVE_MAXIMUM],
        ),
    ],
)
def test_a_remaining_term_past_the_maximum_or_loan_term_gets_no_figure(
    overrides: dict[str, object], codes: list[str]
) -> None:
    for detail in ("summary", "full"):
        myr = _review(response_detail=detail, **overrides)["minimum_yearly_repayment"]

        assert myr["reason_codes"] == codes, detail
        assert len(myr["reasons"]) == len(codes), detail
        assert (myr["verdict"] == "REFUSED") == bool(codes), detail
        assert (myr["myr_required"] is None) == bool(codes), detail
        assert (myr["shortfall"] is None) == bool(codes), detail
        assert myr["experimental_deemed_dividend_exposure"] is None, detail


def test_a_term_refusal_keeps_the_engines_refused_shape() -> None:
    myr = _review(response_detail="full", remaining_term_years="8")["minimum_yearly_repayment"]

    assert myr["reasons"][0].startswith(
        "remaining_term_years is 8, a remaining term of 8 under s 109E(6), longer than the "
        "7-year maximum term this row's security facts give under s 109N(3)"
    )
    assert myr["reasons"][0].endswith("No MYR figure is emitted.")
    assert myr["remaining_term_years_used"] == "8"
    assert myr["benchmark_rate"] == "0.0877"
    assert myr["payments_applied"] == "108770.00"
    assert len(myr["statutory_trace"]) == 2


def test_a_term_refusal_the_engine_already_gives_is_not_given_twice(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from dataclasses import replace

    from div7aloan import MyrVerdict

    from aus_accounting_mcp.adapters import div7a

    engine = div7a.minimum_yearly_repayment

    def refusing(facts):  # an engine release that refuses by itself
        return replace(
            engine(facts),
            verdict=MyrVerdict.REFUSED,
            myr_required=None,
            shortfall=None,
            experimental_deemed_dividend_exposure=None,
            reasons=("the engine's own reason",),
            reason_codes=(ABOVE_MAXIMUM,),
        )

    monkeypatch.setattr(div7a, "minimum_yearly_repayment", refusing)
    myr = _review(remaining_term_years="8")["minimum_yearly_repayment"]

    assert myr["reason_codes"] == [ABOVE_MAXIMUM]
    assert myr["reasons"] == ["the engine's own reason"]


ABOVE_TERM_LEFT = "REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED"


def _loan_made(year_loan_made: str, year_of_income: str, **overrides: object) -> dict[str, object]:
    return {
        "year_loan_made": year_loan_made,
        "year_of_income": year_of_income,
        "interest_rate_for_years_after_year_loan_made": "0.0900",
        **overrides,
    }


@pytest.mark.parametrize(
    ("overrides", "codes"),
    [
        # A 7-year loan made in 2024-25: the first repayment year, nothing elapsed.
        (_loan_made("2024-25", "2025-26", remaining_term_years="7"), []),
        (_loan_made("2024-25", "2025-26", remaining_term_years="8"), [ABOVE_MAXIMUM]),
        # The second: 1 year has, so 6 remain.
        (_loan_made("2024-25", "2026-27", remaining_term_years="6"), []),
        (_loan_made("2024-25", "2026-27", remaining_term_years="7"), [ABOVE_TERM_LEFT]),
        (_loan_made("2024-25", "2026-27", remaining_term_years="6.5"), [ABOVE_TERM_LEFT]),
        (
            _loan_made("2024-25", "2026-27", maximum_term_years="5", remaining_term_years="6"),
            [ABOVE_LOAN_TERM],
        ),
        # A loan made in 2022-23: 3 years have.
        (_loan_made("2022-23", "2026-27", remaining_term_years="4"), []),
        (_loan_made("2022-23", "2026-27", remaining_term_years="5"), [ABOVE_TERM_LEFT]),
        # A fractional term rounds up after the subtraction: 20.5 less 1 is 20.
        (
            _loan_made("2024-25", "2026-27", **SECURED, maximum_term_years="20.5",
                       remaining_term_years="20"),
            [],
        ),
        (
            _loan_made("2024-25", "2026-27", **SECURED, maximum_term_years="20.5",
                       remaining_term_years="20.5"),
            [ABOVE_TERM_LEFT],
        ),
        # The term rounds up before the years come off, so a term a hair over 6
        # keeps 7 years in the first repayment year, however many digits it has.
        (
            _loan_made("2024-25", "2025-26", maximum_term_years="6." + "0" * 27 + "1",
                       remaining_term_years="7"),
            [],
        ),
        # A 1-year term has no years left in its second repayment year.
        (
            _loan_made("2024-25", "2026-27", maximum_term_years="1", remaining_term_years="1"),
            [ABOVE_TERM_LEFT],
        ),
        (
            _loan_made("2024-25", "2026-27", maximum_term_years="1", remaining_term_years="0"),
            ["REFUSED_REMAINING_TERM_NOT_POSITIVE"],
        ),
        # Secured and extended under s 109E(3A) in 2024-25: the new amalgamated
        # loan s 109E(3B) treats as made just before 1 July 2024.
        (
            _loan_made("2023-24", "2024-25", **SECURED, maximum_term_years="25",
                       remaining_term_years="25"),
            [],
        ),
        (
            _loan_made("2023-24", "2026-27", **SECURED, maximum_term_years="25",
                       remaining_term_years="23"),
            [],
        ),
        (
            _loan_made("2023-24", "2026-27", **SECURED, maximum_term_years="25",
                       remaining_term_years="24"),
            [ABOVE_TERM_LEFT],
        ),
        (
            _loan_made("2024-25", "2026-27", written_agreement=False, remaining_term_years="7"),
            ["REFUSED_GATE_NOT_COMPLYING"],
        ),
        (
            _loan_made("2024-25", "2024-25", remaining_term_years="7"),
            ["REFUSED_YEAR_IS_YEAR_OF_LOAN"],
        ),
    ],
)
def test_a_remaining_term_past_the_term_left_gets_no_figure(
    overrides: dict[str, object], codes: list[str]
) -> None:
    for detail in ("summary", "full"):
        myr = _review(response_detail=detail, **overrides)["minimum_yearly_repayment"]

        assert myr["reason_codes"] == codes, detail
        assert (myr["myr_required"] is None) == bool(codes), detail
        assert (myr["shortfall"] is None) == bool(codes), detail


def test_the_term_left_refusal_counts_the_years_since_the_loan_year() -> None:
    myr = _review(**_loan_made("2022-23", "2026-27", remaining_term_years="5"))[
        "minimum_yearly_repayment"
    ]

    assert "From the end of 2022-23 to the end of 2025-26 is 3 years, so the remaining " \
        "term is at most 4;" in myr["reasons"][0]
    assert myr["reasons"][0].endswith("No MYR figure is emitted.")


def test_a_term_the_years_outrun_leaves_a_remaining_term_of_nil() -> None:
    # 2.5 years less 3: the limit is 0, not the -0 a ceiling of -0.5 prints.
    myr = _review(
        **_loan_made("2022-23", "2026-27", maximum_term_years="2.5", remaining_term_years="1")
    )["minimum_yearly_repayment"]

    assert myr["reason_codes"] == [ABOVE_TERM_LEFT]
    assert "which leaves a remaining term of 0, so the loan should" in myr["reasons"][0]


def test_the_term_left_refusal_says_how_to_enter_an_extended_loan() -> None:
    myr = _review(**_loan_made("2022-23", "2026-27", remaining_term_years="5"))[
        "minimum_yearly_repayment"
    ]

    assert myr["reasons"][0].endswith(
        "so its year_loan_made is the year before that later year, and its "
        "maximum_term_years, as the ATO works it, is 25 years less the time the loan had "
        "already run before the extension. No MYR figure is emitted."
    )


def _description(name: str) -> str:
    tools = asyncio.run(mcp.list_tools())
    tool = next(t for t in tools if t.name == "review_div7a_loan")
    return tool.input_schema["properties"][name]["description"]


def test_the_loan_year_description_gives_the_s_109e_3b_entry_rule() -> None:
    assert (
        "For a loan secured by a mortgage and extended under s 109E(3A), the year before the "
        "year of the extension" in _description("year_loan_made")
    )


def test_the_term_description_gives_an_extended_loans_term() -> None:
    assert (
        "For a loan extended under s 109E(3A), 25 less the years it had already run before "
        "the extension" in _description("maximum_term_years")
    )


def test_the_remaining_term_description_gives_its_limit() -> None:
    description = _description("remaining_term_years")

    assert (
        "at most maximum_term_years less the whole years from the end of year_loan_made"
        in description
    )
    assert "A longer term, or one above the s 109N(3) maximum, is REFUSED." in description
