"""div7a-loan-review: an experimental review aid for ITAA 1936 Division 7A
loan terms and minimum yearly repayments.

Not tax, legal or financial advice, and not a Division 7A determination.
Verify every output against the compiled Act and the ATO's own materials
before acting on it.

The four functions below are the surface the aus-accounting-mcp adapter will
import. They are stable; the modules behind them are not.
"""

__version__ = "0.1.0"

#: The date the compiled Act and the benchmark rate table were last read.
LAW_CONTENT_DATE = "2026-08-31"

#: The compilation of ITAA 1936 this engine was written against.
LAW_COMPILATION = "C1936A00027, compilation in force 1 July 2026"

from .gate import GateFacts, GateResult, complying_loan_gate
from .myr import MyrFacts, MyrResult, minimum_yearly_repayment
from .rates import RateResult, benchmark_rate
from .register import ReviewLine, ReviewReport, review_register
from .verdicts import GateVerdict, MyrVerdict, RateVerdict, RowStatus
from .years import YearOfIncome, parse_year

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
