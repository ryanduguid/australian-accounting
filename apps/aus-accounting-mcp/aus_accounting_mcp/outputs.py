"""MCP result contracts; calculations and provenance remain owned by the engines.

Typed dictionaries preserve existing JSON objects. Separate variants require
summary/full and CTR/BAS fields without inserting defaults. Extra fields are
retained so engine audit information is never discarded during serialization.
"""

from datetime import date
from typing import Annotated, Any, Literal

from pydantic import (
    AfterValidator,
    ConfigDict,
    Field,
    RootModel,
    TypeAdapter,
    model_validator,
    with_config,
)
from typing_extensions import TypedDict


DecimalText = Annotated[
    str,
    Field(
        description="Finite engine decimal string, including exponent notation; retain its precision.",
        pattern=r"^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$",
    ),
]
MaybeDecimal = Annotated[
    DecimalText | None,
    Field(description="Engine decimal string, or null when unavailable; null is not zero."),
]


def _iso_date(value: str) -> str:
    date.fromisoformat(value)
    return value


DateText = Annotated[
    str,
    Field(
        description="Date in YYYY-MM-DD form.",
        pattern=r"^[0-9]{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12][0-9]|3[01])$",
        json_schema_extra={"format": "date"},
    ),
    AfterValidator(_iso_date),
]
YearText = Annotated[
    str,
    Field(
        description="Income or dataset year in YYYY-YY form.",
        pattern=r"^[0-9]{4}-[0-9]{2}$",
    ),
]
Caveats = Annotated[
    list[str], Field(description="Limitations to retain when presenting the result.")
]
Reasons = Annotated[list[str], Field(description="Engine reasons for the reported verdict.")]
Trace = Annotated[
    list[str], Field(description="Engine statutory citations and audit trace; full only.")
]
Provenance = Annotated[
    dict[str, Any],
    Field(description="Unmodified engine source metadata and review provenance."),
]


@with_config(ConfigDict(extra="allow", strict=True))
class ResultObject(TypedDict):
    """Preserve additional engine fields and reject coercion of declared values."""


class EngineResult(ResultObject):
    ok: Annotated[
        Literal[True],
        Field(description="The review ran; this does not mean compliance or a known verdict."),
    ]
    engine: Annotated[str, Field(description="Delegated distribution that produced this result.")]
    engine_version: Annotated[str, Field(description="Installed version of that engine.")]


class Industry(ResultObject):
    name: Annotated[str, Field(description="Business-type name accepted by get_ato_benchmarks.")]
    key_ratio: Annotated[str, Field(description="ATO key-ratio identifier for this industry.")]


class IndustryList(EngineResult):
    benchmark_year: YearText
    count: Annotated[int, Field(description="Number of industries matching the search.")]
    total_business_types: Annotated[
        int, Field(description="Total industries in the selected dataset.")
    ]
    industries: Annotated[
        list[Industry], Field(description="Matching industries; empty if no match.")
    ]
    source: Provenance


class BenchmarkRatio(ResultObject):
    ratio: Annotated[str, Field(description="Ratio identifier.")]
    label: Annotated[str, Field(description="Human-readable ratio name.")]
    value: MaybeDecimal
    percent: Annotated[
        str | None, Field(description="Formatted percentage, or null if unavailable.")
    ]
    benchmark_min: MaybeDecimal
    benchmark_max: MaybeDecimal
    status: Annotated[
        str,
        Field(
            description="Engine comparison status; not_supplied means facts do not establish the ratio."
        ),
    ]
    is_key_ratio: Annotated[bool, Field(description="Whether this is the selected ATO key ratio.")]


class BenchmarkComparison(EngineResult):
    benchmark_year: YearText
    business_type: Annotated[str, Field(description="Selected ATO industry name.")]
    key_ratio: Annotated[str, Field(description="Selected ATO key-ratio identifier.")]
    turnover: MaybeDecimal
    turnover_basis: Annotated[
        str | None, Field(description="Engine denominator basis; null if unestablished.")
    ]
    turnover_band: Annotated[
        dict[str, str] | None,
        Field(description="Selected dataset band and label, or null if unavailable."),
    ]
    figures: Annotated[
        dict[str, MaybeDecimal], Field(description="Engine figures; unevidenced totals are null.")
    ]
    bucket_totals: Annotated[
        dict[str, MaybeDecimal], Field(description="Bucket amounts; omitted inputs remain null.")
    ]
    ratios: Annotated[
        list[BenchmarkRatio], Field(description="Comparisons, including unevidenced ratios.")
    ]
    unreviewed_accounts: Annotated[
        None, Field(description="Unknown: this tool receives totals, not an account ledger.")
    ]
    notes: Annotated[list[str], Field(description="Dataset and missing-fact explanations.")]
    checks_to_make: Annotated[
        list[str], Field(description="Suggested human checks, not findings of wrongdoing.")
    ]
    source: Provenance
    disclaimer: Annotated[
        str, Field(description="Engine limitations on using benchmark comparisons.")
    ]
    supplied_buckets: Annotated[
        list[str], Field(description="Buckets explicitly supplied by the operator.")
    ]
    omitted_buckets: Annotated[
        list[str], Field(description="Buckets not supplied; never evidence of zero.")
    ]
    complete_buckets: Annotated[
        bool, Field(description="Whether all required expense buckets were supplied.")
    ]


class PaydayAssessment(ResultObject):
    employee_id: Annotated[str, Field(description="Operator reference echoed from the input.")]
    qe_day: DateText
    sg_amount: DecimalText
    remitted: Annotated[
        DateText | None, Field(description="Remittance date, YYYY-MM-DD, or null if unknown.")
    ]
    received: Annotated[
        DateText | None, Field(description="Fund-receipt date, YYYY-MM-DD, or null if unknown.")
    ]
    due: Annotated[
        DateText | None,
        Field(description="Engine deadline, YYYY-MM-DD, or null when not applicable."),
    ]
    pathway: Annotated[str, Field(description="Deadline pathway chosen by the engine.")]
    verdict: Annotated[
        str, Field(description="Engine contribution verdict; read alongside caveats and pathway.")
    ]
    days_late: Annotated[
        int | None, Field(description="Engine days late, or null when not established.")
    ]
    lateness_basis: Annotated[
        str | None, Field(description="Receipt or assessment basis used to measure lateness.")
    ]
    base_shortfall: MaybeDecimal
    final_shortfall: MaybeDecimal
    notional_earnings: MaybeDecimal
    experimental_sgc_low: MaybeDecimal
    experimental_sgc_high: MaybeDecimal
    uplift: Annotated[
        dict[str, dict[str, DecimalText]] | None,
        Field(
            description="Experimental uplift scenarios by history and disclosure timing, or null."
        ),
    ]
    notes: Annotated[list[str], Field(description="Engine assessment notes.")]
    caveats: Caveats
    horizon_verdicts: Annotated[
        list[str] | None, Field(description="Engine horizon verdicts, when available.")
    ]


class PaydayReview(EngineResult):
    law_content_date: DateText
    as_at: Annotated[DateText, Field(description="Explicit operator assessment date, YYYY-MM-DD.")]
    disclaimer: Annotated[
        str, Field(description="Experimental review and fund-receipt limitations.")
    ]
    result: Annotated[
        PaydayAssessment,
        Field(description="Contribution assessment; not a compliance determination."),
    ]


class VerificationSource(ResultObject):
    verify_at: Annotated[
        str | None,
        Field(description="Engine verification URL; empty or null when none is available."),
    ]


class Div7aResult(EngineResult):
    law_content_date: DateText
    law_compilation: Annotated[str, Field(description="Compiled law identified by the engine.")]
    disclaimer: Annotated[
        str, Field(description="Experimental scope and human-review requirements.")
    ]


class SummaryDetails(ResultObject):
    response_detail: Annotated[Literal["summary"], Field(description="Summary response marker.")]
    source: Annotated[VerificationSource, Field(description="Concise verification source.")]


class RateFields(Div7aResult):
    year_of_income: YearText
    verdict: Annotated[
        Literal["KNOWN", "UNKNOWN"], Field(description="Whether the engine has a reviewed rate.")
    ]
    benchmark_rate: Annotated[
        DecimalText | None,
        Field(description="Decimal fraction, e.g. 0.08 means 8%; null if UNKNOWN."),
    ]
    reason: Annotated[
        str | None,
        Field(description="Engine explanation when the rate is unavailable; otherwise null."),
    ]


class RateSummary(RateFields, SummaryDetails):
    """Concise rate result with source link."""


@with_config(
    ConfigDict(
        extra="allow",
        strict=True,
        json_schema_extra={
            "not": {"required": ["response_detail"]},
        },
    )
)
class RateFull(RateFields):
    provenance: Provenance
    statutory_trace: Trace


class Div7aRate(RootModel[RateSummary | RateFull]):
    model_config = ConfigDict(json_schema_extra={"type": "object"})

    @model_validator(mode="before")
    @classmethod
    def require_summary_source(cls, value: Any) -> Any:
        if isinstance(value, dict) and "response_detail" in value:
            # Do not let a malformed summary fall back to the full union branch.
            TypeAdapter(RateSummary).validate_python(value)
        return value


class Div7aGate(ResultObject):
    verdict: Annotated[
        Literal["COMPLYING", "NOT_COMPLYING", "UNKNOWN"],
        Field(description="Reviewed s 109N gate only; UNKNOWN never means false or compliant."),
    ]
    loan_id: Annotated[str, Field(description="Operator loan reference.")]
    benchmark_year_used: Annotated[
        YearText | None, Field(description="Engine benchmark year, YYYY-YY, if known.")
    ]
    benchmark_rate: MaybeDecimal
    maximum_term_years_allowed: MaybeDecimal
    reasons: Reasons
    caveats: Caveats


class GateFull(Div7aGate):
    benchmark_provenance: Provenance | None
    limbs: Annotated[list[dict[str, str]], Field(description="Individual s 109N findings.")]
    statutory_trace: Trace


class Div7aRepayment(ResultObject):
    verdict: Annotated[
        str,
        Field(
            description="Engine s 109E outcome, including MYR_MET, MYR_SHORT, UNKNOWN or REFUSED."
        ),
    ]
    loan_id: Annotated[str, Field(description="Operator loan reference.")]
    year_of_income: YearText
    gate_verdict: Annotated[
        str | None, Field(description="s 109N gate outcome used in the repayment review.")
    ]
    benchmark_rate: MaybeDecimal
    amalgamated_loan_unpaid_at_end_of_previous_year: MaybeDecimal
    remaining_term_years_used: MaybeDecimal
    myr_required: Annotated[
        DecimalText | None, Field(description="Required repayment in AUD; null if not determined.")
    ]
    payments_applied: MaybeDecimal
    shortfall: MaybeDecimal
    experimental_deemed_dividend_exposure: Annotated[
        DecimalText | None,
        Field(
            description="Experimental AUD exposure only, not an assessed dividend; null if unknown."
        ),
    ]
    rounding: Annotated[str, Field(description="Rounding rule reported by the engine.")]
    reasons: Reasons
    caveats: Caveats


class RepaymentFull(Div7aRepayment):
    benchmark_provenance: Provenance | None
    statutory_trace: Trace


class ReviewSummary(Div7aResult, SummaryDetails):
    gate: Annotated[Div7aGate, Field(description="s 109N gate outcome and reasons.")]
    minimum_yearly_repayment: Annotated[
        Div7aRepayment, Field(description="s 109E review or refusal.")
    ]


@with_config(
    ConfigDict(
        extra="allow",
        strict=True,
        json_schema_extra={
            "not": {"required": ["response_detail"]},
        },
    )
)
class ReviewFull(Div7aResult):
    gate: Annotated[GateFull, Field(description="s 109N gate with full audit fields.")]
    minimum_yearly_repayment: Annotated[
        RepaymentFull, Field(description="s 109E review with full audit fields.")
    ]


class Div7aReview(RootModel[ReviewSummary | ReviewFull]):
    model_config = ConfigDict(json_schema_extra={"type": "object"})

    @model_validator(mode="before")
    @classmethod
    def require_summary_source(cls, value: Any) -> Any:
        if isinstance(value, dict) and "response_detail" in value:
            TypeAdapter(ReviewSummary).validate_python(value)
        return value


class ScopeRefusal(ResultObject):
    ok: Annotated[Literal[False], Field(description="Unsupported request was refused.")]
    available: Annotated[
        Literal[False], Field(description="This compatibility tool cannot calculate a repayment.")
    ]
    reviewed_engine: Annotated[
        Literal[True], Field(description="Separate tools expose the reviewed engine scope.")
    ]
    code: Annotated[
        Literal["ERR_POLICY_DIV7A_SCOPE_REFUSED"],
        Field(description="Machine-readable refusal code."),
    ]
    reason: Annotated[str, Field(description="Supported alternatives and excluded matters.")]


class FixtureFields(ResultObject):
    synthetic: Annotated[
        Literal[True], Field(description="Fabricated test data; never real client results.")
    ]
    not_a_lodgment: Annotated[Literal[True], Field(description="Never a lodgment-ready payload.")]
    entity: Annotated[dict[str, Any], Field(description="Fabricated identity and period fields.")]


CTR_SECTIONS = ("income_statement", "reconciliation")
BAS_SECTIONS = ("gst_labels", "payg_withholding_labels", "summary")


@with_config(
    ConfigDict(
        extra="allow",
        strict=True,
        json_schema_extra={
            "not": {"anyOf": [{"required": [name]} for name in BAS_SECTIONS]},
        },
    )
)
class CtrFixture(FixtureFields):
    form_type: Literal["CTR_AU_2025"]
    income_statement: Annotated[
        dict[str, DecimalText], Field(description="Synthetic CTR income figures.")
    ]
    reconciliation: Annotated[
        dict[str, DecimalText], Field(description="Synthetic CTR reconciliation.")
    ]


@with_config(
    ConfigDict(
        extra="allow",
        strict=True,
        json_schema_extra={
            "not": {"anyOf": [{"required": [name]} for name in CTR_SECTIONS]},
        },
    )
)
class BasFixture(FixtureFields):
    form_type: Literal["BAS_AU_ACTIVITY_STATEMENT"]
    gst_labels: Annotated[dict[str, DecimalText], Field(description="Synthetic BAS GST labels.")]
    payg_withholding_labels: Annotated[
        dict[str, DecimalText], Field(description="Synthetic BAS PAYG labels.")
    ]
    summary: Annotated[dict[str, DecimalText], Field(description="Synthetic BAS total.")]


class SyntheticFixture(RootModel[CtrFixture | BasFixture]):
    model_config = ConfigDict(json_schema_extra={"type": "object"})

    @model_validator(mode="before")
    @classmethod
    def reject_mixed_forms(cls, value: Any) -> Any:
        if isinstance(value, dict):
            form = value.get("form_type")
            forbidden = (
                BAS_SECTIONS
                if form == "CTR_AU_2025"
                else (CTR_SECTIONS if form == "BAS_AU_ACTIVITY_STATEMENT" else ())
            )
            if any(name in value for name in forbidden):
                raise ValueError("Fixture contains sections from the other form")
        return value
