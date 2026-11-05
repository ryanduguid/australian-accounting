# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""A plain question reaches the corpus, and a partial hit is never reported as a match.

Every title and provision here is fabricated.
"""

import asyncio

import pytest
from jsonschema import Draft202012Validator

import synthetic_corpus
from aus_accounting_mcp import corpus as corpus_module
from aus_accounting_mcp.server import mcp

SCHEMAS = {tool.name: tool.output_schema for tool in asyncio.run(mcp.list_tools())}
PRINCIPAL_ACT = "Income Tax Assessment Act 1997"
MINOR_ACT = "Synthetic Levy Act 2099"


def call(name, **arguments):
    payload = asyncio.run(mcp.call_tool(name, arguments)).structured_content
    Draft202012Validator(SCHEMAS[name]).validate(payload)
    return payload


def write_title(root, register_id, act, rows, **options):
    synthetic_corpus.write(
        root / "markdown" / register_id / "sections.jsonl",
        [
            synthetic_corpus.section(
                register_id, f"{index:04d}", label, text, act=act, heading=heading, **options
            )
            for index, (label, heading, text) in enumerate(rows, start=1)
        ],
    )


@pytest.fixture
def root(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    write_title(tmp_path, "C9999A00010", MINOR_ACT, [
        ("10-1", "Motor vehicle cap", "The cap on a motor vehicle is set by regulation."),
        ("10-2", "Home based business", "A home based business keeps a separate office."),
        ("10-3", "Occupancy expenses", "Occupancy expenses of a home are not deductible."),
    ])
    write_title(tmp_path, "C2099A00038", PRINCIPAL_ACT, [
        ("20-1", "Fuel records", "An entity keeps fuel records for a motor vehicle."),
        ("20-2", "Scar tissue", "A scarred asset is written off."),
    ])
    return tmp_path


QUESTION = "What can I claim for the running costs of my home office?"


def test_a_plain_question_that_found_nothing_returns_labelled_partial_hits(root):
    result = call("search_tax_legislation", query=QUESTION)

    assert result["matches"] == []
    assert [hit["section"] for hit in result["partial_hits"]] == ["10-2"]
    first = result["partial_hits"][0]
    assert first["match"] == "partial"
    assert first["matched_words"] == ["home", "office"]
    assert first["missing_words"] == ["claim", "running", "costs"]
    assert "partial_hits are not matches" in result["notice"]
    assert "not that it answers the question" in result["notice"]
    # The no-match guidance stays, because no row holds every word.
    assert "No provision holds every word" in result["notice"]


def test_a_partial_hit_keeps_its_citation(root):
    hit = call("search_tax_legislation", query=QUESTION)["partial_hits"][0]

    assert hit["row_id"] == "C9999A00010:0002:10-2"
    assert hit["act"] == MINOR_ACT
    assert hit["compilation_date"] == "2098-07-01"
    assert hit["register_page"] == "https://example.invalid/C9999A00010/latest"
    assert hit["attribution"] == synthetic_corpus.ATTRIBUTION


def test_dropping_stop_words_turns_a_plain_question_into_a_match(root):
    result = call("search_tax_legislation", query="What is the cap on a motor vehicle?")

    assert [match["section"] for match in result["matches"]] == ["10-1"]
    assert result["dropped_words"] == ["what", "is", "the", "on", "a"]
    assert "match" not in result["matches"][0]


def test_an_all_words_hit_ranks_above_a_partial_hit_from_a_principal_act(root):
    # 20-1 sits in a principal Act and holds "motor" and "vehicle" in its text; 10-1, in a
    # minor title, holds every word. Every other signal favours the partial hit.
    result = call("search_tax_legislation", query="motor vehicle cap")

    assert [match["section"] for match in result["matches"]] == ["10-1"]
    assert [hit["section"] for hit in result["partial_hits"]] == ["20-1"]
    assert result["partial_hits"][0]["act"] == PRINCIPAL_ACT
    assert result["partial_hits"][0]["match"] == "partial"
    assert result["partial_hits"][0]["missing_words"] == ["cap"]
    assert "partial_hits are not matches" in result["notice"]


def test_no_partial_hit_appears_in_matches(root):
    for query in (QUESTION, "motor vehicle cap", "fuel home"):
        result = call("search_tax_legislation", query=query)
        words = [word for word in query.casefold().rstrip("?").split()
                 if word not in corpus_module.STOP_WORDS]
        for match in result["matches"]:
            text = " ".join(str(match[field]).casefold() for field in corpus_module.SECTION_FIELDS)
            assert all(word in text for word in words)
        held = {hit["row_id"] for hit in result["matches"]}
        assert not held & {hit["row_id"] for hit in result["partial_hits"]}


def test_a_row_holding_more_of_the_words_comes_first(root):
    # 20-1 holds 3 of the 4 words, 10-1 holds 2, and 10-1's are in its heading.
    result = call("search_tax_legislation", query="motor vehicle fuel licence")

    assert [hit["section"] for hit in result["partial_hits"]] == ["20-1", "10-1"]


def test_words_in_a_heading_break_a_tie_then_a_principal_act(root):
    write_title(root, "C9999A00011", MINOR_ACT, [
        ("30-1", "Ferry fares", "Text about a tram."),
        ("30-2", "Unrelated", "Ferry services and a tram are mentioned here."),
    ])
    write_title(root, "C2099A00039", PRINCIPAL_ACT, [
        ("40-1", "Unrelated", "Ferry services and a tram are mentioned here too."),
    ])

    result = call("search_tax_legislation", query="ferry tram zebra")

    assert [hit["section"] for hit in result["partial_hits"]] == ["30-1", "40-1", "30-2"]


def test_a_partial_word_must_be_a_whole_word(root):
    # "scarred" holds "car" only inside a longer word, so it does not count here.
    assert not call("search_tax_legislation", query="car asset licence")["partial_hits"]

    hit = call("search_tax_legislation", query="car asset written licence")["partial_hits"][0]
    assert hit["matched_words"] == ["asset", "written"]
    assert hit["missing_words"] == ["car", "licence"]


def test_one_shared_word_is_not_offered(root):
    # "home" is in 3 rows, but one shared word was no evidence of relevance.
    assert call("search_tax_legislation", query="home gym zebra")["partial_hits"] == []


def test_a_possessive_is_not_a_word(root):
    # Word splitting turns "entity's" into "entity" and "s", which must not count.
    result = call("search_tax_legislation", query="An entity's zebra")

    assert result["dropped_words"] == ["an", "s"]
    assert result["partial_hits"] == []


def test_an_act_name_is_no_evidence_about_a_provision(root):
    # Every row of the principal Act holds "income" and "tax" in its name.
    result = call("search_tax_legislation", query="What is the income tax rate in Tasmania?")

    assert result["partial_hits"] == []


def test_a_word_only_in_the_act_name_is_reported_as_missing(root):
    write_title(root, "C2099A00038", PRINCIPAL_ACT, [
        ("20-1", "Fuel records", "An entity keeps fuel records for a motor vehicle."),
    ])

    result = call("search_tax_legislation", query="income fuel records queensland")

    hit = result["partial_hits"][0]
    assert hit["matched_words"] == ["fuel", "records"]
    assert hit["missing_words"] == ["income", "queensland"]
    assert "the heading and text of each hold only" in result["notice"]


def test_a_section_label_is_not_the_provision_s_wording(root):
    write_title(root, "C9999A00011", MINOR_ACT, [
        ("99-1", "Fuel", "The fuel of a motor is not covered."),
        ("10-9", "Fuel", "Section 99-1 sets the fuel cap of a motor vehicle."),
    ])

    result = call("search_tax_legislation", query="How does section 99-1 treat a motor vehicle?")

    assert [hit["section"] for hit in result["partial_hits"]] == ["10-9"]
    assert result["partial_hits"][0]["missing_words"] == ["treat"]


def test_another_section_number_is_not_a_partial_hit(root):
    result = call("search_tax_legislation", query="How does section 99-1 treat a motor vehicle?")

    assert result["partial_hits"] == []


def rate_rows(root):
    synthetic_corpus.write(root / "rates" / "rates.jsonl", [
        synthetic_corpus.rate("old", "Caps", "The concessional contributions cap is $30,000.",
                              amounts=["$30,000"], years=["2024-25"]),
        synthetic_corpus.rate("new", "Caps", "For 2026-27 the concessional cap is $32,500.",
                              amounts=["$32,500"], years=["2026-27"]),
    ])


def test_a_row_for_another_year_is_not_a_partial_hit(root):
    rate_rows(root)

    result = call(
        "search_tax_rates", query="What is the concessional contributions cap for 2026-27?"
    )

    assert [hit["rate_id"] for hit in result["partial_hits"]] == ["new"]
    assert result["partial_hits"][0]["missing_words"] == ["contributions"]


def test_a_word_with_a_letter_and_a_digit_is_held_to_the_same_rule(root):
    write_title(root, "C9999A00011", MINOR_ACT, [
        ("10-9", "Form", "The fuel cap of a motor vehicle."),
    ])

    assert call("search_tax_legislation", query="fuel cap form s99")["partial_hits"] == []


@pytest.mark.parametrize("word", ["will", "us"])
def test_a_word_with_a_tax_meaning_is_not_a_stop_word(root, word):
    result = call("search_tax_legislation", query=f"a {word} trust")

    assert result["dropped_words"] == ["a"]


def test_a_substring_match_is_not_also_reported_as_a_partial_hit(root):
    # "scarred" holds "car" as a substring, so the all-words search takes the row;
    # the whole-word test must not then offer it again as a hit lacking "car".
    result = call("search_tax_legislation", query="car asset written")

    assert [match["section"] for match in result["matches"]] == ["20-2"]
    assert result["partial_hits"] == []


def test_a_phrase_of_stop_words_still_ranks_as_a_phrase(root):
    write_title(root, "C9999A00012", MINOR_ACT, [
        ("50-1", "Wheat and cost", "Text."),
        ("50-2", "Cost of wheat", "Text."),
    ])

    result = call("search_tax_legislation", query="cost of wheat")

    assert [match["section"] for match in result["matches"]] == ["50-2", "50-1"]
    assert result["dropped_words"] == ["of"]


def test_a_query_of_only_stop_words_is_searched_as_written(root):
    result = call("search_tax_legislation", query="by regulation of the")

    assert result["dropped_words"] == ["by", "of", "the"]

    result = call("search_tax_legislation", query="is by the")

    assert [match["section"] for match in result["matches"]] == ["10-1"]
    assert result["dropped_words"] == []
    assert result["partial_hits"] == []


@pytest.mark.parametrize("query", ["What about zebras?", "motor zebra"])
def test_a_query_too_short_to_be_partly_held_offers_nothing(root, query):
    result = call("search_tax_legislation", query=query)

    assert result["matches"] == []
    assert result["partial_hits"] == []
    assert "partial_hits" not in result["notice"]
    assert "No provision holds every word" in result["notice"]


def test_partial_hits_keep_the_act_and_currency_filters(root):
    write_title(root, "C9999A00013", MINOR_ACT, [
        ("60-1", "Home office", "Superseded running costs text."),
    ], current=False)

    scoped = call("search_tax_legislation", query=QUESTION, act="income tax assessment")
    assert scoped["partial_hits"] == []

    current = call("search_tax_legislation", query=QUESTION)
    assert "60-1" not in {hit["section"] for hit in current["partial_hits"]}

    every = call("search_tax_legislation", query=QUESTION, in_force_only=False)
    assert "60-1" in {hit["section"] for hit in every["partial_hits"]}


def test_pages_run_on_from_matches_into_partial_hits_without_a_repeat(root):
    first = call("search_tax_legislation", query="motor vehicle cap home", limit=1)
    assert first["matches"] == []
    assert [hit["section"] for hit in first["partial_hits"]] == ["10-1"]
    assert first["has_more"] is True

    first = call("search_tax_legislation", query="motor vehicle cap", limit=1)
    assert [match["section"] for match in first["matches"]] == ["10-1"]
    assert first["partial_hits"] == []
    assert first["has_more"] is True

    second = call("search_tax_legislation", query="motor vehicle cap", limit=1,
                  offset=first["next_offset"])
    assert second["matches"] == []
    assert [hit["section"] for hit in second["partial_hits"]] == ["20-1"]
    assert second["has_more"] is False
    assert second["next_offset"] is None
    # The no-match hint is for a search with no match, not for a later page of one.
    assert "No provision holds every word" not in second["notice"]


def test_a_full_page_of_matches_never_reads_for_partial_hits(root, monkeypatch):
    write_title(root, "C9999A00014", MINOR_ACT, [
        ("70-1", "Ferry tram zebra", "Text."),
        ("70-2", "Ferry tram zebra", "Text."),
    ])
    monkeypatch.setattr(corpus_module, "_partial", lambda *_: pytest.fail("read for partials"))

    result = call("search_tax_legislation", query="ferry tram zebra", limit=1)

    assert result["has_more"] is True
    assert result["partial_hits"] == []


def test_a_rate_search_labels_its_partial_hits(root):
    synthetic_corpus.write(root / "rates" / "rates.jsonl", [
        synthetic_corpus.rate("r1", "Levies", "The fuel toll cap is $9.", amounts=["$9"],
                              years=["2098-99"]),
        synthetic_corpus.rate("r2", "Levies", "The fuel toll is $3.", amounts=["$3"],
                              years=["2098-99"]),
        synthetic_corpus.rate("r3", "Levies", "A toll cap of $5.", amounts=["$5"],
                              years=["2097-98"]),
        synthetic_corpus.rate("r4", "Levies", "A cap of $6.", amounts=["$6"], years=["2097-98"]),
    ])

    result = call("search_tax_rates", query="What is the fuel toll cap?")

    assert [match["rate_id"] for match in result["matches"]] == ["r1"]
    assert [hit["rate_id"] for hit in result["partial_hits"]] == ["r2", "r3"]
    assert result["partial_hits"][0]["match"] == "partial"
    assert result["partial_hits"][0]["matched_words"] == ["fuel", "toll"]
    assert result["partial_hits"][0]["missing_words"] == ["cap"]
    assert "partial_hits are not matches" in result["notice"]

    narrowed = call("search_tax_rates", query="fuel toll cap", year="2098-99")
    assert [hit["rate_id"] for hit in narrowed["partial_hits"]] == ["r2"]

    assert not call("search_tax_rates", query="fuel toll cap", year="1999-00")["partial_hits"]
