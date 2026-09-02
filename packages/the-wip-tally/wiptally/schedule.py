"""AASB 15 WIP arithmetic for a single unit of account.

This module computes. It does not decide whether a performance obligation
transfers over time, whether a claim is enforceable, or whether a contract is
onerous. Those stay with the person running hardhat-ledger's WIP skill.
"""

from __future__ import annotations

from decimal import Decimal

from .model import (
    FADE_FLAG_POINTS,
    PROGRESS_COST_TO_COST,
    PROGRESS_OUTPUT,
    PROGRESS_RIGHT_TO_INVOICE,
    RETENTION_RECEIVABLE,
    RETENTION_REVIEW,
    ContractInput,
    ContractPosition,
)
from .money import as_money, as_points

ZERO = Decimal("0.00")
STALE_SPEND = Decimal("1000.00")


class ScheduleError(ValueError):
    """Raised when the facts cannot produce a schedule row."""


def _where(contract: ContractInput, field: str) -> str:
    return f"row {contract.line_number} ({contract.contract_id}): {field}"


def _require_non_negative(contract: ContractInput, value: Decimal, field: str) -> None:
    if value < 0:
        raise ScheduleError(f"{_where(contract, field)} is negative")


def measure(contract: ContractInput) -> ContractPosition:
    """Return the WIP position for one contract."""
    _require_non_negative(contract, contract.original_contract_sum, "original_contract_sum")
    _require_non_negative(contract, contract.approved_variations, "approved_variations")
    _require_non_negative(
        contract, contract.unapproved_variations_estimate, "unapproved_variations_estimate"
    )
    _require_non_negative(contract, contract.costs_incurred, "costs_incurred")
    _require_non_negative(
        contract, contract.inefficiency_rework_wastage, "inefficiency_rework_wastage"
    )
    _require_non_negative(contract, contract.uninstalled_materials, "uninstalled_materials")
    _require_non_negative(
        contract, contract.estimated_cost_to_complete, "estimated_cost_to_complete"
    )
    _require_non_negative(contract, contract.certified_billings, "certified_billings")
    _require_non_negative(contract, contract.uncertified_claims, "uncertified_claims")
    _require_non_negative(contract, contract.retention_withheld, "retention_withheld")
    _require_non_negative(contract, contract.committed_outstanding, "committed_outstanding")

    excluded_from_progress = (
        contract.inefficiency_rework_wastage + contract.uninstalled_materials
    )
    if excluded_from_progress > contract.costs_incurred:
        raise ScheduleError(
            f"{_where(contract, 'B19 exclusions')} "
            f"({excluded_from_progress}) exceed costs incurred "
            f"({contract.costs_incurred})"
        )

    approved_price = as_money(contract.original_contract_sum + contract.approved_variations)
    variable_included = as_money(
        contract.unapproved_variations_estimate * contract.constraint_include_ratio
    )
    variable_excluded = as_money(contract.unapproved_variations_estimate - variable_included)
    transaction_price = as_money(approved_price + variable_included)

    progress_cost = as_money(contract.costs_incurred - excluded_from_progress)
    estimated_cost_at_completion = as_money(
        contract.costs_incurred + contract.estimated_cost_to_complete
    )
    progress_eac = as_money(estimated_cost_at_completion - excluded_from_progress)

    flags: list[str] = []
    notes_method = contract.progress_method

    if notes_method == PROGRESS_COST_TO_COST:
        percent_complete, revenue_to_date = _cost_to_cost_revenue(
            contract, transaction_price, progress_cost, progress_eac, flags
        )
    elif notes_method == PROGRESS_OUTPUT:
        flags.append("progress_method_not_cost_to_cost")
        if contract.output_percent is None:
            raise ScheduleError(
                f"{_where(contract, 'output_percent')} is required when "
                f"progress_method is output"
            )
        percent_complete = contract.output_percent
        revenue_to_date = as_money(transaction_price * percent_complete)
    elif notes_method == PROGRESS_RIGHT_TO_INVOICE:
        flags.append("progress_method_not_cost_to_cost")
        if progress_eac <= 0:
            percent_complete = ZERO
        else:
            percent_complete = progress_cost / progress_eac
        revenue_to_date = contract.certified_billings
    else:
        raise ScheduleError(
            f"{_where(contract, 'progress_method')} {notes_method!r} is not supported"
        )

    net = as_money(revenue_to_date - contract.certified_billings)
    if net > 0:
        contract_asset, contract_liability = net, ZERO
    elif net < 0:
        contract_asset, contract_liability = ZERO, as_money(-net)
    else:
        contract_asset, contract_liability = ZERO, ZERO

    gross_profit = as_money(transaction_price - estimated_cost_at_completion)
    margin = _margin(transaction_price, gross_profit)
    prior_margin = _prior_margin(contract)
    fade_points = _fade_points(margin, prior_margin)
    period_revenue = None
    if contract.prior_revenue_to_date is not None:
        period_revenue = as_money(revenue_to_date - contract.prior_revenue_to_date)

    gst_on_certified = as_money(contract.certified_billings * contract.gst_rate)
    gst_on_retention = as_money(contract.retention_withheld * contract.gst_rate)

    _flag_commercial(contract, flags, variable_excluded, fade_points, gross_profit)

    return ContractPosition(
        contract=contract,
        approved_price=approved_price,
        variable_consideration_included=variable_included,
        variable_consideration_excluded=variable_excluded,
        transaction_price=transaction_price,
        progress_cost=progress_cost,
        estimated_cost_at_completion=estimated_cost_at_completion,
        progress_eac=progress_eac,
        percent_complete=percent_complete,
        revenue_to_date=revenue_to_date,
        contract_asset=contract_asset,
        contract_liability=contract_liability,
        gross_profit_at_completion=gross_profit,
        margin_at_completion=margin,
        prior_margin_at_completion=prior_margin,
        profit_fade_points=fade_points,
        period_revenue=period_revenue,
        gst_on_certified_billings=gst_on_certified,
        gst_on_retention=gst_on_retention,
        flags=flags,
    )


def _cost_to_cost_revenue(
    contract: ContractInput,
    transaction_price: Decimal,
    progress_cost: Decimal,
    progress_eac: Decimal,
    flags: list[str],
) -> tuple[Decimal, Decimal]:
    if not contract.outcome_reasonably_measurable:
        flags.append("outcome_not_reasonably_measurable")
        recoverable = (
            contract.recoverable_costs if contract.recoverable_costs is not None else progress_cost
        )
        _require_non_negative(contract, recoverable, "recoverable_costs")
        # AASB 15 para 45: recognise revenue only to the extent of the costs
        # incurred. Recoverable cost above cost to date is refused rather than
        # reduced to it, because the excess is a mapping error the operator has
        # to resolve, not a figure this engine may quietly rewrite.
        if recoverable > contract.costs_incurred:
            raise ScheduleError(
                f"{_where(contract, 'recoverable_costs')} {recoverable} exceeds "
                f"costs incurred {contract.costs_incurred}; para 45 recognises "
                f"revenue only to the extent of the costs incurred"
            )
        # The default is progress cost, which has the B19 exclusions taken out.
        # Uninstalled material cost that is in fact recoverable therefore has to
        # be supplied in recoverable_costs; it is not added back here.
        return ZERO, as_money(recoverable)

    if progress_eac <= 0:
        raise ScheduleError(
            f"{_where(contract, 'progress_eac')} is {progress_eac} after B19 "
            f"exclusions; cost-to-cost progress is undefined"
        )
    if progress_cost > progress_eac:
        raise ScheduleError(
            f"{_where(contract, 'progress_cost')} {progress_cost} exceeds "
            f"progress EAC {progress_eac}"
        )

    percent_complete = progress_cost / progress_eac
    # B19(b): uninstalled materials that do not depict progress are recognised
    # at cost (zero margin). The remainder of the transaction price follows POC.
    poc_base = as_money(transaction_price - contract.uninstalled_materials)
    if poc_base < 0:
        raise ScheduleError(
            f"{_where(contract, 'uninstalled_materials')} {contract.uninstalled_materials} "
            f"exceeds transaction price {transaction_price}"
        )
    revenue = as_money(contract.uninstalled_materials + poc_base * percent_complete)
    return percent_complete, revenue


def _margin(transaction_price: Decimal, gross_profit: Decimal) -> Decimal | None:
    if transaction_price == 0:
        return None
    return gross_profit / transaction_price


def _prior_margin(contract: ContractInput) -> Decimal | None:
    prior_tp = contract.prior_transaction_price
    prior_eac = contract.prior_estimated_cost_at_completion
    if prior_tp is None or prior_eac is None:
        return None
    if prior_tp == 0:
        return None
    return as_money(prior_tp - prior_eac) / prior_tp


def _fade_points(margin: Decimal | None, prior_margin: Decimal | None) -> Decimal | None:
    # Quantised here, not in the formatter, so the fade that is flagged is the
    # same fade the console and the review pack print.
    if margin is None or prior_margin is None:
        return None
    return as_points((prior_margin - margin) * Decimal(100))


def _flag_commercial(
    contract: ContractInput,
    flags: list[str],
    variable_excluded: Decimal,
    fade_points: Decimal | None,
    gross_profit: Decimal,
) -> None:
    if variable_excluded > 0:
        flags.append("variable_consideration_constrained")
    if contract.uncertified_claims > 0:
        flags.append("uncertified_claims_present")
    if contract.retention_classification == RETENTION_REVIEW and contract.retention_withheld > 0:
        flags.append("retention_classification_review")
    elif contract.retention_classification == RETENTION_RECEIVABLE and contract.retention_withheld > 0:
        flags.append("retention_presented_as_receivable_confirm_para_108")
    elif contract.retention_withheld > 0:
        flags.append("retention_remains_in_contract_balance")

    if fade_points is not None and fade_points >= FADE_FLAG_POINTS:
        flags.append("profit_fade")

    if (
        contract.prior_costs_incurred is not None
        and contract.prior_estimated_cost_to_complete is not None
        and contract.estimated_cost_to_complete == contract.prior_estimated_cost_to_complete
        and as_money(contract.costs_incurred - contract.prior_costs_incurred) >= STALE_SPEND
    ):
        flags.append("stale_cost_to_complete")

    # AASB 137 does not wait for the outcome to be reasonably measurable, and an
    # unmeasurable job is the one most likely to be underwater.
    if gross_profit < 0:
        flags.append("negative_margin_at_completion")
        flags.append("onerous_contract_review_aasb_137")
        if contract.assets_used_carrying and contract.assets_used_carrying > 0:
            flags.append("impair_contract_assets_before_provision_aasb_137_69")

    if contract.committed_outstanding > contract.estimated_cost_to_complete:
        flags.append("committed_exceeds_etc")
    if (
        contract.estimated_cost_to_complete > 0
        and contract.committed_outstanding == 0
        and contract.costs_incurred > 0
    ):
        flags.append("etc_has_no_commitments")
