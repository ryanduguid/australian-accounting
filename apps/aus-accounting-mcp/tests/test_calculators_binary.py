# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""The tax worksheets run only the pinned calculators executable, and refuse clearly without it."""

import asyncio
import hashlib
import json
import os
import subprocess
import threading
from pathlib import Path

import pytest
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp import freshness, resources
from aus_accounting_mcp.adapters import tax
from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import mcp

GST = {"kind": "gst", "scope_confirmed": True, "year": "2025-26",
       "amount": "1100", "gst_inclusive": True}
REAL = tax._executable()


def _gst() -> dict:
    result = asyncio.run(mcp.call_tool("calculate_tax_worksheet", {"facts": GST}))
    assert not result.is_error
    return result.structured_content


def _answer(stdout: str = "", returncode: int = 0, stderr: str = ""):
    """A stand-in for subprocess.run that gives every call the same reply."""
    def run(command, **_options):
        return subprocess.CompletedProcess(command, returncode, stdout, stderr)
    return run


@pytest.fixture
def missing(monkeypatch, tmp_path):
    monkeypatch.setenv(tax.BINARY_VARIABLE, str(tmp_path / "absent"))


def test_the_variable_is_used_before_path(monkeypatch, tmp_path) -> None:
    assert REAL, "the suite needs the calculators executable on PATH or in the variable"
    monkeypatch.setenv("PATH", str(tmp_path))
    monkeypatch.setenv(tax.BINARY_VARIABLE, REAL)
    assert _gst()["engine_version"] == tax.CALCULATORS_VERSION
    monkeypatch.delenv(tax.BINARY_VARIABLE)
    monkeypatch.setenv("PATH", str(Path(REAL).parent))
    assert _gst()["amounts"]["gst"] == "100.00"
    assert tax.reported_version() == tax.CALCULATORS_VERSION


def test_a_relative_variable_is_refused(monkeypatch, tmp_path) -> None:
    # Resolved by the operating system, a bare name could run a file in the working directory.
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv(tax.BINARY_VARIABLE, tax.CALCULATORS)
    with pytest.raises(ToolError, match=f"{tax.BINARY_VARIABLE} must be an absolute path"):
        asyncio.run(mcp.call_tool("calculate_tax_worksheet", {"facts": GST}))
    assert tax.reported_version() is None


def test_the_path_search_never_takes_the_working_directory(monkeypatch, tmp_path) -> None:
    # Windows' search, and shutil.which there, try the working directory before PATH.
    for name in (tax.CALCULATORS, f"{tax.CALCULATORS}.exe"):
        (tmp_path / name).write_text("")
        (tmp_path / name).chmod(0o755)
    monkeypatch.chdir(tmp_path)
    monkeypatch.delenv(tax.BINARY_VARIABLE, raising=False)
    monkeypatch.setenv("PATH", os.pathsep.join([str(tmp_path / "empty"), os.curdir, ""]))
    assert tax._executable() is None
    monkeypatch.setenv("PATH", os.pathsep.join([os.curdir, str(Path(REAL).parent)]))
    assert Path(tax._executable()).parent == Path(REAL).parent


def test_without_the_executable_the_worksheet_says_how_to_install_it(monkeypatch, tmp_path):
    monkeypatch.delenv(tax.BINARY_VARIABLE, raising=False)
    monkeypatch.setenv("PATH", str(tmp_path))
    with pytest.raises(ToolError, match=f"No {tax.CALCULATORS} executable was found") as caught:
        asyncio.run(mcp.call_tool("calculate_tax_worksheet", {"facts": GST}))
    assert tax.INSTALL in str(caught.value)
    assert tax.reported_version() is None


def test_without_the_executable_every_other_tool_and_resource_still_answers(missing) -> None:
    rate = asyncio.run(mcp.call_tool("get_div7a_benchmark_rate", {"year_of_income": "2025-26"}))
    assert not rate.is_error
    scope = resources.scope()["calculation_worksheets"]
    assert isinstance(scope, str) and tax.INSTALL in scope
    assert tax.INSTALL in resources.disclaimer()
    versions = {engine["distribution"]: engine["version"]
                for engine in resources.component_versions()["engines"]}
    assert versions[tax.CALCULATORS] is None
    report = freshness.data_freshness(tool="calculate_tax_worksheet")
    [dataset] = report["datasets"]
    assert dataset["dataset"] == "tax_worksheets"
    assert tax.INSTALL in dataset["description"]
    assert dataset["effective_periods"] == dataset["source_checked"] == "UNKNOWN"
    with pytest.raises(ToolError, match="did not run"):
        asyncio.run(mcp.call_tool("calculate_tax_worksheet", {"facts": GST}))


@pytest.mark.parametrize("reply", [
    "australian-tax-calculators 1.0.1\n",
    "australian-tax-calculators 1.0.0-rc.1\n",
    "australian-tax-calculators 1.0.0 extra\n",
    "australian-tax-calculators  1.0.0\n",
    "other-tool 1.0.0\n",
    "",
])
def test_any_other_version_is_refused(monkeypatch, reply) -> None:
    monkeypatch.setattr(tax.subprocess, "run", _answer(reply))
    with pytest.raises(tax.CalculatorsUnavailable, match="--version answered") as caught:
        tax.calculate(tax._FACTS.validate_python(GST))
    assert f"not '{tax.CALCULATORS} {tax.CALCULATORS_VERSION}'" in str(caught.value)


@pytest.fixture
def impostor(monkeypatch, tmp_path):
    """An executable that is no release, answering --version as the pinned one would."""
    binary = tmp_path / tax.CALCULATORS
    binary.write_bytes(b"not a release executable")
    monkeypatch.setenv(tax.BINARY_VARIABLE, str(binary))
    monkeypatch.delenv(tax.DIGEST_VARIABLE, raising=False)
    runs = []

    def run(command, **_options):
        runs.append(command)
        return subprocess.CompletedProcess(command, 0, f"{tax.CALCULATORS} "
                                           f"{tax.CALCULATORS_VERSION}\n", "")
    monkeypatch.setattr(tax.subprocess, "run", run)
    return binary, runs


def test_an_executable_with_another_digest_is_never_run(impostor) -> None:
    _, runs = impostor
    with pytest.raises(tax.CalculatorsUnavailable, match="neither a") as caught:
        tax.worksheet_catalogue()
    assert tax.INSTALL in str(caught.value)
    assert tax.reported_version() is None
    assert runs == []


def test_the_operators_digest_admits_a_source_build(monkeypatch, impostor) -> None:
    binary, runs = impostor
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    monkeypatch.setenv(tax.DIGEST_VARIABLE, digest)
    assert tax._binary() == str(binary)
    monkeypatch.setenv(tax.DIGEST_VARIABLE, digest.upper())
    assert tax.reported_version() == tax.CALCULATORS_VERSION
    assert len(runs) == 2
    for named in (digest[:-1], f" {digest}", f"{digest[:-1]}g"):
        monkeypatch.setenv(tax.DIGEST_VARIABLE, named)
        with pytest.raises(tax.CalculatorsUnavailable, match="64 hexadecimal digits"):
            tax._binary()
    assert len(runs) == 2


def test_a_file_larger_than_any_build_is_refused_unread(monkeypatch, impostor) -> None:
    binary, runs = impostor
    monkeypatch.setenv(tax.DIGEST_VARIABLE, hashlib.sha256(binary.read_bytes()).hexdigest())
    monkeypatch.setattr(tax, "MAX_BYTES", binary.stat().st_size - 1)
    with pytest.raises(tax.CalculatorsUnavailable, match="larger than any"):
        tax._binary()
    assert runs == []


@pytest.mark.skipif(not hasattr(os, "mkfifo"), reason="POSIX FIFOs only")
def test_a_fifo_on_path_is_refused_without_waiting_for_a_writer(monkeypatch, tmp_path) -> None:
    fifo = tmp_path / tax.CALCULATORS
    os.mkfifo(fifo, 0o700)
    monkeypatch.setenv(tax.BINARY_VARIABLE, str(fifo))
    # A writer after 5 seconds ends a read that waits for one, so a regression fails
    # here instead of hanging the suite.
    writer = threading.Timer(5, lambda: os.close(os.open(fifo, os.O_WRONLY)))
    writer.daemon = True
    writer.start()
    try:
        with pytest.raises(tax.CalculatorsUnavailable, match="is not a regular file"):
            tax._trusted(str(fifo))
        assert writer.is_alive()
    finally:
        writer.cancel()


def test_a_version_that_exits_nonzero_is_refused(monkeypatch) -> None:
    monkeypatch.setattr(tax.subprocess, "run", _answer("australian-tax-calculators 1.0.0\n", 1))
    with pytest.raises(tax.CalculatorsUnavailable, match="--version answered"):
        tax.worksheet_catalogue()
    assert tax.reported_version() is None


def test_the_reported_version_is_the_executables_own(monkeypatch) -> None:
    monkeypatch.setattr(tax.subprocess, "run", _answer("australian-tax-calculators 0.9.0\n"))
    assert tax.reported_version() == "0.9.0"
    monkeypatch.setattr(tax.subprocess, "run", _answer("australian-tax-calculators\n"))
    assert tax.reported_version() is None


def _versioned(reply: subprocess.CompletedProcess):
    """--version answers correctly; every other command gets `reply`."""
    def run(command, **_options):
        if command[1:] == ["--version"]:
            return subprocess.CompletedProcess(command, 0, f"{tax.CALCULATORS} "
                                               f"{tax.CALCULATORS_VERSION}\n", "")
        return reply
    return run


@pytest.mark.parametrize("returncode,stdout", [
    (1, ""), (2, ""), (2, '{"ok": true}'), (2, '{"ok": false}'), (0, "not json"), (0, "[]"),
    (101, '{"ok": false}'),
])
def test_a_reply_that_is_neither_a_result_nor_a_refusal_is_unavailability(
        monkeypatch, returncode, stdout) -> None:
    reply = subprocess.CompletedProcess([], returncode, stdout, "thread 'main' panicked\n")
    monkeypatch.setattr(tax.subprocess, "run", _versioned(reply))
    with pytest.raises(tax.CalculatorsUnavailable, match=f"exited {returncode}: thread 'main'"):
        tax.calculate(tax._FACTS.validate_python(GST))


def test_a_refusal_keeps_the_engines_message(monkeypatch) -> None:
    refusal = json.dumps({"ok": False, "code": "scope.unconfirmed", "message": "Establish it."})
    monkeypatch.setattr(tax.subprocess, "run", _versioned(
        subprocess.CompletedProcess([], 2, refusal, "error: Establish it.\n")))
    with pytest.raises(InputError, match="^Establish it.$"):
        tax.calculate(tax._FACTS.validate_python(GST))


@pytest.mark.parametrize("error", [
    subprocess.TimeoutExpired(["x"], tax.TIMEOUT_SECONDS), PermissionError("denied"),
])
def test_an_executable_that_cannot_run_or_hangs_is_unavailability(monkeypatch, error) -> None:
    def run(command, **_options):
        raise error
    monkeypatch.setattr(tax.subprocess, "run", run)
    with pytest.raises(tax.CalculatorsUnavailable, match="did not run"):
        tax.worksheet_catalogue()
    assert tax.reported_version() is None


def test_the_executable_gets_python_decimal_text(monkeypatch) -> None:
    sent = []
    real = subprocess.run

    def run(command, **options):
        sent.append((command[1:], options.get("input")))
        return real(command, **options)
    monkeypatch.setattr(tax.subprocess, "run", run)
    facts = {"kind": "depreciation", "scope_confirmed": True, "year": "2025-26",
             "cost": " 1000.50 ", "effective_life": "5.0", "taxable_use": "1",
             "days": 365, "method": "prime_cost"}
    tax.calculate(tax._FACTS.validate_python(facts))
    [(arguments, payload)] = [call for call in sent if call[0][0] == "depreciation"]
    assert arguments == ["depreciation", "--refusal-json"]
    assert json.loads(payload) == {"scope_confirmed": True, "year": "2025-26", "cost": "1000.50",
                                   "effective_life": "5.0", "taxable_use": "1", "days": 365,
                                   "method": "prime_cost"}


def test_no_shell_reads_the_command(monkeypatch) -> None:
    seen = {}
    real = subprocess.run

    def run(command, **options):
        seen.update(options)
        return real(command, **options)
    monkeypatch.setattr(tax.subprocess, "run", run)
    _gst()
    assert "shell" not in seen and seen["timeout"] == tax.TIMEOUT_SECONDS
