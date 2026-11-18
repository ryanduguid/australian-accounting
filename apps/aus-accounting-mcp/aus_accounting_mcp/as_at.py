# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""A corpus title's standing against one date, from a row's own fields.

The corpus builder's `as_at` stage classifies its manifest in Rust, and this server reads
sections.jsonl directly, so the same 4 statuses are worked out here. docs/scope.md in
PaulGoodman is the contract: its status names, field names, next steps and
limits are repeated exactly. A status is about a title's compilation window, never about
the law on that date. A corpus can hold earlier compilations of a title under
compilations/; corpus.py picks the one whose window covers the date, and each is classified
here like any other.
"""

from __future__ import annotations

from datetime import date
from typing import Any

from .errors import InputError

CAPTURED = "captured"
BEFORE_COMPILATION = "before_compilation"
SUPERSEDED = "superseded"
AFTER_RETRIEVAL = "after_retrieval"
STATUSES = (CAPTURED, BEFORE_COMPILATION, SUPERSEDED, AFTER_RETRIEVAL)
NEXT_STEPS = {
    BEFORE_COMPILATION: "read the Register's earlier compilations",
    SUPERSEDED: "a later version started; check the Register and application provisions",
    AFTER_RETRIEVAL: "run check_current and read the Register",
}
COVERS = (
    "captured means the build holds the compilation for that date; it is not a finding "
    "that the text was the law then."
)
AS_AT_NOTICE = (
    "as_at reads each title's compilation window, not the law on that date. " + COVERS + " "
    "A compilation does not show application, saving or transitional provisions elsewhere, "
    "modification by another law, or an amendment with retrospective effect registered "
    "after the build was retrieved, so a date inside a window can still carry text amended "
    "without a new compilation. Read the Register's version history and endnotes for any "
    "question about a past date."
)
REFUSED_NOTICE = (
    "refusals names each title withheld for this date, with its status, window and "
    "next_step. Empty matches alone do not show whether rows were withheld: check "
    "matching_rows and partial_rows in each refusal. A refused title can have zero "
    "matching rows. A refusal is the answer for that date: the corpus holds no text of "
    "that title for it. A title the corpus holds in more than one compilation lists each "
    "window in held_compilations, and its as_at and row counts are those of its latest "
    "compilation, or for a read, of the latest one holding the row read."
)
UNCLASSIFIABLE = (
    "as_at needs each row's compilation_date, retrieved and version_is_current, and "
    "superseded_from when the version is not current, so this corpus cannot answer for a "
    "date until PaulGoodman rebuilds it: tell the user, and do not retry."
)


def is_date(value: object) -> bool:
    """Whether value is a canonical YYYY-MM-DD date; other forms Python parses are not."""
    if not isinstance(value, str):
        return False
    try:
        return date.fromisoformat(value).isoformat() == value
    except ValueError:
        return False


def checked_date(value: str) -> str:
    """value, refused unless it is a real date written YYYY-MM-DD."""
    if not is_date(value):
        raise InputError(f"as_at must be a YYYY-MM-DD date; got {value!r}")
    return value


def _broken(row: dict[str, Any], problem: str) -> InputError:
    return InputError(f"{row.get('register_id')!s:.32}: {problem}. {UNCLASSIFIABLE}")


def _date(row: dict[str, Any], field: str) -> str:
    value = row.get(field)
    if not is_date(value):
        raise _broken(row, f"{field} is not a YYYY-MM-DD date")
    return str(value)


def _superseded(row: dict[str, Any], start: str, retrieved: str) -> str | None:
    """The start of the later version, or None for a current version.

    A row always records version_is_current. Unlike the builder's manifest, which omits it
    for a current title, a row without it cannot be taken as current, because `captured`
    would then rest on a fact the corpus does not hold.
    """
    current = row.get("version_is_current")
    if not isinstance(current, bool):
        raise _broken(row, "version_is_current must be true or false")
    if current:
        if row.get("superseded_from") is not None:
            raise _broken(row, "superseded_from must be null when version_is_current is true")
        return None
    superseded = _date(row, "superseded_from")
    if not start < superseded <= retrieved:
        raise _broken(
            row, "superseded_from must fall after compilation_date and on or before retrieved"
        )
    return superseded


def _status(on: str, start: str, superseded: str | None, retrieved: str) -> str:
    if on < start:
        return BEFORE_COMPILATION
    if superseded is not None:
        return SUPERSEDED if on >= superseded else CAPTURED
    return AFTER_RETRIEVAL if on > retrieved else CAPTURED


def standing(row: dict[str, Any], on: str) -> dict[str, Any]:
    """The row's status against on, its window, what the status covers and the next step.

    Canonical dates order as text. A row whose dates cannot all be true is refused rather
    than reported as held, as the builder refuses such a manifest entry.
    """
    start = _date(row, "compilation_date")
    retrieved = _date(row, "retrieved")
    if start > retrieved:
        raise _broken(row, "compilation_date is after retrieved")
    superseded = _superseded(row, start, retrieved)
    status = _status(on, start, superseded, retrieved)
    return {
        "date": on,
        "status": status,
        "compilation_date": start,
        "superseded_from": superseded,
        "retrieved": retrieved,
        "covers": COVERS,
        "next_step": NEXT_STEPS.get(status),
    }
