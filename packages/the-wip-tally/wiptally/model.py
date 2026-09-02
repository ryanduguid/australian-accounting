"""Contract facts the schedule consumes, and the computed position it returns."""

from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal

PROGRESS_COST_TO_COST = "cost_to_cost"
PROGRESS_OUTPUT = "output"
PROGRESS_RIGHT_TO_INVOICE = "right_to_invoice"
PROGRESS_METHODS = frozenset(
    {PROGRESS_COST_TO_COST, PROGRESS_OUTPUT, PROGRESS_RIGHT_TO_INVOICE}
)

RETENTION_RECEIVABLE = "receivable"
RETENTION_CONDITIONAL = "conditional"
RETENTION_REVIEW = "review"
RETENTION_CLASSES = frozenset(
    {RETENTION_RECEIVABLE, RETENTION_CONDITIONAL, RETENTION_REVIEW}
)

FADE_FLAG_POINTS = Decimal("1.00")


@dataclass(frozen=True)
class ContractInput:
    """One unit of account. Combination under AASB 15 para 17 is an operator fact."""

    contract_id: str
    line_number: int
    customer: str
    description: str
    original_contract_sum: Decimal
    approved_variations: Decimal
    unapproved_variations_estimate: Decimal
    constraint_include_ratio: Decimal
    costs_incurred: Decimal
    inefficiency_rework_wastage: Decimal
    uninstalled_materials: Decimal
    estimated_cost_to_complete: Decimal
    certified_billings: Decimal
    uncertified_claims: Decimal
    retention_withheld: Decimal
    retention_classification: str
    committed_outstanding: Decimal
    outcome_reasonably_measurable: bool
    recoverable_costs: Decimal | None
    progress_method: str
    output_percent: Decimal | None
    prior_transaction_price: Decimal | None
    prior_estimated_cost_at_completion: Decimal | None
    prior_costs_incurred: Decimal | None
    prior_estimated_cost_to_complete: Decimal | None
    prior_revenue_to_date: Decimal | None
    gst_rate: Decimal
    assets_used_carrying: Decimal | None


@dataclass
class ContractPosition:
    """Computed WIP position for one contract. Money fields are cent-quantised."""

    contract: ContractInput
    approved_price: Decimal
    variable_consideration_included: Decimal
    variable_consideration_excluded: Decimal
    transaction_price: Decimal
    progress_cost: Decimal
    estimated_cost_at_completion: Decimal
    progress_eac: Decimal
    percent_complete: Decimal
    revenue_to_date: Decimal
    contract_asset: Decimal
    contract_liability: Decimal
    gross_profit_at_completion: Decimal
    margin_at_completion: Decimal | None
    prior_margin_at_completion: Decimal | None
    profit_fade_points: Decimal | None
    period_revenue: Decimal | None
    gst_on_certified_billings: Decimal
    gst_on_retention: Decimal
    flags: list[str] = field(default_factory=list)

    @property
    def needs_review(self) -> bool:
        return any(flag in REVIEW_FLAGS for flag in self.flags)


# Flags that force exit code 2. Informational flags stay on the row.
REVIEW_FLAGS = frozenset(
    {
        "profit_fade",
        "stale_cost_to_complete",
        "onerous_contract_review_aasb_137",
        "impair_contract_assets_before_provision_aasb_137_69",
        "outcome_not_reasonably_measurable",
        "variable_consideration_constrained",
        "committed_exceeds_etc",
        "etc_has_no_commitments",
        "uncertified_claims_present",
        "retention_classification_review",
        "progress_method_not_cost_to_cost",
        "negative_margin_at_completion",
    }
)


@dataclass
class Schedule:
    """All contracts in the file, plus the portfolio totals that must not be netted."""

    as_at: str
    positions: list[ContractPosition]
    source_name: str

    @property
    def total_contract_assets(self) -> Decimal:
        return sum((row.contract_asset for row in self.positions), Decimal("0.00"))

    @property
    def total_contract_liabilities(self) -> Decimal:
        return sum((row.contract_liability for row in self.positions), Decimal("0.00"))

    @property
    def total_revenue_to_date(self) -> Decimal:
        return sum((row.revenue_to_date for row in self.positions), Decimal("0.00"))

    @property
    def review_rows(self) -> list[ContractPosition]:
        return [row for row in self.positions if row.needs_review]
