"""Shared MCP money parsing. No statutory arithmetic lives here."""

from __future__ import annotations

from decimal import Decimal, InvalidOperation

from .errors import InputError

MAX_MONEY_MAGNITUDE = Decimal("1000000000000.00")
MAX_MONEY_DECIMAL_PLACES = 2


def parse_amount(value: str, field: str) -> Decimal:
    """Parse a required decimal-string amount with Codex #1 domain bounds."""
    text = str(value).strip()
    if not text:
        raise InputError(f"{field} is required")
    try:
        amount = Decimal(text)
    except InvalidOperation as exc:
        raise InputError(f"{field}: {value!r} is not a decimal amount") from exc
    if not amount.is_finite():
        raise InputError(f"{field}: {value!r} is not a finite amount")
    if amount.copy_abs() > MAX_MONEY_MAGNITUDE:
        raise InputError(f"{field} absolute value must not exceed AUD {MAX_MONEY_MAGNITUDE}")
    exponent = amount.as_tuple().exponent
    if isinstance(exponent, int) and exponent < -MAX_MONEY_DECIMAL_PLACES:
        raise InputError(
            f"{field} must have no more than {MAX_MONEY_DECIMAL_PLACES} decimal places"
        )
    return amount


def parse_optional_amount(value: str | None, field: str) -> Decimal | None:
    if value is None:
        return None
    text = str(value).strip()
    if text == "":
        return None
    return parse_amount(text, field)
