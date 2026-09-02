"""Parsing operator-supplied facts, including the ones they cannot establish.

Division 7A turns on facts a calculator cannot see: whether an agreement was
in writing, whether it was in place before the lodgment day, whether a
payment is a genuine repayment under s 109R. This engine takes those as
assertions from the operator and never infers them from narrative or from
bank movement.

"unknown" is a value here, not a missing value. It is carried through to the
verdict as UNKNOWN and is never coerced to False, because s 109N(1) is a set
of positive requirements: a limb that has not been established is not the
same as a limb that has failed, and only one of those two is a finding.
"""
from __future__ import annotations

from decimal import Decimal

from .money import MoneyError, parse_money, parse_ratio, parse_rate
from .years import YearError, YearOfIncome, parse_year

#: Spellings of "I do not know" accepted in a CSV cell. A blank cell counts:
#: an operator who leaves a column empty has not asserted the fact.
UNKNOWN_TOKENS = frozenset({"", "unknown", "unk", "n/a", "na", "?", "-"})

_TRUE = frozenset({"true", "t", "yes", "y", "1"})
_FALSE = frozenset({"false", "f", "no", "n", "0"})


class FactError(ValueError):
    """A supplied fact is neither a usable value nor a spelling of unknown."""


def is_unknown(raw: object) -> bool:
    return raw is None or str(raw).strip().lower() in UNKNOWN_TOKENS


def parse_tristate(raw: object, where: str) -> bool | None:
    """true / false / unknown. Returns None for unknown.

    Anything unrecognised is an error rather than a silent False. A typo like
    "ture" that fell through to False would report NOT_COMPLYING on a written
    agreement that exists, which is the wrong answer in the more damaging
    direction.
    """
    if is_unknown(raw):
        return None
    text = str(raw).strip().lower()
    if text in _TRUE:
        return True
    if text in _FALSE:
        return False
    raise FactError(
        f"{where} is {raw!r}; write true, false, or unknown. This engine will not "
        "read an unrecognised value as false"
    )


def optional_money(raw: object, where: str) -> Decimal | None:
    if is_unknown(raw):
        return None
    try:
        return parse_money(raw, where)
    except MoneyError as exc:
        raise FactError(str(exc))


def optional_rate(raw: object, where: str) -> Decimal | None:
    if is_unknown(raw):
        return None
    try:
        return parse_rate(raw, where)
    except MoneyError as exc:
        raise FactError(str(exc))


def optional_ratio(raw: object, where: str) -> Decimal | None:
    if is_unknown(raw):
        return None
    try:
        return parse_ratio(raw, where)
    except MoneyError as exc:
        raise FactError(str(exc))


def optional_years(raw: object, where: str) -> Decimal | None:
    """A term or remaining term, in years. Non-negative, finite, may be
    fractional: s 109E(6) contemplates a difference that is not a whole
    number and rounds it up."""
    if is_unknown(raw):
        return None
    try:
        return parse_ratio(raw, where)
    except MoneyError as exc:
        raise FactError(str(exc))


def optional_year_of_income(raw: object, where: str) -> YearOfIncome | None:
    if is_unknown(raw):
        return None
    try:
        return parse_year(raw, where)
    except YearError as exc:
        raise FactError(str(exc))
