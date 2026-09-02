"""The minimum yearly repayment: ITAA 1936 s 109E(5) and (6).

s 109E(5): "The minimum yearly repayment of an amalgamated loan for a year of
income is the amount worked out using the formula in subsection (6)."

s 109E(6), reproduced from the compiled Act:

    Amount of the loan not repaid by the
    end of the previous year of income     x   Current year's benchmark interest rate
    ------------------------------------------------------------------------------
                                                                   Remaining term
                    /                    1                     \\
            1  -   |  --------------------------------------    |
                    \\   1 + Current year's benchmark interest   /
                                        rate

    where:
      "current year's benchmark interest rate" is the benchmark interest rate
      for the year of income for which the minimum yearly repayment is being
      worked out.
      "remaining term" is the difference between:
        (a) the number of years in the longest term of any of the constituent
            loans that the amalgamated loan takes account of; and
        (b) the number of years between the end of the private company's year
            of income in which the loan was made and the end of the private
            company's year of income before the year of income for which the
            minimum yearly repayment is being worked out;
      rounded up to the next higher whole number if the difference is not
      already a whole number.

This engine takes the remaining term as the operator's (a) minus (b), because
computing it would require forming the amalgamated loan from its constituent
loans, and v1 does not do that. It applies the statutory rounding-up rule to
whatever it is given.

What this module refuses, rather than guesses:
  - the year the loan was made. s 109E(1)(a) applies only where the
    amalgamated loan was made "in an earlier year of income", and s 109P puts
    the year of making outside s 109D entirely. A minimum yearly repayment
    for the year the loan was made is the wrong question, not a hard sum.
  - a loan that is not on s 109N terms. s 109E operates on an amalgamated
    loan, which under s 109E(3)(b) is built out of loans that would be
    dividends "apart from section 109N". A loan that never met s 109N is
    already a s 109D dividend in the year it was made; a repayment schedule
    does not save it, and this engine will not print one as though it might.
  - whether a payment counts. s 109R takes some payments out of the
    reckoning, and that turns on what a reasonable person would conclude
    about intention. The operator asserts the amount applied.

The shortfall this module reports is a review aid. It is not the dividend:
s 109E(2) makes the dividend the shortfall "subject to section 109Y", which
caps the Division's dividends at the company's distributable surplus, and
s 109E(1)(d) removes the dividend entirely where s 109Q applies. Neither is
modelled.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from decimal import ROUND_CEILING, Decimal, localcontext

from .facts import optional_money, optional_year_of_income, optional_years
from .gate import GateResult
from .money import FORMULA_PRECISION, ROUNDING, cents_str, to_cents
from .rates import BenchmarkTable, RateOverride, RateResult, benchmark_rate
from .verdicts import GateVerdict, MyrVerdict, RateVerdict
from .years import YearOfIncome


@dataclass(frozen=True)
class MyrFacts:
    """Operator assertions for one amalgamated loan, one later year of income."""

    year_of_income: YearOfIncome | None = None
    amalgamated_loan_unpaid_at_end_of_previous_year: Decimal | None = None
    remaining_term_years: Decimal | None = None
    payments_applied_during_the_year: Decimal | None = None
    gate_result: GateResult | None = None
    year_loan_made: YearOfIncome | None = None
    loan_id: str = ""

    @classmethod
    def from_mapping(
        cls,
        row: dict,
        year_of_income: YearOfIncome,
        gate_result: GateResult | None = None,
        where: str = "row",
    ) -> "MyrFacts":
        return cls(
            loan_id=str(row.get("loan_id", "") or "").strip(),
            year_of_income=year_of_income,
            amalgamated_loan_unpaid_at_end_of_previous_year=optional_money(
                row.get("amalgamated_loan_unpaid_at_end_of_previous_year"),
                f"{where} amalgamated_loan_unpaid_at_end_of_previous_year",
            ),
            remaining_term_years=optional_years(
                row.get("remaining_term_years"), f"{where} remaining_term_years"
            ),
            payments_applied_during_the_year=optional_money(
                row.get("payments_applied_during_the_year"),
                f"{where} payments_applied_during_the_year",
            ),
            gate_result=gate_result,
            year_loan_made=optional_year_of_income(
                row.get("year_loan_made"), f"{where} year_loan_made"
            ),
        )


@dataclass(frozen=True)
class MyrResult:
    verdict: MyrVerdict
    loan_id: str = ""
    year_of_income: str = ""
    gate_verdict: GateVerdict | None = None
    benchmark: RateResult | None = None
    amalgamated_loan_unpaid_at_end_of_previous_year: Decimal | None = None
    remaining_term_years_used: Decimal | None = None
    myr_required: Decimal | None = None
    payments_applied: Decimal | None = None
    shortfall: Decimal | None = None
    experimental_deemed_dividend_exposure: Decimal | None = None
    reasons: tuple[str, ...] = field(default_factory=tuple)
    caveats: tuple[str, ...] = field(default_factory=tuple)
    statutory_trace: tuple[str, ...] = field(default_factory=tuple)

    def to_json_dict(self) -> dict:
        def money(value: Decimal | None) -> str | None:
            return None if value is None else cents_str(value)

        return {
            "verdict": self.verdict.value,
            "loan_id": self.loan_id,
            "year_of_income": self.year_of_income,
            "gate_verdict": None if self.gate_verdict is None else self.gate_verdict.value,
            "benchmark_rate": None if self.benchmark is None else self.benchmark.rate_text,
            "benchmark_provenance": (
                None if self.benchmark is None else self.benchmark.to_json_dict()["provenance"]
            ),
            "amalgamated_loan_unpaid_at_end_of_previous_year": money(
                self.amalgamated_loan_unpaid_at_end_of_previous_year
            ),
            "remaining_term_years_used": (
                None if self.remaining_term_years_used is None else str(self.remaining_term_years_used)
            ),
            "myr_required": money(self.myr_required),
            "payments_applied": money(self.payments_applied),
            "shortfall": money(self.shortfall),
            "experimental_deemed_dividend_exposure": money(
                self.experimental_deemed_dividend_exposure
            ),
            "rounding": ROUNDING,
            "reasons": list(self.reasons),
            "caveats": list(self.caveats),
            "statutory_trace": list(self.statutory_trace),
        }


def statutory_remaining_term(supplied: Decimal) -> Decimal:
    """Apply the s 109E(6) rounding rule: up to the next higher whole number
    if the difference is not already a whole number."""
    return supplied.to_integral_value(rounding=ROUND_CEILING)


def minimum_yearly_repayment_amount(
    principal: Decimal, rate: Decimal, remaining_term: Decimal
) -> Decimal:
    """The bare s 109E(6) formula, quantised to cents.

    Intermediate arithmetic runs at FORMULA_PRECISION significant digits in a
    local decimal context, so the result never depends on the ambient context
    a caller happens to have set, and is then quantised once at the end. The
    Act prescribes no rounding for this amount; ROUND_HALF_UP is this
    engine's documented choice.
    """
    if rate <= 0:
        raise ValueError("the s 109E(6) formula is undefined at a nil benchmark rate")
    if remaining_term <= 0:
        raise ValueError("the s 109E(6) formula is undefined at a nil remaining term")
    with localcontext() as ctx:
        ctx.prec = FORMULA_PRECISION
        numerator = principal * rate
        denominator = Decimal(1) - (Decimal(1) / (Decimal(1) + rate)) ** int(remaining_term)
        raw = numerator / denominator
    return to_cents(raw)


_GENUINE_REPAYMENT_CAVEAT = (
    "s 109R: this engine did not decide whether the payments applied are genuine "
    "repayments. A payment made where the borrower intended to draw a similar or "
    "larger loan is left out of the reckoning by s 109R(2), and that turns on what "
    "a reasonable person would conclude. The figure used is the operator's assertion."
)

_NOT_A_DETERMINATION_CAVEAT = (
    "Any shortfall shown is an experimental review aid, not an ATO assessment and "
    "not a s 109E(1) determination. s 109E(2) makes the dividend the shortfall "
    "subject to s 109Y, which caps Division 7A dividends at the company's "
    "distributable surplus, and s 109E(1)(d) removes the dividend where s 109Q "
    "applies. Neither is modelled here, and neither is s 109RB."
)


def minimum_yearly_repayment(
    facts: MyrFacts,
    *,
    table: BenchmarkTable | None = None,
    override: RateOverride | None = None,
) -> MyrResult:
    """Work out the s 109E minimum yearly repayment and any shortfall.

    Returns REFUSED where the question is outside this engine, UNKNOWN where a
    fact needed to answer it was not established, and never a number in
    either case.
    """
    refusals: list[str] = []
    unknowns: list[str] = []
    caveats: list[str] = [_GENUINE_REPAYMENT_CAVEAT, _NOT_A_DETERMINATION_CAVEAT]

    year = facts.year_of_income
    if year is None:
        return MyrResult(
            verdict=MyrVerdict.UNKNOWN,
            loan_id=facts.loan_id,
            reasons=("year_of_income was not supplied.",),
            caveats=tuple(caveats),
        )

    gate = facts.gate_result
    gate_verdict = None if gate is None else gate.verdict
    if gate is None:
        refusals.append(
            "No s 109N gate result was supplied. s 109E works on an amalgamated loan, "
            "which s 109E(3)(b) builds out of loans that would be dividends apart from "
            "s 109N. Run the gate first."
        )
    elif gate.verdict is GateVerdict.NOT_COMPLYING:
        refusals.append(
            "The s 109N gate returned NOT_COMPLYING. A constituent loan of an "
            "amalgamated loan is one that would be a s 109D dividend apart from "
            "s 109N (s 109E(3)(b)), so a loan that fails s 109N is not one, and s 109E "
            "produces no minimum yearly repayment for it. Where no other Subdivision D "
            "provision applies, s 109D(1) instead treats it as a dividend in the year "
            "it was made, for the amount unpaid before the lodgment day (s 109D(1AA)). "
            "No MYR figure is emitted. Reasons: " + "; ".join(gate.reasons)
        )
    elif gate.verdict is GateVerdict.UNKNOWN:
        refusals.append(
            "The s 109N gate returned UNKNOWN, so it is not established that this loan "
            "is an amalgamated loan on complying terms. Reasons: " + "; ".join(gate.reasons)
        )

    if facts.year_loan_made is not None:
        if facts.year_loan_made == year:
            refusals.append(
                f"The year of income requested ({year.label}) is the year the loan was "
                "made. s 109E(1)(a) applies only where the amalgamated loan was made in "
                "an earlier year of income, and s 109P puts an amalgamated loan outside "
                "s 109D in the year it is made. There is no minimum yearly repayment for "
                "the year of the loan."
            )
        elif facts.year_loan_made > year:
            refusals.append(
                f"The year of income requested ({year.label}) is before the year the loan "
                f"was made ({facts.year_loan_made.label}). s 109E(1)(a) requires an "
                "earlier year of making."
            )
    else:
        caveats.append(
            "year_loan_made was not supplied, so this engine has not established that "
            "the amalgamated loan was made in an earlier year of income as s 109E(1)(a) "
            "requires. The supplied balance unpaid at the end of the previous year "
            "implies it; nothing here verifies it."
        )

    rate = benchmark_rate(year, table=table, override=override)
    if rate.verdict is not RateVerdict.KNOWN or rate.rate is None:
        unknowns.append(rate.reason or f"No reviewed benchmark rate for {year.label}.")

    principal = facts.amalgamated_loan_unpaid_at_end_of_previous_year
    if principal is None:
        unknowns.append(
            "amalgamated_loan_unpaid_at_end_of_previous_year was not established. This "
            "engine does not form the amalgamated loan from its constituent loans "
            "(s 109E(3)); the operator supplies the balance."
        )

    payments = facts.payments_applied_during_the_year
    if payments is None:
        unknowns.append(
            "payments_applied_during_the_year was not established. Bank credits are not "
            "a substitute: s 109R decides which payments count, and this engine does "
            "not apply it."
        )

    term_used: Decimal | None = None
    if facts.remaining_term_years is None:
        unknowns.append("remaining_term_years was not established.")
    else:
        term_used = statutory_remaining_term(facts.remaining_term_years)
        if term_used <= 0:
            refusals.append(
                f"remaining_term_years is {facts.remaining_term_years}, which leaves a "
                f"remaining term of {term_used} under s 109E(6). The formula divides by a "
                "term that is nil at that point, and the loan should have been repaid in "
                "full by the end of the previous year of income. No MYR figure is emitted."
            )
        elif term_used != facts.remaining_term_years:
            caveats.append(
                f"remaining_term_years {facts.remaining_term_years} was rounded up to "
                f"{term_used} under the closing words of s 109E(6)."
            )

    if rate.rate is not None and rate.rate <= 0:
        refusals.append(
            f"The benchmark rate for {year.label} is {rate.rate_text}. The s 109E(6) "
            "formula divides by 1 minus a power of 1/(1+rate), which is nil at a nil "
            "rate. No MYR figure is emitted."
        )

    trace: list[str] = [
        "ITAA 1936 s 109E(1): a private company is taken to pay a dividend at the end "
        "of the current year where an amalgamated loan made in an earlier year is not "
        "repaid and payments fall short of the minimum yearly repayment.",
        "s 109E(5): the minimum yearly repayment is worked out using the formula in "
        "s 109E(6).",
    ]

    if refusals:
        return MyrResult(
            verdict=MyrVerdict.REFUSED,
            loan_id=facts.loan_id,
            year_of_income=year.label,
            gate_verdict=gate_verdict,
            benchmark=rate,
            amalgamated_loan_unpaid_at_end_of_previous_year=principal,
            remaining_term_years_used=term_used,
            payments_applied=payments,
            reasons=tuple(refusals),
            caveats=tuple(caveats),
            statutory_trace=tuple(trace),
        )
    if unknowns:
        return MyrResult(
            verdict=MyrVerdict.UNKNOWN,
            loan_id=facts.loan_id,
            year_of_income=year.label,
            gate_verdict=gate_verdict,
            benchmark=rate,
            amalgamated_loan_unpaid_at_end_of_previous_year=principal,
            remaining_term_years_used=term_used,
            payments_applied=payments,
            reasons=tuple(unknowns),
            caveats=tuple(caveats),
            statutory_trace=tuple(trace),
        )

    assert principal is not None and payments is not None
    assert term_used is not None and rate.rate is not None

    required = minimum_yearly_repayment_amount(principal, rate.rate, term_used)
    applied = to_cents(payments)
    shortfall = required - applied
    if shortfall < 0:
        shortfall = Decimal("0.00")
    shortfall = to_cents(shortfall)

    met = shortfall == Decimal("0.00")
    verdict = MyrVerdict.MYR_MET if met else MyrVerdict.MYR_SHORT

    trace.extend(
        [
            "s 109E(6) formula: MYR = (amount of the loan not repaid by the end of the "
            "previous year of income x current year's benchmark interest rate) / "
            "(1 - (1 / (1 + current year's benchmark interest rate)) ^ remaining term).",
            f"Amount of the loan not repaid by the end of the previous year of income = "
            f"{cents_str(principal)}.",
            f"Current year's benchmark interest rate ({year.label}, s 109N(2)) = "
            f"{rate.rate_text}, from RBA table {rate.rba_table} series {rate.rba_series}, "
            f"{rate.rba_month} figure.",
            f"Remaining term (s 109E(6), rounded up to a whole number) = {term_used}.",
            f"MYR = ({cents_str(principal)} x {rate.rate_text}) / "
            f"(1 - (1 / (1 + {rate.rate_text})) ^ {term_used}) = {cents_str(required)} "
            f"(quantised to cents, {ROUNDING}; the Act prescribes no rounding).",
            f"Payments applied during the year (operator assertion, s 109R not applied) = "
            f"{cents_str(applied)}.",
            f"Shortfall (s 109E(1)(c)) = max(0, {cents_str(required)} - "
            f"{cents_str(applied)}) = {cents_str(shortfall)}.",
        ]
    )

    return MyrResult(
        verdict=verdict,
        loan_id=facts.loan_id,
        year_of_income=year.label,
        gate_verdict=gate_verdict,
        benchmark=rate,
        amalgamated_loan_unpaid_at_end_of_previous_year=principal,
        remaining_term_years_used=term_used,
        myr_required=required,
        payments_applied=applied,
        shortfall=shortfall,
        experimental_deemed_dividend_exposure=None if met else shortfall,
        reasons=(),
        caveats=tuple(caveats),
        statutory_trace=tuple(trace),
    )
