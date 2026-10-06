"""Shared MCP money parsing. No statutory arithmetic lives here."""

from __future__ import annotations

import re
from decimal import Decimal, InvalidOperation

from pydantic.json_schema import JsonDict

from .errors import InputError

MAX_MONEY_MAGNITUDE = Decimal("1000000000000.00")
MAX_MONEY_DECIMAL_PLACES = 2
# Plain notation: ASCII digits, an optional leading minus and at most two decimal
# places, with surrounding whitespace ignored. Decimal() alone also reads
# exponents, underscores, a leading plus, ".5" and "12.", which a money argument
# should never use. Every money parameter publishes this pattern in its schema.
MONEY_PATTERN = r"^\s*-?[0-9]+(?:\.[0-9]{1,2})?\s*$"
MONEY_SCHEMA: JsonDict = {"pattern": MONEY_PATTERN}
# More decimal places pass this check so the decimal-place message below explains them.
_PLAIN_DECIMAL = re.compile(r"-?[0-9]+(?:\.[0-9]+)?")


def parse_amount(value: str, field: str) -> Decimal:
    """Parse a required plain decimal-string amount with Codex #1 domain bounds."""
    text = str(value).strip()
    if not text:
        raise InputError(f"{field} is required")
    try:
        amount = Decimal(text)
    except InvalidOperation as exc:
        raise InputError(f"{field}: {value!r} is not a decimal amount") from exc
    if not amount.is_finite():
        raise InputError(f"{field}: {value!r} is not a finite amount")
    if _PLAIN_DECIMAL.fullmatch(text) is None:
        raise InputError(f"{field}: {value!r} is not a decimal amount")
    if amount.copy_abs() > MAX_MONEY_MAGNITUDE:
        raise InputError(f"{field} absolute value must not exceed AUD {MAX_MONEY_MAGNITUDE}")
    exponent = amount.as_tuple().exponent
    if isinstance(exponent, int) and exponent < -MAX_MONEY_DECIMAL_PLACES:
        raise InputError(
            f"{field} must have no more than {MAX_MONEY_DECIMAL_PLACES} decimal places"
        )
    return amount


def parse_optional_amount(value: str | None, field: str) -> Decimal | None:
    """Parse an optional amount; an unknown amount is omitted or null, never blank."""
    if value is None:
        return None
    if not str(value).strip():
        raise InputError(f"{field}: omit an unknown amount or send null, not a blank string")
    return parse_amount(value, field)
