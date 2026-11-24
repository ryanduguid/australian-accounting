# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Synthetic checks for a definition's found status and for the provisions citing a section.

Every row here is fabricated. The real corpus is the operator's own folder and never
enters this repository.
"""

import asyncio

import pytest
from jsonschema import Draft202012Validator
from mcp.server.mcpserver.exceptions import ToolError

import synthetic_corpus
from aus_accounting_mcp.server import mcp
from synthetic_corpus import LEVY_ACT, section

SCHEMAS = {tool.name: tool.output_schema for tool in asyncio.run(mcp.list_tools())}
OTHER_ACT = "Synthetic Other Act 2099"
TARGET = "C9999A00001:0002:5-10"


def call(name, **arguments):
    result = asyncio.run(mcp.call_tool(name, arguments))
    payload = result.structured_content
    Draft202012Validator(SCHEMAS[name]).validate(payload)
    return payload


@pytest.fixture
def corpus(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    return synthetic_corpus.build(tmp_path)


@pytest.fixture
def citing(corpus):
    """The synthetic corpus plus provisions that cite, or only seem to cite, s 5-10."""
    levy = corpus / "markdown" / "C9999A00001" / "sections.jsonl"
    rows = [
        section("C9999A00001", "0001", "1", "This Act may be cited as the Levy Act.",
                act=LEVY_ACT),
        section("C9999A00001", "0002", "5-10",
                "The synthetic levy rate is 7% of the assessable amount.", act=LEVY_ACT),
        section("C9999A00001", "0003", "5-15",
                "Opening words.\n\nThe rate in subsection 5-10(2) and section 5-15 "
                "applies.\n\nClosing words.", act=LEVY_ACT, heading="5-15 Exemption"),
        section("C9999A00001", "0004", "5-20",
                "A rate under s 5‑10 is not changed by section 5-100.", act=LEVY_ACT,
                heading="5-20 Unchanged"),
        section("C9999A00001", "0005", "5-25", "Section 5-10 once applied here.",
                act=LEVY_ACT, current=False, heading="5-25 Former rule"),
        section("C9999A00001", "0006", "5-30",
                "See sections 5-5 to 5-15 and section 5-10 of this Act.", act=LEVY_ACT,
                heading="5-30 Guide"),
    ]
    synthetic_corpus.write(levy, rows)
    synthetic_corpus.write(
        corpus / "markdown" / "C9999A00009" / "sections.jsonl",
        [
            section("C9999A00009", "0001", "3",
                    "The amount under section 5-10 of the Synthetic Levy Act 2099 is "
                    "reported.", act=OTHER_ACT),
            section("C9999A00009", "0002", "4", "This section 5-10 is the Act's own.",
                    act=OTHER_ACT),
            section("C9999A00009", "0003", "6",
                    "Section 5-10 of the Synthetic Charge Act 2099 does not apply.",
                    act=OTHER_ACT),
            section("C9999A00009", "0004", "Schedule 1", "Forms.", act=OTHER_ACT,
                    heading="Schedule 1 Forms"),
        ],
    )
    return corpus


def test_a_term_with_its_own_entry_is_defined(corpus):
    result = call("define_tax_term", term="small entity")

    assert result["defined"] is True
    assert result["nearest_terms"] == []
    assert result["defined_note"] is None


def test_a_term_with_only_longer_entries_is_not_defined_and_names_them(corpus):
    result = call("define_tax_term", term="cap")

    assert result["defined"] is False
    assert result["nearest_terms"] == ["small entity cap, for a year"]
    assert [entry["match"] for entry in result["definitions"]] == ["partial"]
    assert "not a finding that the law leaves it undefined" in result["defined_note"]
    assert "ordinary meaning" in result["defined_note"]


def test_a_term_with_no_entry_is_not_defined_and_has_no_nearest_terms(corpus):
    result = call("define_tax_term", term="quantum widget")

    assert result["defined"] is False
    assert result["nearest_terms"] == []
    assert result["definitions"] == []
    assert result["defined_note"]


def test_a_term_defined_only_in_a_superseded_compilation_is_not_defined_in_force(corpus):
    assert call("define_tax_term", term="old term")["defined"] is False
    assert call("define_tax_term", term="old term", in_force_only=False)["defined"] is True


@pytest.mark.parametrize(("wording", "cited"), [
    ("Section 5-10 applies to the levy.", True),
    ("Under subsection 5-10(1) the rate applies.", True),
    ("See ss 5-5 and 5-10.", True),
    # "the Act" as a defined short title may name another Act.
    ("The rate in section 5-10 of the Act applies.", False),
    ("Section 5-10 of Schedule 1 applies.", False),
    ("Apply section 5-10 in Australia.", True),
    ("Apply section 5-10 of this Act in Australia.", True),
    ("Apply section 5-10 in Part 2.", True),
    ("See sections 5-5, 5-10 and 5-15.", True),
    ("See section 5-5 or 5-10.", True),
    # A further digit, here U+0663, makes it another section.
    ("The rate in section 5-10٣ applies.", False),
    ("The rate in section 5-10 of the Schedule applies.", False),
    ("The rate in section 5-10 of the Other (General) Rules 2099 applies.", False),
])
def test_ordinary_forms_are_read_and_a_title_it_cannot_name_is_not(corpus, wording, cited):
    synthetic_corpus.write(
        corpus / "markdown" / "C9999A00001" / "sections.jsonl",
        [
            section("C9999A00001", "0002", "5-10", "The rate is 7%.", act=LEVY_ACT),
            section("C9999A00001", "0003", "5-15", wording, act=LEVY_ACT,
                    heading="5-15 Use"),
        ],
    )

    result = call("find_citing_provisions", row_id=TARGET)

    assert [entry["section"] for entry in result["cited_by"]] == (["5-15"] if cited else [])


@pytest.mark.parametrize("cited", ["C9999A00001:0002:2", "C9999A00001:0003:6"])
def test_a_subsection_number_or_a_longer_number_is_not_another_section(corpus, cited):
    synthetic_corpus.write(
        corpus / "markdown" / "C9999A00001" / "sections.jsonl",
        [
            section("C9999A00001", "0002", "2", "Rate.", act=LEVY_ACT),
            section("C9999A00001", "0003", "6", "Base.", act=LEVY_ACT),
            section("C9999A00001", "0004", "7",
                    "Under section 5-10(2)(a) and section 6٣ nothing changes.",
                    act=LEVY_ACT, heading="7 Use"),
        ],
    )

    assert call("find_citing_provisions", row_id=cited)["cited_by"] == []


def test_a_dictionary_citation_of_a_section_is_listed(corpus):
    result = call("find_citing_provisions", row_id="C9999A00004:0004:4")

    assert result["provision"] == {
        "row_id": "C9999A00004:0004:4", "register_id": "C9999A00004",
        "act": "Synthetic Glossary Act 2099", "section": "4",
        "heading": "4 Extended definition of covered body",
    }
    [entry] = result["cited_by"]
    assert entry["row_id"] == "C9999A00004:0002:2"
    assert entry["citations"] == ["section 4"]
    assert entry["text"] == "entity has the meaning given by section 4."
    assert entry["register_page"] == "https://example.invalid/C9999A00004/latest"
    assert result["has_more"] is False
    assert "not proof that nothing cites" in result["notice"]
    assert result["corpus"]["licence"] == "CC BY 4.0"


def test_citations_follow_the_title_they_name(citing):
    result = call("find_citing_provisions", row_id=TARGET)

    assert [(entry["register_id"], entry["section"], entry["citations"])
            for entry in result["cited_by"]] == [
        ("C9999A00001", "5-15", ["subsection 5-10(2)"]),
        ("C9999A00001", "5-20", ["s 5‑10"]),
        ("C9999A00001", "5-30", ["section 5-10 of this Act"]),
        ("C9999A00009", "3", ["section 5-10 of the Synthetic Levy Act 2099"]),
    ]
    # The paragraph holding the citation, not the whole provision.
    assert result["cited_by"][0]["text"] == (
        "The rate in subsection 5-10(2) and section 5-15 applies."
    )


def test_a_superseded_citing_provision_is_left_out_unless_asked_for(citing):
    result = call("find_citing_provisions", row_id=TARGET, in_force_only=False)

    former = [entry for entry in result["cited_by"] if entry["section"] == "5-25"]
    assert len(former) == 1
    assert former[0]["version_is_current"] is False
    assert former[0]["caveats"]


def test_the_limit_reports_more(citing):
    result = call("find_citing_provisions", row_id=TARGET, limit=1)

    assert [entry["section"] for entry in result["cited_by"]] == ["5-15"]
    assert result["has_more"] is True


def test_a_section_cited_by_nothing_returns_an_empty_list(citing):
    result = call("find_citing_provisions", row_id="C9999A00001:0001:1")

    assert result["cited_by"] == []
    assert result["has_more"] is False


@pytest.mark.parametrize("row_id", ["not a row id", "C9999A00009:0004:Schedule 1",
                                    "C9999A00077:0001:1"])
def test_a_row_that_cannot_be_cited_by_number_is_refused(citing, row_id):
    with pytest.raises(ToolError):
        call("find_citing_provisions", row_id=row_id)


def test_an_index_that_is_not_utf8_stops_the_scan(citing):
    broken = citing / "markdown" / "C9999A00008" / "sections.jsonl"
    broken.parent.mkdir(parents=True)
    broken.write_bytes(b'{"text": "section 5-10 \xff"}\n')

    with pytest.raises(ToolError, match="UTF-8"):
        call("find_citing_provisions", row_id=TARGET)
