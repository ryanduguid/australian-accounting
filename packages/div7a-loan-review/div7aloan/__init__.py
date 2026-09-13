"""div7a-loan-review: an experimental review aid for ITAA 1936 Division 7A
loan terms and minimum yearly repayments.

Not tax, legal or financial advice, and not a Division 7A determination.
Verify every output against the compiled Act and the ATO's own materials
before acting on it.

The 4 functions below are the stable surface imported by the
aus-accounting-mcp adapter. The modules behind them are not stable.
"""

__version__ = "0.1.2"

#: The date the compiled Act and the benchmark rate table were last read.
LAW_CONTENT_DATE = "2026-08-31"

#: The compilation of ITAA 1936 this engine was written against.
LAW_COMPILATION = "C1936A00027, compilation in force 1 July 2026"

# The version and the law dates sit above these imports on purpose: they are
# the first thing a reader of this module needs, and a reader of the built
# wheel's metadata reaches them without importing the engine.
from .gate import GateFacts, GateResult, complying_loan_gate  # noqa: E402
from .myr import MyrFacts, MyrResult, minimum_yearly_repayment  # noqa: E402
from .rates import RateResult, benchmark_rate  # noqa: E402
from .register import ReviewLine, ReviewReport, review_register  # noqa: E402
from .verdicts import GateVerdict, MyrVerdict, RateVerdict, RowStatus  # noqa: E402
from .years import YearOfIncome, parse_year  # noqa: E402

__all__ = [
    "LAW_COMPILATION",
    "LAW_CONTENT_DATE",
    "GateFacts",
    "GateResult",
    "GateVerdict",
    "MyrFacts",
    "MyrResult",
    "MyrVerdict",
    "RateResult",
    "RateVerdict",
    "ReviewLine",
    "ReviewReport",
    "RowStatus",
    "YearOfIncome",
    "benchmark_rate",
    "complying_loan_gate",
    "minimum_yearly_repayment",
    "parse_year",
    "review_register",
    "__version__",
]
