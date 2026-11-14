# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""context7.json must name the calculator's withholding worksheets as they are at this commit."""

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
