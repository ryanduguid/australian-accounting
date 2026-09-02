"""The complying-loan gate: ITAA 1936 s 109N(1).

    "A private company that makes a loan to an entity in one of the private
    company's years of income is not taken under section 109D to pay a
    dividend at the end of the year of income because of the loan if, before
    the lodgment day for the year of income:
      (a) the agreement that the loan was made under is in writing; and
      (b) the rate of interest payable on the loan for years of income after
          the year in which the loan is made equals or exceeds the benchmark
          interest rate for the year; and
      (c) the term of the loan does not exceed the term (the maximum term)
          for that kind of loan worked out under subsection (3)."
    -- s 109N(1)

Every limb must be established. A limb that fails on a supplied fact gives
NOT_COMPLYING. A limb that has not been established gives UNKNOWN, and
UNKNOWN is never rounded down to "probably fine" or up to "assume the worst":
it is the finding that a human still has work to do.

The engine does not compute the lodgment day. Under s 109D(6) that is the
earlier of the due date for the company's return for the year of income and
the day the return is lodged, which needs a lodgment-program date this engine
does not hold. The operator asserts the boolean.

Not modelled: the refinancing reductions to the maximum term in s 109N(3A)
to (3D), s 109M arm's-length loans, s 109NA liquidator's distributions,
s 109NB employee-share-scheme loans, and every other Subdivision D exclusion.
A caveat is emitted where one of those could bear on the answer.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal

from .facts import (
    FactError,
    optional_rate,
    optional_ratio,
    optional_year_of_income,
    optional_years,
    parse_tristate,
)
from .money import rate_str
from .rates import BenchmarkTable, RateOverride, RateResult, benchmark_rate
from .verdicts import GateVerdict, RateVerdict, StrEnum
from .years import YearOfIncome

#: s 109N(3)(a): 25 years where the loan is fully secured by a registered
#: mortgage over real property and the security covers at least 110% of it.
SECURED_MAXIMUM_TERM = Decimal("25")

#: s 109N(3)(b): 7 years for any other loan.
DEFAULT_MAXIMUM_TERM = Decimal("7")

#: s 109N(3)(a)(ii): "at least 110% of the amount of the loan".
REQUIRED_SECURITY_COVERAGE = Decimal("1.10")


class LimbState(StrEnum):
    PASS = "PASS"
    FAIL = "FAIL"
    UNKNOWN = "UNKNOWN"


@dataclass(frozen=True)
class Limb:
    """One requirement of s 109N(1), and what the supplied facts did to it."""

    cite: str
    requirement: str
    state: LimbState
    finding: str

    def to_json_dict(self) -> dict:
        return {
            "cite": self.cite,
            "requirement": self.requirement,
            "state": self.state.value,
            "finding": self.finding,
        }


@dataclass(frozen=True)
class GateFacts:
    """Operator assertions about one loan. Nothing here is inferred."""

    #: The term of the loan under the written agreement, in years. It is
    #: tested against the maximum term worked out under s 109N(3); the name
    #: follows the published API surface rather than the Act's own use of
    #: "maximum term" for the statutory cap.
    maximum_term_years: Decimal | None = None
    written_agreement: bool | None = None
    terms_in_place_before_lodgment_day: bool | None = None
    secured_by_registered_mortgage_over_real_property: bool | None = None
    security_coverage_at_first_made: Decimal | None = None
    interest_rate_for_years_after_year_loan_made: Decimal | None = None
    year_of_income_being_tested: YearOfIncome | None = None
    year_loan_made: YearOfIncome | None = None
    loan_id: str = ""

    @classmethod
    def from_mapping(cls, row: dict, where: str = "row") -> "GateFacts":
        """Build from a CSV row. Raises FactError on an unreadable cell."""
        return cls(
            loan_id=str(row.get("loan_id", "") or "").strip(),
            written_agreement=parse_tristate(
                row.get("written_agreement"), f"{where} written_agreement"
            ),
            terms_in_place_before_lodgment_day=parse_tristate(
                row.get("terms_in_place_before_lodgment_day"),
                f"{where} terms_in_place_before_lodgment_day",
            ),
            maximum_term_years=optional_years(
                row.get("maximum_term_years"), f"{where} maximum_term_years"
            ),
            secured_by_registered_mortgage_over_real_property=parse_tristate(
                row.get("secured_by_registered_mortgage_over_real_property"),
                f"{where} secured_by_registered_mortgage_over_real_property",
            ),
            security_coverage_at_first_made=optional_ratio(
                row.get("security_coverage_at_first_made"),
                f"{where} security_coverage_at_first_made",
            ),
            interest_rate_for_years_after_year_loan_made=optional_rate(
                row.get("interest_rate_for_years_after_year_loan_made"),
                f"{where} interest_rate_for_years_after_year_loan_made",
            ),
            year_of_income_being_tested=optional_year_of_income(
                row.get("year_of_income_being_tested"), f"{where} year_of_income_being_tested"
            ),
            year_loan_made=optional_year_of_income(
                row.get("year_loan_made"), f"{where} year_loan_made"
            ),
        )


@dataclass(frozen=True)
class GateResult:
    verdict: GateVerdict
    loan_id: str = ""
    benchmark_year_used: str = ""
    benchmark: RateResult | None = None
    maximum_term_years_allowed: Decimal | None = None
    limbs: tuple[Limb, ...] = field(default_factory=tuple)
    reasons: tuple[str, ...] = field(default_factory=tuple)
    caveats: tuple[str, ...] = field(default_factory=tuple)
    statutory_trace: tuple[str, ...] = field(default_factory=tuple)

    def to_json_dict(self) -> dict:
        return {
            "verdict": self.verdict.value,
            "loan_id": self.loan_id,
            "benchmark_year_used": self.benchmark_year_used,
            "benchmark_rate": None if self.benchmark is None else self.benchmark.rate_text,
            "benchmark_provenance": (
                None if self.benchmark is None else self.benchmark.to_json_dict()["provenance"]
            ),
            "maximum_term_years_allowed": (
                None
                if self.maximum_term_years_allowed is None
                else str(self.maximum_term_years_allowed)
            ),
            "limbs": [limb.to_json_dict() for limb in self.limbs],
            "reasons": list(self.reasons),
            "caveats": list(self.caveats),
            "statutory_trace": list(self.statutory_trace),
        }


def _written_agreement_limb(facts: GateFacts) -> Limb:
    cite = "s 109N(1)(a)"
    requirement = "the agreement that the loan was made under is in writing"
    if facts.written_agreement is None:
        return Limb(
            cite,
            requirement,
            LimbState.UNKNOWN,
            "written_agreement was not established. This engine will not read the "
            "absence of an assertion as an absence of an agreement.",
        )
    if facts.written_agreement:
        return Limb(cite, requirement, LimbState.PASS, "Operator asserts a written agreement.")
    return Limb(
        cite,
        requirement,
        LimbState.FAIL,
        "Operator asserts there is no written agreement.",
    )


def _lodgment_day_limb(facts: GateFacts) -> Limb:
    cite = "s 109N(1) chapeau, with s 109D(6)"
    requirement = "the criteria were met before the lodgment day for the year of income"
    if facts.terms_in_place_before_lodgment_day is None:
        return Limb(
            cite,
            requirement,
            LimbState.UNKNOWN,
            "terms_in_place_before_lodgment_day was not established. The lodgment day "
            "under s 109D(6) is the earlier of the due date for the company's return "
            "and the day it was lodged; this engine does not compute either.",
        )
    if facts.terms_in_place_before_lodgment_day:
        return Limb(
            cite,
            requirement,
            LimbState.PASS,
            "Operator asserts the terms were in place before the lodgment day.",
        )
    return Limb(
        cite,
        requirement,
        LimbState.FAIL,
        "Operator asserts the terms were not in place before the lodgment day.",
    )


def _interest_limb(facts: GateFacts, rate: RateResult) -> Limb:
    cite = "s 109N(1)(b)"
    requirement = (
        "the rate of interest payable for years of income after the year the loan "
        "was made equals or exceeds the benchmark interest rate"
    )
    supplied = facts.interest_rate_for_years_after_year_loan_made
    if supplied is None:
        return Limb(
            cite,
            requirement,
            LimbState.UNKNOWN,
            "interest_rate_for_years_after_year_loan_made was not established.",
        )
    if rate.verdict is not RateVerdict.KNOWN or rate.rate is None:
        return Limb(
            cite,
            requirement,
            LimbState.UNKNOWN,
            f"No reviewed benchmark rate for {rate.year_of_income}, so the supplied "
            f"rate {rate_str(supplied)} cannot be tested against a floor.",
        )
    if supplied >= rate.rate:
        return Limb(
            cite,
            requirement,
            LimbState.PASS,
            f"Supplied rate {rate_str(supplied)} is at or above the "
            f"{rate.year_of_income} benchmark {rate.rate_text}.",
        )
    return Limb(
        cite,
        requirement,
        LimbState.FAIL,
        f"Supplied rate {rate_str(supplied)} is below the {rate.year_of_income} "
        f"benchmark {rate.rate_text}.",
    )


def _maximum_term(facts: GateFacts) -> tuple[Decimal | None, str, tuple[str, ...]]:
    """Work out the s 109N(3) maximum term, or None where it is not established.

    The unknown case is narrower than it looks. Where the loan's own term is
    at or under 7 years the answer is the same whichever limb of s 109N(3)
    applies, so unestablished security facts do not make the term limb
    unknown. Only a term above 7 years actually depends on them.
    """
    secured = facts.secured_by_registered_mortgage_over_real_property
    coverage = facts.security_coverage_at_first_made
    caveats: tuple[str, ...] = ()

    if secured is False:
        return DEFAULT_MAXIMUM_TERM, "s 109N(3)(b): 7 years for a loan that is not secured", ()
    if secured is True:
        if coverage is None:
            return (
                None,
                "s 109N(3)(a)(ii): security coverage at the time the loan was first made "
                "was not established",
                (),
            )
        if coverage >= REQUIRED_SECURITY_COVERAGE:
            return (
                SECURED_MAXIMUM_TERM,
                f"s 109N(3)(a): 25 years; registered mortgage over real property and "
                f"coverage {coverage} is at least {REQUIRED_SECURITY_COVERAGE}",
                (
                    "s 109N(3A) to (3D) reduce the maximum term where the loan "
                    "refinances an earlier loan. Refinancing is not modelled; if this "
                    "loan refinanced another, the 25 years may be shorter.",
                ),
            )
        return (
            DEFAULT_MAXIMUM_TERM,
            f"s 109N(3)(b): 7 years; coverage {coverage} is below the "
            f"{REQUIRED_SECURITY_COVERAGE} that s 109N(3)(a)(ii) requires, so the "
            "25-year limb is not available",
            (),
        )
    return (
        None,
        "s 109N(3): whether the loan is secured by a registered mortgage over real "
        "property was not established",
        caveats,
    )


def _term_limb(facts: GateFacts) -> tuple[Limb, Decimal | None, tuple[str, ...]]:
    cite = "s 109N(1)(c)"
    requirement = "the term of the loan does not exceed the maximum term under s 109N(3)"
    term = facts.maximum_term_years
    allowed, basis, caveats = _maximum_term(facts)

    if term is None:
        return (
            Limb(cite, requirement, LimbState.UNKNOWN, "maximum_term_years was not established."),
            allowed,
            caveats,
        )
    if allowed is None:
        # The security facts only matter above 7 years: below that, both
        # limbs of s 109N(3) permit the term.
        if term <= DEFAULT_MAXIMUM_TERM:
            return (
                Limb(
                    cite,
                    requirement,
                    LimbState.PASS,
                    f"Term {term} years is within the 7 years s 109N(3)(b) allows for any "
                    "loan, so the unestablished security facts cannot change this limb.",
                ),
                DEFAULT_MAXIMUM_TERM,
                caveats,
            )
        return (
            Limb(cite, requirement, LimbState.UNKNOWN, f"{basis}, and the term {term} years "
                 f"exceeds the {DEFAULT_MAXIMUM_TERM} years available without it."),
            None,
            caveats,
        )
    if term <= allowed:
        return (
            Limb(cite, requirement, LimbState.PASS, f"Term {term} years is within {allowed} ({basis})."),
            allowed,
            caveats,
        )
    return (
        Limb(cite, requirement, LimbState.FAIL, f"Term {term} years exceeds {allowed} ({basis})."),
        allowed,
        caveats,
    )


def complying_loan_gate(
    facts: GateFacts,
    *,
    table: BenchmarkTable | None = None,
    override: RateOverride | None = None,
) -> GateResult:
    """Test one loan against s 109N(1) on the operator's own facts.

    The benchmark floor year is facts.year_of_income_being_tested where the
    operator nominates one, and otherwise facts.year_loan_made, which is the
    year s 109N(1)(b) itself points at. Where the two differ the result
    carries a caveat saying so: testing a later year is a practice check
    against a risen benchmark, not the s 109N(1)(b) test.
    """
    caveats: list[str] = []
    reasons: list[str] = []

    floor_year = facts.year_of_income_being_tested or facts.year_loan_made
    if floor_year is None:
        return GateResult(
            verdict=GateVerdict.UNKNOWN,
            loan_id=facts.loan_id,
            reasons=(
                "Neither year_of_income_being_tested nor year_loan_made was supplied, "
                "so there is no year to read a benchmark interest rate for and "
                "s 109N(1)(b) cannot be tested.",
            ),
            statutory_trace=("ITAA 1936 s 109N(1): every limb must be met before the lodgment day.",),
        )

    rate = benchmark_rate(floor_year, table=table, override=override)

    if (
        facts.year_loan_made is not None
        and facts.year_of_income_being_tested is not None
        and facts.year_of_income_being_tested != facts.year_loan_made
    ):
        caveats.append(
            f"s 109N(1)(b) sets the floor by reference to the benchmark rate for the "
            f"year the loan was made ({facts.year_loan_made.label}). This run tested "
            f"the rate against {floor_year.label} because the operator nominated it. "
            "A later-year comparison is a practice check on a risen benchmark, not "
            "the s 109N(1)(b) test itself. See evaluation/div7a_myr/README.md."
        )

    limbs = [
        _written_agreement_limb(facts),
        _lodgment_day_limb(facts),
        _interest_limb(facts, rate),
    ]
    term_limb, allowed, term_caveats = _term_limb(facts)
    limbs.append(term_limb)
    caveats.extend(term_caveats)

    states = [limb.state for limb in limbs]
    if LimbState.FAIL in states:
        verdict = GateVerdict.NOT_COMPLYING
    elif LimbState.UNKNOWN in states:
        verdict = GateVerdict.UNKNOWN
    else:
        verdict = GateVerdict.COMPLYING

    for limb in limbs:
        if limb.state is not LimbState.PASS:
            reasons.append(f"{limb.cite}: {limb.finding}")

    caveats.append(
        "COMPLYING here means the four limbs of s 109N(1) are established on the "
        "facts supplied. It is not a finding that no dividend arises: Subdivision D "
        "holds other exclusions, s 109T to s 109X reach interposed entities, and "
        "s 109XA reaches unpaid present entitlements. None of those is modelled."
    )

    trace = (
        "ITAA 1936 s 109N(1): a loan is not taken under s 109D to be a dividend if, "
        "before the lodgment day for the year of income, the agreement is in "
        "writing, the interest rate meets the benchmark, and the term is within the "
        "s 109N(3) maximum.",
        f"Benchmark interest rate read for {rate.year_of_income} (s 109N(2)): "
        f"{rate.rate_text if rate.rate_text else 'UNKNOWN'}.",
        "s 109D(6): lodgment day is the earlier of the due date for the company's "
        "return and the day it was lodged; asserted by the operator, not computed here.",
    )

    return GateResult(
        verdict=verdict,
        loan_id=facts.loan_id,
        benchmark_year_used=rate.year_of_income,
        benchmark=rate,
        maximum_term_years_allowed=allowed,
        limbs=tuple(limbs),
        reasons=tuple(reasons),
        caveats=tuple(caveats),
        statutory_trace=trace,
    )
