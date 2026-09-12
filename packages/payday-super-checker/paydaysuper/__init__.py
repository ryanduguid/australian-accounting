"""payday-super-checker: payday-super deadline and SG-charge exposure checker.

Educational tool. Not legal, tax or financial advice. Verify outcomes
against the ATO's own materials before acting.
"""

__version__ = "0.1.3"

LAW_CONTENT_DATE = "2026-08-15"

# The version and the law dates sit above these imports on purpose: they are
# the first thing a reader of this module needs, and a reader of the built
# wheel's metadata reaches them without importing the engine.
from .report import Result, assess  # noqa: E402

__all__ = ["LAW_CONTENT_DATE", "Result", "assess", "__version__"]
