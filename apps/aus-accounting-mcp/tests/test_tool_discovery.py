"""Check the discovery contract clients receive through MCP tools/list."""

import ast
import asyncio
import io
import tokenize
from pathlib import Path

import pytest

from aus_accounting_mcp import server
from aus_accounting_mcp.server import mcp

TOOLS = asyncio.run(mcp.list_tools())


@pytest.mark.parametrize("tool", TOOLS, ids=lambda tool: tool.name)
def test_every_input_has_a_description_in_the_public_schema(tool) -> None:
    properties = tool.input_schema["properties"]
    # refuse_div7a is the one tool that takes nothing: a refusal needs no facts,
    # and a published input would only invite inventing them.
    assert properties or tool.name == "refuse_div7a"
    missing = [name for name, schema in properties.items() if not schema.get("description")]
    assert not missing, f"{tool.name} has undocumented inputs: {missing}"


def _plain_string_literal(token: str) -> str | None:
    try:
        value = ast.literal_eval(token)
    except (ValueError, SyntaxError):
        return None
    return value if isinstance(value, str) else None


def test_no_description_fuses_two_words_across_a_line_break() -> None:
    # Descriptions are wrapped as adjacent string literals. A literal ending in a
    # letter followed by one starting with a letter reaches the model as a single
    # run-together word, such as "ratiodenominator".
    source = Path(server.__file__).read_text(encoding="utf-8")
    tokens = [
        token for token in tokenize.generate_tokens(io.StringIO(source).readline)
        if token.type not in (tokenize.NL, tokenize.COMMENT)
    ]
    fused = [
        first.start[0]
        for first, second in zip(tokens, tokens[1:])
        if first.type == second.type == tokenize.STRING
        and (left := _plain_string_literal(first.string)) is not None
        and (right := _plain_string_literal(second.string)) is not None
        and left[-1:].isalnum()
        and right[:1].isalnum()
    ]
    assert not fused, f"literals joined without a space at lines {fused}"


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
    ("list_ato_benchmark_industries", "search"): 120,
    ("list_ato_benchmark_industries", "year"): 7,
    ("get_ato_benchmarks", "industry"): 120,
    ("get_ato_benchmarks", "year"): 7,
    ("calc_payday_super_deadline", "qe_day"): 40,
    ("calc_payday_super_deadline", "sg_amount"): 60,
    ("calc_payday_super_deadline", "as_at"): 40,
    ("calc_payday_super_deadline", "remitted"): 40,
    ("calc_payday_super_deadline", "received"): 40,
    ("calc_payday_super_deadline", "next_standard_qe_day"): 40,
    ("calc_payday_super_deadline", "remitted_amount"): 60,
    ("calc_payday_super_deadline", "matched_amount"): 60,
    ("get_div7a_benchmark_rate", "year_of_income"): 7,
    ("review_div7a_loan", "year_of_income"): 7,
    ("review_div7a_loan", "year_loan_made"): 7,
    ("review_div7a_loan", "maximum_term_years"): 30,
    ("review_div7a_loan", "security_coverage_at_first_made"): 30,
    ("review_div7a_loan", "interest_rate_for_years_after_year_loan_made"): 30,
    ("review_div7a_loan", "amalgamated_loan_unpaid_at_end_of_previous_year"): 60,
    ("review_div7a_loan", "remaining_term_years"): 30,
    ("review_div7a_loan", "payments_applied_during_the_year"): 60,
    ("generate_synthetic_sbr_fixture", "form_type"): 20,
    ("generate_synthetic_sbr_fixture", "revenue_or_sales"): 60,
    ("review_payday_super_contributions", "as_at"): 40,
    ("build_payday_super_evidence_pack", "as_at"): 40,
    ("search_accounting_library", "query"): 200,
    ("read_accounting_library", "path"): 500,
    ("search_tax_legislation", "query"): 200,
    ("search_tax_legislation", "act"): 200,
    ("read_tax_legislation_section", "row_id"): 300,
    ("define_tax_term", "term"): 200,
    ("define_tax_term", "act"): 200,
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


def test_every_money_input_shares_the_worksheet_money_bound() -> None:
    """Every amount the tools take goes through aus_accounting_mcp.money, the same
    boundary the worksheet Money alias describes, so they all carry its length."""
    from aus_accounting_mcp.adapters.tax import Money

    money = _declared_max_length(Money.__metadata__[0].metadata)
    amounts = {
        key for key in BOUNDED_STRING_INPUTS
        if key[1].endswith(("amount", "principal", "revenue_or_sales"))
        or key[1].startswith(("amalgamated_loan", "payments_applied"))
    }
    assert len(amounts) == 6
    assert {BOUNDED_STRING_INPUTS[key] for key in amounts} == {money}


def test_the_div7a_ratio_bounds_match_the_worksheet_ratio_type() -> None:
    """Terms, coverage and the interest rate reach the engine's ratio parsers,
    which put no ceiling on magnitude, so the facade's own Ratio alias is the
    only width they are held to."""
    from aus_accounting_mcp.adapters.tax import Ratio

    ratio = _declared_max_length(Ratio.__metadata__[0].metadata)
    for name in (
        "maximum_term_years",
        "remaining_term_years",
        "security_coverage_at_first_made",
        "interest_rate_for_years_after_year_loan_made",
    ):
        assert BOUNDED_STRING_INPUTS[("review_div7a_loan", name)] == ratio


def test_the_year_bound_is_the_width_of_the_yyyy_yy_label() -> None:
    """Both engines read a year of income as exactly YYYY-YY: div7a-loan-review
    through parse_year, ato-benchmark-compare as the dataset file name. The
    bound admits every year either one ships and nothing wider."""
    from atobenchmark.dataset import DATA_DIR, available_years
    from div7aloan import parse_year

    bound = BOUNDED_STRING_INPUTS[("get_div7a_benchmark_rate", "year_of_income")]
    shipped = available_years(DATA_DIR)
    assert shipped
    assert {len(year) for year in shipped} == {bound}
    widest = "2026-27"
    parse_year(widest)
    assert len(widest) == bound
    for key in (
        ("list_ato_benchmark_industries", "year"),
        ("get_ato_benchmarks", "year"),
        ("review_div7a_loan", "year_of_income"),
        ("review_div7a_loan", "year_loan_made"),
    ):
        assert BOUNDED_STRING_INPUTS[key] == bound


def test_the_date_bound_admits_the_longest_date_the_engine_reads() -> None:
    """The payday dates go to payday-super-checker's parse_date_text, whose
    widest shapes are a spelled-out month with a 12-hour time and an ISO
    date-time with a 6-digit fraction. Each must fit the published bound."""
    from datetime import datetime

    from paydaysuper.csv_io import DATE_FORMATS, TIME_FORMATS, parse_date_text

    bound = BOUNDED_STRING_INPUTS[("calc_payday_super_deadline", "qe_day")]
    moment = datetime(2027, 9, 13, 22, 30, 45, 123456)
    shapes = [moment.strftime(fmt) for fmt in DATE_FORMATS]
    shapes += [
        moment.strftime(f"{fmt} {time_fmt}")
        for fmt in DATE_FORMATS
        for time_fmt in TIME_FORMATS
    ]
    shapes.append(moment.isoformat())
    for shape in shapes:
        assert parse_date_text(shape) == moment.date(), shape
        assert len(shape) <= bound, shape
    for key in (
        ("calc_payday_super_deadline", "as_at"),
        ("calc_payday_super_deadline", "remitted"),
        ("calc_payday_super_deadline", "received"),
        ("calc_payday_super_deadline", "next_standard_qe_day"),
        ("review_payday_super_contributions", "as_at"),
        ("build_payday_super_evidence_pack", "as_at"),
    ):
        assert BOUNDED_STRING_INPUTS[key] == bound


def test_the_industry_bound_admits_every_shipped_business_type_name() -> None:
    """industry and search are matched against the dataset's business-type
    names, so the bound has to admit the longest name in every shipped year."""
    from atobenchmark.dataset import DATA_DIR, available_years, load

    bound = BOUNDED_STRING_INPUTS[("get_ato_benchmarks", "industry")]
    assert BOUNDED_STRING_INPUTS[("list_ato_benchmark_industries", "search")] == bound
    longest = max(
        len(business_type.name)
        for year in available_years(DATA_DIR)
        for business_type in load(year).business_types
    )
    assert longest <= bound
