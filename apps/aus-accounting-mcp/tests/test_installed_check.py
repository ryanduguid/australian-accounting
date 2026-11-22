# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""The release check refuses a wheel whose rate records differ from the reviewed dataset."""

import importlib.util
from pathlib import Path

import pytest

from aus_accounting_mcp import reviewed_rates

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "check_installed_worksheets.py"
SPEC = importlib.util.spec_from_file_location("check_installed_worksheets", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


@pytest.fixture
def folders(tmp_path):
    installed, reviewed = tmp_path / "installed", tmp_path / "reviewed"
    for folder in (installed, reviewed):
        folder.mkdir()
        (folder / "one.json").write_text('{"value": 1}\n', encoding="utf-8")
        (folder / "two.json").write_text('{"value": 2}\n', encoding="utf-8")
    return installed, reviewed


def test_the_same_records_pass_and_are_counted(folders):
    assert CHECK.check_records(*folders) == 2


def test_a_missing_record_fails_and_is_named(folders):
    installed, reviewed = folders
    (installed / "two.json").unlink()

    with pytest.raises(ValueError, match=r"2 reviewed, 1 installed, differing \['two.json'\]"):
        CHECK.check_records(installed, reviewed)


def test_an_extra_record_fails(folders):
    installed, reviewed = folders
    (installed / "three.json").write_text("{}", encoding="utf-8")

    with pytest.raises(ValueError, match="three.json"):
        CHECK.check_records(installed, reviewed)


def test_a_changed_figure_with_the_same_name_fails(folders):
    installed, reviewed = folders
    (installed / "one.json").write_text('{"value": 9}\n', encoding="utf-8")

    with pytest.raises(ValueError, match="one.json"):
        CHECK.check_records(installed, reviewed)


def test_a_dataset_that_cannot_be_found_fails_rather_than_matching_nothing(tmp_path):
    empty = tmp_path / "empty"
    empty.mkdir()

    with pytest.raises(ValueError, match="0 reviewed"):
        CHECK.check_records(empty, tmp_path / "gone")


def test_the_script_compares_the_real_dataset_in_this_checkout():
    assert CHECK.check_records(reviewed_rates.RECORDS, CHECK.REVIEWED_RECORDS) == 74


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
