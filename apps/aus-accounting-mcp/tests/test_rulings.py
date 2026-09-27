"""Synthetic checks for cited retrieval from configured ATO rulings runs.

Every run here is fabricated in the shape the corpus builder's rulings stage writes.
The real runs are the operator's own folder and never enter this repository.
"""

import asyncio
import json
import os
from pathlib import Path

import pytest
from jsonschema import Draft202012Validator
from mcp.server.mcpserver.exceptions import ToolError

import synthetic_rulings as fixture
from aus_accounting_mcp import rulings as rulings_module
from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import mcp

SCHEMAS = {tool.name: tool.output_schema for tool in asyncio.run(mcp.list_tools())}


def call(name, **arguments):
    result = asyncio.run(mcp.call_tool(name, arguments))
    payload = result.structured_content
    Draft202012Validator(SCHEMAS[name]).validate(payload)
    return payload


@pytest.fixture
def runs(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_RULINGS_ROOT", str(tmp_path))
    return fixture.build(tmp_path)


def _rewrite_manifest(run: Path, change) -> None:
    path = run / "manifest.json"
    manifest = json.loads(path.read_text(encoding="utf-8"))
    change(manifest)
    path.write_text(json.dumps(manifest), encoding="utf-8")


def _rewrite_rows(run: Path, change) -> None:
    path = run / "rulings.jsonl"
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
    rows = change(rows)
    path.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")


def test_search_returns_a_citable_paragraph_from_the_serving_run(runs):
    result = call("search_ato_rulings", query="synthetic reimbursement agreement")
    top = result["matches"][0]

    assert top["row_ref"] == f"{fixture.NEWER_RUN}|{fixture.RULING}|2"
    assert top["paragraph"] == "2"
    assert top["family"] == "Taxation Ruling"
    assert top["serving"] is True
    assert top["fetched_on"] == "2099-02-01"
    assert top["source_url"].endswith(fixture.RULING)
    assert "OLDER COPY" not in json.dumps(result["matches"])
    corpus = result["corpus"]
    assert corpus["runs_configured"] == 2
    assert corpus["documents_serving"] == 4
    assert corpus["documents_withheld"] == 1
    assert corpus["reuse_notices"][0]["notice"] == fixture.NOTICE
    assert "endorses" in corpus["non_endorsement"]
    assert "not been assessed" in result["notice"]


def test_a_later_exclusion_withholds_the_earlier_copy(runs):
    assert not call("search_ato_rulings", query="synthetic withheld expense")["matches"]

    # The older run holds the ruling (lines 1-3), the guideline (4-5), then this.
    # A row_ref kept from before the exclusion must not reach the withheld text.
    older = f"{fixture.OLDER_RUN}|{fixture.EDITED_OLD}|6"
    with pytest.raises(ToolError, match="excluded that document"):
        call("read_ato_ruling", row_ref=older)


def test_a_replaced_copy_stays_readable_and_says_so(runs):
    excerpt = call("read_ato_ruling", row_ref=f"{fixture.OLDER_RUN}|{fixture.RULING}|2")

    assert excerpt["paragraph"]["text"].startswith("OLDER COPY")
    assert excerpt["paragraph"]["serving"] is False
    assert any("newer copy" in caveat for caveat in excerpt["paragraph"]["caveats"])


def test_edited_private_advice_always_carries_the_reliance_caveat(runs):
    result = call("search_ato_rulings", query="synthetic boat")
    assert result["matches"]
    for match in result["matches"]:
        assert match["family"] == "Edited version of private advice"
        assert any("cannot be relied on by anyone" in caveat for caveat in match["caveats"])
        assert any("not an ATO pinpoint" in caveat for caveat in match["caveats"])

    read = call("read_ato_ruling", row_ref=result["matches"][0]["row_ref"], neighbours=1)
    for paragraph in [read["paragraph"], *read["before"], *read["after"]]:
        assert any("cannot be relied on by anyone" in caveat for caveat in paragraph["caveats"])


def test_family_filter_and_display_order(runs):
    both = call("search_ato_rulings", query="synthetic reimbursement")
    families = [match["family"] for match in both["matches"]]
    assert "Practical Compliance Guideline" in families
    assert families.index("Taxation Ruling") < families.index("Practical Compliance Guideline")
    assert "not a statement of legal authority" in both["corpus"]["ranking"]

    only = call(
        "search_ato_rulings", query="synthetic reimbursement", family="compliance guideline"
    )
    assert {match["family"] for match in only["matches"]} == {"Practical Compliance Guideline"}


def test_a_word_only_in_the_address_does_not_match(runs):
    # Every row's source_url holds "docid", and the prefilter reads the raw line.
    result = call("search_ato_rulings", query="docid")
    assert result["matches"] == []
    assert "No match is not evidence" in result["notice"]


def test_paging_is_stable(runs):
    first = call("search_ato_rulings", query="synthetic", limit=2)
    assert first["has_more"] is True
    second = call("search_ato_rulings", query="synthetic", limit=2, offset=first["next_offset"])
    refs = [match["row_ref"] for match in first["matches"] + second["matches"]]
    assert len(refs) == len(set(refs)) == 4


def test_neighbours_come_only_from_the_cited_document(runs):
    # The statement's first paragraph follows the ruling's last in the same run.
    read = call(
        "read_ato_ruling", row_ref=f"{fixture.NEWER_RUN}|{fixture.STATEMENT}|4", neighbours=2
    )
    assert read["before"] == []
    assert [paragraph["paragraph"] for paragraph in read["after"]] == ["2"]
    assert read["after"][0]["docid"] == fixture.STATEMENT

    ruling = call(
        "read_ato_ruling", row_ref=f"{fixture.NEWER_RUN}|{fixture.RULING}|3", neighbours=5
    )
    assert [paragraph["paragraph"] for paragraph in ruling["before"]] == ["1", "2"]
    assert ruling["after"] == []


def test_a_long_paragraph_is_read_in_parts(runs):
    row_ref = f"{fixture.NEWER_RUN}|{fixture.STATEMENT}|5"
    first = call("read_ato_ruling", row_ref=row_ref)
    assert first["next_start"] == 12000
    assert len(first["paragraph"]["text"]) == 12000
    assert first["paragraph"]["total_chars"] == fixture.LONG_TEXT_CHARS
    rest = call("read_ato_ruling", row_ref=row_ref, start=first["next_start"])
    assert rest["next_start"] is None
    assert len(rest["paragraph"]["text"]) == fixture.LONG_TEXT_CHARS - 12000
    with pytest.raises(ToolError, match="past the end"):
        call("read_ato_ruling", row_ref=row_ref, start=fixture.LONG_TEXT_CHARS)

    search = call("search_ato_rulings", query="ATO view")
    hit = next(match for match in search["matches"] if match["row_ref"] == row_ref)
    assert len(hit["text"]) == rulings_module.SEARCH_TEXT_CHARS
    assert any("truncated" in caveat for caveat in hit["caveats"])


@pytest.mark.parametrize(
    "row_ref, message",
    [
        ("not a ref", "row_ref returned by search"),
        (f"missing-run|{fixture.RULING}|1", "No rulings run with that name"),
        (f"{fixture.NEWER_RUN}|{fixture.GUIDELINE}|1", "holds no document with that docid"),
        (f"{fixture.NEWER_RUN}|{fixture.RULING}|4", "belongs to another document"),
        (f"{fixture.NEWER_RUN}|{fixture.RULING}|999", "No paragraph at that row_ref"),
        (f"{fixture.NEWER_RUN}|../../etc|1", "row_ref returned by search"),
    ],
)
def test_bad_row_refs_are_refused(runs, row_ref, message):
    with pytest.raises(ToolError, match=message):
        call("read_ato_ruling", row_ref=row_ref)


def test_a_single_run_can_be_configured_directly(runs, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_RULINGS_ROOT", str(runs / fixture.OLDER_RUN))
    result = call("search_ato_rulings", query="OLDER COPY")
    assert result["matches"][0]["serving"] is True
    assert result["corpus"]["runs_configured"] == 1


def test_a_same_day_tie_goes_to_the_later_folder_name(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_RULINGS_ROOT", str(tmp_path))
    for name, text in (("run-b", "second fetch wording"), ("run-a", "first fetch wording")):
        fixture.write_run(
            tmp_path / name, "2099-03-01", [fixture.ruling("2099-03-01", second=text)]
        )
    [match] = call("search_ato_rulings", query="fetch wording")["matches"]
    assert match["run"] == "run-b"


def test_an_unconfigured_folder_is_an_input_error(monkeypatch):
    monkeypatch.delenv("AUS_ACCOUNTING_RULINGS_ROOT", raising=False)
    with pytest.raises(InputError, match="AUS_ACCOUNTING_RULINGS_ROOT"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_a_folder_without_runs_is_refused(tmp_path, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_RULINGS_ROOT", str(tmp_path))
    (tmp_path / "notes").mkdir()
    with pytest.raises(InputError, match="No rulings runs"):
        rulings_module.search_rulings("synthetic", 5, 0)


def _older(runs: Path) -> Path:
    return runs / fixture.OLDER_RUN


def _append_case_variant(manifest) -> None:
    """A second record for the first document, spelled in lower case with a slash."""
    first = manifest["documents"][0]
    manifest["documents"].append(dict(first, docid=first["docid"].lower() + "/"))


@pytest.mark.parametrize(
    "change, message",
    [
        (lambda m: m.update(stage="other"), "stage must be 'rulings'"),
        (lambda m: m.update(fetched_on="01/01/2099"), "fetched_on must be"),
        (lambda m: m.update(rows=m["rows"] + 1), "rows does not equal"),
        (lambda m: m["documents"][0].update(fetched_on="2099-01-02"), "different date"),
        (lambda m: m["documents"][0].update(family="Opinion"), "unsupported family"),
        (_append_case_variant, "repeats an earlier docid"),
        (lambda m: m["excluded"].append({"docid": m["documents"][0]["docid"], "reason": "x"}),
         "also accepted"),
    ],
)
def test_an_inconsistent_manifest_is_refused(runs, change, message):
    _rewrite_manifest(_older(runs), change)
    with pytest.raises(InputError, match=message):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_rows_must_agree_with_their_manifest(runs):
    def undeclared(rows):
        return rows + [dict(rows[0], docid="TXR/TR20999/NAT/ATO/00001")]

    _rewrite_rows(_older(runs), undeclared)
    with pytest.raises(InputError, match="does not list"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_a_row_with_another_hash_is_refused(runs):
    _rewrite_rows(_older(runs), lambda rows: [dict(rows[0], source_sha256="0" * 64), *rows[1:]])
    with pytest.raises(InputError, match="disagrees with its manifest"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_a_missing_row_is_refused(runs):
    _rewrite_rows(_older(runs), lambda rows: rows[:-1])
    with pytest.raises(InputError, match="holds 0 rows"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_a_duplicate_manifest_key_is_refused(runs):
    path = _older(runs) / "manifest.json"
    text = path.read_text(encoding="utf-8")
    path.write_text(text.replace('"stage": "rulings"', '"stage": "rulings", "stage": "rulings"', 1),
                    encoding="utf-8")
    with pytest.raises(InputError, match="duplicate keys"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_a_run_missing_its_rows_is_refused(runs):
    (_older(runs) / "rulings.jsonl").unlink()
    with pytest.raises(InputError, match="must hold rulings.jsonl"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_an_oversized_folder_is_refused_before_anything_is_parsed(runs, monkeypatch):
    def parsed(*_):
        raise AssertionError("a manifest was parsed before the size check")

    monkeypatch.setattr(rulings_module, "MAX_CORPUS_BYTES", 10)
    monkeypatch.setattr(rulings_module, "_manifest", parsed)
    with pytest.raises(InputError, match="MB of rows; configure a smaller folder"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_an_oversized_manifest_is_refused(runs, monkeypatch):
    monkeypatch.setattr(rulings_module, "MAX_MANIFEST_BYTES", 10)
    with pytest.raises(InputError, match="manifest over 10 bytes"):
        rulings_module.search_rulings("synthetic", 5, 0)


def test_a_linked_run_is_not_followed(runs, tmp_path_factory):
    outside = fixture.write_run(
        tmp_path_factory.mktemp("outside") / "run-2099-09-01", "2099-09-01",
        [fixture.ruling("2099-09-01", second="linked outside wording")],
    )
    try:
        os.symlink(outside, runs / "run-2099-09-01", target_is_directory=True)
    except (OSError, NotImplementedError) as exc:  # pragma: no cover - platform dependent
        pytest.skip(f"cannot create a directory link here: {exc}")
    assert not rulings_module.search_rulings("linked outside wording", 5, 0)["matches"]


def test_a_changed_run_is_validated_again(runs):
    rulings_module.search_rulings("synthetic", 5, 0)
    _rewrite_manifest(_older(runs), lambda m: m.update(stage="other"))
    with pytest.raises(InputError, match="stage must be"):
        rulings_module.search_rulings("synthetic", 5, 0)
