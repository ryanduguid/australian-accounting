"""Lodgment and payment due dates under the ATO's 2026–27 lodgment program.

A preparation aid, not a determination: it lodges nothing and contacts no service.
"""

from __future__ import annotations

from ._version import __version__
from .due_dates import Facts, due_dates

__all__ = ["Facts", "__version__", "due_dates"]
