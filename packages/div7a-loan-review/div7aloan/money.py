"""Decimal money and rate handling.

Every amount in this engine is a decimal.Decimal built from a string. There
is no float anywhere: the s 109E(6) formula divides by a term that approaches
zero as the remaining term shortens, and binary floating point turns a cent
of rounding there into dollars of phantom shortfall.

Money is quantised to cents with ROUND_HALF_UP. The Act prescribes no
rounding for the minimum yearly repayment, so the choice is this engine's and
not the Commissioner's; it is stated here, in the README, and in the
statutory trace of every result. See evaluation/div7a_myr/README.md.
"""
from __future__ import annotations

from decimal import Decimal, InvalidOperation, localcontext

CENTS = Decimal("0.01")
MAX_MONEY_MAGNITUDE = Decimal("1000000000000.00")
MAX_MONEY_DECIMAL_PLACES = 2

# ROUND_HALF_UP is the ordinary accounting convention and the one a reviewer
# re-performing on paper will reach for. It is not a statutory rule.
ROUNDING = "ROUND_HALF_UP"

# Intermediate precision for the s 109E(6) formula. (1 + r) raised to a
# 25-year term carries about 100 significant digits if held exactly; 60 is
# far more than cents need and keeps every run deterministic across
# platforms, because the result never depends on the ambient decimal context
# a caller happens to have set.
FORMULA_PRECISION = 60

# A benchmark rate above this is a typo, not a rate. The RBA indicator rate
# for bank variable housing loans has never approached 100% a year, so the
# ceiling costs nothing real and catches the two hand-edit slips that move
# money: a dropped decimal point (877 for 0.0877) and a stray minus sign.
RATE_CEILING = Decimal("1")


class MoneyError(ValueError):
    """A supplied amount, rate or ratio is not a usable number."""


def _decimal(raw: object, where: str) -> Decimal:
    """Build a Decimal from operator input, refusing the values that poison a
    report silently.

    decimal.InvalidOperation is an ArithmeticError rather than a ValueError,
    so an unguarded conversion escapes the CLI's error handling and prints a
    traceback. A NaN escapes nothing at all: it compares false against every
    threshold, so a NaN payment would sail through the shortfall test and
    report MYR_MET.
    """
    text = str(raw).strip()
    if not text:
        raise MoneyError(f"{where} is empty; supply a number or the word 'unknown'")
    try:
        value = Decimal(text)
    except InvalidOperation:
        raise MoneyError(f"{where} is {raw!r}, which is not a number")
    if not value.is_finite():
        raise MoneyError(
            f"{where} is {raw!r}; nan and infinity compare false against every "
            "threshold in this engine and would silently produce a clean verdict"
        )
    return value


def parse_money(raw: object, where: str) -> Decimal:
    """A non-negative amount of money, in dollars and cents."""
    value = _decimal(raw, where)
    if value < 0:
        raise MoneyError(
            f"{where} is {raw!r}; a negative amount here inverts the shortfall and "
            "reports exposure as money owed back to the borrower"
        )
    if value > MAX_MONEY_MAGNITUDE:
        raise MoneyError(
            f"{where} is {raw!r}; amounts cannot exceed "
            f"{MAX_MONEY_MAGNITUDE} AUD"
        )
    exponent = value.as_tuple().exponent
    if isinstance(exponent, int) and exponent < -MAX_MONEY_DECIMAL_PLACES:
        raise MoneyError(
            f"{where} is {raw!r}; amounts cannot have more than "
            f"{MAX_MONEY_DECIMAL_PLACES} decimal places"
        )
    return value


def parse_rate(raw: object, where: str) -> Decimal:
    """An annual interest rate as a decimal fraction: 0.0877, not 8.77."""
    value = _decimal(raw, where)
    if value < 0:
        raise MoneyError(f"{where} is {raw!r}; an interest rate cannot be negative")
    if value > RATE_CEILING:
        raise MoneyError(
            f"{where} is {raw!r}, above {RATE_CEILING} (100% a year). Rates are "
            "decimal fractions in this engine: write 8.77 per cent as 0.0877"
        )
    return value


def parse_ratio(raw: object, where: str) -> Decimal:
    """A non-negative ratio, such as security cover expressed against the loan.

    1.10 is the 110% that s 109N(3)(a)(ii) requires. No ceiling applies: a
    property can be worth many times the loan it secures.
    """
    value = _decimal(raw, where)
    if value < 0:
        raise MoneyError(f"{where} is {raw!r}; a coverage ratio cannot be negative")
    return value


def to_cents(value: Decimal) -> Decimal:
    """Quantise to cents with ROUND_HALF_UP, in a context wide enough that the
    quantisation itself cannot raise InvalidOperation on a large amount."""
    with localcontext() as ctx:
        ctx.prec = FORMULA_PRECISION
        return value.quantize(CENTS, rounding=ROUNDING)


def cents_str(value: Decimal) -> str:
    """The JSON and CSV form of an amount: a quoted decimal string, always two
    places. JSON numbers are IEEE doubles in most parsers, so an amount that
    left this engine as a JSON number would arrive at the reader as a float."""
    return str(to_cents(value))


def rate_str(value: Decimal) -> str:
    """The JSON form of a rate: the table's own decimal string, unrounded."""
    return str(value)
