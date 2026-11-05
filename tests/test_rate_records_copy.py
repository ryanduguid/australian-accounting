# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""The MCP server's bundled rate records must equal the reviewed dataset.

No published engine carries packages/au-tax-rates-data, so aus-accounting-mcp ships a
copy for get_reviewed_rate. This sits at the root because a change to the dataset alone
must fail here. Standard library only, and no component is imported.
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATASET = ROOT / "packages" / "au-tax-rates-data" / "data"
COPY = ROOT / "apps" / "aus-accounting-mcp" / "aus_accounting_mcp" / "rate_records"
HOW = (
    "Copy packages/au-tax-rates-data/data/*.json over "
    "apps/aus-accounting-mcp/aus_accounting_mcp/rate_records/, removing any file it no "
    "longer holds."
)


def _records(folder: Path) -> dict[str, object]:
    return {
        path.name: json.loads(path.read_text(encoding="utf-8"))
        for path in folder.iterdir()
    }


def test_the_bundled_records_equal_the_reviewed_dataset() -> None:
    dataset, copy = _records(DATASET), _records(COPY)

    assert sorted(copy) == sorted(dataset), HOW
    for name, record in dataset.items():
        assert copy[name] == record, f"{name} differs. {HOW}"
