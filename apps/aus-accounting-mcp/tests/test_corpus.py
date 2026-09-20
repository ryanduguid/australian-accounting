"""Synthetic checks for cited retrieval from a configured legislation corpus.

Every row here is fabricated. The real corpus is the operator's own folder and never
enters this repository.
"""

import asyncio
import json
import os

import pytest
from jsonschema import Draft202012Validator
from mcp.server.mcpserver.exceptions import ToolError

import synthetic_corpus
from aus_accounting_mcp import corpus as corpus_module
from aus_accounting_mcp.server import mcp

SCHEMAS = {tool.name: tool.output_schema for tool in asyncio.run(mcp.list_tools())}
ATTRIBUTION = synthetic_corpus.ATTRIBUTION


def call(name, **arguments):
    result = asyncio.run(mcp.call_tool(name, arguments))
    payload = result.structured_content
    Draft202012Validator(SCHEMAS[name]).validate(payload)
    return payload


@pytest.fixture
def corpus(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    return synthetic_corpus.build(tmp_path)


def test_search_returns_a_traceable_citation(corpus):
    result = call("search_tax_legislation", query="synthetic levy rate")
    hit = next(match for match in result["matches"] if match["register_id"] == "C9999A00001")

    assert hit["row_id"] == "C9999A00001:0002:5-10"
    assert hit["act"] == "Synthetic Levy Act 2099"
    assert hit["section"] == "5-10"
    assert hit["compilation_number"] == "12"
    assert hit["compilation_date"] == "2098-07-01"
    assert hit["register_page"] == "https://example.invalid/C9999A00001/latest"
    assert hit["licence"] == "CC BY 4.0"
    assert hit["attribution"] == ATTRIBUTION
    assert "7%" in hit["text"]
    assert result["corpus"]["retrieved"] == "2099-01-01"
    assert "never" in result["notice"]


def test_a_superseded_compilation_is_left_out_unless_asked_for(corpus):
    assert not call("search_tax_legislation", query="not applied here")["matches"]

    result = call("search_tax_legislation", query="not applied here", in_force_only=False)
    hit = result["matches"][0]

    assert hit["version_is_current"] is False
    assert any("not the current version" in caveat for caveat in hit["caveats"])


def test_in_force_only_keeps_a_row_with_unrecorded_currency(corpus):
    path = corpus / "markdown" / "C9999A00001" / "sections.jsonl"
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
    del rows[1]["version_is_current"]
    synthetic_corpus.write(path, rows)

    hit = call("search_tax_legislation", query="synthetic levy rate")["matches"][0]

    assert hit["row_id"] == "C9999A00001:0002:5-10"
    assert hit["version_is_current"] is None


def test_search_matches_every_word_and_filters_by_act(corpus):
    assert not call("search_tax_legislation", query="levy unrelatedword")["matches"]

    filtered = call(
        "search_tax_legislation", query="synthetic levy rate", act="charge act", in_force_only=False
    )

    assert [match["register_id"] for match in filtered["matches"]] == ["C9999A00002"]


def test_a_word_only_in_the_stored_metadata_is_not_a_match(corpus):
    # "attribution" appears in every raw index line as a field name, and the
    # attribution text itself is not searchable content.
    assert not call("search_tax_legislation", query="attribution")["matches"]
    assert not call("search_tax_legislation", query="synthetic register")["matches"]


def test_paging_continues_from_the_returned_offset(corpus):
    first = call("search_tax_legislation", query="synthetic", limit=2)

    assert len(first["matches"]) == 2
    assert first["has_more"] is True
    assert first["next_offset"] == 2

    second = call("search_tax_legislation", query="synthetic", limit=2, offset=first["next_offset"])

    assert [match["row_id"] for match in second["matches"]] != [
        match["row_id"] for match in first["matches"]
    ]


def test_search_truncates_a_long_provision_and_reports_the_whole_length(corpus):
    hit = call("search_tax_legislation", query="xxxx")["matches"][0]

    assert len(hit["text"]) == 1200
    assert hit["total_chars"] == synthetic_corpus.LONG_TEXT_CHARS
    assert any("truncated" in caveat for caveat in hit["caveats"])


def test_read_returns_the_cited_provision_in_full(corpus):
    result = call("read_tax_legislation_section", row_id="C9999A00001:0003:5-15")

    assert result["section"]["text"] == "A synthetic levy exemption applies to a small entity."
    assert result["section"]["section"] == "5-15"
    assert result["corpus"]["licence"] == "CC BY 4.0"


def test_read_stops_at_the_character_bound(corpus):
    result = call("read_tax_legislation_section", row_id="C9999A00002:0002:3")

    assert len(result["section"]["text"]) == 12000
    assert result["section"]["total_chars"] == synthetic_corpus.LONG_TEXT_CHARS


@pytest.mark.parametrize(
    "row_id",
    ["C9999A00001:0002:9-99", "C9999A00003:0001:1", "../../etc:0001:1", "not-a-row-id"],
)
def test_read_refuses_an_unknown_or_unsafe_row_id(corpus, row_id):
    with pytest.raises(ToolError):
        call("read_tax_legislation_section", row_id=row_id)


def test_rates_keep_the_amounts_years_and_provision(corpus):
    result = call("search_tax_rates", query="cap")
    hit = result["matches"][0]

    assert hit["rate_id"] == "R00002"
    assert hit["amounts"] == ["$30,000"]
    assert hit["years"] == ["2098-99"]
    assert hit["section"] == "9-20"
    assert hit["register_page"] == "https://example.invalid/C9999A00001/latest"


def test_rates_filter_by_stated_year(corpus):
    dated = call("search_tax_rates", query="cap", year="2098-99")["matches"]
    assert [row["rate_id"] for row in dated] == ["R00002"]
    assert not call("search_tax_rates", query="cap", year="2097-98")["matches"]
    # A blank year is no filter, the same as omitting it.
    assert call("search_tax_rates", query="cap", year=" ")["matches"][0]["rate_id"] == "R00002"


def test_rates_filter_by_topic(corpus):
    assert not call("search_tax_rates", query="cap", topic="levies")["matches"]
    assert call("search_tax_rates", query="rate", topic="levies")["matches"][0][
        "rate_id"
    ] == "R00001"


def test_unconfigured_corpus_is_an_input_error(monkeypatch):
    monkeypatch.delenv("AUS_ACCOUNTING_CORPUS_ROOT", raising=False)
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="levy")


def test_a_folder_without_title_indexes_is_refused(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="levy")

    (tmp_path / "markdown").mkdir()
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="levy")


def test_a_corpus_without_rates_refuses_the_rate_search(corpus):
    (corpus / "rates" / "rates.jsonl").unlink()
    with pytest.raises(ToolError):
        call("search_tax_rates", query="cap")


def test_unknown_arguments_are_rejected(corpus):
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="levy", unknown=True)


def test_a_root_that_is_not_a_folder_is_refused(tmp_path, monkeypatch):
    target = tmp_path / "corpus.txt"
    target.write_text("not a folder", encoding="utf-8")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(target))
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="levy")


def test_a_query_without_a_word_is_refused(corpus):
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="   ")
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="levy", act="  ")


def test_an_unparsable_row_is_skipped_rather_than_failing_the_search(corpus):
    index = corpus / "markdown" / "C9999A00001" / "sections.jsonl"
    index.write_text(
        '{not json at all: synthetic levy rate\n'
        '["synthetic", "levy", "rate"]\n' + index.read_text(encoding="utf-8"),
        encoding="utf-8",
    )

    result = call("search_tax_legislation", query="synthetic levy rate")

    assert result["matches"][0]["row_id"] == "C9999A00001:0002:5-10"


def test_a_title_folder_without_an_index_is_skipped(corpus):
    (corpus / "markdown" / "C9999A00003").mkdir()

    assert call("search_tax_legislation", query="synthetic levy rate")["matches"]


def test_a_corpus_without_a_manifest_still_cites_each_row(corpus):
    (corpus / "sources.json").unlink()

    result = call("search_tax_legislation", query="synthetic levy rate")

    assert result["corpus"] == {}
    assert result["matches"][0]["attribution"] == ATTRIBUTION


def test_an_unreadable_manifest_is_not_fatal(corpus):
    (corpus / "sources.json").write_text("{ broken", encoding="utf-8")

    assert call("search_tax_legislation", query="synthetic levy rate")["corpus"] == {}


def test_a_corpus_past_the_scan_bounds_is_refused(corpus, monkeypatch):
    monkeypatch.setattr(corpus_module, "MAX_CORPUS_BYTES", 10)
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="synthetic levy rate")

    monkeypatch.setattr(corpus_module, "MAX_INDEX_FILES", 1)
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="synthetic levy rate")


def test_the_act_filter_is_not_satisfied_by_body_text(corpus):
    # "exemption" is in one provision's text, not in any Act name, so the filter
    # must reject the row even though the stored line holds both words.
    assert not call("search_tax_legislation", query="exemption", act="levy exemption")["matches"]


def test_the_topic_filter_is_not_satisfied_by_row_content(corpus):
    assert not call("search_tax_rates", query="rate", topic="levies rate")["matches"]


def test_the_offset_boundary_asks_for_a_narrower_query(corpus, monkeypatch):
    monkeypatch.setattr(corpus_module, "MAX_OFFSET", 1)

    page = call("search_tax_legislation", query="synthetic", limit=2)

    assert page["has_more"] is True
    assert page["next_offset"] is None
    assert "narrow the query" in page["notice"]


ACCENTED_ROW_ID = "C9999A00001:0004:5-20\u00e9"


@pytest.mark.parametrize("ensure_ascii", [True, False])
def test_escaped_and_literal_indexes_search_and_read_alike(tmp_path, monkeypatch, ensure_ascii):
    """An index written with ensure_ascii=True hid its accented text from the raw prefilter."""
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    synthetic_corpus.build(tmp_path)
    index = tmp_path / "markdown" / "C9999A00001" / "sections.jsonl"
    rows = [json.loads(line) for line in index.read_text(encoding="utf-8").splitlines()]
    rows.append(synthetic_corpus.section(
        "C9999A00001", "0004", "5-20\u00e9",
        "A synthetic d\u00e9duction applies to the N\u00fa\u00f1ez levy.",
        act=synthetic_corpus.LEVY_ACT,
    ))
    index.write_text(
        "".join(json.dumps(row, ensure_ascii=ensure_ascii) + "\n" for row in rows),
        encoding="utf-8",
    )
    assert ("\\u00e9" in index.read_text(encoding="utf-8")) is ensure_ascii

    result = call("search_tax_legislation", query="d\u00e9duction")
    assert [match["row_id"] for match in result["matches"]] == [ACCENTED_ROW_ID]
    read = call("read_tax_legislation_section", row_id=ACCENTED_ROW_ID)
    assert "N\u00fa\u00f1ez" in read["section"]["text"]


def test_an_escaped_solidus_row_id_reads_back_without_neighbours(tmp_path, monkeypatch):
    """JSON may spell "/" as "\\/"; search returned such a row_id and a default read lost it."""
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    synthetic_corpus.build(tmp_path)
    index = tmp_path / "markdown" / "C9999A00001" / "sections.jsonl"
    row = synthetic_corpus.section(
        "C9999A00001", "0004", "5/10", "A synthetic apportionment applies.",
        act=synthetic_corpus.LEVY_ACT,
    )
    line = json.dumps(row, ensure_ascii=True).replace("/", "\\/") + "\n"
    assert "5\\/10" in line and "\\u" not in line
    with index.open("a", encoding="utf-8") as stream:
        stream.write(line)

    result = call("search_tax_legislation", query="apportionment")
    assert [match["row_id"] for match in result["matches"]] == ["C9999A00001:0004:5/10"]
    read = call("read_tax_legislation_section", row_id="C9999A00001:0004:5/10")
    assert read["section"]["section"] == "5/10"
    assert read["before"] == [] and read["after"] == []


def test_a_linked_markdown_directory_is_refused_by_read_as_well_as_search(tmp_path, monkeypatch):
    """A direct read followed a link to markdown that search had already refused."""
    real = synthetic_corpus.build(tmp_path / "real")
    root = tmp_path / "root"
    root.mkdir()
    try:
        os.symlink(real / "markdown", root / "markdown", target_is_directory=True)
    except (OSError, NotImplementedError) as exc:  # pragma: no cover - no symlink privilege
        pytest.skip(f"symbolic links unavailable: {exc}")
    (root / "sources.json").write_bytes((real / "sources.json").read_bytes())
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(root))

    with pytest.raises(ToolError):
        call("read_tax_legislation_section", row_id="C9999A00001:0003:5-15")
    with pytest.raises(ToolError):
        call("search_tax_legislation", query="synthetic levy rate")


def test_read_widens_to_the_provisions_either_side(corpus):
    alone = call("read_tax_legislation_section", row_id="C9999A00001:0002:5-10")

    assert alone["before"] == [] and alone["after"] == []

    widened = call("read_tax_legislation_section", row_id="C9999A00001:0002:5-10", neighbours=1)

    assert [row["row_id"] for row in widened["before"]] == ["C9999A00001:0001:1"]
    assert [row["row_id"] for row in widened["after"]] == ["C9999A00001:0003:5-15"]
    assert widened["after"][0]["text"] == "A synthetic levy exemption applies to a small entity."
    assert widened["after"][0]["attribution"] == ATTRIBUTION
    assert widened["section"]["row_id"] == "C9999A00001:0002:5-10"


def test_neighbours_stop_at_the_edges_of_the_title(corpus):
    result = call("read_tax_legislation_section", row_id="C9999A00001:0001:1", neighbours=5)

    assert result["before"] == []
    assert [row["row_id"] for row in result["after"]] == [
        "C9999A00001:0002:5-10", "C9999A00001:0003:5-15"
    ]


def test_a_neighbour_is_truncated_like_a_search_match(corpus):
    result = call("read_tax_legislation_section", row_id="C9999A00002:0001:2", neighbours=1)
    neighbour = result["after"][0]

    assert len(neighbour["text"]) == 1200
    assert neighbour["total_chars"] == synthetic_corpus.LONG_TEXT_CHARS
    assert any("truncated" in caveat for caveat in neighbour["caveats"])


@pytest.mark.parametrize("neighbours", [-1, 6, "1"])
def test_neighbours_outside_the_bound_are_rejected(corpus, neighbours):
    with pytest.raises(ToolError):
        call("read_tax_legislation_section", row_id="C9999A00001:0002:5-10",
             neighbours=neighbours)


def test_a_definition_carries_its_entry_and_the_dictionary_citation(corpus):
    result = call("define_tax_term", term="small entity")
    first = result["definitions"][0]

    assert first["match"] == "exact"
    assert first["head"] == "small entity"
    assert first["text"] == (
        "small entity: an entity is a small entity for a year if:\n\n"
        "- (a) its *assessable amount for the year is below the cap; and\n\n"
        "- (b) it is not a *large entity."
    )
    assert first["row_id"] == "C9999A00004:0002:2"
    assert first["act"] == "Synthetic Glossary Act 2099"
    assert first["section"] == "2"
    assert first["compilation_date"] == "2098-07-01"
    assert first["register_page"] == "https://example.invalid/C9999A00004/latest"
    assert first["attribution"] == ATTRIBUTION
    assert [(entry["head"], entry["match"]) for entry in result["definitions"]] == [
        ("small entity", "exact"), ("small entity cap, for a year", "partial")
    ]
    assert result["has_more"] is False
    assert "undefined" in result["notice"]
    assert result["corpus"]["licence"] == "CC BY 4.0"


def test_a_note_stays_with_the_definition_it_follows(corpus):
    result = call("define_tax_term", term="assessable amount")

    assert result["definitions"][0]["text"] == (
        "assessable amount means the amount an entity reports for the year.\n\n"
        "Note: The amount is reported on the approved form."
    )
    # "(2) A term used in a note ..." is a subsection, not a definition head.
    assert not call("define_tax_term", term="term used")["definitions"]


def test_lettered_conditions_stay_with_the_definition_they_complete(corpus):
    """"(a)" and "(b)" paragraphs were read as subsection labels and dropped without a caveat."""
    result = call("define_tax_term", term="qualifying receipt")

    assert result["definitions"][0]["text"] == (
        "qualifying receipt means a receipt if:\n\n"
        "(a) it is issued to the entity; and\n\n"
        "(b) it is dated in the year."
    )
    assert not call("define_tax_term", term="term used")["definitions"]


def test_a_qualified_head_and_a_non_breaking_hyphen_still_match_exactly(corpus):
    cap = call("define_tax_term", term="Small Entity Cap")["definitions"][0]
    tagged = call("define_tax_term", term="165-cc tagged asset")["definitions"][0]

    assert cap["match"] == "exact" and cap["head"] == "small entity cap, for a year"
    assert tagged["match"] == "exact" and tagged["head"] == "165\u2011CC tagged asset"


def test_a_superseded_dictionary_is_left_out_unless_asked_for(corpus):
    assert not call("define_tax_term", term="old term")["definitions"]

    hit = call("define_tax_term", term="old term", in_force_only=False)["definitions"][0]

    assert hit["match"] == "exact"
    assert hit["version_is_current"] is False
    assert any("not the current version" in caveat for caveat in hit["caveats"])


def test_definitions_filter_by_act_and_put_the_exact_match_first(corpus):
    assert not call("define_tax_term", term="entity", act="levy act")["definitions"]

    page = call("define_tax_term", term="entity", act="glossary act", limit=1)

    assert [entry["head"] for entry in page["definitions"]] == ["entity"]
    assert page["has_more"] is True


def test_a_word_outside_a_dictionary_section_is_not_a_definition(corpus):
    # Every Levy Act provision mentions the levy; none of them is a dictionary.
    assert not call("define_tax_term", term="levy")["definitions"]
    # An operative section headed "Extended definition of ..." is not a dictionary
    # either, so its opening sentence is not an entry for the words it contains.
    assert not call("define_tax_term", term="covered body")["definitions"]
    assert not call("define_tax_term", term="body")["definitions"]


def test_a_definition_beyond_the_partial_cap_is_reported_not_returned(corpus, monkeypatch):
    monkeypatch.setattr(corpus_module, "MAX_DEFINITIONS", 1)

    result = call("define_tax_term", term="entity", limit=20)

    assert [entry["match"] for entry in result["definitions"]] == ["exact", "partial"]
    assert result["has_more"] is True


def test_a_term_without_a_word_is_refused(corpus):
    with pytest.raises(ToolError):
        call("define_tax_term", term=" ")


def test_a_malformed_line_beside_the_cited_row_costs_no_neighbour_slot(corpus):
    index = corpus / "markdown" / "C9999A00001" / "sections.jsonl"
    rows = index.read_text(encoding="utf-8").splitlines()
    index.write_text(
        "\n".join([rows[0], "{not json", rows[1], '["not", "an", "object"]', rows[2]]) + "\n",
        encoding="utf-8",
    )

    result = call("read_tax_legislation_section", row_id="C9999A00001:0002:5-10", neighbours=1)

    assert [row["row_id"] for row in result["before"]] == ["C9999A00001:0001:1"]
    assert [row["row_id"] for row in result["after"]] == ["C9999A00001:0003:5-15"]


def test_a_definition_lookup_refuses_a_corpus_past_the_scan_bounds(corpus, monkeypatch):
    monkeypatch.setattr(corpus_module, "MAX_CORPUS_BYTES", 10)
    with pytest.raises(ToolError):
        call("define_tax_term", term="small entity")


def test_a_default_read_returns_no_context_even_past_an_escaped_row(corpus):
    # An index written with ensure_ascii=True spells a non-ASCII character as a JSON
    # escape, and such a line is parsed regardless of the row_id prefilter. That must
    # not turn a plain read into one with context nobody asked for.
    index = corpus / "markdown" / "C9999A00001" / "sections.jsonl"
    rows = [json.loads(line) for line in index.read_text(encoding="utf-8").splitlines()]
    rows.insert(2, synthetic_corpus.section("C9999A00001", "0002a", "5-12",
                                            "A r\u00e9sum\u00e9 of the levy.",
                                            act=synthetic_corpus.LEVY_ACT))
    index.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")

    plain = call("read_tax_legislation_section", row_id="C9999A00001:0002:5-10")
    widened = call("read_tax_legislation_section", row_id="C9999A00001:0002:5-10", neighbours=1)

    assert plain["before"] == [] and plain["after"] == []
    assert [row["row_id"] for row in widened["after"]] == ["C9999A00001:0002a:5-12"]
