"""Exact decimal handling on the wire, in both directions.

Reading is the easy half: a decimal string becomes a `Decimal` with
`Decimal(text)`, which is exact, and anything that is not a decimal string is
refused rather than coerced.

Writing is the half that needs care. The LodgeiT request schemas declare their
money fields as JSON `number`, not as strings (see `contracts/README.md`), so a
request has to carry a JSON number. `json.dumps` reaches that number through
`float`, and `float("0.1")` is not `0.1`. This module serialises a `Decimal`
straight to its own digits instead, so `Decimal("55000.10")` goes onto the wire
as `55000.10` and never as `55000.099999999998`.

`tests/test_wire_precision.py` checks the emitted bytes, not the round trip
through `json.loads`, because a round trip through Python floats would hide the
defect this module exists to prevent.
"""

from __future__ import annotations

from decimal import Decimal, InvalidOperation
from typing import Any

MAX_SCALE = 10
MAX_MAGNITUDE = Decimal("1e15")


class DecimalWireError(ValueError):
    """A value cannot be represented exactly on the wire, or was not a decimal."""


def parse_decimal(value: object, field: str) -> Decimal:
    """A decimal string to a Decimal, exactly. A float is refused.

    A JSON float has already lost whatever the provider meant by the time it
    reaches Python, so accepting one here would be accepting a number this
    package cannot vouch for.
    """
    if isinstance(value, bool) or isinstance(value, float):
        raise DecimalWireError(
            f"{field}: {value!r} arrived as a JSON number. This adapter reads money and "
            "rates as decimal strings, because a float has already lost precision."
        )
    if isinstance(value, int):
        return Decimal(value)
    if not isinstance(value, str) or not value.strip():
        raise DecimalWireError(f"{field}: {value!r} is not a decimal string")
    try:
        parsed = Decimal(value.strip())
    except InvalidOperation as exc:
        raise DecimalWireError(f"{field}: {value!r} is not a decimal") from exc
    if not parsed.is_finite():
        raise DecimalWireError(f"{field}: {value!r} is not finite")
    return parsed


def check_writable(value: Decimal, field: str) -> Decimal:
    """Refuse a Decimal that cannot go on the wire as exact digits."""
    if not value.is_finite():
        raise DecimalWireError(f"{field}: {value} is not finite")
    if value.copy_abs() >= MAX_MAGNITUDE:
        raise DecimalWireError(f"{field}: {value} exceeds the supported magnitude")
    exponent = value.as_tuple().exponent
    if isinstance(exponent, int) and exponent < -MAX_SCALE:
        raise DecimalWireError(f"{field}: {value} has more than {MAX_SCALE} decimal places")
    return value


def format_number(value: Decimal) -> str:
    """A Decimal as JSON number text, exactly, with no exponent.

    `str(Decimal("1E+3"))` is `1E+3`, which is legal JSON but reads badly in
    evidence, so the value is normalised to plain digits first.
    """
    text = format(value, "f")
    if text in ("-0", "-0.0"):
        return "0"
    return text


def _escape(text: str) -> str:
    out = ['"']
    for character in text:
        if character == '"':
            out.append('\\"')
        elif character == "\\":
            out.append("\\\\")
        elif character == "\n":
            out.append("\\n")
        elif character == "\r":
            out.append("\\r")
        elif character == "\t":
            out.append("\\t")
        elif ord(character) < 0x20:
            out.append(f"\\u{ord(character):04x}")
        else:
            out.append(character)
    out.append('"')
    return "".join(out)


def dumps(value: Any, *, field: str = "body") -> str:
    """Serialise to JSON, emitting a Decimal as its own exact digits.

    Small by design: request bodies here are flat mappings of strings, numbers,
    booleans and lists. Anything else is refused rather than guessed at, which
    is what keeps the output exact.
    """
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, Decimal):
        return format_number(check_writable(value, field))
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        raise DecimalWireError(
            f"{field}: a float reached the serialiser. Use Decimal so the digits on the "
            "wire are the digits you meant."
        )
    if isinstance(value, str):
        return _escape(value)
    if isinstance(value, (list, tuple)):
        items = [dumps(item, field=f"{field}[{i}]") for i, item in enumerate(value)]
        return "[" + ",".join(items) + "]"

    if isinstance(value, dict):
        parts = []
        for key, item in value.items():
            if not isinstance(key, str):
                raise DecimalWireError(f"{field}: object keys must be strings, got {key!r}")
            parts.append(f"{_escape(key)}:{dumps(item, field=f'{field}.{key}')}")
        return "{" + ",".join(parts) + "}"
    raise DecimalWireError(f"{field}: {type(value).__name__} cannot be serialised exactly")
