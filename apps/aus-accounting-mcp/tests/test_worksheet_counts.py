# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Every public count of worksheet kinds agrees, and the release check refuses one
that differs from the catalogue the pinned calculators executable serves."""

import importlib.util
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "check_installed_worksheets.py"
SPEC = importlib.util.spec_from_file_location("check_installed_worksheets", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


def test_every_public_text_states_one_worksheet_count():
    # The release check compares this count with the pinned executable's catalogue.
    stated = CHECK.stated_counts(CHECK.REPOSITORY)
    assert len({count for counts in stated.values() for count in counts}) == 1


def test_a_text_that_drops_its_count_fails_and_is_named(tmp_path):
    for name in CHECK.COUNT_STATEMENTS:
        (tmp_path / name).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / name).write_text(
            (CHECK.REPOSITORY / name).read_text(encoding="utf-8"), encoding="utf-8")
    pyproject = tmp_path / "apps/aus-accounting-mcp/pyproject.toml"
    pyproject.write_text(CHECK.COUNT.sub(
        lambda found: "fourteen" + found.group(0)[len(found.group(1)):],
        pyproject.read_text(encoding="utf-8")), encoding="utf-8")

    with pytest.raises(ValueError, match="pyproject.toml states a worksheet count 0 times"):
        CHECK.stated_counts(tmp_path)


def test_a_count_the_executable_does_not_serve_fails_and_is_named():
    CHECK.check_counts({"a": [70], "b": [70, 70]}, 70)

    with pytest.raises(ValueError, match=r"serves 70 worksheet kinds; \['b'\]"):
        CHECK.check_counts({"a": [70], "b": [66, 70]}, 70)
