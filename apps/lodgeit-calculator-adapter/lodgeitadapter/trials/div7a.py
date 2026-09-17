"""Division 7A: the local engine against the provider's overlapping scope.

Before any number is compared, six conventions have to be lined up. They are
listed here rather than buried, because every one of them could produce a
difference that looks like an arithmetic defect and is not.

1. **Period.** The provider's `fy2026` is the income year ending 30 June 2026,
   which the local engine labels `2025-26`. Checked on 18 September 2026: the
   provider's `fy2026` rate document carries period_start 2025-07-01 and
   period_end 2026-06-30 with value 0.0837, and the local table records
   2025-26 at 0.0837. `fy2025` is 0.0877 against the local 2024-25 at 0.0877.
2. **Rate unit.** Both hold the benchmark rate as a decimal fraction of one,
   not a percentage.
3. **Balance.** The provider's `amalgamated_base` and the local
   `amalgamated_loan_unpaid_at_end_of_previous_year` are the same quantity:
   the amount of the loan not repaid at the end of the previous income year.
4. **Remaining term.** The provider derives it from the origination date, the
   income year start and `is_first_real_myr_year`, giving `term - 1` in the
   first real MYR year. The local engine takes the remaining term as an
   operator assertion and applies the s 109E(6) rounding-up rule to it. A case
   here supplies the same integer to both, computed from the same origination
   facts, so a difference is arithmetic and not bookkeeping.
5. **Repayments.** The provider takes repayment events and aggregates them on
   a daily-balance basis; the local engine takes one figure the operator
   asserts. A case supplies repayments whose sum is that figure, and
   `total_repayments` is compared so an aggregation difference shows up as
   itself.
6. **Rounding.** Both quantise to cents. The Act prescribes none, and both
   sides document their own choice.

What is never mapped across: the provider's `is_complying` and
`deemed_dividend` are labels for its own arithmetic. They are not a Division 7A
determination, and they are not put into the local engine's `GateVerdict`. The
local engine's exclusions stand: it does not characterise repayments under
s 109R, does not form the amalgamated loan from constituent loans, does not
model distributable surplus under s 109Y and does not apply s 109RB. A trial
that quietly borrowed the provider's compliance label would be throwing all of
that away.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from decimal import Decimal
from pathlib import Path
from typing import Any

from ..client import LodgeitClient, Status
from . import Comparison, Evaluation, compare_values

CALC_URI = "urn:sbrm:calculator:div7a:at"
_HERE = Path(__file__).resolve().parent
_FIXTURE_DIRS = (_HERE.parent.parent / "fixtures", _HERE.parent / "fixtures")


def cases_path() -> Path:
    """The case file, in a source checkout or inside a built wheel."""
    for candidate in _FIXTURE_DIRS:
        path = candidate / "div7a-cases.json"
        if path.is_file():
            return path
    return _FIXTURE_DIRS[0] / "div7a-cases.json"

#: The provider's period URN for an income year ending 30 June NNNN, against
#: the local engine's label for the same year.
PERIOD_MAP = {
    "urn:sbrm:period:div7a:fy2025": "2024-25",
    "urn:sbrm:period:div7a:fy2026": "2025-26",
}


@dataclass(frozen=True)
class Case:
    case_id: str
    period_uri: str
    amalgamated_base: Decimal
    loan_term_years: int
    loan_origination_date: str
    income_year_start_date: str
    is_first_real_myr_year: bool
    repayments: tuple[dict[str, Any], ...]
    remaining_term_years: int
    expected: dict[str, Decimal]
    derivation: str
    expect_local: str
    expect_evaluation: str
    note: str = ""

    @property
    def local_year(self) -> str | None:
        """The local income-year label, or None when the period is unmapped.

        An unmapped period is not an error here: it is the case where one side
        covers a year the other does not, which the trial reports rather than
        guessing a mapping for.
        """
        return PERIOD_MAP.get(self.period_uri)

    @property
    def repayments_total(self) -> Decimal:
        return sum((Decimal(item["amount"]) for item in self.repayments), Decimal("0"))


def load_cases(path: Path | None = None) -> list[Case]:
    payload = json.loads((path or cases_path()).read_text(encoding="utf-8"))
    cases = []
    for raw in payload["cases"]:
        cases.append(
            Case(
                case_id=raw["case_id"],
                period_uri=raw["period_uri"],
                amalgamated_base=Decimal(raw["amalgamated_base"]),
                loan_term_years=raw["loan_term_years"],
                loan_origination_date=raw["loan_origination_date"],
                income_year_start_date=raw["income_year_start_date"],
                is_first_real_myr_year=raw["is_first_real_myr_year"],
                repayments=tuple(raw["repayments"]),
                remaining_term_years=raw["remaining_term_years"],
                expected={key: Decimal(value) for key, value in raw["expected"].items()},
                derivation=raw["derivation"],
                expect_local=raw["expect_local"],
                expect_evaluation=raw["expect_evaluation"],
                note=raw.get("note", ""),
            )
        )
    return cases


def build_request(case: Case) -> dict[str, Any]:
    """The provider request body. Money stays Decimal to the serialiser."""
    return {
        "amalgamated_base": case.amalgamated_base,
        "loan_term_years": case.loan_term_years,
        "loan_origination_date": case.loan_origination_date,
        "income_year_start_date": case.income_year_start_date,
        "is_first_real_myr_year": case.is_first_real_myr_year,
        "repayments": [
            {"amount": Decimal(item["amount"]), "date": item["date"]} for item in case.repayments
        ],
    }


def run_local(case: Case) -> tuple[str, dict[str, Decimal], tuple[str, ...]]:
    """The local engine's answer for one case, with its verdict and reasons.

    Imported here rather than at module scope so the trial module can be read,
    and its offline tests run, without the engine installed.
    """
    from div7aloan.gate import GateResult  # noqa: PLC0415
    from div7aloan.myr import MyrFacts, minimum_yearly_repayment  # noqa: PLC0415
    from div7aloan.verdicts import GateVerdict  # noqa: PLC0415
    from div7aloan.years import parse_year  # noqa: PLC0415

    if case.local_year is None:
        return "UNMAPPED_PERIOD", {}, (
            f"{case.period_uri} has no entry in PERIOD_MAP, so no local income year is "
            "established for it. A mapping is a reviewed fact, not a guess from the label.",
        )
    year = parse_year(case.local_year, "case")
    made = parse_year(_origination_year_label(case), "case")
    # The gate is supplied as COMPLYING on the case's own asserted facts. This
    # is the scope alignment, not a finding: s 109E works on an amalgamated
    # loan, which s 109E(3)(b) builds from loans that would be dividends apart
    # from s 109N, so a trial of the repayment arithmetic has to start from a
    # loan that met s 109N. It is an input to the comparison, never a
    # conclusion the provider handed over.
    # No reasons on the gate result: the engine pairs every reason with a
    # stable reason code, and a trial has no business inventing one. The note
    # about where this gate came from travels in the comparison's own reasons.
    gate = GateResult(
        verdict=GateVerdict.COMPLYING,
        loan_id=case.case_id,
        benchmark_year_used=made.label,
    )
    facts = MyrFacts(
        loan_id=case.case_id,
        year_of_income=year,
        amalgamated_loan_unpaid_at_end_of_previous_year=case.amalgamated_base,
        remaining_term_years=Decimal(case.remaining_term_years),
        payments_applied_during_the_year=case.repayments_total,
        gate_result=gate,
        year_loan_made=made,
    )
    result = minimum_yearly_repayment(facts)
    notes: tuple[str, ...] = (
        "The s 109N gate was supplied to this trial as an asserted fact, so the repayment "
        "arithmetic could be compared. It was not derived from the provider and it is not a "
        "conclusion about the loan.",
    )
    values: dict[str, Decimal] = {}
    if result.myr_required is not None:
        values["statutory_myr"] = result.myr_required
    if result.payments_applied is not None:
        values["total_repayments"] = result.payments_applied
    if result.shortfall is not None:
        values["shortfall"] = result.shortfall
    if result.benchmark is not None and result.benchmark.rate is not None:
        values["benchmark_rate"] = result.benchmark.rate
    return str(result.verdict), values, notes + tuple(result.reasons)


def _origination_year_label(case: Case) -> str:
    """The income year the loan was made in, as the local engine labels it."""
    year = int(case.loan_origination_date[:4])
    month = int(case.loan_origination_date[5:7])
    start = year if month >= 7 else year - 1
    return f"{start}-{str(start + 1)[2:]}"


def extract_upstream(result: dict) -> dict[str, Decimal]:
    """The provider's figures, as Decimals, for the fields both sides have."""
    from ..decimals import parse_decimal  # noqa: PLC0415

    values: dict[str, Decimal] = {}
    for name in ("statutory_myr", "total_repayments", "shortfall", "benchmark_rate"):
        if name in result:
            values[name] = parse_decimal(result[name], name)
    return values


def evaluate(case: Case, client: LodgeitClient | None = None) -> Comparison:
    """Run one case on both sides and say what the difference means."""
    local_verdict, local_values, local_reasons = run_local(case)
    if local_verdict == "UNMAPPED_PERIOD":
        return Comparison(
            case_id=case.case_id, evaluation=Evaluation.UNSUPPORTED_PERIOD,
            expected=case.expected, local_verdict=local_verdict, reasons=local_reasons,
        )
    if local_verdict in ("REFUSED", "UNKNOWN"):
        return Comparison(
            case_id=case.case_id, evaluation=Evaluation.LOCAL_REFUSED,
            expected=case.expected, local=local_values, local_verdict=local_verdict,
            reasons=local_reasons,
        )
    if client is None:
        return Comparison(
            case_id=case.case_id, evaluation=Evaluation.NOT_RUN,
            expected=case.expected, local=local_values, local_verdict=local_verdict,
            reasons=("No client supplied: the local side ran and the provider was not called.",),
        )
    outcome = client.invoke(CALC_URI, case.period_uri, build_request(case))
    return evaluate_outcome(case, local_verdict, local_values, local_reasons, outcome)


def evaluate_outcome(
    case: Case,
    local_verdict: str,
    local_values: dict[str, Decimal],
    local_reasons: tuple[str, ...],
    outcome,
) -> Comparison:
    """The evaluation half, separated so it can be driven from a fixture."""
    common: dict[str, Any] = dict(
        case_id=case.case_id, expected=case.expected, local=local_values,
        local_verdict=local_verdict, upstream_status=str(outcome.status),
    )
    if outcome.status is Status.UPSTREAM_NOT_FOUND:
        return Comparison(evaluation=Evaluation.UNSUPPORTED_PERIOD,
                          reasons=(
                              f"the provider does not accept {case.period_uri}",
                              ) + outcome.findings,
                          **common)
    if outcome.status in (Status.UPSTREAM_UNAVAILABLE, Status.UPSTREAM_THROTTLED,
                          Status.REFUSED_TO_SEND):
        return Comparison(
            evaluation=Evaluation.UPSTREAM_UNAVAILABLE,
            reasons=outcome.findings,
            **common,
        )
    if outcome.status is Status.UPSTREAM_REFUSED:
        return Comparison(
            evaluation=Evaluation.SCOPE_MISMATCH,
            reasons=(
                "the provider refused this case: "
                f"{outcome.upstream_refusal_class or 'no class given'}",
            ) + outcome.findings,
            **common,
        )
    if outcome.status is not Status.COMPUTED:
        return Comparison(
            evaluation=Evaluation.CONTRACT_FAILURE,
            reasons=outcome.findings,
            **common,
        )

    upstream = extract_upstream(outcome.result or {})
    evaluation, differences, reasons = compare_values(case.expected, local_values, upstream)
    extra: list[str] = list(reasons)
    recorded_term = (outcome.result or {}).get("remaining_term_years")
    if recorded_term is not None and str(recorded_term) != str(case.remaining_term_years):
        extra.append(
            f"remaining term: the provider used {recorded_term}, the case supplied "
            f"{case.remaining_term_years}. A term difference is a convention difference, "
            "not an arithmetic one."
        )
        evaluation = Evaluation.SCOPE_MISMATCH
    if "is_complying" in (outcome.result or {}):
        extra.append(
            "the provider returned is_complying; it is its own label for its own arithmetic "
            "and is not carried into any local verdict"
        )
    return Comparison(evaluation=evaluation, upstream=upstream, differences=differences,
                      reasons=tuple(local_reasons) + tuple(extra), **common)
