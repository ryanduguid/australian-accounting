"""The fast-check bridge cannot skip work, pass a failure or leak its worker.

Each component that uses tests/_fastcheck.py keeps an identical copy of this file.
"""

import json
import shutil
import subprocess
from pathlib import Path

import pytest

import _fastcheck


def in_range(value):
    assert 0 <= value <= 10_000


def run(test, runs=40, **options):
    return _fastcheck.run_property(
        test, name="bridge_control_integer", seed=17, runs=runs, **options
    )


def test_the_same_seed_repeats_the_same_cases():
    first, second = run(in_range), run(in_range)
    assert first["accepted_cases"] == second["accepted_cases"]
    assert first["numRuns"] == second["numRuns"] == first["accepted_calls"] == 40


def test_discarded_inputs_do_not_reduce_the_accepted_budget():
    def upper_half(value):
        _fastcheck.require_case(value >= 5_000)

    result = run(upper_half)
    assert result["accepted_calls"] == result["numRuns"] == 40
    assert result["numSkips"] == result["discarded_calls"] > 0


def test_a_failure_is_shrunk_and_its_replay_repeats_the_counterexample():
    def below_half(value):
        assert value < 5_000

    with pytest.raises(_fastcheck.PropertyFailure) as failure:
        run(below_half)
    first = failure.value.result
    assert first["counterexample"] == [{"value": 5_000}]
    assert first["numShrinks"] > 0
    replay = {"seed": first["seed"], "path": first["counterexamplePath"]}
    with pytest.raises(_fastcheck.PropertyFailure) as again:
        run(below_half, replay=replay)
    assert again.value.result["counterexample"] == first["counterexample"]
    assert again.value.result["numShrinks"] == 0


def test_a_python_error_is_reported_as_a_failure():
    def broken(value):
        raise ValueError("deliberate")

    with pytest.raises(_fastcheck.PropertyFailure) as failure:
        run(broken, runs=1)
    assert isinstance(failure.value.__cause__, ValueError)


def test_an_unknown_property_fails_closed():
    with pytest.raises(RuntimeError, match="Unknown property: no_such_property"):
        _fastcheck.run_property(in_range, name="no_such_property", seed=17, runs=1)


@pytest.mark.parametrize("runs", [0, -1, True, 1.5])
def test_an_invalid_budget_is_refused(runs):
    with pytest.raises(ValueError, match="positive integer"):
        run(in_range, runs=runs)


@pytest.fixture
def worker(tmp_path, monkeypatch):
    """A worker directory whose lock and installation can be changed."""
    directory = tmp_path / "fastcheck"
    shutil.copytree(_fastcheck.WORKER, directory, ignore=shutil.ignore_patterns("node_modules"))
    (directory / "node_modules").mkdir()
    shutil.copy(directory / "package-lock.json", directory / "node_modules" / ".package-lock.json")
    monkeypatch.setattr(_fastcheck, "WORKER", directory)
    return directory


def test_a_missing_installation_is_bootstrapped_before_the_property(worker, monkeypatch):
    # The release workflow runs a bare pytest in a fresh checkout.
    shutil.rmtree(worker / "node_modules")
    real_node_modules = Path(_fastcheck.__file__).with_name("fastcheck") / "node_modules"
    calls = []

    def bootstrap(command, **kwargs):
        calls.append(command)
        shutil.copytree(real_node_modules, worker / "node_modules")
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr(_fastcheck.subprocess, "run", bootstrap)
    assert run(in_range)["numRuns"] == 40
    assert calls == [[_fastcheck.sys.executable, str(worker / "bootstrap.py")]]


def test_a_failed_bootstrap_fails_closed(worker, monkeypatch):
    shutil.rmtree(worker / "node_modules")
    monkeypatch.setattr(
        _fastcheck.subprocess,
        "run",
        lambda command, **kwargs: subprocess.CompletedProcess(command, 1, "", "npm ERR! offline"),
    )
    with pytest.raises(RuntimeError, match="(?s)bootstrap failed.*bootstrap.py.*npm ERR! offline"):
        run(in_range)


def test_a_stale_installation_fails_closed(worker):
    installed = worker / "node_modules" / ".package-lock.json"
    lock = json.loads(installed.read_text(encoding="utf-8"))
    lock["packages"]["node_modules/fast-check"]["version"] = "0.0.1"
    installed.write_text(json.dumps(lock), encoding="utf-8")
    with pytest.raises(RuntimeError, match="does not match its lock"):
        run(in_range)


def test_a_missing_module_fails_closed(worker):
    # The lock matches, but the package itself is absent from node_modules.
    with pytest.raises(RuntimeError, match="Cannot find module 'fast-check'"):
        run(in_range)


def test_a_hung_worker_is_killed_and_reaped(worker, monkeypatch):
    (worker / "runner.cjs").write_text("setInterval(() => {}, 1000);\n", encoding="utf-8")
    monkeypatch.setattr(_fastcheck, "WORKER_TIMEOUT", 0.5)
    started = []

    class Recorded(subprocess.Popen):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, **kwargs)
            started.append(self)

    monkeypatch.setattr(_fastcheck.subprocess, "Popen", Recorded)
    with pytest.raises(RuntimeError, match="ended without a result"):
        run(in_range)
    assert len(started) == 1 and started[0].returncode is not None
