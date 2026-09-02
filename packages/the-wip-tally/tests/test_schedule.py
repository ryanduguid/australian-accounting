from __future__ import annotations

from decimal import Decimal

import pytest

from wiptally.model import ContractInput
from wiptally.money import points
from wiptally.schedule import ScheduleError, measure

ZERO = Decimal("0.00")


def _contract(**overrides: object) -> ContractInput:
    values: dict[str, object] = dict(
        contract_id="JOB-1",
        line_number=2,
        customer="Example Principal Pty Ltd",
        description="Synthetic civil package",
        original_contract_sum=Decimal("1000000.00"),
        approved_variations=ZERO,
        unapproved_variations_estimate=ZERO,
        constraint_include_ratio=Decimal("0"),
        costs_incurred=Decimal("400000.00"),
        inefficiency_rework_wastage=ZERO,
        uninstalled_materials=ZERO,
        estimated_cost_to_complete=Decimal("400000.00"),
        certified_billings=Decimal("450000.00"),
        uncertified_claims=ZERO,
        retention_withheld=Decimal("45000.00"),
        retention_classification="conditional",
        committed_outstanding=Decimal("380000.00"),
        outcome_reasonably_measurable=True,
        recoverable_costs=None,
        progress_method="cost_to_cost",
        output_percent=None,
        prior_transaction_price=Decimal("1000000.00"),
        prior_estimated_cost_at_completion=Decimal("800000.00"),
        prior_costs_incurred=Decimal("200000.00"),
        prior_estimated_cost_to_complete=Decimal("600000.00"),
        prior_revenue_to_date=Decimal("250000.00"),
        gst_rate=Decimal("0.10"),
        assets_used_carrying=None,
    )
    values.update(overrides)
    return ContractInput(**values)  # type: ignore[arg-type]


def test_clean_cost_to_cost_underbilling() -> None:
    position = measure(_contract())
    assert position.transaction_price == Decimal("1000000.00")
    assert position.percent_complete == Decimal("0.5")
    assert position.revenue_to_date == Decimal("500000.00")
    assert position.contract_asset == Decimal("50000.00")
    assert position.contract_liability == ZERO
    assert position.gross_profit_at_completion == Decimal("200000.00")
    assert position.margin_at_completion == Decimal("0.2")
    assert position.profit_fade_points == ZERO
    assert position.period_revenue == Decimal("250000.00")
    assert position.gst_on_certified_billings == Decimal("45000.00")
    assert position.gst_on_retention == Decimal("4500.00")
    assert "profit_fade" not in position.flags
    assert position.needs_review is False


def test_b19_wastage_is_stripped_from_progress_but_stays_in_margin() -> None:
    position = measure(
        _contract(
            original_contract_sum=Decimal("800000.00"),
            costs_incurred=Decimal("500000.00"),
            inefficiency_rework_wastage=Decimal("80000.00"),
            estimated_cost_to_complete=Decimal("300000.00"),
            certified_billings=Decimal("500000.00"),
            prior_transaction_price=Decimal("800000.00"),
            prior_estimated_cost_at_completion=Decimal("650000.00"),
            prior_costs_incurred=Decimal("350000.00"),
            prior_estimated_cost_to_complete=Decimal("300000.00"),
            prior_revenue_to_date=Decimal("400000.00"),
            retention_classification="review",
            retention_withheld=Decimal("40000.00"),
            committed_outstanding=Decimal("250000.00"),
        )
    )
    assert position.progress_cost == Decimal("420000.00")
    assert position.progress_eac == Decimal("720000.00")
    assert position.estimated_cost_at_completion == Decimal("800000.00")
    assert position.revenue_to_date == Decimal("466666.67")
    assert position.contract_liability == Decimal("33333.33")
    assert position.contract_asset == ZERO
    assert position.gross_profit_at_completion == ZERO
    assert "profit_fade" in position.flags
    assert "stale_cost_to_complete" in position.flags
    assert position.needs_review is True


def test_uninstalled_materials_earn_zero_margin() -> None:
    position = measure(
        _contract(
            original_contract_sum=Decimal("500000.00"),
            costs_incurred=Decimal("420000.00"),
            uninstalled_materials=Decimal("70000.00"),
            estimated_cost_to_complete=Decimal("150000.00"),
            certified_billings=Decimal("300000.00"),
            committed_outstanding=Decimal("200000.00"),
            prior_transaction_price=Decimal("500000.00"),
            prior_estimated_cost_at_completion=Decimal("480000.00"),
            prior_costs_incurred=Decimal("200000.00"),
            prior_estimated_cost_to_complete=Decimal("280000.00"),
            prior_revenue_to_date=Decimal("180000.00"),
            assets_used_carrying=Decimal("40000.00"),
            retention_withheld=Decimal("25000.00"),
        )
    )
    assert position.percent_complete == Decimal("0.7")
    assert position.revenue_to_date == Decimal("371000.00")
    assert position.contract_asset == Decimal("71000.00")
    assert position.gross_profit_at_completion == Decimal("-70000.00")
    assert "onerous_contract_review_aasb_137" in position.flags
    assert "impair_contract_assets_before_provision_aasb_137_69" in position.flags
    assert "committed_exceeds_etc" in position.flags


def test_constrained_variable_consideration_is_not_full_claim() -> None:
    position = measure(
        _contract(
            original_contract_sum=Decimal("2000000.00"),
            approved_variations=Decimal("100000.00"),
            unapproved_variations_estimate=Decimal("400000.00"),
            constraint_include_ratio=Decimal("0.25"),
            costs_incurred=Decimal("1500000.00"),
            estimated_cost_to_complete=Decimal("500000.00"),
            certified_billings=Decimal("1800000.00"),
            uncertified_claims=Decimal("200000.00"),
            retention_withheld=Decimal("90000.00"),
            retention_classification="review",
            committed_outstanding=Decimal("480000.00"),
            prior_transaction_price=Decimal("2100000.00"),
            prior_estimated_cost_at_completion=Decimal("1900000.00"),
            prior_costs_incurred=Decimal("900000.00"),
            prior_estimated_cost_to_complete=Decimal("1000000.00"),
            prior_revenue_to_date=Decimal("990000.00"),
        )
    )
    assert position.transaction_price == Decimal("2200000.00")
    assert position.variable_consideration_included == Decimal("100000.00")
    assert position.variable_consideration_excluded == Decimal("300000.00")
    assert position.revenue_to_date == Decimal("1650000.00")
    assert position.contract_liability == Decimal("150000.00")
    assert "variable_consideration_constrained" in position.flags
    assert "uncertified_claims_present" in position.flags
    # Fade is under one point, so it is not a review flag.
    assert position.profit_fade_points is not None
    assert position.profit_fade_points < Decimal("1.00")
    assert "profit_fade" not in position.flags


def test_para_45_recognises_recoverable_cost_only() -> None:
    position = measure(
        _contract(
            original_contract_sum=Decimal("600000.00"),
            costs_incurred=Decimal("100000.00"),
            estimated_cost_to_complete=Decimal("400000.00"),
            certified_billings=ZERO,
            outcome_reasonably_measurable=False,
            recoverable_costs=Decimal("100000.00"),
            retention_withheld=ZERO,
            committed_outstanding=ZERO,
            prior_transaction_price=None,
            prior_estimated_cost_at_completion=None,
            prior_costs_incurred=None,
            prior_estimated_cost_to_complete=None,
            prior_revenue_to_date=None,
        )
    )
    assert position.revenue_to_date == Decimal("100000.00")
    assert position.percent_complete == Decimal("0.00")
    assert position.contract_asset == Decimal("100000.00")
    assert "outcome_not_reasonably_measurable" in position.flags


def test_para_45_revenue_cannot_exceed_costs_incurred() -> None:
    """Para 45 recognises revenue only to the extent of the costs incurred.

    A mis-mapped column would otherwise book a $900,000 contract asset against
    $100,000 of cost on a $600,000 job.
    """
    with pytest.raises(ScheduleError):
        measure(
            _contract(
                original_contract_sum=Decimal("600000.00"),
                costs_incurred=Decimal("100000.00"),
                estimated_cost_to_complete=Decimal("400000.00"),
                certified_billings=ZERO,
                outcome_reasonably_measurable=False,
                recoverable_costs=Decimal("900000.00"),
            )
        )


def test_onerous_flag_does_not_depend_on_a_measurable_outcome() -> None:
    """AASB 137's test does not wait for the outcome to become measurable.

    The row still writes a negative gross profit at completion, so suppressing
    the flag would leave the figure and the verdict disagreeing.
    """
    position = measure(
        _contract(
            original_contract_sum=Decimal("600000.00"),
            costs_incurred=Decimal("100000.00"),
            estimated_cost_to_complete=Decimal("900000.00"),
            certified_billings=ZERO,
            outcome_reasonably_measurable=False,
            recoverable_costs=Decimal("100000.00"),
            assets_used_carrying=Decimal("40000.00"),
        )
    )
    assert position.gross_profit_at_completion == Decimal("-400000.00")
    assert "negative_margin_at_completion" in position.flags
    assert "onerous_contract_review_aasb_137" in position.flags
    assert "impair_contract_assets_before_provision_aasb_137_69" in position.flags


def test_profit_fade_flag_reads_the_fade_that_is_displayed() -> None:
    """A displayed 1.00-point fade must carry the flag that 1.00 points earns.

    Prior EAC $790,010 against $800,000 is a raw 0.999-point movement, which
    prints as 1.00 points.
    """
    position = measure(
        _contract(prior_estimated_cost_at_completion=Decimal("790010.00"))
    )
    assert position.profit_fade_points is not None
    assert points(position.profit_fade_points) == "1.00 points"
    assert "profit_fade" in position.flags


def test_right_to_invoice_sets_revenue_equal_to_certified_billings() -> None:
    position = measure(_contract(progress_method="right_to_invoice"))
    assert position.revenue_to_date == Decimal("450000.00")
    assert position.contract_asset == ZERO
    assert position.contract_liability == ZERO
    assert "progress_method_not_cost_to_cost" in position.flags


def test_output_method_uses_supplied_percent() -> None:
    position = measure(
        _contract(
            progress_method="output",
            output_percent=Decimal("0.40"),
        )
    )
    assert position.revenue_to_date == Decimal("400000.00")
    assert position.percent_complete == Decimal("0.40")


def test_contracts_are_not_netted_by_the_caller() -> None:
    under = measure(_contract())
    over = measure(
        _contract(
            contract_id="JOB-2",
            certified_billings=Decimal("700000.00"),
            retention_withheld=ZERO,
        )
    )
    assets = under.contract_asset + over.contract_asset
    liabilities = under.contract_liability + over.contract_liability
    assert assets == Decimal("50000.00")
    assert liabilities == Decimal("200000.00")
    assert assets - liabilities != under.contract_asset  # a net figure would hide the pair


def test_b19_exclusions_cannot_exceed_cost() -> None:
    with pytest.raises(ScheduleError):
        measure(
            _contract(
                costs_incurred=Decimal("100.00"),
                inefficiency_rework_wastage=Decimal("80.00"),
                uninstalled_materials=Decimal("30.00"),
            )
        )


def test_negative_etc_is_refused() -> None:
    with pytest.raises(ScheduleError):
        measure(_contract(estimated_cost_to_complete=Decimal("-1.00")))
