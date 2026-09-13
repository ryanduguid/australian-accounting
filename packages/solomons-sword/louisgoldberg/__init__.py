"""
Solomon's Sword: trust distribution and section 100A / 99B risk engine
The distribution is `solomons-sword`; the import package remains `louisgoldberg`.
"""

from importlib.metadata import PackageNotFoundError
from importlib.metadata import version as _dist_version

try:
    __version__ = _dist_version("solomons-sword")
except PackageNotFoundError:  # running from a source tree without installation
    __version__ = "0.0.0.dev0"
__author__ = "Ryan Duguid"

from .division6 import (
    BeneficiaryEntitlement,
    TrustIncomeAssessment,
    calculate_proportionate_share,
)
from .section99b import (
    ForeignTrustReceipt,
    Section99BAssessment,
    evaluate_section99b_liability,
)
from .section100a import (
    Section100AAssessment,
    Section100ARiskZone,
    evaluate_section100a_risk,
)
from .trust_resolution import (
    TrustResolutionSchedule,
    validate_trust_resolution,
)

__all__ = [
    "TrustIncomeAssessment",
    "BeneficiaryEntitlement",
    "calculate_proportionate_share",
    "Section100ARiskZone",
    "Section100AAssessment",
    "evaluate_section100a_risk",
    "ForeignTrustReceipt",
    "Section99BAssessment",
    "evaluate_section99b_liability",
    "TrustResolutionSchedule",
    "validate_trust_resolution",
]
