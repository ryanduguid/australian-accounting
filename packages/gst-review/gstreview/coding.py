"""Give each coded bank line a review state, checking GST codes against registration.

The review reads facts the caller supplies: bank lines with their coding and each
entity's GST registration dates. It contacts no service and writes nothing.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import date, datetime
from decimal import Decimal
from typing import Any

from ._version import __version__

SOURCE_CHECKED = "2026-10-19"
# Compilation 96 (C2026C00081), in force from 1 January 2026.
GST_ACT = (
    "https://www.legislation.gov.au/C2004A00446/2026-01-01/2026-01-01/text/original/"
    "epub/OEBPS/document_1/document_1.html"
)

TAXABLE = "taxable"
GST_FREE = "gst_free"
INPUT_TAXED = "input_taxed"
OUTSIDE_GST = "outside_gst"
TREATMENTS = frozenset({TAXABLE, GST_FREE, INPUT_TAXED, OUTSIDE_GST})

# A convenience for common Xero and MYOB AU codes. Callers with other codes, or
# with renamed rates in their own file, pass their own table.
DEFAULT_GST_CODES: dict[str, str] = {
    "GST on Income": TAXABLE,
    "GST on Expenses": TAXABLE,
    "GST on Capital": TAXABLE,
    "GST Free Income": GST_FREE,
    "GST Free Expenses": GST_FREE,
    "GST Free Capital": GST_FREE,
    "GST Free Exports": GST_FREE,
    "Input Taxed": INPUT_TAXED,
    "BAS Excluded": OUTSIDE_GST,
    "GST": TAXABLE,
    "CAP": TAXABLE,
    "FRE": GST_FREE,
    "EXP": GST_FREE,
    "INP": INPUT_TAXED,
    "N-T": OUTSIDE_GST,
}

STATES = (
    "unknown_gst_code",
    "gst_differs_from_registration",
    "between_entities",
    "coded",
    "suggested",
    "nothing_suggested",
)

NOT_DECIDED = [
    "Whether an entity was required to be registered (GST Act s 23-5). The review uses "
    "only the registration dates supplied.",
    "Whether a supply is taxable, GST-free or input taxed on its merits. The review checks "
    "GST codes against registration, not against the supply.",
    "Whether a matched pair between entities is a loan, capital, a distribution or payment "
    "for a supply between them.",
    "Whether a suggested coding is right.",
    "Direction is read from the sign: a negative amount is a payment. Refunds, reversals "
    "and creditable importations (GST Act Division 15) are not distinguished.",
]

MAX_MATCH_WINDOW_DAYS = 31
_MONEY_LIMIT = Decimal("1e12")


@dataclass(frozen=True)
class Registration:
    """An entity's GST registration: first and last registered day, inclusive.

    ``registered_from=None`` means not registered at any time in the review.
    ``registered_to=None`` means still registered.
    """

    registered_from: date | None
    registered_to: date | None = None

    def covers(self, day: date) -> bool:
        if self.registered_from is None or day < self.registered_from:
            return False
        return self.registered_to is None or day <= self.registered_to


@dataclass(frozen=True)
class BankLine:
    """One bank line. A negative amount is money out, a positive amount money in.

    ``account`` and ``gst_code`` are the coding on the line; ``confirmed`` says a
    person has accepted it. ``suggested_*`` hold an unaccepted proposal.
    ``supplier_registered`` is the caller's fact about the other party, if known.
    """

    id: str
    entity: str
    date: date
    amount: Decimal
    description: str = ""
    account: str | None = None
    gst_code: str | None = None
    confirmed: bool = False
    suggested_account: str | None = None
    suggested_gst_code: str | None = None
    supplier_registered: bool | None = None


def review_coding(
    lines: list[BankLine],
    registrations: dict[str, Registration],
    *,
    match_window_days: int = 3,
    gst_codes: dict[str, str] | None = None,
) -> dict[str, Any]:
    """Return a review state and reason for every line, in input order."""
    codes = DEFAULT_GST_CODES if gst_codes is None else gst_codes
    _validate(lines, registrations, match_window_days, codes)
    counterparts = _match_between_entities(lines, match_window_days)

    reviewed = []
    for line in lines:
        state, reason, cited = _state(line, registrations[line.entity], codes,
                                      counterparts.get(line.id))
        reviewed.append({
            "id": line.id,
            "entity": line.entity,
            "date": line.date.isoformat(),
            "amount": str(line.amount),
            "state": state,
            "reason": reason,
            "counterpart": counterparts.get(line.id),
            "sections": cited,
        })

    return {
        "ok": True,
        "engine": "gst-review",
        "engine_version": __version__,
        "review": "gst_coding",
        "lines": reviewed,
        "counts": {state: sum(1 for row in reviewed if row["state"] == state)
                   for state in STATES},
        "facts_used": _facts_used(lines, registrations, match_window_days, codes),
        "not_decided": list(NOT_DECIDED),
        "sources": [GST_ACT],
        "source_checked": SOURCE_CHECKED,
        "warnings": [
            "Registration dates, coding and supplier status are the caller's facts and are "
            "not independently verified.",
            "Review aid only, not advice, an assessment or a lodgment. Obtain human review.",
        ],
    }


def _facts_used(lines: list[BankLine], registrations: dict[str, Registration],
                window: int, codes: dict[str, str]) -> dict[str, Any]:
    entities = sorted({line.entity for line in lines})
    return {
        "registrations": {
            entity: {"registered_from": _iso(registrations[entity].registered_from),
                     "registered_to": _iso(registrations[entity].registered_to)}
            for entity in entities
        },
        "match_window_days": window,
        "gst_codes": dict(codes),
    }


def _state(line: BankLine, registration: Registration, codes: dict[str, str],
           counterpart: str | None) -> tuple[str, str, list[str]]:
    code = line.gst_code if line.gst_code is not None else line.suggested_gst_code
    if code is not None and code not in codes:
        return ("unknown_gst_code",
                f"GST code {code!r} is not in the code table, so its treatment is unknown.", [])

    if code is not None and codes[code] == TAXABLE:
        if not registration.covers(line.date):
            return ("gst_differs_from_registration",
                    f"Coded {code!r}, which charges or claims GST, but {line.entity} is not "
                    f"registered for GST on {line.date.isoformat()}. A taxable supply and a "
                    "creditable acquisition each need the entity registered or required to "
                    "be registered.",
                    ["GST Act s 9-5(d)", "GST Act s 11-5(d)"])
        if line.amount < 0 and line.supplier_registered is False:
            return ("gst_differs_from_registration",
                    f"Coded {code!r}, which claims GST, but the supplier is recorded as not "
                    "registered. A supply by an entity that is neither registered nor required "
                    "to be registered is not a taxable supply, so no input tax credit arises.",
                    ["GST Act s 9-5(d)", "GST Act s 11-5(b)"])

    if counterpart is not None:
        return ("between_entities",
                f"Opposite amount on line {counterpart!r} of another entity within the match "
                "window: likely a transfer between entities, not income or expense.", [])

    if line.confirmed:
        return ("coded", "Coding confirmed.", [])
    if line.account or line.gst_code or line.suggested_account or line.suggested_gst_code:
        return ("suggested", "Coding proposed but not confirmed.", [])
    return ("nothing_suggested", "No account or GST code proposed.", [])


def _match_between_entities(lines: list[BankLine], window: int) -> dict[str, str]:
    """Pair lines one to one: opposite amounts, different entities, nearest date first."""
    by_amount: dict[Decimal, list[BankLine]] = {}
    for line in lines:
        by_amount.setdefault(line.amount, []).append(line)

    matched: dict[str, str] = {}
    for line in lines:
        if line.id in matched or line.amount == 0:
            continue
        candidates = [
            other for other in by_amount.get(-line.amount, [])
            if other.id not in matched and other.entity != line.entity
            and abs((other.date - line.date).days) <= window
        ]
        if candidates:
            # min keeps the first in input order among equal distances.
            other = min(candidates, key=lambda other: abs((other.date - line.date).days))
            matched[line.id] = other.id
            matched[other.id] = line.id
    return matched


def _validate(lines: list[BankLine], registrations: dict[str, Registration],
              window: int, codes: dict[str, str]) -> None:
    # bool is an int subclass, so True would otherwise pass as a 1-day window.
    if (isinstance(window, bool) or not isinstance(window, int)
            or not 0 <= window <= MAX_MATCH_WINDOW_DAYS):
        raise ValueError(f"match_window_days must be an integer from 0 to "
                         f"{MAX_MATCH_WINDOW_DAYS}.")
    unknown = {treatment for treatment in codes.values() if treatment not in TREATMENTS}
    if unknown:
        raise ValueError(f"Unknown GST treatments in the code table: {sorted(unknown)}.")
    for entity, registration in registrations.items():
        start, end = registration.registered_from, registration.registered_to
        if not _is_date(start, optional=True) or not _is_date(end, optional=True):
            raise ValueError(f"Registration dates for {entity!r} must be dates or None.")
        if end is not None and (start is None or end < start):
            raise ValueError(f"Registration for {entity!r} ends before it starts.")

    seen: set[str] = set()
    for line in lines:
        if not isinstance(line.id, str) or not line.id or line.id in seen:
            raise ValueError(f"Line ids must be unique non-empty strings: {line.id!r}.")
        seen.add(line.id)
        if line.entity not in registrations:
            raise ValueError(f"No registration facts for entity {line.entity!r}.")
        if not _is_date(line.date, optional=False):
            raise ValueError(f"Line {line.id!r} needs a date.")
        if (not isinstance(line.amount, Decimal) or not line.amount.is_finite()
                or abs(line.amount) > _MONEY_LIMIT
                or int(line.amount.as_tuple().exponent) < -2):
            raise ValueError(f"Line {line.id!r} amount must be a finite Decimal, at most "
                             "2 decimal places and 1e12.")
        if not isinstance(line.confirmed, bool) or not isinstance(
                line.supplier_registered, (bool, type(None))):
            raise ValueError(f"Line {line.id!r} flags must be booleans.")
        if line.confirmed and not (line.account and line.gst_code):
            raise ValueError(f"Line {line.id!r} is confirmed without an account and GST code.")


def _is_date(value: object, *, optional: bool) -> bool:
    if value is None:
        return optional
    return isinstance(value, date) and not isinstance(value, datetime)


def _iso(value: date | None) -> str | None:
    return None if value is None else value.isoformat()
