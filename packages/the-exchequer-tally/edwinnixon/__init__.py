"""
The Exchequer Tally: corporate tax rate, franking account and Division 203
benchmark engine for Australian companies. The distribution is
`the-exchequer-tally`; the import package remains `edwinnixon`.

Outputs are review aids, not tax advice or determinations.
"""

from importlib.metadata import PackageNotFoundError
from importlib.metadata import version as _dist_version

try:
    __version__ = _dist_version("the-exchequer-tally")
except PackageNotFoundError:  # running from a source tree without installation
    __version__ = "0.0.0.dev0"
__author__ = "Ryan Duguid"

from .benchmark_rule import (
    BenchmarkRuleValidator,
    BenchmarkRuleViolation,
    DistributionEvent,
)
from .corporate_tax import (
    BaseRateEntityTest,
    CorporateTaxRate,
    determine_corporate_tax_rate,
    determine_max_franking_rate,
)
from .distribution_statement import (
    DistributionStatement,
    generate_distribution_statement,
)
from .franking_account import (
    FrankingAccount,
    FrankingDeficitResult,
    FrankingEntry,
    FrankingEntryType,
)

__all__ = [
    "CorporateTaxRate",
    "BaseRateEntityTest",
    "determine_corporate_tax_rate",
    "determine_max_franking_rate",
    "FrankingAccount",
    "FrankingEntry",
    "FrankingEntryType",
    "FrankingDeficitResult",
    "BenchmarkRuleValidator",
    "DistributionEvent",
    "BenchmarkRuleViolation",
    "DistributionStatement",
    "generate_distribution_statement",
]
