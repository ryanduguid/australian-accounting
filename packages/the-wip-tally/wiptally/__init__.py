"""Deterministic AASB 15 construction WIP schedule. Review aid, not a determination."""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError
from importlib.metadata import version as _dist_version

from .money import parse_money
from .schedule import measure

try:
    __version__ = _dist_version("the-wip-tally")
except PackageNotFoundError:  # running from a source tree without installation
    __version__ = "0.0.0.dev0"

__all__ = ["__version__", "measure", "parse_money"]
