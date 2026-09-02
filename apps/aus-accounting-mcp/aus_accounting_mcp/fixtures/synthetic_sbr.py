"""Synthetic SBR fixtures for agent testing. Not lodgment payloads."""

from __future__ import annotations

from decimal import Decimal
from typing import Any


def generate_synthetic_ctr_payload(
    company_name: str = "Synthetix Pty Ltd",
    tfn_masked: str = "XXX-XXX-123",
    abn: str = "11 222 333 444",
    financial_year: int = 2025,
    gross_revenue: Decimal = Decimal("2500000.00"),
    cost_of_sales: Decimal | None = None,
    deductible_operating_expenses: Decimal | None = None,
    non_deductible_entertainment: Decimal = Decimal("15000.00"),
    is_base_rate_entity: bool = True,
) -> dict[str, Any]:
    """Generate a synthetic Company Tax Return fixture. Not a lodgment."""
    cos = (
        cost_of_sales
        if cost_of_sales is not None
        else (gross_revenue * Decimal("0.40")).quantize(Decimal("0.01"))
    )
    expenses = (
        deductible_operating_expenses
        if deductible_operating_expenses is not None
        else (gross_revenue * Decimal("0.30")).quantize(Decimal("0.01"))
    )

    gross_profit = gross_revenue - cos
    accounting_net_profit = gross_profit - expenses - non_deductible_entertainment
    taxable_income = accounting_net_profit + non_deductible_entertainment
    tax_rate = Decimal("0.25") if is_base_rate_entity else Decimal("0.30")
    tax_liability = (taxable_income * tax_rate).quantize(Decimal("0.01"))

    return {
        "synthetic": True,
        "not_a_lodgment": True,
        "form_type": "CTR_AU_2025",
        "entity": {
            "legal_name": company_name,
            "tfn": tfn_masked,
            "abn": abn,
            "financial_year": financial_year,
            "base_rate_entity": is_base_rate_entity,
        },
        "income_statement": {
            "total_business_income": str(gross_revenue),
            "cost_of_sales": str(cos),
            "gross_profit": str(gross_profit),
            "total_expenses": str(expenses + non_deductible_entertainment),
            "operating_net_profit": str(accounting_net_profit),
        },
        "reconciliation": {
            "accounting_profit": str(accounting_net_profit),
            "add_back_non_deductible": str(non_deductible_entertainment),
            "taxable_income": str(taxable_income),
            "applicable_tax_rate": str(tax_rate),
            "gross_tax_liability": str(tax_liability),
        },
    }


def generate_synthetic_bas_payload(
    entity_name: str = "Synthetix Pty Ltd",
    abn: str = "11 222 333 444",
    quarter_ended: str = "2025-03-31",
    total_sales_g1: Decimal = Decimal("660000.00"),
    capital_purchases_g10: Decimal = Decimal("55000.00"),
    non_capital_purchases_g11: Decimal | None = None,
    total_salary_wages_w1: Decimal = Decimal("150000.00"),
    payg_withheld_w2: Decimal = Decimal("37500.00"),
) -> dict[str, Any]:
    """Generate a synthetic BAS fixture. Not a lodgment."""
    g11 = (
        non_capital_purchases_g11
        if non_capital_purchases_g11 is not None
        else (total_sales_g1 * Decimal("0.40")).quantize(Decimal("0.01"))
    )

    gst_collected_1a = (total_sales_g1 / Decimal("11.0")).quantize(Decimal("0.01"))
    gst_purchases_1b = ((capital_purchases_g10 + g11) / Decimal("11.0")).quantize(Decimal("0.01"))
    net_gst = gst_collected_1a - gst_purchases_1b
    net_bas_payable = net_gst + payg_withheld_w2

    return {
        "synthetic": True,
        "not_a_lodgment": True,
        "form_type": "BAS_AU_ACTIVITY_STATEMENT",
        "entity": {
            "name": entity_name,
            "abn": abn,
            "quarter_ended": quarter_ended,
        },
        "gst_labels": {
            "G1_total_sales": str(total_sales_g1),
            "G10_capital_purchases": str(capital_purchases_g10),
            "G11_non_capital_purchases": str(g11),
            "1A_gst_on_sales": str(gst_collected_1a),
            "1B_gst_on_purchases": str(gst_purchases_1b),
            "net_gst": str(net_gst),
        },
        "payg_withholding_labels": {
            "W1_total_salary_wages": str(total_salary_wages_w1),
            "W2_amounts_withheld": str(payg_withheld_w2),
        },
        "summary": {
            "total_payable_to_ato": str(net_bas_payable),
        },
    }
