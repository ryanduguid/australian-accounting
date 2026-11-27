# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""context7.json must pass Context7's schema, index component documentation and
name the calculator's withholding worksheets as its command table describes them."""

import json
import os
import re
import urllib.error
import urllib.request
from pathlib import Path

import pytest
from jsonschema import Draft7Validator

ROOT = Path(__file__).resolve().parents[1]
CONFIG = json.loads((ROOT / "context7.json").read_text(encoding="utf-8"))
SCHEMA_URL = "https://context7.com/schema/context7.json"
ARGUMENTS = dict(re.findall(
    r"^\| `([a-z0-9-]+)` \|(.*)$",
    (ROOT / "packages/australian-tax-calculators/README.md").read_text(encoding="utf-8"),
    flags=re.MULTILINE,
))


def published_schema() -> dict:
    """The schema, or a skip on a runner with no network; GitHub Actions never skips."""
    try:
        with urllib.request.urlopen(SCHEMA_URL, timeout=30) as response:
            return json.load(response)
    except urllib.error.URLError as error:
        # An offline runner fails to connect. An HTTP error means the network
        # answered, and hosted CI has the network, so those failures are real.
        if isinstance(error, urllib.error.HTTPError) or os.environ.get("GITHUB_ACTIONS") == "true":
            raise
        pytest.skip(f"needs network to fetch {SCHEMA_URL}: {error.reason}")


def test_config_passes_the_published_schema() -> None:
    # Context7 publishes the schema only at this URL; the copy in upstash/context7 is older.
    assert CONFIG["$schema"] == SCHEMA_URL
    schema = published_schema()
    assert [error.message for error in Draft7Validator(schema).iter_errors(CONFIG)] == []


def unreachable(*_args, **_kwargs):
    raise urllib.error.URLError(OSError("Network is unreachable"))


def test_an_offline_runner_skips_the_schema_fetch(monkeypatch) -> None:
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.setattr(urllib.request, "urlopen", unreachable)
    with pytest.raises(pytest.skip.Exception, match="needs network"):
        published_schema()


def test_github_actions_fails_rather_than_skips_without_the_schema(monkeypatch) -> None:
    monkeypatch.setenv("GITHUB_ACTIONS", "true")
    monkeypatch.setattr(urllib.request, "urlopen", unreachable)
    with pytest.raises(urllib.error.URLError):
        published_schema()


def test_an_http_error_fails_everywhere(monkeypatch) -> None:
    def refused(*_args, **_kwargs):
        raise urllib.error.HTTPError(SCHEMA_URL, 404, "Not Found", None, None)

    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.setattr(urllib.request, "urlopen", refused)
    with pytest.raises(urllib.error.HTTPError):
        published_schema()


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


def test_the_age_pension_rule_names_every_stated_circumstance() -> None:
    """The circumstances age-pension's example states false must each be named in its rule."""
    metadata = (ROOT / "packages/australian-tax-calculators/src/metadata.rs").read_text(encoding="utf-8")
    example = metadata[metadata.rindex('"age_pension" => vec!['):]
    facts = json.loads(re.search(r'r#"(\{.*?\})"#', example).group(1))
    circumstances = {name for name, stated in facts.items() if stated is False}
    assert len(circumstances) == 6
    rule = next(rule for rule in CONFIG["rules"] if "`age-pension`'s facts" in rule)
    assert {name for name in circumstances if name not in rule} == set()


def test_the_age_pension_rules_name_each_conditional_circumstance_and_its_parent() -> None:
    """A circumstance age-pension reads only under another must be named with that parent."""
    engine = (ROOT / "packages/australian-tax-calculators/src/engine/age_pension.rs").read_text(encoding="utf-8")
    pairs = re.findall(r'\(\s*"(\w+)",\s*"[^"]*",\s*Some\("(\w+)"\)', engine)
    assert sorted(pairs) == [("rate_before_november_1986", "armed_services_payment"),
                             ("remote_area_child", "remote_area")]
    rule = next(rule for rule in CONFIG["rules"] if "only where" in rule and "`age-pension`" in rule)
    assert [(child, parent) for child, parent in pairs if f"{child} " not in rule or f"{parent} is true" not in rule] == []


def test_folders_are_directories() -> None:
    assert all((ROOT / folder).is_dir() for folder in CONFIG["folders"])
