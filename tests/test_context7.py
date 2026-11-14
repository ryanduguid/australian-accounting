# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""context7.json must index component documentation and name the calculator's withholding worksheets."""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONFIG = json.loads((ROOT / "context7.json").read_text(encoding="utf-8"))
COMMANDS = set(re.findall(
    r"^\| `([a-z0-9-]+)` \|",
    (ROOT / "packages/australian-tax-calculators/README.md").read_text(encoding="utf-8"),
    flags=re.MULTILINE,
))


def test_rules_name_only_calculator_commands_that_exist() -> None:
    named = {name for rule in CONFIG["rules"] for name in re.findall(r"`([^`]+)`", rule)}
    assert named <= COMMANDS


def test_rules_name_every_withholding_worksheet() -> None:
    rules = " ".join(CONFIG["rules"])
    withholding = {command for command in COMMANDS if command.endswith("-withholding")}
    assert withholding
    assert {command for command in withholding if f"`{command}`" not in rules} == set()


def test_folders_hold_component_documentation_not_the_discovery_note() -> None:
    folders = [ROOT / folder for folder in CONFIG["folders"]]
    assert all(folder.is_dir() for folder in folders)
    assert not any((ROOT / "docs/discovery.md").is_relative_to(folder) for folder in folders)
