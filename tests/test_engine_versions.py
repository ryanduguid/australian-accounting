"""The engine version quoted beside a number must be the engine that produced it."""

from __future__ import annotations

import importlib.metadata

from aus_accounting_mcp.adapters.benchmarks import BENCHMARK_VERSION
from aus_accounting_mcp.adapters.payday import PAYDAY_VERSION


def test_quoted_engine_versions_match_the_installed_engines() -> None:
    # Both constants are the engines' own __version__ attributes, and every tool
    # response reports them as engine_version. ato-benchmark-compare 0.1.3 shipped
    # __version__ = "0.1.2", so the citation trail understated the engine while
    # that pin stood. This fails if a pinned engine ever does that again.
    assert BENCHMARK_VERSION == importlib.metadata.version("ato-benchmark-compare")
    assert PAYDAY_VERSION == importlib.metadata.version("payday-super-checker")
