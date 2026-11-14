# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""context7.json must pass Context7's schema, index component documentation and
name the calculator's withholding worksheets as its command table describes them."""

import json
import re
import urllib.request
from pathlib import Path

from jsonschema import Draft7Validator

ROOT = Path(__file__).resolve().parents[1]
CONFIG = json.loads((ROOT / "context7.json").read_text(encoding="utf-8"))
SCHEMA_URL = "https://context7.com/schema/context7.json"
ARGUMENTS = dict(re.findall(
    r"^\| `([a-z0-9-]+)` \|(.*)$",
    (ROOT / "packages/australian-tax-calculators/README.md").read_text(encoding="utf-8"),
    flags=re.MULTILINE,
))


def test_config_passes_the_published_schema() -> None:
    # Context7 publishes the schema only at this URL; the copy in upstash/context7 is older.
    assert CONFIG["$schema"] == SCHEMA_URL
    with urllib.request.urlopen(SCHEMA_URL, timeout=30) as response:
        schema = json.load(response)
    assert [error.message for error in Draft7Validator(schema).iter_errors(CONFIG)] == []


def test_rules_name_only_calculator_commands_that_exist() -> None:
    named = {name for rule in CONFIG["rules"] for name in re.findall(r"`([^`]+)`", rule)}
    assert named <= ARGUMENTS.keys()


def test_rules_name_every_withholding_worksheet() -> None:
    rules = " ".join(CONFIG["rules"])
    withholding = {command for command in ARGUMENTS if command.endswith("-withholding")}
    assert withholding
    assert {command for command in withholding if f"`{command}`" not in rules} == set()


def test_rules_name_every_command_that_takes_a_pay_date() -> None:
    dated = {command for command, arguments in ARGUMENTS.items() if "`pay_date`" in arguments}
    assert dated
    rule = next(rule for rule in CONFIG["rules"] if "pay date" in rule)
    assert {command for command in dated if f"`{command}`" not in rule} == set()


def test_folders_are_directories() -> None:
    assert all((ROOT / folder).is_dir() for folder in CONFIG["folders"])
