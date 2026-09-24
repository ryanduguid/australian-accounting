"""The command answers --help and --version instead of waiting silently for a client."""

import pytest

from aus_accounting_mcp import __version__, cli


def test_version_prints_and_exits_without_starting_the_server(capsys, monkeypatch):
    monkeypatch.setattr(cli, "run_stdio", lambda: pytest.fail("server started"))

    with pytest.raises(SystemExit) as stopped:
        cli.main(["--version"])

    assert stopped.value.code == 0
    assert capsys.readouterr().out.strip() == f"aus-accounting-mcp {__version__}"


def test_help_names_the_retrieval_folders(capsys, monkeypatch):
    monkeypatch.setattr(cli, "run_stdio", lambda: pytest.fail("server started"))

    with pytest.raises(SystemExit) as stopped:
        cli.main(["--help"])

    out = capsys.readouterr().out
    assert stopped.value.code == 0
    assert "AUS_ACCOUNTING_LIBRARY_ROOT" in out and "AUS_ACCOUNTING_CORPUS_ROOT" in out


def test_no_arguments_starts_the_server(monkeypatch):
    started = []
    monkeypatch.setattr(cli, "run_stdio", lambda: started.append(True))

    cli.main([])

    assert started == [True]


def test_an_unknown_argument_is_refused(monkeypatch):
    monkeypatch.setattr(cli, "run_stdio", lambda: pytest.fail("server started"))

    with pytest.raises(SystemExit) as stopped:
        cli.main(["--bogus"])

    assert stopped.value.code == 2
