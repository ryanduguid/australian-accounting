"""The sequential release script refuses before tagging and stops on the first failure.

Standard library only; git, gh and PyPI are replaced by an in-memory repository.

    python -m unittest -v tests/test_release_script.py
"""

from __future__ import annotations

import importlib.util
import io
import json
import subprocess
import sys
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[1]


def _load(name: str, path: Path):
    """Import a module by path, so this runs under unittest and pytest alike."""
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


release = _load("release", ROOT / ".github" / "ci" / "release.py")
REQUIRED = _load("test_release_checks", ROOT / "tests" / "test_release_checks.py").REQUIRED
SHA = "a" * 40
MOVED = "b" * 40
RUNS = f"repos/{release.REPO}/actions/workflows/"


CALLER = """jobs:
  release:
    with:
      required-checks: |
        .github/workflows/ci.yml: {name} / test (3.14)
        .github/workflows/boundaries.yml: boundaries
      source-directory: {root}/{name}
"""


def component_files(name: str, version: str, root: str = "packages") -> dict[str, str]:
    return {
        f".github/workflows/release-{name}.yml": CALLER.format(name=name, root=root),
        f"{root}/{name}/pyproject.toml": f'[project]\nname = "{name}"\nversion = "{version}"\n',
        f"{root}/{name}/RELEASE_NOTES.md": f"# v{version}\n\nNotes.\n",
    }


class FakeRepository:
    """Answers the git and gh commands the script runs, and records them."""

    def __init__(self) -> None:
        self.main = SHA
        self.files = {
            **component_files("payday-super-checker", "0.2.3"),
            **component_files("solomons-sword", "0.3.3"),
            **component_files("aus-accounting-mcp", "0.6.1", root="apps"),
        }
        self.immutable: object = {"enabled": True}
        self.local_tags: set[str] = set()
        self.remote_tags: set[str] = set()
        self.releases: set[str] = set()
        self.pypi_published: set[tuple[str, str]] = set()
        self.pypi_error: int | None = None
        self.gh_error: str | None = None
        # Tag -> run conclusion; a tag missing here never starts a run.
        self.conclusions: dict[str, str] = {}
        self.move_main_after_push = False
        self.calls: list[list[str]] = []
        self.pushed: list[str] = []
        self.runs: dict[int, str] = {}
        # Push runs of main for SHA: run id -> (workflow path, {job name: conclusion}).
        self.ci: dict[int, tuple[str, dict[str, str | None]]] = {
            101: (".github/workflows/ci.yml", {
                f"{name} / test (3.14)": "success"
                for name in ("payday-super-checker", "solomons-sword", "aus-accounting-mcp")
            }),
            102: (".github/workflows/boundaries.yml", {"boundaries": "success"}),
        }
        self.ci_event = "push"

    def __call__(self, command: Sequence[str]) -> subprocess.CompletedProcess[str]:
        command = list(command)
        self.calls.append(command)
        return self.answer(command)

    def done(self, stdout: str = "", returncode: int = 0, stderr: str = ""):
        return subprocess.CompletedProcess([], returncode, stdout, stderr)

    def answer(self, command: list[str]) -> subprocess.CompletedProcess[str]:
        match command:
            case ["git", "ls-remote", "origin", "refs/heads/main"]:
                return self.done(f"{self.main}\trefs/heads/main\n")
            case ["git", "ls-remote", "origin", ref]:
                tag = ref.removeprefix("refs/tags/")
                return self.done(f"{SHA}\t{ref}\n" if tag in self.remote_tags else "")
            case ["git", "fetch", *_]:
                return self.done()
            case ["git", "show", spec]:
                sha, path = spec.split(":", 1)
                if sha == SHA and path in self.files:
                    return self.done(self.files[path])
                return self.done(returncode=128, stderr="fatal: path does not exist")
            case ["git", "rev-parse", "-q", "--verify", ref]:
                return self.done(returncode=0 if ref.removeprefix("refs/tags/") in self.local_tags else 1)
            case ["git", "tag", "-a", tag, "-m", _, sha]:
                assert sha == SHA
                self.local_tags.add(tag)
                return self.done()
            case ["git", "push", "origin", *refs]:
                assert len(refs) == 1, f"pushed {refs} at once"
                tag = refs[0].removeprefix("refs/tags/")
                self.pushed.append(tag)
                self.remote_tags.add(tag)
                if self.move_main_after_push:
                    self.main = MOVED
                return self.done()
            case ["gh", "api", "-H", _, endpoint]:
                return self.api(endpoint)
        raise AssertionError(f"unexpected command {command}")

    def api(self, endpoint: str) -> subprocess.CompletedProcess[str]:
        if self.gh_error:
            return self.done(returncode=1, stderr=self.gh_error)
        missing = self.done(returncode=1, stderr="gh: Not Found (HTTP 404)")
        if endpoint.endswith("/immutable-releases"):
            return self.done(json.dumps(self.immutable))
        if "/releases/tags/" in endpoint:
            tag = endpoint.split("/releases/tags/", 1)[1]
            return self.done("{}") if tag in self.releases else missing
        if endpoint.startswith(RUNS):
            tag = endpoint.split("branch=", 1)[1].split("&", 1)[0]
            if tag not in self.conclusions or tag not in self.pushed:
                return self.done(json.dumps({"workflow_runs": []}))
            run_id = self.pushed.index(tag) + 1
            self.runs[run_id] = tag
            url = f"https://example.invalid/runs/{run_id}"
            return self.done(json.dumps({"workflow_runs": [{"id": run_id, "head_sha": SHA, "html_url": url}]}))
        if endpoint.startswith(f"repos/{release.REPO}/actions/runs?head_sha={SHA}&"):
            runs = [
                {"id": run_id, "path": path, "head_sha": SHA, "event": self.ci_event,
                 "head_branch": "main", "html_url": f"https://example.invalid/ci/{run_id}"}
                for run_id, (path, _) in self.ci.items()
            ]
            return self.done(json.dumps({"total_count": len(runs), "workflow_runs": runs}))
        if endpoint.endswith("/jobs?per_page=100"):
            run_id = int(endpoint.split("/actions/runs/", 1)[1].split("/", 1)[0])
            jobs = [{"name": name, "status": "completed" if conclusion else "in_progress",
                     "conclusion": conclusion}
                    for name, conclusion in self.ci[run_id][1].items()]
            return self.done(json.dumps({"jobs": jobs}))
        if "/actions/runs/" in endpoint:
            tag = self.runs[int(endpoint.rsplit("/", 1)[1])]
            return self.done(json.dumps({"status": "completed", "conclusion": self.conclusions[tag]}))
        raise AssertionError(f"unexpected endpoint {endpoint}")

    def pypi(self, name: str, version: str) -> int:
        if self.pypi_error is not None:
            return self.pypi_error
        return 200 if (name, version) in self.pypi_published else 404


class ReleaseScriptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = FakeRepository()
        self.output: list[str] = []

    def release(self, *argv: str) -> tuple[int, str]:
        releaser = release.Releaser(
            run=self.repo, pypi=self.repo.pypi, sleep=lambda _: None, out=self.output.append
        )
        stderr = io.StringIO()
        with redirect_stderr(stderr):
            code = release.main(list(argv), releaser)
        return code, stderr.getvalue()

    def assert_refused(self, message: str, *argv: str) -> None:
        code, stderr = self.release(*argv)
        self.assertEqual(code, 1, stderr)
        self.assertIn(message, stderr)
        self.assertEqual(self.repo.pushed, [])
        self.assertEqual(self.repo.local_tags, set())

    # Refusals before any tag.

    def test_refuses_a_component_named_twice(self) -> None:
        self.assert_refused("named more than once: solomons-sword", "solomons-sword", "solomons-sword")

    def test_refuses_without_release_immutability(self) -> None:
        self.repo.immutable = {"enabled": False}
        self.assert_refused("immutability is not enabled", "solomons-sword")

    def test_refuses_a_name_that_is_not_a_component(self) -> None:
        self.assert_refused("'../solomons-sword' is not a component name", "../solomons-sword")

    def test_refuses_a_version_that_cannot_be_a_tag(self) -> None:
        self.repo.files["packages/solomons-sword/pyproject.toml"] = (
            '[project]\nname = "solomons-sword"\nversion = "0.3.3rc1"\n'
        )
        self.assert_refused("'0.3.3rc1' is not MAJOR.MINOR.PATCH", "solomons-sword")

    def test_refuses_a_component_without_a_release_workflow(self) -> None:
        self.assert_refused(
            "no .github/workflows/release-the-exchequer-tally.yml",
            "solomons-sword", "the-exchequer-tally",
        )

    def test_refuses_a_workflow_without_a_component(self) -> None:
        self.repo.files[".github/workflows/release-ghost.yml"] = "on: push\n"
        self.assert_refused("ghost: no pyproject.toml", "ghost")

    def test_refuses_a_name_that_is_not_the_component(self) -> None:
        self.repo.files["packages/solomons-sword/pyproject.toml"] = (
            '[project]\nname = "louisgoldberg"\nversion = "0.3.3"\n'
        )
        self.assert_refused("pyproject name is 'louisgoldberg'", "solomons-sword")

    def test_refuses_notes_for_another_version(self) -> None:
        self.repo.files["packages/solomons-sword/RELEASE_NOTES.md"] = "# v0.3.2\n"
        self.assert_refused("must start with '# v0.3.3'", "solomons-sword")

    def test_refuses_a_local_tag(self) -> None:
        self.repo.local_tags.add("solomons-sword/v0.3.3")
        code, stderr = self.release("payday-super-checker", "solomons-sword")
        self.assertEqual(code, 1)
        self.assertIn("solomons-sword/v0.3.3: a local tag that is not on GitHub", stderr)
        self.assertEqual(self.repo.pushed, [])

    def test_refuses_a_pushed_tag_that_never_released(self) -> None:
        self.repo.remote_tags.add("solomons-sword/v0.3.3")
        self.assert_refused("release this change under the next version", "payday-super-checker", "solomons-sword")

    def test_refuses_an_existing_github_release(self) -> None:
        self.repo.releases.add("solomons-sword/v0.3.3")
        self.assert_refused("a GitHub release already exists", "solomons-sword")

    def test_refuses_a_version_already_on_pypi(self) -> None:
        self.repo.pypi_published.add(("aus-accounting-mcp", "0.6.1"))
        self.assert_refused("aus-accounting-mcp 0.6.1: already on PyPI", "aus-accounting-mcp")

    def test_refuses_when_pypi_cannot_answer(self) -> None:
        self.repo.pypi_error = 503
        self.assert_refused("PyPI answered HTTP 503", "solomons-sword")

    def test_refuses_when_github_cannot_answer(self) -> None:
        self.repo.gh_error = "HTTP 502: Bad Gateway"
        self.assert_refused("HTTP 502", "solomons-sword")

    def test_refuses_a_failed_required_check(self) -> None:
        self.repo.ci[101][1]["solomons-sword / test (3.14)"] = "failure"
        self.assert_refused(
            "required check '.github/workflows/ci.yml: solomons-sword / test (3.14)' is failure",
            "payday-super-checker", "solomons-sword",
        )

    def test_refuses_a_required_check_still_running(self) -> None:
        self.repo.ci[102][1]["boundaries"] = None
        self.assert_refused("'.github/workflows/boundaries.yml: boundaries' is in_progress", "solomons-sword")

    def test_refuses_a_missing_required_job(self) -> None:
        del self.repo.ci[101][1]["solomons-sword / test (3.14)"]
        self.assert_refused("solomons-sword / test (3.14)' matches 0 jobs", "solomons-sword")

    def test_refuses_without_a_push_run_of_a_required_workflow(self) -> None:
        self.repo.ci_event = "pull_request"
        self.assert_refused("no push run of .github/workflows/ci.yml on main", "solomons-sword")

    def test_refuses_a_caller_without_required_checks(self) -> None:
        self.repo.files[".github/workflows/release-solomons-sword.yml"] = "on: push\n"
        self.assert_refused("names no required-checks", "solomons-sword")

    def test_reads_every_selector_from_each_release_caller(self) -> None:
        for caller, expected in REQUIRED.items():
            workflow = (ROOT / ".github/workflows" / caller).read_text()
            selectors = [f"{path}: {job}" for path, job in release.required_checks(workflow)]
            self.assertEqual(selectors, expected, caller)

    def test_dry_run_checks_everything_and_tags_nothing(self) -> None:
        code, stderr = self.release("--dry-run", "payday-super-checker", "solomons-sword")
        self.assertEqual(code, 0, stderr)
        self.assertEqual(self.repo.pushed, [])
        self.assertIn("  solomons-sword/v0.3.3", self.output)

    # Stops after tagging begins.

    def test_releases_in_order_with_one_tag_per_push(self) -> None:
        self.repo.conclusions = {"payday-super-checker/v0.2.3": "success", "solomons-sword/v0.3.3": "success"}
        code, stderr = self.release("payday-super-checker", "solomons-sword")
        self.assertEqual(code, 0, stderr)
        self.assertEqual(self.repo.pushed, ["payday-super-checker/v0.2.3", "solomons-sword/v0.3.3"])
        pushes = [index for index, call in enumerate(self.repo.calls) if call[:2] == ["git", "push"]]
        polls = [index for index, call in enumerate(self.repo.calls) if "/actions/runs/" in call[-1]]
        # The second tag is pushed only after the first run is seen to finish.
        self.assertLess(polls[0], pushes[1])

    def test_stops_when_main_moves_before_the_next_tag(self) -> None:
        self.repo.conclusions = {"payday-super-checker/v0.2.3": "success"}
        self.repo.move_main_after_push = True
        code, stderr = self.release("payday-super-checker", "solomons-sword", "aus-accounting-mcp")
        self.assertEqual(code, 1)
        self.assertIn(f"main moved from {SHA} to {MOVED}, so solomons-sword/v0.3.3 was not tagged", stderr)
        self.assertIn("not tagged: aus-accounting-mcp/v0.6.1", stderr)
        self.assertEqual(self.repo.pushed, ["payday-super-checker/v0.2.3"])

    def test_stops_when_no_run_starts(self) -> None:
        code, stderr = self.release("payday-super-checker", "solomons-sword")
        self.assertEqual(code, 1)
        self.assertIn("no release-payday-super-checker.yml run after 60 seconds", stderr)
        self.assertIn("not tagged: solomons-sword/v0.3.3", stderr)
        self.assertEqual(self.repo.pushed, ["payday-super-checker/v0.2.3"])

    def test_stops_on_the_first_failed_run(self) -> None:
        self.repo.conclusions = {"payday-super-checker/v0.2.3": "failure", "solomons-sword/v0.3.3": "success"}
        code, stderr = self.release("payday-super-checker", "solomons-sword")
        self.assertEqual(code, 1)
        self.assertIn("payday-super-checker/v0.2.3: release run https://example.invalid/runs/1 concluded failure", stderr)
        self.assertIn("not tagged: solomons-sword/v0.3.3", stderr)
        self.assertEqual(self.repo.pushed, ["payday-super-checker/v0.2.3"])


if __name__ == "__main__":
    unittest.main()
