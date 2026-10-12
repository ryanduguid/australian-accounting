"""Replay the fuzz seed corpus so a broken harness fails ordinary CI."""

from __future__ import annotations

import sys
from pathlib import Path
from unittest import mock

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from fuzz import payday_super_import  # noqa: E402

SEEDS = sorted((ROOT / "fuzz" / "corpus").iterdir())


def test_the_corpus_is_not_empty() -> None:
    assert SEEDS  # nosec B101: pytest outcome assertion


@pytest.mark.parametrize("seed", SEEDS, ids=lambda seed: seed.name)
def test_seed_replays_cleanly(seed: Path) -> None:
    payday_super_import.test_one_input(seed.read_bytes())


def test_unexpected_reader_errors_escape() -> None:
    importers = payday_super_import.importers
    with mock.patch.object(importers, "read_payroll", side_effect=TypeError("bug")):
        with pytest.raises(TypeError, match="bug"):
            payday_super_import.test_one_input(b"")
