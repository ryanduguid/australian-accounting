# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Shipped industry-page data checks for ato-benchmark-compare.

`examples/build_other_benchmarks.rs` (`[[example]] test = true`) carries the
builder's unit tests under `cargo test`. This module holds the shipped JSON
data under `packages/ato-benchmark-compare/data/` to the shipped workbook JSON
and to the engine's canonical dataset digest.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "packages" / "ato-benchmark-compare"
DATA = ROOT / "data"


def _canonical_digest(text: str) -> str:
    canonical = json.dumps(
        json.loads(text), sort_keys=True, separators=(",", ":"), ensure_ascii=True
    )
    return hashlib.sha256(canonical.encode("ascii")).hexdigest()


def test_the_shipped_page_file_matches_the_shipped_workbook() -> None:
    pages = json.loads((DATA / "other-benchmarks-2023-24.json").read_text(encoding="utf-8"))
    workbook = json.loads((DATA / "benchmarks-2023-24.json").read_text(encoding="utf-8"))
    assert pages["benchmark_year"] == workbook["benchmark_year"]
    assert pages["source"]["dataset_sha256"] == workbook["source"]["sha256"]
    names = {entry["name"] for entry in pages["business_types"]}
    excluded = {entry["name"] for entry in pages["excluded_pages"]}
    assert not names & excluded
    assert names | excluded == {bt["name"] for bt in workbook["business_types"]}
    assert excluded == {"Hardware and building supplies retailing"}


def test_the_shipped_page_file_counts() -> None:
    pages = json.loads((DATA / "other-benchmarks-2023-24.json").read_text(encoding="utf-8"))
    ranges = [
        value
        for entry in pages["business_types"]
        for band in entry["turnover_bands"]
        for key, value in band.items()
        if key != "band" and value is not None
    ]
    assert len(pages["business_types"]) == 99
    assert len(ranges) == 708
    assert sum(1 for value in ranges if value["min"] == value["max"]) == 35


def test_the_builder_digest_is_the_one_the_engine_checks() -> None:
    """The engine refuses a page file whose dataset_json_sha256 is not its own digest of
    the dataset, so the shipped file's value is the engine's digest."""
    for pages_path in sorted(DATA.glob("other-benchmarks-*.json")):
        pages = json.loads(pages_path.read_text(encoding="utf-8"))
        dataset = (DATA / pages_path.name.removeprefix("other-")).read_text(encoding="utf-8")
        assert _canonical_digest(dataset) == pages["source"]["dataset_json_sha256"]
