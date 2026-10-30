"""Write a ``due_dates`` result as an iCalendar file (RFC 5545).

Each DETERMINED row becomes one all-day event. An UNKNOWN row has no date to put in
a calendar, so it is returned apart and never given its nominal date.
"""

from __future__ import annotations

import hashlib
import json
from datetime import date, timedelta
from typing import Any

_PRODID = "-//Ryan Duguid//lodgment-calendar//EN"
_MAX_OCTETS = 75


def to_ics(result: dict[str, Any], *, uid_scope: str = "") -> dict[str, Any]:
    """Return ``{"ics": text, "unknown": rows}`` for a result from ``due_dates``.

    ``ics`` is None when no row is DETERMINED, because a calendar must hold an event.
    An event's UID follows the request, period and kind, so it stays the same when the
    date moves. Whether a calendar program then updates the event on a later import is
    up to that program. ``uid_scope`` keeps 2 clients on the same terms from sharing
    UIDs; pass a client code. Only its hash is written.
    """
    if not isinstance(uid_scope, str):
        raise ValueError(f"uid_scope must be text; got {uid_scope!r}")
    rows = result["obligations"]
    unknown = [row for row in rows if row["status"] == "UNKNOWN"]
    events = [_event(row, result, uid_scope) for row in rows if row["status"] == "DETERMINED"]
    if not events:
        return {"ics": None, "unknown": unknown}
    lines = ["BEGIN:VCALENDAR", "VERSION:2.0", f"PRODID:{_PRODID}", "CALSCALE:GREGORIAN"]
    if unknown:
        names = "; ".join(f"{row['obligation']} ({row['kind']})" for row in unknown)
        lines.append("DESCRIPTION:" + _text(f"UNKNOWN and not in this calendar: {names}"))
    for event in events:
        lines.extend(event)
    lines.append("END:VCALENDAR")
    return {"ics": "".join(_fold(line) + "\r\n" for line in lines), "unknown": unknown}


def _event(row: dict[str, Any], result: dict[str, Any], uid_scope: str) -> list[str]:
    due = date.fromisoformat(row["due_date"])
    identity = json.dumps([uid_scope, result["request"], row["period"], row["kind"]],
                          sort_keys=True)
    details = [row["summary"], f"Rule: {row['rule_id']}"]
    if row["nominal_due_date"] != row["due_date"]:
        details.append(f"Rule's date before the next-business-day roll: {row['nominal_due_date']}")
    details += [f"Source: {row['source_provision']}, {row['source_url']}",
                f"Source checked: {row['source_checked']}",
                *result["warnings"][:1]]
    title = f"{row['obligation']}: {row['kind']}"
    return [
        "BEGIN:VEVENT",
        f"UID:{hashlib.sha256(identity.encode()).hexdigest()}@lodgment-calendar",
        # A fixed stamp keeps the same result byte-identical from one export to the next.
        # shortcut: no SEQUENCE or revision history, so a calendar program may keep a stale
        # or duplicate event when a date moves. Upgrade: record each UID's revisions.
        f"DTSTAMP:{row['source_checked'].replace('-', '')}T000000Z",
        f"DTSTART;VALUE=DATE:{due:%Y%m%d}",
        f"DTEND;VALUE=DATE:{due + timedelta(days=1):%Y%m%d}",
        "SUMMARY:" + _text(title),
        "DESCRIPTION:" + _text("\n".join(details)),
        "END:VEVENT",
    ]


def _text(value: str) -> str:
    """Escape a TEXT value (RFC 5545 section 3.3.11)."""
    return (value.replace("\\", "\\\\").replace(";", "\\;").replace(",", "\\,")
            .replace("\r\n", "\n").replace("\r", "\n").replace("\n", "\\n"))


def _fold(line: str) -> str:
    """Fold at 75 octets without splitting a character (RFC 5545 section 3.1)."""
    parts: list[str] = []
    current, size = "", 0
    for char in line:
        width = len(char.encode())
        if size + width > _MAX_OCTETS:
            parts.append(current)
            current, size = " ", 1
        current += char
        size += width
    parts.append(current)
    return "\r\n".join(parts)
