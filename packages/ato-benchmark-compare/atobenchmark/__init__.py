"""Compare a set of profit and loss figures against the ATO small business benchmarks."""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError
from importlib.metadata import version as _dist_version

from .dataset import load
from .money import parse_amount
from .report import compare, to_evidenced_dict

try:
    __version__ = _dist_version("ato-benchmark-compare")
except PackageNotFoundError:  # running from a source tree without installation
    __version__ = "0.0.0.dev0"

__all__ = ["__version__", "compare", "load", "parse_amount", "to_evidenced_dict"]
