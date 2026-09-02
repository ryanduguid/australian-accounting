"""The year of income, written the way the Act's readers write it: 2026-27.

A private company's year of income under Division 7A runs 1 July to 30 June
unless the company has a substituted accounting period. This engine assumes
the ordinary 1 July start, because s 109N(2) reads the benchmark rate off the
last RBA publication "before the start of the year of income" and a
substituted period moves that date. A company with a substituted accounting
period is outside v1: see the refusals in the README.
"""
from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import date
from functools import total_ordering

_LABEL = re.compile(r"^(\d{4})-(\d{2})$")

# Typo guards, not statutory limits. Division 7A applies to loans made on or
# after 4 December 1997 (s 109D(5)), and the frozen rate table refuses every
# year it has not reviewed, so the real gate is in rates.py.
_EARLIEST = 1900
_LATEST = 2100


class YearError(ValueError):
    """A year of income label is not in the form 2026-27."""


@total_ordering
@dataclass(frozen=True)
class YearOfIncome:
    """A year of income identified by the calendar year it starts in."""

    start_year: int

    def __post_init__(self) -> None:
        if not isinstance(self.start_year, int) or isinstance(self.start_year, bool):
            raise YearError(f"year of income start {self.start_year!r} must be a whole year")
        if not _EARLIEST <= self.start_year <= _LATEST:
            raise YearError(
                f"year of income {self.start_year}-.. is outside {_EARLIEST}-{_LATEST}; "
                "this looks like a typo rather than a year"
            )

    @property
    def label(self) -> str:
        return f"{self.start_year}-{(self.start_year + 1) % 100:02d}"

    @property
    def starts_on(self) -> date:
        """1 July. s 109N(2) takes the rate last published before this date."""
        return date(self.start_year, 7, 1)

    def __str__(self) -> str:
        return self.label

    def __lt__(self, other: object) -> bool:
        if not isinstance(other, YearOfIncome):
            return NotImplemented
        return self.start_year < other.start_year


def parse_year(raw: object, where: str = "year of income") -> YearOfIncome:
    """Parse '2026-27'.

    The second half is checked against the first rather than ignored. '2026-28'
    and '2026-26' are the two ways a hand-typed year goes wrong, and either one
    silently reads the wrong benchmark rate off the table if only the first
    half is used.
    """
    text = str(raw).strip()
    match = _LABEL.match(text)
    if match is None:
        raise YearError(f"{where} {raw!r} must be written like 2026-27")
    start = int(match.group(1))
    if f"{(start + 1) % 100:02d}" != match.group(2):
        raise YearError(
            f"{where} {raw!r} is not a year of income: {start} is followed by "
            f"{(start + 1) % 100:02d}, not {match.group(2)}"
        )
    return YearOfIncome(start)
