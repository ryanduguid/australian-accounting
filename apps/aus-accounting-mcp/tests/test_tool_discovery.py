"""Check the discovery contract clients receive through MCP tools/list."""

import asyncio

import pytest

from aus_accounting_mcp.server import mcp

TOOLS = asyncio.run(mcp.list_tools())


@pytest.mark.parametrize("tool", TOOLS, ids=lambda tool: tool.name)
def test_every_input_has_a_description_in_the_public_schema(tool) -> None:
    properties = tool.input_schema["properties"]
    assert properties
    missing = [name for name, schema in properties.items() if not schema.get("description")]
    assert not missing, f"{tool.name} has undocumented inputs: {missing}"


@pytest.mark.parametrize("tool", TOOLS, ids=lambda tool: tool.name)
def test_every_tool_has_a_readable_title_for_a_host_menu(tool) -> None:
    # A host lists these to a person, and the raw identifier is not the label to
    # show them. Prose, not a symbol: no underscores, and not the identifier
    # itself.
    assert tool.title
    assert tool.title != tool.name
    assert "_" not in tool.title
    assert tool.title[0].isupper()


def test_no_tool_reachable_only_by_supplying_facts_it_ignores() -> None:
    # refuse_div7a exists to answer questions this server does not review, and
    # those arrive with no loan facts. A required input on a tool that ignores
    # its inputs would make a model fabricate one to reach the refusal.
    refusal = next(tool for tool in TOOLS if tool.name == "refuse_div7a")

    assert not refusal.input_schema.get("required")


@pytest.mark.parametrize("tool", TOOLS, ids=lambda tool: tool.name)
def test_local_tools_disclose_their_effects_to_clients(tool) -> None:
    assert tool.annotations is not None
    annotations = tool.annotations.model_dump(by_alias=True)
    assert annotations["readOnlyHint"] is True
    assert annotations["destructiveHint"] is False
    assert annotations["idempotentHint"] is True
    assert annotations["openWorldHint"] is False


# The declared length of every bounded string input, as a client sees it in
# tools/list. A bound that disappears from the public schema lets an operator
# reference or an amount arrive unbounded, which is what these pin.
BOUNDED_STRING_INPUTS = {
    ("get_ato_benchmarks", "turnover"): 60,
    ("get_ato_benchmarks", "other_income"): 60,
    ("get_ato_benchmarks", "cost_of_sales"): 60,
    ("get_ato_benchmarks", "cost_of_sales_labour"): 60,
    ("get_ato_benchmarks", "salary_wages"): 60,
    ("get_ato_benchmarks", "contractor_commission"): 60,
    ("get_ato_benchmarks", "associated_persons"): 60,
    ("get_ato_benchmarks", "rent"): 60,
    ("get_ato_benchmarks", "motor_vehicle"): 60,
    ("get_ato_benchmarks", "other_expense"): 60,
    ("get_ato_benchmarks", "w1"): 60,
    ("calc_payday_super_deadline", "employee_id"): 120,
    ("review_div7a_loan", "loan_id"): 120,
    ("generate_synthetic_sbr_fixture", "entity_name"): 120,
    ("search_accounting_library", "query"): 200,
    ("read_accounting_library", "path"): 500,
    ("search_tax_legislation", "query"): 200,
    ("search_tax_legislation", "act"): 200,
    ("read_tax_legislation_section", "row_id"): 300,
    ("search_tax_rates", "query"): 200,
    ("search_tax_rates", "topic"): 100,
}


def _string_leaves(schema):
    """Every string branch of an input schema, including an optional one.

    An optional input is published as anyOf[{string}, {null}], so a bound
    declared on the string branch is invisible to a naive top-level read.
    """
    if "anyOf" in schema:
        for branch in schema["anyOf"]:
            yield from _string_leaves(branch)
    elif schema.get("type") == "string" and "enum" not in schema:
        yield schema


@pytest.mark.parametrize(
    "key,expected",
    sorted(BOUNDED_STRING_INPUTS.items()),
    ids=lambda value: value if isinstance(value, int) else ".".join(value),
)
def test_every_bounded_string_input_publishes_its_length(key, expected) -> None:
    tool_name, input_name = key
    tool = next(tool for tool in TOOLS if tool.name == tool_name)
    leaves = list(_string_leaves(tool.input_schema["properties"][input_name]))
    assert leaves, f"{tool_name}.{input_name} is not a string input"
    assert [leaf.get("maxLength") for leaf in leaves] == [expected] * len(leaves)


def _declared_max_length(constraints) -> int:
    return next(
        item.max_length for item in constraints if getattr(item, "max_length", None)
    )


def test_the_operator_reference_bound_matches_the_engine_model() -> None:
    """An employee reference reaching the grouped tool is bounded by the
    adapter's own model. The single-contribution tool declares its own Field,
    so the 2 can drift apart without either one looking wrong on its own."""
    from aus_accounting_mcp.adapters.payday import ContributionInput

    declared = _declared_max_length(
        ContributionInput.model_fields["employee_id"].metadata
    )
    assert declared == BOUNDED_STRING_INPUTS[("calc_payday_super_deadline", "employee_id")]


def test_the_benchmark_money_bound_matches_the_worksheet_money_type() -> None:
    """The benchmark tool spells its money inputs out one by one rather than
    reusing the Money alias, so the bound is pinned to that alias here."""
    from aus_accounting_mcp.adapters.tax import Money

    declared = _declared_max_length(Money.__metadata__[0].metadata)
    assert declared == BOUNDED_STRING_INPUTS[("get_ato_benchmarks", "turnover")]
