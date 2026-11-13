# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Thin adapter over div7a-loan-review.

The delegated engine owns the statutory tests, rates, arithmetic, refusals and
serialisation. This module translates MCP arguments, applies the facade's money
boundary and adds the remaining-term refusals the pinned release lacks.
"""

from __future__ import annotations

from dataclasses import replace
from typing import Any, Literal

from div7aloan import (
    LAW_COMPILATION,
    LAW_CONTENT_DATE,
    GateFacts,
    GateResult,
    GateVerdict,
    MyrFacts,
    MyrResult,
    MyrVerdict,
    YearOfIncome,
    __version__ as DIV7A_VERSION,
    benchmark_rate,
    complying_loan_gate,
    minimum_yearly_repayment,
    parse_year,
)
from div7aloan.gate import _maximum_term
from div7aloan.myr import statutory_remaining_term

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.money import parse_optional_amount

DISCLAIMER = (
    "Experimental review aid. Not a Division 7A determination, an ATO assessment "
    "or professional advice. The operator supplies the statutory facts. Verify the "
    "result against the compiled Act and obtain human review before acting."
)


def _validate_response_detail(response_detail: str) -> None:
    if response_detail not in {"summary", "full"}:
        raise InputError("response_detail must be 'summary' or 'full'")


def get_benchmark_rate(
    year_of_income: str,
    response_detail: Literal["summary", "full"] = "summary",
) -> dict[str, Any]:
    _validate_response_detail(response_detail)
    try:
        result = benchmark_rate(year_of_income)
    except ValueError as exc:
        raise InputError(str(exc)) from exc
    payload = result.to_json_dict()
    payload.update(
        {
            "ok": True,
            "engine": "div7a-loan-review",
            "engine_version": DIV7A_VERSION,
            "law_content_date": LAW_CONTENT_DATE,
            "law_compilation": LAW_COMPILATION,
            "disclaimer": DISCLAIMER,
        }
    )
    if response_detail == "full":
        return payload
    return {
        **{key: payload[key] for key in (
            "ok",
            "engine",
            "engine_version",
            "law_content_date",
            "law_compilation",
            "disclaimer",
            "year_of_income",
            "verdict",
            "benchmark_rate",
            "reason",
        )},
        "response_detail": "summary",
        "source": {
            "verify_at": payload["provenance"]["verify_at"],
            "rate_table_uris": payload["manifest"]["rate_table_uris"],
        },
    }


# Division 7A Loan Review 1.0.0 refuses a remaining term that cannot hold on a
# loan the gate passed (src/myr.rs, repayment_within); the pinned 0.3.2 works a
# lower repayment from it. These copy its conditions, codes and wording. Remove
# them once review_loan calls an engine release that gives these refusals
# itself, such as 1.0.0's review-register; its minimum-yearly-repayment takes
# no loan term and does not. Until then a code the engine gave is not repeated.
def _remaining_term_refusal(
    gate: GateResult, gate_facts: GateFacts, myr_facts: MyrFacts
) -> tuple[str, str] | None:
    supplied = myr_facts.remaining_term_years
    if supplied is None or gate.verdict is not GateVerdict.COMPLYING:
        return None
    used = statutory_remaining_term(supplied)
    # Not gate.maximum_term_years_allowed: with the security unknown the gate
    # still allows 7 years, where the security facts give no maximum and the
    # Rust engine does not give this refusal.
    maximum = _maximum_term(gate_facts)[0]
    if maximum is not None and used > maximum:
        return (
            "REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM",
            f"remaining_term_years is {supplied}, a remaining term of {used} under "
            f"s 109E(6), longer than the {maximum}-year maximum term this row's "
            "security facts give under s 109N(3) before any s 109N(3A) to (3D) "
            "reduction, which can only shorten it, yet the gate found the loan on "
            "s 109N terms. Both cannot hold: each constituent loan is one s 109N "
            "saves (s 109E(3)(b)), so its term is within its maximum term "
            "(s 109N(1)(c)); the constituent loans share one maximum term "
            "(s 109E(3)(c)); and the remaining term is the longest constituent "
            "term less the years since the loan year (s 109E(6)). Correct the "
            "remaining term or the security facts. Enter a loan secured and "
            "extended under s 109E(3A) as the new amalgamated loan s 109E(3B) "
            "creates, with its 25-year security facts. No MYR figure is emitted.",
        )
    term = gate_facts.maximum_term_years
    if term is not None and used > statutory_remaining_term(term):
        return (
            "REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM",
            f"remaining_term_years is {supplied}, a remaining term of {used} under "
            f"s 109E(6), longer than the {term}-year term of the loan in "
            "maximum_term_years. The remaining term is the longest constituent-loan "
            "term less the years from the end of the loan year to the end of the "
            "previous year of income (s 109E(6)), so it cannot be longer than that "
            "term, and a longer one gives a lower minimum yearly repayment than the "
            "Act requires. Correct remaining_term_years or, where a constituent loan "
            "of this amalgamated loan has a longer term, enter that longest term in "
            "maximum_term_years. No MYR figure is emitted.",
        )
    made, year = myr_facts.year_loan_made, myr_facts.year_of_income
    if term is None or made is None or year is None or not made < year:
        return None
    # s 109E(6)(b): the whole years from the end of the loan year to the end of
    # the year before this one, and the loan's term less them, rounded up. The
    # term is rounded up first: the same figure, as the years are whole, and
    # exact, where subtracting first would round a long term to the context
    # precision.
    elapsed = year.start_year - made.start_year - 1
    left = statutory_remaining_term(term) - elapsed
    if used <= 0 or used <= left:
        return None
    previous = YearOfIncome(year.start_year - 1)
    years = "year" if elapsed == 1 else "years"
    limit = (
        f"so the remaining term is at most {left}"
        if left > 0
        else f"which leaves a remaining term of {left}, so the loan should have been "
        "repaid in full by the end of the previous year of income"
    )
    return (
        "REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED",
        f"remaining_term_years is {supplied}, a remaining term of {used} under "
        f"s 109E(6), longer than the {term}-year term in maximum_term_years less "
        "the years since the loan year. s 109E(6) makes the remaining term the "
        "difference between (a) 'the number of years in the longest term of any of "
        "the constituent loans that the amalgamated loan takes account of' and (b) "
        "'the number of years between the end of the private company's year of "
        "income in which the loan was made and the end of the private company's "
        "year of income before the year of income for which the minimum yearly "
        "repayment is being worked out', rounded up if not already a whole number. "
        f"From the end of {made.label} to the end of {previous.label} is {elapsed} "
        f"{years}, {limit}; a longer remaining term gives a lower minimum yearly "
        "repayment than the Act requires. Correct remaining_term_years, "
        "year_loan_made or maximum_term_years. A loan secured and extended under "
        "s 109E(3A) is the new amalgamated loan s 109E(3B) treats as made just "
        "before the start of the later year of income, so its year_loan_made is the "
        "year before that later year. No MYR figure is emitted.",
    )


def _refuse(myr: MyrResult, code: str, reason: str) -> MyrResult:
    if code in myr.reason_codes:
        return myr
    # A refusal joins the engine's refusals and replaces its unknowns.
    earlier = myr.verdict is MyrVerdict.REFUSED
    return replace(
        myr,
        verdict=MyrVerdict.REFUSED,
        myr_required=None,
        shortfall=None,
        experimental_deemed_dividend_exposure=None,
        reasons=(*myr.reasons, reason) if earlier else (reason,),
        reason_codes=(*myr.reason_codes, code) if earlier else (code,),
        # The engine's refused result keeps only its s 109E(1) and (5) lines.
        statutory_trace=myr.statutory_trace[:2],
    )


def review_loan(
    *,
    year_of_income: str,
    year_loan_made: str | None = None,
    written_agreement: bool | None = None,
    terms_in_place_before_lodgment_day: bool | None = None,
    maximum_term_years: str | None = None,
    secured_by_registered_mortgage_over_real_property: bool | None = None,
    security_coverage_at_first_made: str | None = None,
    interest_rate_for_years_after_year_loan_made: str | None = None,
    amalgamated_loan_unpaid_at_end_of_previous_year: str | None = None,
    remaining_term_years: str | None = None,
    payments_applied_during_the_year: str | None = None,
    loan_id: str = "mcp-div7a-1",
    response_detail: Literal["summary", "full"] = "summary",
) -> dict[str, Any]:
    _validate_response_detail(response_detail)
    try:
        gate_facts = GateFacts.from_mapping(
            {
                "loan_id": loan_id,
                "year_loan_made": year_loan_made,
                "written_agreement": written_agreement,
                "terms_in_place_before_lodgment_day": terms_in_place_before_lodgment_day,
                "maximum_term_years": maximum_term_years,
                "secured_by_registered_mortgage_over_real_property": (
                    secured_by_registered_mortgage_over_real_property
                ),
                "security_coverage_at_first_made": security_coverage_at_first_made,
                "interest_rate_for_years_after_year_loan_made": (
                    interest_rate_for_years_after_year_loan_made
                ),
            },
            "MCP review_div7a_loan",
        )
        gate = complying_loan_gate(gate_facts)
        year = parse_year(year_of_income)
        parsed_myr = MyrFacts.from_mapping(
            {
                "loan_id": loan_id,
                "year_loan_made": year_loan_made,
                "remaining_term_years": remaining_term_years,
            },
            year,
            gate,
            "MCP review_div7a_loan",
        )
        myr = minimum_yearly_repayment(
            MyrFacts(
                loan_id=parsed_myr.loan_id,
                year_of_income=parsed_myr.year_of_income,
                year_loan_made=parsed_myr.year_loan_made,
                gate_result=gate,
                remaining_term_years=parsed_myr.remaining_term_years,
                amalgamated_loan_unpaid_at_end_of_previous_year=parse_optional_amount(
                    amalgamated_loan_unpaid_at_end_of_previous_year,
                    "amalgamated_loan_unpaid_at_end_of_previous_year",
                ),
                payments_applied_during_the_year=parse_optional_amount(
                    payments_applied_during_the_year,
                    "payments_applied_during_the_year",
                ),
            )
        )
    except ValueError as exc:
        raise InputError(str(exc)) from exc
    refusal = _remaining_term_refusal(gate, gate_facts, parsed_myr)
    if refusal is not None:
        myr = _refuse(myr, *refusal)

    gate_payload: dict[str, Any] = gate.to_json_dict()
    repayment_payload: dict[str, Any] = myr.to_json_dict()
    payload: dict[str, Any] = {
        "ok": True,
        "engine": "div7a-loan-review",
        "engine_version": DIV7A_VERSION,
        "law_content_date": LAW_CONTENT_DATE,
        "law_compilation": LAW_COMPILATION,
        "disclaimer": DISCLAIMER,
        "gate": gate_payload,
        "minimum_yearly_repayment": repayment_payload,
    }
    if response_detail == "full":
        return payload

    provenance = (
        gate_payload["benchmark_provenance"]
        or repayment_payload["benchmark_provenance"]
        or {}
    )
    return {
        **{key: payload[key] for key in (
            "ok",
            "engine",
            "engine_version",
            "law_content_date",
            "law_compilation",
            "disclaimer",
        )},
        "response_detail": "summary",
        "source": {
            "verify_at": provenance.get("verify_at"),
            "rate_table_uris": (
                gate_payload["manifest"]["rate_table_uris"]
                or repayment_payload["manifest"]["rate_table_uris"]
            ),
        },
        "gate": {key: gate_payload[key] for key in (
            "verdict",
            "loan_id",
            "benchmark_year_used",
            "benchmark_rate",
            "maximum_term_years_allowed",
            "reasons",
            "reason_codes",
            "caveats",
        )},
        "minimum_yearly_repayment": {key: repayment_payload[key] for key in (
            "verdict",
            "loan_id",
            "year_of_income",
            "gate_verdict",
            "benchmark_rate",
            "amalgamated_loan_unpaid_at_end_of_previous_year",
            "remaining_term_years_used",
            "myr_required",
            "payments_applied",
            "shortfall",
            "experimental_deemed_dividend_exposure",
            "rounding",
            "reasons",
            "reason_codes",
            "caveats",
        )},
    }
