# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""`aus-accounting-mcp doctor` says what is installed and exits non-zero on a broken folder."""

import pytest

import synthetic_corpus
import synthetic_rulings
from aus_accounting_mcp import __version__, cli, corpus, doctor
from aus_accounting_mcp.errors import InputError

ROOTS = (
    "AUS_ACCOUNTING_LIBRARY_ROOT", "AUS_ACCOUNTING_CORPUS_ROOT", "AUS_ACCOUNTING_RULINGS_ROOT",
)


@pytest.fixture(autouse=True)
def no_roots(monkeypatch):
    for variable in ROOTS:
        monkeypatch.delenv(variable, raising=False)
    monkeypatch.setattr(cli, "run_stdio", lambda: pytest.fail("server started"))


def run(capsys):
    with pytest.raises(SystemExit) as stopped:
        cli.main(["doctor"])
    return stopped.value.code, capsys.readouterr().out


def test_with_no_folders_it_reports_the_install_and_exits_zero(capsys):
    code, out = run(capsys)

    assert code == 0
    assert out.startswith(f"aus-accounting-mcp {__version__}\n")
    assert "not that a figure is current" in out
    assert "  reviewed_ato_rates: 2025-07-01 to 2027-06-30\n" in out
    assert "  ato_small_business_benchmarks:2023-24: 2023-07-01 to 2024-06-30\n" in out
    assert "  gst_review_rules: UNKNOWN\n" in out
    for variable in ROOTS:
        assert f"  {variable}: not set\n" in out
    assert "broken" not in out


def test_an_empty_variable_counts_as_not_set(capsys, monkeypatch):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", "")

    code, out = run(capsys)

    assert code == 0
    assert "  AUS_ACCOUNTING_CORPUS_ROOT: not set\n" in out


def test_a_working_library_reports_its_rows(capsys, monkeypatch, tmp_path):
    (tmp_path / "a.md").write_text("# One\ntwo\n", encoding="utf-8")
    (tmp_path / "b.md").write_text("three\n", encoding="utf-8")
    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 0
    assert f"AUS_ACCOUNTING_LIBRARY_ROOT: OK: {tmp_path}: 3 lines in 2 Markdown files\n" in out


def test_a_working_corpus_reports_its_rows(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    rows = sum(
        len(path.read_text(encoding="utf-8").splitlines())
        for path in tmp_path.glob("markdown/*/sections.jsonl")
    )
    rates = len((tmp_path / "rates" / "rates.jsonl").read_text(encoding="utf-8").splitlines())
    titles = len(list(tmp_path.glob("markdown/*/sections.jsonl")))
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 0
    assert (f"AUS_ACCOUNTING_CORPUS_ROOT: OK: {tmp_path}: {rows} section rows in "
            f"{titles} title indexes, {rates} rate rows\n") in out


def assert_rates_are_broken(capsys):
    code, out = run(capsys)

    assert code == 1
    assert "it has no usable rates/rates.jsonl, so search_tax_rates will fail." in out
    with pytest.raises(InputError, match="no rates/rates.jsonl"):
        corpus.search_rates("levy", 5, 0)


def test_a_corpus_without_a_rates_index_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    (tmp_path / "rates" / "rates.jsonl").unlink()
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    assert_rates_are_broken(capsys)


def test_a_linked_rates_index_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))
    monkeypatch.setattr(corpus, "_linked", lambda path: path.name == "rates.jsonl")

    assert_rates_are_broken(capsys)


def test_working_rulings_report_the_rows_they_serve(capsys, monkeypatch, tmp_path):
    synthetic_rulings.build(tmp_path)
    monkeypatch.setenv("AUS_ACCOUNTING_RULINGS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 0
    # The older run's copy of a re-fetched ruling is not served, so it is not counted.
    assert "paragraph rows from" in out
    assert "documents withheld\n" in out
    served = out.split("AUS_ACCOUNTING_RULINGS_ROOT: OK: ")[1].split(": ", 1)[1].split()[0]
    held = sum(
        len(path.read_text(encoding="utf-8").splitlines())
        for path in tmp_path.glob("*/rulings.jsonl")
    )
    assert 0 < int(served) < held


@pytest.mark.parametrize("variable", ROOTS)
def test_a_missing_folder_is_broken_and_exits_one(capsys, monkeypatch, tmp_path, variable):
    missing = tmp_path / "gone"
    monkeypatch.setenv(variable, str(missing))

    code, out = run(capsys)

    assert code == 1
    assert f"{variable}: BROKEN: {missing}: " in out
    assert "\n1 configured folder is broken.\n" in out
    # The MCP tools' words to a model read wrongly on a person's terminal.
    assert "do not retry" not in out


def test_every_broken_folder_is_reported(capsys, monkeypatch, tmp_path):
    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path / "gone"))
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path / "also-gone"))

    code, out = run(capsys)

    assert code == 1
    assert out.count("BROKEN") == 2
    assert "\n2 configured folders are broken.\n" in out


def test_a_library_with_no_markdown_is_broken(capsys, monkeypatch, tmp_path):
    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "it holds no rows" in out


def test_a_library_file_search_would_skip_is_broken(capsys, monkeypatch, tmp_path):
    (tmp_path / "good.md").write_text("fine\n", encoding="utf-8")
    (tmp_path / "bad.md").write_bytes(b"\xff\xfe not utf-8")
    monkeypatch.setenv("AUS_ACCOUNTING_LIBRARY_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "Markdown files search skips: 1, first bad.md." in out


def test_a_corpus_folder_of_the_wrong_shape_is_broken(capsys, monkeypatch, tmp_path):
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "must contain a markdown directory" in out


def test_a_malformed_corpus_row_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    index = next(tmp_path.glob("markdown/*/sections.jsonl"))
    with index.open("a", encoding="utf-8") as stream:
        stream.write("\nnot json\n")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "lines a search cannot return: 1." in out


@pytest.mark.parametrize("line", [
    '{"text": "no row id"}',
    '{"row_id": "C9999A00001:9:9"}',
    '{"row_id": 7, "text": "a number is no row id"}',
    '{"row_id": "C9999A00001:9:9", "text": null}',
])
def test_a_row_a_search_could_never_return_is_broken(capsys, monkeypatch, tmp_path, line):
    synthetic_corpus.build(tmp_path)
    with next(tmp_path.glob("markdown/*/sections.jsonl")).open("a", encoding="utf-8") as stream:
        stream.write(line + "\n")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "lines a search cannot return: 1." in out


def test_a_rates_row_needs_a_rate_id_and_content(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    with (tmp_path / "rates" / "rates.jsonl").open("a", encoding="utf-8") as stream:
        stream.write('{"rate_id": "r9"}\n{"content": "no id"}\n')
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "lines a search cannot return: 2." in out


def test_an_empty_rates_index_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    (tmp_path / "rates" / "rates.jsonl").write_text("", encoding="utf-8")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "its rates index holds no rows" in out


def test_a_title_folder_with_no_index_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    (tmp_path / "markdown" / "C9999A00077").mkdir()
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "entries under markdown holding no index a search reads: 1." in out


def test_a_file_beside_the_title_folders_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    (tmp_path / "markdown" / "stray.txt").write_text("not a title", encoding="utf-8")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "entries under markdown holding no index a search reads: 1." in out


def test_a_hidden_entry_is_not_a_title(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    (tmp_path / "markdown" / ".cache").mkdir()
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, _ = run(capsys)

    assert code == 0


def test_a_row_nested_past_the_limit_is_broken_not_a_crash(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    with next(tmp_path.glob("markdown/*/sections.jsonl")).open("a", encoding="utf-8") as stream:
        stream.write("[" * 1_000_000 + "\n")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "AUS_ACCOUNTING_CORPUS_ROOT: BROKEN" in out


def test_a_malformed_rates_row_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    with (tmp_path / "rates" / "rates.jsonl").open("a", encoding="utf-8") as stream:
        stream.write("not json\n")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "lines a search cannot return: 1." in out


def test_a_corpus_that_cannot_be_decoded_is_broken(capsys, monkeypatch, tmp_path):
    synthetic_corpus.build(tmp_path)
    next(tmp_path.glob("markdown/*/sections.jsonl")).write_bytes(b"\xff\xfe")
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "AUS_ACCOUNTING_CORPUS_ROOT: BROKEN" in out


def test_a_corpus_of_empty_indexes_is_broken(capsys, monkeypatch, tmp_path):
    index = tmp_path / "markdown" / "C9999A00001" / "sections.jsonl"
    index.parent.mkdir(parents=True)
    index.write_text("", encoding="utf-8")
    synthetic_corpus.write(tmp_path / "rates" / "rates.jsonl", [
        synthetic_corpus.rate("r1", "Levies", "A cap of $6.", amounts=["$6"], years=["2098-99"]),
    ])
    monkeypatch.setenv("AUS_ACCOUNTING_CORPUS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "it holds no rows" in out


def test_rulings_a_manifest_disagrees_with_are_broken(capsys, monkeypatch, tmp_path):
    synthetic_rulings.build(tmp_path)
    rows = next(tmp_path.glob("*/rulings.jsonl"))
    rows.write_text(rows.read_text(encoding="utf-8") + "not json\n", encoding="utf-8")
    monkeypatch.setenv("AUS_ACCOUNTING_RULINGS_ROOT", str(tmp_path))

    code, out = run(capsys)

    assert code == 1
    assert "has a malformed row" in out


def test_the_help_names_the_command(capsys):
    with pytest.raises(SystemExit) as stopped:
        cli.main(["--help"])

    assert stopped.value.code == 0
    out = capsys.readouterr().out
    assert "doctor" in out and doctor.HELP.split()[0] in out
