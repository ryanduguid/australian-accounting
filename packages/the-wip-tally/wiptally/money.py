"""Amount parsing and formatting.

Everything downstream works in Decimal. No amount is ever converted to float, so
a cost-to-cost ratio of 2/7 compares equal to itself rather than to a binary
approximation of 0.285714.
"""

from __future__ import annotations

import re
from decimal import Decimal, InvalidOperation, ROUND_HALF_UP

CENTS = Decimal("0.01")
RATIO_PLACES = Decimal("0.00000001")
PERCENT_POINTS = Decimal("0.01")

_STRIP_RE = re.compile(r"[\s$,]")
_TRAILING_SIGNS = ("CR", "DR")
_PERCENT_RE = re.compile(r"^([+-]?\d*\.?\d+)\s*%$")
_YES = frozenset({"yes", "y", "true", "1"})
_NO = frozenset({"no", "n", "false", "0"})


class AmountError(ValueError):
    """Raised when a cell cannot be read as an amount, ratio, or yes/no flag."""


def parse_amount(raw: str, where: str = "amount") -> Decimal:
    """Parse an accounting amount to an unquantised Decimal.

    Accepts a plain number, thousands separators, a leading currency symbol,
    parentheses for negatives and a trailing CR marker. Rejects NaN, Infinity
    and anything Decimal would otherwise swallow silently.
    """
    if raw is None:
        raise AmountError(f"{where}: no amount given")
    text = str(raw).strip()
    if not text:
        raise AmountError(f"{where}: no amount given")

    negative = False
    suffix = text[-2:].upper()
    if len(text) > 2 and suffix in _TRAILING_SIGNS:
        negative = suffix == "CR"
        text = text[:-2].strip()

    if text.startswith("$"):
        text = text[1:].strip()

    if text.startswith("(") and text.endswith(")"):
        negative = not negative
        text = text[1:-1].strip()

    cleaned = _STRIP_RE.sub("", text)
    if not cleaned:
        raise AmountError(f"{where}: no amount given")
    if not re.fullmatch(r"[+-]?\d*\.?\d+", cleaned):
        raise AmountError(f"{where}: {raw!r} is not an amount")

    try:
        value = Decimal(cleaned)
    except InvalidOperation as exc:
        raise AmountError(f"{where}: {raw!r} is not an amount") from exc
    if not value.is_finite():
        raise AmountError(f"{where}: {raw!r} is not a finite amount")
    return -value if negative else value


def as_money(value: Decimal) -> Decimal:
    """Quantise to cents with ROUND_HALF_UP. Collapse signed zero to 0.00."""
    quantised = value.quantize(CENTS, rounding=ROUND_HALF_UP)
    if quantised == 0:
        return Decimal("0.00")
    return quantised


def parse_money(raw: str, where: str = "amount") -> Decimal:
    """Parse an amount and quantise it to cents.

    A non-zero value that rounds to nothing is refused rather than disappearing
    into a WIP schedule as a silent nil.
    """
    value = parse_amount(raw, where)
    quantised = as_money(value)
    if value != 0 and quantised == 0:
        raise AmountError(
            f"{where}: {raw!r} is smaller than half a cent and would round to nil"
        )
    return quantised


def parse_ratio(raw: str, where: str = "ratio") -> Decimal:
    """Parse a ratio in 0-1 form, or a percentage with a % suffix.

    0.25 and 25% are the same number. A bare 25 is refused: it is ambiguous
    between 25% and a 25x constraint, and either misread wrecks the schedule.
    """
    if raw is None:
        raise AmountError(f"{where}: no ratio given")
    text = str(raw).strip()
    if not text:
        raise AmountError(f"{where}: no ratio given")
    percent = _PERCENT_RE.fullmatch(text)
    if percent:
        value = parse_amount(percent.group(1), where) / Decimal(100)
    else:
        value = parse_amount(text, where)
        if value > 1:
            raise AmountError(
                f"{where}: {raw!r} is greater than 1. Write it as a percentage "
                f"with a % suffix, or as a ratio between 0 and 1."
            )
    if value < 0 or value > 1:
        raise AmountError(f"{where}: {raw!r} is not between 0 and 1")
    return value


def parse_bool(raw: str, where: str = "flag") -> bool:
    if raw is None:
        raise AmountError(f"{where}: no yes/no value given")
    text = str(raw).strip().casefold()
    if text in _YES:
        return True
    if text in _NO:
        return False
    raise AmountError(f"{where}: {raw!r} is not yes or no")


def money(value: Decimal) -> str:
    """Format an amount for display, with thousands separators and two decimals."""
    quantised = as_money(value)
    return f"{quantised:,.2f}"


def percent(value: Decimal) -> str:
    """Format a ratio as a percentage to two decimal places."""
    scaled = (value * 100).quantize(PERCENT_POINTS, rounding=ROUND_HALF_UP)
    return f"{scaled}%"


def as_points(value: Decimal) -> Decimal:
    """Quantise a margin movement to the two places it is displayed at.

    The flag threshold and the printed figure have to be the same number, so
    every margin movement is put through here before either is taken.
    """
    quantised = value.quantize(PERCENT_POINTS, rounding=ROUND_HALF_UP)
    if quantised == 0:
        return Decimal("0.00")
    return quantised


def points(value: Decimal) -> str:
    """Format a margin movement in percentage points."""
    return f"{as_points(value)} points"
