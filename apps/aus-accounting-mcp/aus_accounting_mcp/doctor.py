# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""`aus-accounting-mcp doctor`: what this install holds and whether its folders work."""

from __future__ import annotations

import os
from typing import Any, Callable

from . import corpus, library, rulings
from .errors import NOT_CONFIGURED
from .freshness import data_freshness

HELP = (
    "Print the installed version, the periods its bundled data records and each configured "
    "retrieval folder with its row count. Exits 1 if a configured folder is broken."
)
NOT_CURRENT = "Bundled data periods (what the release records, not that a figure is current):"
ENVIRONMENT = (
    "Retrieval folders (from this command's environment; an MCP client passes the ones "
    "in its own server configuration):"
)


def _periods(dataset: dict[str, Any]) -> str:
    periods = dataset["effective_periods"]
    if isinstance(periods, str):
        return periods
    return f"{periods[0]['start']} to {periods[-1]['end']}"


def _library(found: dict[str, Any]) -> str:
    if found["skipped"]:
        raise ValueError(
            f"Markdown files search skips: {len(found['skipped'])}, first {found['skipped'][0]}."
        )
    return f"{found['rows']} lines in {found['files']} Markdown files"


def _corpus(found: dict[str, Any]) -> str:
    if found["unreadable"]:
        raise ValueError(f"lines a search cannot return: {found['unreadable']}.")
    if found["skipped_entries"]:
        raise ValueError(
            f"entries under markdown holding no index a search reads: {found['skipped_entries']}."
        )
    rates = found["rate_rows"]
    if rates is None:
        raise ValueError("it has no usable rates/rates.jsonl, so search_tax_rates will fail.")
    if not rates:
        raise ValueError("its rates index holds no rows, so a rate search would come back empty.")
    return f"{found['rows']} section rows in {found['indexes']} title indexes, {rates} rate rows"


def _rulings(found: dict[str, Any]) -> str:
    return (
        f"{found['rows']} paragraph rows from {found['documents']} documents in "
        f"{found['runs']} runs, {found['withheld']} documents withheld"
    )


ROOTS: tuple[tuple[str, Callable[[], dict[str, Any]], Callable[[dict[str, Any]], str]], ...] = (
    ("AUS_ACCOUNTING_LIBRARY_ROOT", library.census, _library),
    ("AUS_ACCOUNTING_CORPUS_ROOT", corpus.census, _corpus),
    ("AUS_ACCOUNTING_RULINGS_ROOT", rulings.census, _rulings),
)


def _check(
    census: Callable[[], dict[str, Any]], describe: Callable[[dict[str, Any]], str]
) -> tuple[bool, str]:
    """Whether the folder works, and a line saying what it holds or why it does not."""
    try:
        found = census()
        held = describe(found)
    # InputError is a ValueError, as a bad row's error is. A row nested past Python's
    # limit raises RecursionError.
    except (OSError, ValueError, RecursionError) as exc:
        return False, str(exc).replace(NOT_CONFIGURED, "").strip()
    if not found["rows"]:
        return False, "it holds no rows, so every search would come back empty."
    return True, held


def run() -> int:
    """Print the report and return the exit code: 1 if a configured folder is broken."""
    report = data_freshness()
    print(f"{report['server']['distribution']} {report['server']['version']}\n")
    print(NOT_CURRENT)
    for dataset in report["datasets"]:
        print(f"  {dataset['dataset']}: {_periods(dataset)}")
    print(f"\n{ENVIRONMENT}")
    broken = 0
    for variable, census, describe in ROOTS:
        root = os.environ.get(variable)
        if not root:
            print(f"  {variable}: not set")
            continue
        ok, line = _check(census, describe)
        broken += not ok
        print(f"  {variable}: {'OK' if ok else 'BROKEN'}: {root}: {line}")
    if broken:
        print(f"\n{broken} configured {'folder is' if broken == 1 else 'folders are'} broken.")
    return 1 if broken else 0
