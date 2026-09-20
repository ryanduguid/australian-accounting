"""
Corporate tax rate determination and Base Rate Entity (BRE) test under
Sections 23AA & 23AB of the Income Tax Rates Act 1986 and Division 328 ITAA 1997.
"""

from dataclasses import dataclass
from decimal import ROUND_HALF_UP, Decimal
from typing import Optional

# Historical Base Rate Entity Tax Rates
BRE_RATES: dict[int, Decimal] = {
    2018: Decimal("0.275"),
    2019: Decimal("0.275"),
    2020: Decimal("0.275"),
    2021: Decimal("0.260"),
    2022: Decimal("0.250"),
    2023: Decimal("0.250"),
    2024: Decimal("0.250"),
    2025: Decimal("0.250"),
    2026: Decimal("0.250"),
    2027: Decimal("0.250"),
}

STANDARD_CORPORATE_RATE = Decimal("0.300")
# Aggregated turnover threshold under s 23AA(b) ITRA 1986. $25M for 2017-18
# (Treasury Laws Amendment (Enterprise Tax Plan) Act 2017 Sch 1 Pt 2); $50M
# from 2018-19 (Sch 1 Pt 3 item 16). Every year the rate table covers is listed
# here: a year with no listed threshold is refused rather than given the last
# threshold Parliament happened to legislate.
TURNOVER_THRESHOLDS: dict[int, Decimal] = {
    2018: Decimal("25000000.00"),
    2019: Decimal("50000000.00"),
    2020: Decimal("50000000.00"),
    2021: Decimal("50000000.00"),
    2022: Decimal("50000000.00"),
    2023: Decimal("50000000.00"),
    2024: Decimal("50000000.00"),
    2025: Decimal("50000000.00"),
    2026: Decimal("50000000.00"),
    2027: Decimal("50000000.00"),
}
BREPI_THRESHOLD_PERCENT = Decimal("80.00")    # Passive income must not exceed 80% (s 23AB)


def turnover_threshold_for(fy: int) -> Decimal:
    try:
        return TURNOVER_THRESHOLDS[fy]
    except KeyError:
        raise ValueError(
            f"No legislated aggregated turnover threshold exists for FY{fy}"
        ) from None


@dataclass(frozen=True)
class BaseRateEntityTest:
    financial_year: int
    aggregated_turnover: Decimal
    assessable_income: Decimal
    passive_income: Decimal  # Base Rate Entity Passive Income (BREPI)

    def __post_init__(self) -> None:
        for name in ("aggregated_turnover", "assessable_income", "passive_income"):
            value = getattr(self, name)
            if not value.is_finite() or value < Decimal("0.00"):
                raise ValueError(f"{name} must be a non-negative finite amount, got {value}")
        if self.passive_income > self.assessable_income:
            raise ValueError("passive_income must not exceed assessable_income")

    @property
    def passive_income_percentage(self) -> Optional[Decimal]:
        """
        Display ratio, rounded to 2dp. The eligibility test compares exactly.
        None where there is no assessable income: the ratio has no denominator,
        and reporting a figure states a passive proportion that was never worked out.
        """
        if self.assessable_income <= Decimal("0.00"):
            return None
        pct = (self.passive_income / self.assessable_income) * Decimal("100.00")
        return pct.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)

    @property
    def is_aggregated_turnover_eligible(self) -> bool:
        return self.aggregated_turnover < turnover_threshold_for(self.financial_year)

    @property
    def is_brepi_eligible(self) -> Optional[bool]:
        """
        The s 23AB passive-income test, or None where it cannot be run.

        Exact comparison by cross-multiplication: rounding the ratio first
        would pass a company whose BREPI is just over the 80% limit. With no
        assessable income there is no proportion to compare, so the test is
        not run and the answer is unknown, the same absence
        `passive_income_percentage` reports. Reading the arithmetic 0 <= 0 as
        satisfied turns a missing income fact into a passed test.
        """
        if self.assessable_income <= Decimal("0.00"):
            return None
        return (
            self.passive_income * Decimal("100.00")
            <= BREPI_THRESHOLD_PERCENT * self.assessable_income
        )

    @property
    def is_base_rate_entity(self) -> Optional[bool]:
        """
        Base rate entity status under s 23AA, or None where it is unknown.

        Failing the turnover test settles the question on its own, so an
        unknown passive-income test only leaves the status unknown where the
        turnover test is met.
        """
        if not self.is_aggregated_turnover_eligible:
            return False
        return self.is_brepi_eligible


@dataclass(frozen=True)
class CorporateTaxRate:
    financial_year: int
    is_base_rate_entity: bool
    applicable_rate: Decimal
    rate_description: str
    statutory_basis: str


def determine_corporate_tax_rate(test: BaseRateEntityTest) -> CorporateTaxRate:
    """
    Determine the company tax rate under s23AA Income Tax Rates Act 1986.
    """
    fy = test.financial_year
    if fy not in BRE_RATES:
        raise ValueError(f"No legislated company-rate table exists for FY{fy}")

    is_bre = test.is_base_rate_entity
    if is_bre is None:
        raise ValueError(
            f"FY{fy}: base rate entity status is not determinable because there is "
            "no assessable income to run the s 23AB passive-income test against; "
            "supply assessable_income for the year"
        )

    threshold_m = turnover_threshold_for(fy) / Decimal("1000000")
    if is_bre:
        rate = BRE_RATES[fy]
        desc = f"Base Rate Entity ({rate * 100:.1f}%)"
        basis = (
            f"s 23AA Income Tax Rates Act 1986; turnover < ${threshold_m:.0f}M "
            "and BREPI <= 80%"
        )
    else:
        rate = STANDARD_CORPORATE_RATE
        desc = "Standard Corporate Tax Rate (30.0%)"
        basis = "s 23(2) Income Tax Rates Act 1986; exceeds turnover or BREPI threshold"

    return CorporateTaxRate(
        financial_year=fy,
        is_base_rate_entity=is_bre,
        applicable_rate=rate,
        rate_description=desc,
        statutory_basis=basis,
    )


def determine_max_franking_rate(
    current_fy: int,
    prior_year_test: Optional[BaseRateEntityTest] = None,
) -> Decimal:
    """
    Determine the corporate tax rate for imputation purposes (the maximum
    franking rate) under s 995-1 ITAA 1997 and s 202-60.

    The rate is the CURRENT year's corporate tax rate, worked out on the
    assumption that the entity's aggregated turnover, BREPI and assessable
    income for the current year equal the prior year's figures. Only the
    amounts are assumed from the prior year; the rate scale and thresholds
    are the current year's. Evidence must be labelled with the immediately
    preceding financial year. Missing or wrong-year evidence is refused.
    """
    if prior_year_test is None:
        raise ValueError(
            f"prior_year_test is required for FY{current_fy}; "
            "the maximum franking rate is not assumed"
        )
    if prior_year_test.financial_year != current_fy - 1:
        raise ValueError(
            f"prior_year_test must be for FY{current_fy - 1}, "
            f"got FY{prior_year_test.financial_year}"
        )
    assumed_current_year = BaseRateEntityTest(
        financial_year=current_fy,
        aggregated_turnover=prior_year_test.aggregated_turnover,
        assessable_income=prior_year_test.assessable_income,
        passive_income=prior_year_test.passive_income,
    )
    return determine_corporate_tax_rate(assumed_current_year).applicable_rate
