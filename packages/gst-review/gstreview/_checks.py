"""Input checks shared by the reviews."""

from __future__ import annotations

from datetime import date, datetime
from decimal import Decimal

MONEY_LIMIT = Decimal("1e12")


def is_money(value: object) -> bool:
    """A finite Decimal of at most 2 decimal places and 1e12 either side of zero."""
    return (isinstance(value, Decimal) and value.is_finite() and abs(value) <= MONEY_LIMIT
            and int(value.as_tuple().exponent) >= -2)


def is_date(value: object, *, optional: bool) -> bool:
    if value is None:
        return optional
    return isinstance(value, date) and not isinstance(value, datetime)
