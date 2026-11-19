# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""The executable gives every output the removed import path gave, bar documented changes.

calculators_0_2_0_oracle.json holds what this server answered through the
australian-tax-calculators 0.2.0 import path, at the commit its provenance
names: each worksheet example in each period with its file note, refusals,
zero and edge amounts, the conformance cases, the scope catalogue, the
disclaimer's worksheet section and the data freshness datasets. The oracle
holds a placeholder for the engine version.

Calculators 1.0.0 changes only what DIVERGED lists, each recorded in
packages/australian-tax-calculators/CHANGELOG.md: newer sources, source dates and
evidence for 7 kinds, and one amount, contribution_caps' carry_forward_applied.
The tool now serves the executable's whole catalogue, so the 6 NARROWED kinds
state the engine's scope, and every kind may offer more periods and inputs
(WIDENED). The facade also refuses a number in exponent or other non-plain
notation in any field, where 0.2.0 refused it only in money, and leaves an
unreadable decimal to the executable's refusal. Every other refusal, every other
amount and every other field must be the oracle's. A diverged field must equal
what the served catalogue states for its kind, so results and catalogue agree.
"""

import asyncio
import functools
import json
import re
from decimal import Decimal, InvalidOperation
from pathlib import Path

import pytest
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp import freshness, resources
from aus_accounting_mcp.adapters import tax
from aus_accounting_mcp.server import mcp

_TEXT = (Path(__file__).parent / "calculators_0_2_0_oracle.json").read_text(encoding="utf-8")
ORACLE = json.loads(_TEXT.replace("<calculators version>", tax.CALCULATORS_VERSION))
CALLS = ORACLE["calculate_tax_worksheet"]
# Result metadata a kind may change, as the changelog records.
METADATA = {"sources", "source_checked"}
# The kinds whose scope the tool once narrowed to what its own schema took.
NARROWED = {"contribution_caps", "depreciation", "fbt", "medicare_levy", "payg_withholding",
            "study_loan_withholding"}
DIVERGED = {kind: METADATA | ({"scope"} if kind in NARROWED else set())
            for kind in [*NARROWED, "study_loan_repayment"]}
# Catalogue fields that now state the engine's whole entry: a superset of the oracle's.
WIDENED = {"supported_periods", "optional_inputs", "required_inputs"}
# Catalogue fields a kind may change, beyond its result metadata.
CATALOGUE_DIVERGED = {
    "contribution_caps": {"source_passage"},
    "depreciation": {"source_passage"},
    "fbt": {"source_passage"},
    "medicare_levy": {"source_passage", "example_evidence"},
    "payg_withholding": {"source_passage", "example_evidence"},
    "study_loan_repayment": {"source_passage", "example_evidence", "source"},
    "study_loan_withholding": {"source_passage", "example_evidence"},
}


@functools.cache
def _catalogue() -> dict:
    return tax.worksheet_catalogue()


def _carry_forward(old: dict, facts: dict) -> str:
    """ITAA 1997 s 291-20(4): only the unused cap that covers contributions over the cap.

    0.2.0 reported all the eligible unused cap; the year's cap is what was available
    without it.
    """
    cap = Decimal(old["concessional_available"]) - Decimal(old["carry_forward_applied"])
    over = max(Decimal(facts["concessional_contributions"].strip()) - cap, Decimal(0))
    return f"{min(Decimal(old['carry_forward_applied']), over):.2f}"


_PREFIX = "Error executing tool calculate_tax_worksheet: "


def _now_refused(facts: dict) -> str | None:
    """The refusal the facade now gives where 0.2.0 answered: a finite number in
    non-plain notation, refused by name in any field, not only in money."""
    for name, value in facts.items():
        if not isinstance(value, str):
            continue
        try:
            finite = Decimal(value).is_finite()
        except InvalidOperation:
            continue
        if finite and not re.fullmatch(r"\s*-?[0-9]+(?:\.[0-9]+)?\s*", value):
            return f"{_PREFIX}{name}: {value!r} is not a decimal amount"
    return None


@pytest.mark.parametrize("case", CALLS, ids=[str(number) for number in range(len(CALLS))])
def test_each_recorded_call_gives_the_same_answer(case) -> None:
    facts = case["arguments"]["facts"]
    try:
        result = asyncio.run(mcp.call_tool("calculate_tax_worksheet", case["arguments"]))
    except ToolError as exc:
        # 0.2.0 parsed a ratio itself; an unreadable one now gets the executable's
        # input.decimal_unreadable message.
        error = _now_refused(facts) or case["error"].replace(
            "[<class 'decimal.ConversionSyntax'>]", "decimal.InvalidOperation")
        assert {"arguments": case["arguments"], "error": error} == {
            "arguments": case["arguments"], "error": str(exc)}
        return
    assert _now_refused(facts) is None
    assert not result.is_error and "result" in case
    got, want = dict(result.structured_content), dict(case["result"])
    kind = facts["kind"]
    entry = _catalogue()[kind]
    for field in DIVERGED.get(kind, set()):
        assert got.pop(field) == entry[field]
        want.pop(field)
    if kind == "contribution_caps":
        got["amounts"] = {**got["amounts"]}
        assert got["amounts"].pop("carry_forward_applied") == _carry_forward(
            want["amounts"], facts)
        want["amounts"] = {**want["amounts"]}
        want["amounts"].pop("carry_forward_applied")
    if "file_note" in got:
        # The note restates the result, so it changes with it. It lists the facts in
        # the catalogue's order, which for quarterly_sg is not 0.2.0's.
        model = tax.TaxFacts.model_validate(facts)
        note = got.pop("file_note")
        assert note == tax.render_file_note(model, result.structured_content)
        old = want.pop("file_note")
        assert kind in DIVERGED or sorted(note.splitlines()) == sorted(old.splitlines())
    assert got == want


def test_the_oracle_covers_results_refusals_and_every_kind() -> None:
    kinds = {case["arguments"]["facts"]["kind"] for case in CALLS}
    assert kinds == set(ORACLE["calculation_worksheets"]) and len(kinds) == 14
    assert sum("result" in case for case in CALLS) > 200
    assert sum("error" in case for case in CALLS) > 90


def test_the_scope_catalogue_keeps_every_scope_source_and_checked_date_but_the_documented():
    # Alcuin F2: every scope, source, checked date, passage, period and example.
    served = json.loads(list(asyncio.run(mcp.read_resource("aus-accounting://scope")))[0].content)
    catalogue = served["calculation_worksheets"]
    assert catalogue.keys() >= ORACLE["calculation_worksheets"].keys()
    for kind, entry in ORACLE["calculation_worksheets"].items():
        now = catalogue[kind]
        allowed = DIVERGED.get(kind, set()) | CATALOGUE_DIVERGED.get(kind, set())
        differing = {field for field in (entry.keys() | now.keys()) - WIDENED
                     if entry.get(field) != now.get(field)}
        assert differing == allowed, kind
        assert entry["required_inputs"].keys() == now["required_inputs"].keys()
        assert all(period in now["supported_periods"] for period in entry["supported_periods"])


def test_the_disclaimer_states_the_served_worksheet_scopes() -> None:
    section = "\n".join(f"{kind}: {entry['scope']}" for kind, entry in _catalogue().items())
    assert f"# australian-tax-calculators\n\n{section}\n\n# gst-review" in resources.disclaimer()
    old = dict(line.split(": ", 1) for line in ORACLE["disclaimer_section"].splitlines())
    assert {kind for kind, scope in old.items() if _catalogue()[kind]["scope"] != scope} == NARROWED


def test_the_freshness_datasets_change_only_with_the_catalogue() -> None:
    datasets = freshness.data_freshness(tool="calculate_tax_worksheet")["datasets"]
    served = {dataset["dataset"]: dataset for dataset in datasets}
    for old in ORACLE["data_freshness"]:
        new = served[old["dataset"]]
        kind = new["dataset"].removeprefix("tax_worksheet:")
        # The engine's periods, a superset of those the tool once took.
        assert all(period in new["effective_periods"] for period in old["effective_periods"])
        new = {**new, "effective_periods": old["effective_periods"]}
        if kind in DIVERGED:
            entry = _catalogue()[kind]
            assert new["source_checked"] == entry["source_checked"]
            assert new["sources"] == entry["sources"]
            assert new["provenance"] == {"source_passage": entry["source_passage"]}
            new = {**new, **{key: old[key] for key in ("source_checked", "sources", "provenance")}}
        assert new == old
