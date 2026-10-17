"""Pytest properties whose inputs fast-check generates, shrinks and replays.

The assertions, the Decimal arithmetic and coverage collection stay in this
Python process. The Node worker in fastcheck/ sends each generated input as one
JSON line and waits for the verdict: pass, fail or discard. Money and rates
arrive as exact decimal strings and become Decimal here. A discarded input does
not count towards the accepted-case budget.

Install the worker's locked npm dependencies first, with the Node toolchain
pinned in the dev extra:

    python tests/fastcheck/bootstrap.py

A missing or stale installation fails the property; nothing is installed or
skipped while the tests run. To replay a reported counterexample, set
FASTCHECK_REPLAY to the JSON object the failure prints.

Each component that uses this keeps an identical copy, because components are
tested and released on their own. tests/test_shared_blocks.py at the
repository root fails when the copies drift.
"""

from __future__ import annotations

import inspect
import json
import os
import subprocess
import threading
from datetime import date
from decimal import Decimal
from pathlib import Path

import nodejs_wheel.executable

WORKER = Path(__file__).with_name("fastcheck")
# One property's whole run, shrinking included, in seconds.
WORKER_TIMEOUT = 120
MAX_LINE = 1_000_000


class DiscardCase(Exception):
    """The generated input is outside the property's precondition."""


class PropertyFailure(AssertionError):
    """A shrunk counterexample, with fast-check's run details for replay."""

    def __init__(self, message: str, result: dict) -> None:
        super().__init__(message)
        self.result = result


def require_case(condition: bool) -> None:
    if not condition:
        raise DiscardCase


def encode(value):
    if isinstance(value, Decimal):
        return {"$decimal": str(value)}
    if isinstance(value, date):
        return {"$date": value.isoformat()}
    if isinstance(value, tuple):
        return {"$tuple": [encode(item) for item in value]}
    if isinstance(value, list):
        return [encode(item) for item in value]
    if isinstance(value, dict):
        return {key: encode(item) for key, item in value.items()}
    return value


def decode(value, types: dict[str, type]):
    if isinstance(value, list):
        return [decode(item, types) for item in value]
    if not isinstance(value, dict):
        return value
    if "$decimal" in value:
        return Decimal(value["$decimal"])
    if "$date" in value:
        return date.fromisoformat(value["$date"])
    if "$tuple" in value:
        return tuple(decode(item, types) for item in value["$tuple"])
    if "$type" in value:
        return types[value["$type"]](**decode(value["fields"], types))
    return {key: decode(item, types) for key, item in value.items()}


def _node() -> Path:
    root = Path(nodejs_wheel.executable.ROOT_DIR)
    return root / "node.exe" if os.name == "nt" else root / "bin" / "node"


def _require_installed() -> None:
    """The worker's node_modules must hold exactly what package-lock.json locks."""
    hint = f"run `python {WORKER / 'bootstrap.py'}` to install the locked fast-check worker"
    installed = WORKER / "node_modules" / ".package-lock.json"
    if not installed.is_file():
        raise RuntimeError(f"fast-check is not installed: {hint}")

    def packages(path: Path) -> dict:
        entries = json.loads(path.read_text(encoding="utf-8"))["packages"]
        return {
            name: (entry.get("version"), entry.get("integrity"))
            for name, entry in entries.items()
            if name
        }

    if packages(installed) != packages(WORKER / "package-lock.json"):
        raise RuntimeError(f"the installed fast-check worker does not match its lock: {hint}")


def _send(process: subprocess.Popen, message: dict) -> None:
    assert process.stdin is not None
    process.stdin.write(json.dumps(message) + "\n")
    process.stdin.flush()


def _receive(process: subprocess.Popen) -> dict:
    assert process.stdout is not None and process.stderr is not None
    line = process.stdout.readline(MAX_LINE + 1)
    if not line or len(line) > MAX_LINE:
        process.kill()
        process.wait(timeout=5)
        raise RuntimeError("fast-check worker ended without a result: " + process.stderr.read())
    return json.loads(line)


def run_property(
    test, *, name: str, seed: int, runs: int, context=None, types=(), replay=None
) -> dict:
    """Check `test` against `runs` accepted inputs from the arbitrary called `name`."""
    if isinstance(runs, bool) or not isinstance(runs, int) or runs <= 0:
        raise ValueError("the accepted-case budget must be a positive integer")
    _require_installed()
    config = {"id": name, "seed": seed, "runs": runs, "context": encode(context or {})}
    if replay is not None:
        config.update(seed=replay["seed"], path=replay["path"])
    known = {kind.__name__: kind for kind in types}
    accepted, failures, discarded = [], [], 0
    with subprocess.Popen(
        [str(_node()), str(WORKER / "runner.cjs")],
        cwd=WORKER,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    ) as process:
        timer = threading.Timer(WORKER_TIMEOUT, process.kill)
        timer.start()
        try:
            _send(process, config)
            while (event := _receive(process)).get("type") == "case":
                raw = event["value"]
                try:
                    test(**decode(raw, known))
                except DiscardCase:
                    status = "discard"
                    discarded += 1
                except Exception as error:
                    status = "fail"
                    failures.append(error)
                else:
                    status = "pass"
                    accepted.append(raw)
                _send(process, {"status": status})
            if event.get("type") != "result":
                raise RuntimeError("unknown fast-check worker message")
            result = event["result"]
            assert process.stdin is not None and process.stderr is not None
            process.stdin.close()
            if process.wait(timeout=5):
                raise RuntimeError("fast-check worker failed: " + process.stderr.read())
        finally:
            timer.cancel()
            if process.poll() is None:
                process.kill()
    result.update(
        accepted_calls=len(accepted),
        discarded_calls=discarded,
        failed_calls=len(failures),
        accepted_cases=accepted,
    )
    if result["failed"]:
        replay_with = {"seed": result["seed"], "path": result["counterexamplePath"]}
        detail = {key: result[key] for key in ("counterexample", "numShrinks", "error")}
        raise PropertyFailure(
            f"fast-check counterexample: {json.dumps(detail)}; "
            f"replay with FASTCHECK_REPLAY='{json.dumps(replay_with)}'",
            result,
        ) from (failures[-1] if failures else None)
    if replay is not None:
        raise AssertionError("the replayed case passed; unset FASTCHECK_REPLAY for the full run")
    if result["interrupted"] or result["numRuns"] != runs or len(accepted) != runs:
        raise AssertionError(f"fast-check did not complete {runs} accepted cases: {result}")
    return result


def fastcheck_property(*, seed: int, runs: int, examples=(), context=None, types=()):
    """Run the explicit examples, then `runs` generated inputs, as one pytest test.

    The arbitrary in fastcheck/properties.cjs has the test's name. `context`
    passes Python values such as supported years to it, and `types` names the
    dataclasses it may build.
    """

    def decorate(test):
        def check() -> None:
            for example in examples:
                test(**example)
            replay = os.environ.get("FASTCHECK_REPLAY")
            run_property(
                test,
                name=test.__name__,
                seed=seed,
                runs=runs,
                context=context,
                types=types,
                replay=json.loads(replay) if replay else None,
            )

        check.__name__ = check.__qualname__ = test.__name__
        check.__doc__ = test.__doc__
        check.__module__ = test.__module__
        check.__signature__ = inspect.Signature()
        # The property's settings, for tooling that inspects or replays it.
        setattr(
            check,
            "fastcheck",
            {
                "test": test,
                "seed": seed,
                "runs": runs,
                "examples": examples,
                "context": context,
                "types": types,
            },
        )
        return check

    return decorate
