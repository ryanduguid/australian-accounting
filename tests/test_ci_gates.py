"""Exercise the merge gates: result rules, job lists and the removed-job tripwire."""

import copy
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github/workflows"
CHECK = ROOT / ".github/ci/check_gates.py"
# Each aggregate that branch protection requires, or that a required one needs.
GATES = {
    "boundaries.yml": "boundaries-gates",
    "ci.yml": "tests-gates",
    "ci-lodgeit-adapter.yml": "lodgeit-gates",
    "ci-package.yml": "gates",
    "codeql.yml": "codeql-gates",
    "joined-fixtures.yml": "public-fixtures-gates",
}
SAMPLE = """name: ci
on: [push]
jobs:
  lint:
    runs-on: ubuntu-latest
  test:
    runs-on: ubuntu-latest
    steps:
      - run: |
          echo "  looks: like a key"
  ci-gates:
    needs: [lint, test]
"""
SAMPLE_PATH = ".github/workflows/ci.yml"
OK = {"lint": {"result": "success"}, "test": {"result": "success"}}


def gate_job(workflow: str, gate: str) -> str:
    return (WORKFLOWS / workflow).read_text(encoding="utf-8").split(f"\n  {gate}:\n", 1)[1]


def job_block(text: str, name: str) -> str:
    match = re.search(rf"(?ms)^  {re.escape(name)}:\n(.*?)(?=^  [\w-]+:|\Z)", text)
    if match is None:
        raise AssertionError(name)
    return match.group(1)


def git(cwd: Path, *args: str) -> bytes:
    executable = shutil.which("git")
    if executable is None:
        raise FileNotFoundError("Git is required for the repository fixtures")
    identity = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.invalid",
                "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.invalid"}
    # Fixed test arguments run the resolved Git executable without a shell.
    # nosemgrep: python.lang.security.audit.dangerous-subprocess-use-audit.dangerous-subprocess-use-audit
    return subprocess.run(  # nosec B603
        [executable, *args], cwd=cwd, check=True, capture_output=True, env={**os.environ, **identity},
    ).stdout


def application_ci():
    sys.path.insert(0, str(CHECK.parent))
    try:
        import app_ci
    finally:
        sys.path.pop(0)
    return app_ci


class EngineSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        application_ci()
        from select_package import parse_changed_paths, should_run

        self.parse = parse_changed_paths
        self.should_run = should_run

    def test_nul_parser_preserves_names_and_rejects_the_whole_malformed_stream(self) -> None:
        names = ["packages/solomons-sword/café.py", "packages/solomons-sword/a\nb\t.py",
                 "packages/solomons-sword/ leading space .py", "packages/solomons-sword/-option.py"]
        self.assertEqual(self.parse("\0".join(names).encode() + b"\0"), names)
        malformed = [b"", b"unterminated", b"bad\xff\0", b"\0", b"a\0\0",
                     b"/absolute/file\0", b"../file\0", b"packages//file\0",
                     b"packages/./file\0", b"packages/solomons-sword/../file\0"]
        streams = malformed + [b"packages/solomons-sword/valid.py\0" + raw
                               for raw in malformed if raw]
        for stream in streams:
            with self.subTest(stream=stream):
                self.assertEqual(self.parse(stream), [])
                for package in application_ci().PACKAGES:
                    self.assertTrue(self.should_run(package, self.parse(stream)))


class EngineGitTests(unittest.TestCase):
    def setUp(self) -> None:
        application_ci()
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.selector = self.root / ".github/ci/select_package.py"
        self.selector.parent.mkdir(parents=True)
        shutil.copyfile(CHECK.parent / "select_package.py", self.selector)
        self.own = "packages/solomons-sword/module.py"
        self.other = "packages/the-wip-tally/module.py"
        for name in (self.own, self.other):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic\n", encoding="utf-8")
        (self.root / "packages/solomons-sword/pyproject.toml").write_text(
            '[project]\nname = "solomons-sword"\nversion = "1.2.3"\n', encoding="utf-8",
        )
        git(self.root, "init", "-q")
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "base")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def select(self, raw: bytes, package: str = "solomons-sword", *args: str) -> str:
        # Fixed fixture arguments run the current interpreter and copied selector without a shell.
        # nosemgrep: python.lang.security.audit.dangerous-subprocess-use-audit.dangerous-subprocess-use-audit
        done = subprocess.run(  # nosec B603
            [sys.executable, str(self.selector), package, *args], cwd=self.root,
            input=raw, capture_output=True, check=False,
        )
        self.assertEqual(done.returncode, 0, done.stderr.decode())
        return done.stdout.decode().strip()

    def diff(self) -> bytes:
        git(self.root, "add", "-A")
        return git(self.root, "-c", "core.quotePath=true", "diff", "--name-only",
                   "--no-renames", "-z", "HEAD", "--")

    def test_unicode_and_ascii_paths_select_only_the_owner(self) -> None:
        for name in ("plain.py", "café.py", " leading space .py", "-option.py"):
            path = self.root / "packages/solomons-sword" / name
            path.write_text("synthetic\n", encoding="utf-8")
            raw = self.diff()
            with self.subTest(name=name):
                self.assertEqual(self.select(raw), "run=true")
                self.assertEqual(self.select(raw, "the-wip-tally"), "run=false")
                from select_package import parse_changed_paths

                self.assertEqual(parse_changed_paths(raw), [f"packages/solomons-sword/{name}"])
            path.unlink()

    def test_cli_selects_an_owner_after_an_unrelated_nul_record(self) -> None:
        raw = b"apps/lodgeit-calculator-adapter/notes.md\0" + self.own.encode() + b"\0"
        self.assertEqual(self.select(raw), "run=true")
        self.assertEqual(self.select(raw, "the-wip-tally"), "run=false")

    @unittest.skipIf(os.name == "nt", "Windows cannot create tab or newline filenames")
    def test_real_git_tab_and_newline_names_are_preserved(self) -> None:
        names = ["packages/solomons-sword/tab\tfile.py", "packages/solomons-sword/line\nfile.py"]
        for name in names:
            (self.root / name).write_text("synthetic\n", encoding="utf-8")
        raw = self.diff()
        from select_package import parse_changed_paths

        self.assertCountEqual(parse_changed_paths(raw), names)
        self.assertEqual(self.select(raw), "run=true")
        self.assertEqual(self.select(raw, "the-wip-tally"), "run=false")

    def test_real_cross_engine_move_selects_both_owners(self) -> None:
        moved = "packages/the-wip-tally/moved.py"
        (self.root / self.own).rename(self.root / moved)
        raw = self.diff()
        from select_package import parse_changed_paths

        self.assertCountEqual(parse_changed_paths(raw), [self.own, moved])
        self.assertEqual(self.select(raw), "run=true")
        self.assertEqual(self.select(raw, "the-wip-tally"), "run=true")

    def test_main_release_pending_and_malformed_input(self) -> None:
        raw = self.other.encode() + b"\0"
        self.assertEqual(self.select(raw), "run=false")
        self.assertEqual(self.select(raw, "solomons-sword", "--on-main"), "run=true")
        git(self.root, "-c", "tag.gpgSign=false", "tag", "solomons-sword/v1.2.3")
        self.assertEqual(self.select(raw, "solomons-sword", "--on-main"), "run=false")
        for malformed in (b"", b"unterminated", b"bad\xff\0", b"valid\0\0"):
            with self.subTest(malformed=malformed):
                self.assertEqual(self.select(malformed), "run=true")
                self.assertEqual(self.select(malformed, "solomons-sword", "--on-main"), "run=true")


class ApplicationSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.ci = application_ci()

    def test_dependencies_match_runtime_and_test_extras(self) -> None:
        packages = {
            tomllib.loads(path.read_text(encoding="utf-8"))["project"]["name"]
            for path in ROOT.glob("packages/*/pyproject.toml")
        }
        self.assertTrue(self.ci.PACKAGES <= packages)
        for app, expected in self.ci.DEPENDENCIES.items():
            project = tomllib.loads((ROOT / "apps" / app / "pyproject.toml").read_text())["project"]
            declared = project.get("dependencies", []) + [
                requirement for extra in project.get("optional-dependencies", {}).values()
                for requirement in extra
            ]
            names = {re.sub(r"[-_.]+", "-", re.split(r"[\s<>=!~\[;@]", dep)[0]).lower()
                     for dep in declared}
            self.assertEqual(names & packages, expected, app)

    def test_only_known_unrelated_components_can_skip(self) -> None:
        from select_package import SHARED_PATHS

        for app, dependencies in self.ci.DEPENDENCIES.items():
            own = f"apps/{app}/docs/REFERENCE.md"
            relevant = [own, "tests/test_components.py", ".github/workflows/ci.yml", *SHARED_PATHS]
            relevant += [f"packages/{name}/module.py" for name in dependencies]
            unrelated = [f"packages/{name}/docs/notes.md" for name in self.ci.PACKAGES - dependencies]
            unrelated += [f"apps/{name}/docs/notes.md" for name in self.ci.DEPENDENCIES if name != app]
            self.assertFalse(self.ci.should_run(app, unrelated))
            for path in relevant + ["new.py", "docs/new.md", "apps/new/module.py",
                                    "packages/new/module.py", "/absolute/file", "../notes", "",
                                    "packages/solomons-sword/../module.py", "packages//file"]:
                with self.subTest(app=app, path=path):
                    self.assertTrue(self.ci.should_run(app, [*unrelated, path]))
            self.assertTrue(self.ci.should_run(app, []))

    def test_nul_paths_survive_unusual_filenames_and_diff_failure_runs_all(self) -> None:
        names = ["packages/solomons-sword/a\nb\t-é.md", "apps/aus-accounting-mcp/-option.py"]
        header = b"tree t\nparent a\nparent b\n\nmessage"
        for output, expected in (("\0".join(names).encode() + b"\0", names),
                                 (b"bad\xff\0", []), (b"unterminated", []), (b"", [])):
            with self.subTest(output=output), patch.object(self.ci.subprocess, "run", side_effect=[
                SimpleNamespace(stdout=header),
                SimpleNamespace(stdout=output),
            ]):
                self.assertEqual(self.ci.changed_paths(ROOT), expected)
        with patch.object(self.ci.subprocess, "run", side_effect=[
            SimpleNamespace(stdout=header), subprocess.CalledProcessError(128, "git"),
        ]):
            self.assertTrue(self.ci.should_run("aus-accounting-mcp", self.ci.changed_paths(ROOT)))
        for header in (b"tree t\n\nroot", b"tree t\nparent a\n\nsingle",
                       b"tree t\nparent a\nparent b\nparent c\n\noctopus"):
            with patch.object(self.ci.subprocess, "run", return_value=SimpleNamespace(stdout=header)):
                self.assertEqual(self.ci.changed_paths(ROOT), [])


class ApplicationGitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.ci = application_ci()
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.own = "apps/aus-accounting-mcp/module.py"
        self.other = "packages/solomons-sword/notes.md"
        for name in (self.own, self.other):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("same content\n", encoding="utf-8")
        git(self.root, "init", "-q", "-b", "main")
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "base")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def merge(self) -> None:
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "--allow-empty", "-m", "feature")
        git(self.root, "checkout", "-q", "main")
        git(self.root, "merge", "-q", "--no-ff", "--no-edit", "feature")

    def select(self, event: str) -> str:
        # The current interpreter runs the repository selector with fixed arguments and no shell.
        # nosemgrep: python.lang.security.audit.dangerous-subprocess-use-audit.dangerous-subprocess-use-audit
        done = subprocess.run(  # nosec B603
            [sys.executable, str(CHECK.parent / "app_ci.py"), "select", "aus-accounting-mcp"],
            cwd=self.root, env={**os.environ, "GITHUB_EVENT_NAME": event},
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(done.returncode, 0, done.stderr)
        return done.stdout.strip()

    def test_irrelevant_merge_can_skip_but_other_events_and_feature_heads_run(self) -> None:
        git(self.root, "checkout", "-q", "-b", "feature")
        (self.root / self.own).write_text("earlier relevant commit\n")
        git(self.root, "commit", "-q", "-am", "relevant")
        (self.root / self.other).write_text("later unrelated commit\n")
        git(self.root, "commit", "-q", "-am", "unrelated")
        self.assertEqual(self.select("pull_request"), "run=true")
        # Restore the app so this merge differs from main only in the unrelated component.
        (self.root / self.own).write_text("same content\n")
        self.merge()
        self.assertEqual(self.select("pull_request"), "run=false")
        for event in ("push", "workflow_dispatch", "", "schedule"):
            with self.subTest(event=event):
                self.assertEqual(self.select(event), "run=true")
        # The merge object still lists its parents, but a shallow checkout has no base.
        (self.root / ".git/shallow").write_text(
            git(self.root, "rev-parse", "HEAD").decode()
        )
        self.assertEqual(self.select("pull_request"), "run=true")

    def test_renames_include_both_roots_in_each_direction(self) -> None:
        for old, new in ((self.own, self.other + ".moved"),
                         (self.other, self.own + ".moved")):
            with self.subTest(old=old):
                git(self.root, "checkout", "-q", "-b", "feature")
                (self.root / old).rename(self.root / new)
                self.merge()
                self.assertCountEqual(self.ci.changed_paths(self.root), [old, new])
                self.assertEqual(self.select("pull_request"), "run=true")
                self.tearDown()
                self.setUp()

    def test_empty_merge_and_non_repository_run_all(self) -> None:
        git(self.root, "checkout", "-q", "-b", "feature")
        self.merge()
        self.assertEqual(self.select("pull_request"), "run=true")
        with tempfile.TemporaryDirectory() as blank:
            self.assertEqual(self.ci.changed_paths(Path(blank)), [])


class ApplicationGateTests(unittest.TestCase):
    def test_gate_truth_table_and_every_malformed_or_unsuccessful_job(self) -> None:
        ci = application_ci()
        for selection, result in (("true", "success"), ("false", "skipped")):
            success = {name: {"result": result} for name in ci.APP_JOBS}
            success.update(changes={"result": "success", "outputs": {"run": selection}},
                           packages={"result": "success"}, future={"result": "success"})
            ci.check_results(success, "pull_request")
            for name in success:
                for bad in ("failure", "cancelled", "skipped", "success", "unknown", None):
                    expected = result if name in ci.APP_JOBS else "success"
                    if bad == expected:
                        continue
                    with self.subTest(selection=selection, name=name, bad=bad):
                        changed = copy.deepcopy(success)
                        changed[name]["result"] = bad
                        with self.assertRaises(ValueError):
                            ci.check_results(changed, "pull_request")
                for malformed in (None, [], "success", {}):
                    with self.subTest(name=name, malformed=malformed):
                        with self.assertRaises(ValueError):
                            ci.check_results({**success, name: malformed}, "pull_request")
                if name in ci.APP_JOBS | {"changes"}:
                    with self.assertRaises(ValueError):
                        ci.check_results({k: v for k, v in success.items() if k != name}, "pull_request")
            for invalid in (None, True, False, "TRUE", "false ", "", [], {}):
                changed = copy.deepcopy(success)
                changed["changes"]["outputs"]["run"] = invalid
                with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                    ci.check_results(changed, "pull_request")
            for malformed in (None, [], "true", {}):
                changed = copy.deepcopy(success)
                changed["changes"]["outputs"] = malformed
                with self.assertRaises(ValueError):
                    ci.check_results(changed, "pull_request")
            if selection == "false":
                for event in ("push", "workflow_dispatch", ""):
                    with self.assertRaises(ValueError):
                        ci.check_results(success, event)
        for malformed in (None, [], "success", {}):
            with self.assertRaises(ValueError):
                ci.check_results(malformed, "pull_request")

    def test_gate_cli_works_with_sparse_checkout_and_rejects_invalid_json(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            shutil.copytree(CHECK.parent, root / ".github/ci", ignore=shutil.ignore_patterns("__pycache__"))
            results = {name: {"result": "skipped"} for name in application_ci().APP_JOBS}
            results["changes"] = {"result": "success", "outputs": {"run": "false"}}
            for raw, expected in ((json.dumps(results), 0), ("{invalid", 1), ("null", 1)):
                # The current interpreter runs the copied repository gate with no shell.
                # nosemgrep: python.lang.security.audit.dangerous-subprocess-use-audit.dangerous-subprocess-use-audit
                done = subprocess.run(  # nosec B603
                    [sys.executable, str(root / ".github/ci/app_ci.py"), "gate"], cwd=root,
                    env={**os.environ, "RESULTS": raw, "GITHUB_EVENT_NAME": "pull_request"},
                    capture_output=True, text=True, check=False,
                )
                self.assertEqual(done.returncode, expected, done.stderr)


class GateWiringTests(unittest.TestCase):
    def test_engine_diff_uses_nul_paths_and_root_retains_all_component_suites(self) -> None:
        package = (WORKFLOWS / "ci-package.yml").read_text(encoding="utf-8")
        self.assertIn('git diff --name-only --no-renames -z "$BASE" "$HEAD" --', package)
        self.assertEqual(package.count('"${ON_MAIN[@]}"'), 2)
        root = job_block((WORKFLOWS / "boundaries.yml").read_text(encoding="utf-8"), "root-checks")
        self.assertIn("run: uv run --locked --group dev pytest\n", root)
        self.assertNotIn("--ignore", root)

    def test_application_selection_wiring_preserves_required_guards(self) -> None:
        for workflow, app in (("ci.yml", "aus-accounting-mcp"),
                              ("ci-lodgeit-adapter.yml", "lodgeit-calculator-adapter")):
            text = (WORKFLOWS / workflow).read_text(encoding="utf-8")
            changes = job_block(text, "changes")
            self.assertIn("fetch-depth: 2", changes)
            self.assertIn("persist-credentials: false", changes)
            self.assertIn(f'run: python3 .github/ci/app_ci.py select {app} >> "$GITHUB_OUTPUT"', changes)
            self.assertIn("run: ${{ steps.select.outputs.run }}", changes)
            for name in application_ci().APP_JOBS:
                job = job_block(text, name)
                self.assertIn("needs: changes\n", job)
                self.assertIn("if: needs.changes.result == 'success' && needs.changes.outputs.run == 'true'", job)
                self.assertNotIn("continue-on-error", job)
            gate = gate_job(workflow, GATES[workflow])
            self.assertIn("run: python3 .github/ci/app_ci.py gate", gate)
            self.assertIn(f"check_gates.py .github/workflows/{workflow} {GATES[workflow]} --no-results", gate)
            if workflow == "ci.yml":
                packages = job_block(text, "packages")
                self.assertNotIn("needs: changes", packages)
        owner = (WORKFLOWS / "boundaries.yml").read_text(encoding="utf-8")
        self.assertIn("run: uv run --locked --group dev pytest", owner)
        self.assertIn("needs: [boundaries, root-checks, rates-dataset]", owner)

    def test_every_gate_runs_the_check_on_its_own_workflow_and_needs_every_job(self) -> None:
        sys.path.insert(0, str(CHECK.parent))
        try:
            import check_gates
        finally:
            sys.path.pop(0)
        for workflow, gate in GATES.items():
            with self.subTest(workflow=workflow):
                job = gate_job(workflow, gate)
                self.assertIn("if: always()", job)
                self.assertIn("fetch-depth: 2", job)
                call = re.search(r"(?m)^        run: python3 \.github/ci/check_gates\.py (\S+) (\S+)(.*)$", job)
                assert call is not None, workflow
                self.assertEqual(call.group(1, 2), (f".github/workflows/{workflow}", gate))
                needs = re.search(r"(?m)^    needs: \[(.*)\]$", job)
                assert needs is not None, workflow
                listed = {name.strip() for name in needs.group(1).split(",")}
                text = (WORKFLOWS / workflow).read_text(encoding="utf-8")
                self.assertEqual(check_gates.job_ids(text) - {gate}, listed)


class CheckGatesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / ".github/workflows").mkdir(parents=True)
        git(self.root, "init", "-q", "-b", "main")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def write(self, text: str) -> None:
        (self.root / SAMPLE_PATH).write_text(text, encoding="utf-8")

    def check(self, results: dict, *extra: str, event: str = "push", body: str = "") -> tuple[int, str]:
        env = {**os.environ, "RESULTS": json.dumps(results), "GITHUB_EVENT_NAME": event, "PR_BODY": body}
        done = subprocess.run([sys.executable, str(CHECK), SAMPLE_PATH, "ci-gates", *extra], cwd=self.root,
                              env=env, capture_output=True, text=True, check=False)
        return done.returncode, (done.stdout + done.stderr).strip()

    def test_results_and_completeness(self) -> None:
        self.write(SAMPLE)
        self.assertEqual(self.check(OK)[0], 0)
        for bad in ("failure", "cancelled", "skipped"):
            self.assertEqual(self.check({**OK, "test": {"result": bad}}), (1, "Jobs did not succeed: test"))
        self.assertEqual(self.check({**OK, "test": {"result": "skipped"}}, "--no-results")[0], 0)
        self.assertEqual(self.check({"lint": OK["lint"]}), (1, "Add these jobs to needs: test"))
        self.assertEqual(self.check({"lint": OK["lint"]}, "--exempt", "test")[0], 0)

    def test_every_job_key_spelling_is_seen_or_fails_closed(self) -> None:
        for spelling in ("extra:", "'extra':", '"extra":', "extra :", "'extra' :", "extra: # note"):
            with self.subTest(spelling=spelling):
                self.write(SAMPLE + f"  {spelling}\n    runs-on: ubuntu-latest\n")
                self.assertEqual(self.check(OK), (1, "Add these jobs to needs: extra"))
        for unreadable in ("? extra", '"ex\\x74ra":'):
            with self.subTest(unreadable=unreadable):
                self.write(SAMPLE + f"  {unreadable}\n    runs-on: ubuntu-latest\n")
                code, out = self.check(OK)
                self.assertEqual(code, 1)
                self.assertIn("Cannot read the job key", out)
        self.write(SAMPLE.replace("\n  ", "\n    "))
        self.assertIn("Cannot find ci-gates", self.check(OK)[1])
        self.write(SAMPLE + "# note\nenv:\n  FOO: bar\n")
        self.assertEqual(self.check(OK)[0], 0)

    def test_the_workflow_must_be_inside_github_workflows(self) -> None:
        self.write(SAMPLE)
        (self.root / "elsewhere.yml").write_text(SAMPLE, encoding="utf-8")
        for outside in ("elsewhere.yml", ".github/workflows/../../elsewhere.yml"):
            env = {**os.environ, "RESULTS": json.dumps(OK), "GITHUB_EVENT_NAME": "push", "PR_BODY": ""}
            done = subprocess.run([sys.executable, str(CHECK), outside, "ci-gates"], cwd=self.root,
                                  env=env, capture_output=True, text=True, check=False)
            self.assertEqual(done.returncode, 1, outside)
            self.assertIn("is not a file in .github/workflows", done.stderr)

    def test_a_removed_job_needs_a_comment_declaring_it_on_a_pull_request(self) -> None:
        without_test = SAMPLE.replace(
            "  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: |\n"
            '          echo "  looks: like a key"\n',
            "",
        ).replace("[lint, test]", "[lint]")
        lint_only = {"lint": OK["lint"]}
        # A declaration naming a job that still exists (lint) would waive a later removal.
        removed_message = f"These jobs were removed from {SAMPLE_PATH}: test"
        cases = (("", removed_message), ("# removed-jobs: test\n", ""), ("# removed-jobs: tests\n", removed_message),
                 ("# removed-jobs: lint, test\n", "declarations name jobs that still exist: lint"))
        for comment, message in cases:
            expected = 1 if message else 0
            with self.subTest(comment=comment):
                self.tearDown()
                self.setUp()
                self.write(SAMPLE)
                git(self.root, "add", "-A")
                git(self.root, "commit", "-q", "-m", "base")
                git(self.root, "checkout", "-q", "-b", "feature")
                self.write(comment + without_test)
                git(self.root, "commit", "-q", "-am", "drop test")
                git(self.root, "checkout", "-q", "main")
                git(self.root, "merge", "-q", "--no-ff", "--no-edit", "feature")
                code, out = self.check(lint_only, event="pull_request")
                self.assertEqual(code, expected, out)
                if message:
                    self.assertIn(message, out)
                # A push run never compares with a base, but a live-job declaration fails anywhere.
                live = "still exist" in message
                self.assertEqual(self.check(lint_only, event="push")[0], 1 if live else 0)

    def test_a_renamed_workflow_is_compared_with_the_base_workflow_holding_its_gate(self) -> None:
        self.write(SAMPLE)
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "base")
        git(self.root, "checkout", "-q", "-b", "feature")
        (self.root / SAMPLE_PATH).unlink()
        renamed = ".github/workflows/build.yml"
        without_test = SAMPLE.replace("  test:\n    runs-on: ubuntu-latest\n", "  unit:\n    runs-on: ubuntu-latest\n")
        (self.root / renamed).write_text(without_test.replace("[lint, test]", "[lint, unit]"), encoding="utf-8")
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "rename and drop test")
        git(self.root, "checkout", "-q", "main")
        git(self.root, "merge", "-q", "--no-ff", "--no-edit", "feature")
        env = {**os.environ, "RESULTS": json.dumps({"lint": OK["lint"], "unit": OK["test"]}),
               "GITHUB_EVENT_NAME": "pull_request"}
        done = subprocess.run([sys.executable, str(CHECK), renamed, "ci-gates"], cwd=self.root, env=env,
                              capture_output=True, text=True, check=False)
        self.assertEqual(done.returncode, 1, done.stdout + done.stderr)
        self.assertIn(f"These jobs were removed from {renamed}: test", done.stderr)

    def test_a_missing_merge_parent_fails_closed(self) -> None:
        self.write(SAMPLE)
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "only")
        self.assertNotEqual(self.check(OK, event="pull_request")[0], 0)


if __name__ == "__main__":
    unittest.main()
