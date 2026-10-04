"""Seeded properties that hold for every input the worksheets accept."""

from decimal import Decimal

from austaxcalc.calculations import (
    capital_gains,
    contribution_caps,
    gst,
    payg_withholding,
    pension_minimum,
    resident_tax,
    study_loan_repayment,
)
from austaxcalc.metadata import (
    PAYG_WITHHOLDING_COEFFICIENTS,
    RESIDENT_TAX_SCALES,
    SUPPORTED_PERIODS,
)
from hypothesis import example, given, seed, settings
from hypothesis import strategies as st

PROPERTY_SETTINGS = settings(max_examples=100, database=None, deadline=None)
MONEY = st.decimals(min_value=Decimal("0"), max_value=Decimal("100000000.00"), places=2)
DOLLARS = st.integers(min_value=0, max_value=2_000_000).map(Decimal)
AGES = st.integers(min_value=0, max_value=150)
PAY = st.decimals(min_value=Decimal("0"), max_value=Decimal("100000.00"), places=2)
PAYG_SCALES = [(year, scale) for year in SUPPORTED_PERIODS["payg_withholding"]
               for scale in sorted(PAYG_WITHHOLDING_COEFFICIENTS[year])]


def amounts(result: dict) -> dict[str, Decimal]:
    return {key: Decimal(value) for key, value in result["amounts"].items()}


def tax(income: Decimal, year: str) -> Decimal:
    return amounts(resident_tax(income, year, True))["basic_income_tax"]


@seed(0x6571)
@PROPERTY_SETTINGS
@given(amount=MONEY, inclusive=st.booleans())
@example(amount=Decimal("0.06"), inclusive=True)
@example(amount=Decimal("0.05"), inclusive=False)
def test_gst_and_the_exclusive_amount_add_to_the_inclusive_amount_to_the_cent(amount, inclusive):
    result = amounts(gst(amount, inclusive, True, "2025-26"))

    assert result["gst"] + result["exclusive"] == result["inclusive"]
    assert result["inclusive" if inclusive else "exclusive"] == amount
    share = result["inclusive"] / 11 if inclusive else result["exclusive"] / 10
    assert abs(result["gst"] - share) <= Decimal("0.005")


@seed(0x7A11)
@PROPERTY_SETTINGS
@given(year=st.sampled_from(SUPPORTED_PERIODS["resident_tax"]), low=DOLLARS, high=DOLLARS)
@example(year="2026-27", low=Decimal(18200), high=Decimal(18201))
@example(year="2026-27", low=Decimal(45000), high=Decimal(45001))
def test_resident_tax_never_falls_and_its_marginal_rate_never_falls(year, low, high):
    low, high = sorted((low, high))
    scale = RESIDENT_TAX_SCALES[year]
    top_rate = max(Decimal(rate) for _, _, rate in scale)

    assert Decimal(0) <= tax(low, year) <= tax(high, year) <= high * top_rate
    assert tax(low + 1, year) - tax(low, year) <= tax(high + 1, year) - tax(high, year)
    tax_free_threshold = scale[0][0]
    if low <= tax_free_threshold:
        assert tax(low, year) == 0


@seed(0x9A76)
@PROPERTY_SETTINGS
@given(year_scale=st.sampled_from(PAYG_SCALES),
       period=st.sampled_from(["weekly", "fortnightly", "monthly"]),
       low=PAY, high=PAY)
@example(year_scale=("2026-27", 2), period="monthly", low=Decimal("3000.32"),
         high=Decimal("3000.33"))
def test_payg_withholding_never_falls_as_earnings_rise(year_scale, period, low, high):
    year, scale = year_scale
    low, high = sorted((low, high))

    def withheld(earnings: Decimal) -> Decimal:
        return amounts(payg_withholding(earnings, period, scale, year, True))["withholding"]

    assert Decimal(0) <= withheld(low) <= withheld(high)


def test_payg_withholding_never_falls_across_any_bracket_boundary():
    # Schedule 1 works on whole-dollar weekly earnings. Whole-dollar steps of a
    # weekly, fortnightly or monthly payment move those by at most $1, so this
    # sweep reaches every weekly amount, and every bracket edge, of each scale.
    for year, scale in PAYG_SCALES:
        rows = PAYG_WITHHOLDING_COEFFICIENTS[year][scale]
        last_limit = max(limit for limit, _, _ in rows if limit is not None)
        for period, weeks in (("weekly", 1), ("fortnightly", 2), ("monthly", 5)):
            previous = Decimal(0)
            for dollars in range((last_limit + 50) * weeks):
                withheld = amounts(payg_withholding(Decimal(dollars), period, scale, year,
                                                    True))["withholding"]
                assert withheld >= previous, (year, scale, period, dollars)
                previous = withheld


@seed(0x5714)
@PROPERTY_SETTINGS
@given(year=st.sampled_from(SUPPORTED_PERIODS["study_loan_repayment"]), low=DOLLARS,
       high=DOLLARS)
@example(year="2026-27", low=Decimal(69528), high=Decimal(69529))
@example(year="2026-27", low=Decimal(186050), high=Decimal(186051))
def test_study_loan_repayment_never_falls_and_never_exceeds_a_tenth(year, low, high):
    low, high = sorted((low, high))

    def repayment(income: Decimal) -> Decimal:
        return amounts(study_loan_repayment(income, year, True))["compulsory_repayment"]

    assert Decimal(0) <= repayment(low) <= repayment(high)
    assert repayment(high) <= high / 10


@seed(0x9E45)
@PROPERTY_SETTINGS
@given(year=st.sampled_from(SUPPORTED_PERIODS["pension_minimum"]), low=MONEY, high=MONEY,
       younger=AGES, older=AGES, days=st.integers(min_value=1, max_value=365))
@example(year="2025-26", low=Decimal("100000.00"), high=Decimal("100000.00"),
         younger=64, older=65, days=365)
def test_pension_minimum_never_falls_with_balance_or_age(year, low, high, younger, older, days):
    low, high = sorted((low, high))
    younger, older = sorted((younger, older))

    def minimum(balance: Decimal, age: int) -> Decimal:
        return amounts(pension_minimum(balance, age, days, year, True))["minimum_payment"]

    assert minimum(low, younger) <= minimum(high, younger) <= minimum(high, older)
    assert minimum(high, older) % 10 == 0


@seed(0xC671)
@PROPERTY_SETTINGS
@given(other=MONEY, discount=MONEY, current=MONEY, prior=MONEY, extra=MONEY)
@example(other=Decimal("100.00"), discount=Decimal("100.00"), current=Decimal("150.00"),
         prior=Decimal("0.00"), extra=Decimal("0.01"))
def test_capital_losses_are_all_accounted_for_and_more_losses_never_raise_the_gain(
        other, discount, current, prior, extra):
    first = amounts(capital_gains(other, discount, current, prior, True, "2025-26"))
    more = amounts(capital_gains(other, discount, current, prior + extra, True, "2025-26"))

    assert first["losses_used"] + first["losses_remaining"] == current + prior
    assert first["losses_used"] <= other + discount
    assert more["net_capital_gain"] <= first["net_capital_gain"] <= other + discount


@seed(0xCA95)
@PROPERTY_SETTINGS
@given(year=st.sampled_from(SUPPORTED_PERIODS["contribution_caps"]), balance=MONEY,
       concessional=MONEY, unused=MONEY, non_concessional=MONEY, under_75=st.booleans())
def test_cap_room_and_excess_reconcile_to_the_cap_available(
        year, balance, concessional, unused, non_concessional, under_75):
    result = amounts(contribution_caps(balance, concessional, unused, non_concessional,
                                       under_75, year, True))

    for kind, contributed in (("concessional", concessional),
                              ("non_concessional", non_concessional)):
        remaining, excess = result[f"{kind}_remaining"], result[f"excess_{kind}"]
        assert remaining - excess == result[f"{kind}_available"] - contributed
        assert remaining == 0 or excess == 0
