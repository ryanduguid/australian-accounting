"""Input checks shared by the reviews."""

from __future__ import annotations

from datetime import date, datetime
from decimal import Context, Decimal

MONEY_LIMIT = Decimal("1e12")
# The reviews add, negate and compare in this context, never the caller's: a caller
# that lowered precision would otherwise round amounts before they are totalled. An
# amount within the limit has at most 15 significant digits, so 40 leaves room for a
# total over any number of lines a caller can supply.
EXACT = Context(prec=40)


def is_money(value: object) -> bool:
    """A finite Decimal of at most 2 decimal places and 1e12 either side of zero."""
    return (isinstance(value, Decimal) and value.is_finite() and abs(value) <= MONEY_LIMIT
            and int(value.as_tuple().exponent) >= -2)


def is_date(value: object, *, optional: bool) -> bool:
    if value is None:
        return optional
    return isinstance(value, date) and not isinstance(value, datetime)
