"""Guard values written to CSV against spreadsheet formula injection.

Contract identifiers and customer names come from whatever the ledger holds.
The schedule this tool writes is meant to be opened in Excel.
"""

from __future__ import annotations

import re

ALWAYS = ("=", "@")
CONDITIONAL = ("+", "-")

_NUMBER_RE = re.compile(r"^[\d,]*\.?\d+$")


def guard(value: object) -> str:
    text = "" if value is None else str(value)
    if not text:
        return text
    first = text[0]
    if first in ALWAYS:
        return "'" + text
    if first in CONDITIONAL and not _NUMBER_RE.match(text[1:]):
        return "'" + text
    if first in "\t\r\n":
        return "'" + text
    return text
