"""Amount parsing and formatting.

Everything downstream works in Decimal. No amount is ever converted to float, so a
ratio of 0.31 compares equal to the ATO's published 0.31 rather than to a binary
approximation of it.
"""

from __future__ import annotations

import re
from decimal import ROUND_HALF_UP, Decimal, InvalidOperation

CENTS = Decimal("0.01")
PERCENT_PLACES = Decimal("0.01")

# The accounting grammar, shared with the wiptally engine. Thousands separators
# are only accepted in groups of 3: stripping every comma first and parsing
# what is left reads "1,2,3" as 123, which is a typed cell silently turned into a
# number nobody entered.
_NUMBER = r"(?:[0-9]+|[0-9]{1,3}(?:,[0-9]{3})+)(?:\.[0-9]+)?|\.[0-9]+"
_ACCOUNTING_NUMBER = re.compile(
    rf"(?:[+-]?\$?\s*(?:{_NUMBER})|\$[+-]\s*(?:{_NUMBER})|"
    rf"\$?\s*\(\s*(?:{_NUMBER})\s*\)|\(\s*\$?(?:{_NUMBER})\s*\)|"
    rf"\$?\s*(?:{_NUMBER})\s*(?:CR|DR))", re.IGNORECASE,
)


class AmountError(ValueError):
    """Raised when a cell cannot be read as an amount."""


def parse_amount(raw: str, where: str = "amount") -> Decimal:
    """Parse an accounting amount.

    Accepts a plain number, thousands separators in groups of 3, a leading
    currency symbol, parentheses for negatives and a trailing CR or DR marker.
    Rejects anything else, including the strings Decimal itself would happily
    accept such as "NaN" and "Infinity", which parse cleanly and then explode on
    the first comparison, and a cell carrying both a parenthesis and a CR marker,
    where the 2 signs disagree about which way the amount runs.
    """
    if raw is None:
        raise AmountError(f"{where}: no amount given")
    text = str(raw).strip()
    if not text:
        raise AmountError(f"{where}: no amount given")

    if not _ACCOUNTING_NUMBER.fullmatch(text):
        raise AmountError(f"{where}: {raw!r} is not an amount")
    negative = "(" in text or text.upper().endswith("CR")
    cleaned = re.sub(r"[\s$,()]|CR$|DR$", "", text, flags=re.IGNORECASE)

    try:
        value = Decimal(cleaned)
    except InvalidOperation as exc:
        raise AmountError(f"{where}: {raw!r} is not an amount") from exc
    if not value.is_finite():
        raise AmountError(f"{where}: {raw!r} is not a finite amount")
    return -value if negative else value


def _quantise(value: Decimal, places: Decimal) -> Decimal:
    """Round to fixed places, refusing a value the decimal context cannot hold.

    quantize raises InvalidOperation once the result would need more digits than the
    context allows. That is an ArithmeticError, so the CLI's error handler does not
    see it and left alone it ends a run with a traceback after the comparison has
    already been produced. AmountError is a ValueError the handler already catches.
    """
    try:
        quantised = value.quantize(places, rounding=ROUND_HALF_UP)
        return abs(quantised) if quantised == 0 else quantised
    except InvalidOperation as exc:
        raise AmountError(f"{value} has more digits than this tool can report") from exc


def money(value: Decimal) -> str:
    """Format an amount for display, with thousands separators and two decimals."""
    quantised = _quantise(value, CENTS)
    return f"{quantised:,.2f}"


def percent(value: Decimal) -> str:
    """Format a ratio as a percentage of turnover.

    Two decimal places, because the ATO publishes benchmarks to whole percentages and
    a computed 30.96% must not be displayed as 31% next to a verdict of "below the
    31% to 38% range".
    """
    scaled = _quantise(value * 100, PERCENT_PLACES)
    return f"{scaled}%"


def percent_compact(value: Decimal) -> str:
    """Format a published benchmark bound without padding zeros: 29% rather than 29.00%."""
    scaled = _quantise(value * 100, PERCENT_PLACES)
    text = f"{scaled:.2f}".rstrip("0").rstrip(".")
    return f"{text or '0'}%"


def percent_range(low: Decimal, high: Decimal) -> str:
    return f"{percent_compact(low)} to {percent_compact(high)}"
