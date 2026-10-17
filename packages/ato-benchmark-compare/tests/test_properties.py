"""Seeded properties for account identity and mapping round trips."""

from __future__ import annotations

import hashlib
import tempfile
from dataclasses import dataclass
from pathlib import Path

from atobenchmark.mapping import BUCKETS, MappingRow, account_key, read_mapping, write_mapping

from _fastcheck import fastcheck_property

# Input domains live in tests/fastcheck/properties.cjs under each test's name.


@fastcheck_property(seed=0xA70, runs=100, examples=[
    dict(pair=("motor vehicle", "  MOTOR\tVEHICLE\r\n")),
    dict(pair=("strasse", "StraSSE")),
])
def test_case_and_whitespace_variants_keep_one_logical_identity(
    pair: tuple[str, str],
) -> None:
    """Removing case folding, trimming or whitespace collapse must break this."""
    canonical, variant = pair
    expected = hashlib.sha256(canonical.encode("utf-8")).hexdigest()

    assert account_key(canonical) == expected
    assert account_key(variant) == expected


@dataclass(frozen=True)
class RowSpec:
    row: MappingRow
    identity: str


# The specs are unique by identity, as a reviewed mapping requires.
@fastcheck_property(
    seed=0xA701D,
    runs=100,
    context={"buckets": sorted(BUCKETS)},
    types=(RowSpec, MappingRow),
)
def test_generated_reviewed_mappings_round_trip_without_identity_loss(
    specs: list[RowSpec],
) -> None:
    """Dropping CSV guarding, keyed identity or a reviewed field must break this."""
    rows = [spec.row for spec in specs]
    expected = {
        hashlib.sha256(spec.identity.encode("utf-8")).hexdigest(): spec.row for spec in specs
    }

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "mapping.csv"
        write_mapping(path, rows)

        assert read_mapping(path) == expected
