"""Runtime checks for the facts supplied to the trust calculations."""

from decimal import Decimal

# The default decimal context's largest exponent, fixed here so that a caller's
# context cannot move it. A larger amount can expand enormously in fixed point:
# 1E+999999999 alone would print about a billion digits.
MAX_SUPPORTED_ADJUSTED_EXPONENT = 999_999


def validate_boolean_facts(**facts: object) -> None:
    """Leave unknown facts for each calculation to handle; refuse other types."""
    for name, value in facts.items():
        if value is not None and type(value) is not bool:
            raise ValueError(f"{name} must be True, False or None")


def validate_printable_amount(name: str, value: Decimal) -> None:
    """Refuse a nonzero amount too large to write out in fixed point."""
    if value and value.adjusted() > MAX_SUPPORTED_ADJUSTED_EXPONENT:
        raise ValueError(
            f"{name} {value} is outside the supported range; amounts must be below "
            f"1E+{MAX_SUPPORTED_ADJUSTED_EXPONENT + 1}"
        )
