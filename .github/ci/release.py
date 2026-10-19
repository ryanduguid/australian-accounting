"""Release components one at a time from the current `main` commit.

    python .github/ci/release.py [--dry-run] <component> [<component> ...]

Every component is checked before anything is tagged: it has a release workflow,
its pyproject name is the component, its RELEASE_NOTES.md starts with its
version, its tag exists neither locally nor on GitHub, neither GitHub nor PyPI
has a release of that version, and every job its release workflow lists under
`required-checks` succeeded in the push runs of `main` for that commit. Release
immutability must be enabled. Any
refusal stops the whole run before the first tag, because a pushed tag cannot
be deleted and a tag whose run fails burns its version.

Components are then released in the order given. Each tag is annotated, made on
the `main` commit the checks ran against and pushed on its own: GitHub creates
no push event, so starts no workflow, for any tag when more than 3 arrive in one
push. The next tag waits until this tag's release workflow has concluded
`success`. Release Policy compares the tag with `main` at its test, build and
publish gates, so `main` must not move until the run finishes; if it has moved
before a tag is pushed, nothing further is tagged. The first failure stops the
run and names the components left untagged.

`--dry-run` runs every check and prints the plan without tagging. Run from the
repository root with `gh` authenticated with Administration read access.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
import tomllib
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Callable, Sequence

REPO = "ryanduguid/australian-accounting"
API_VERSION = "X-GitHub-Api-Version: 2026-03-10"
COMPONENT_ROOTS = ("packages", "apps")
# Runs that execute the workflow committed at the commit, as Release Policy counts them.
TRUSTED_EVENTS = frozenset({"push", "workflow_dispatch"})
# A run normally appears within seconds of the push.
RUN_START_POLLS, RUN_START_SECONDS = 12, 5
# The release workflow's own jobs time out well inside an hour.
RUN_END_POLLS, RUN_END_SECONDS = 180, 20

Runner = Callable[[Sequence[str]], "subprocess.CompletedProcess[str]"]


class Refusal(Exception):
    """A check failed; the message says what to do instead."""


@dataclass(frozen=True)
class Plan:
    component: str
    version: str

    @property
    def tag(self) -> str:
        return f"{self.component}/v{self.version}"


def required_checks(workflow: str) -> list[tuple[str, str]]:
    """The `required-checks: |` block of a release caller, as (workflow path, job name)."""
    lines = workflow.splitlines()
    for index, line in enumerate(lines):
        if line.strip() == "required-checks: |":
            indent = len(line) - len(line.lstrip())
            break
    else:
        return []
    selectors = []
    for line in lines[index + 1 :]:
        if line.strip() and len(line) - len(line.lstrip()) <= indent:
            break
        if line.strip():
            path, _, job = line.strip().partition(": ")
            selectors.append((path, job))
    return selectors


def run_command(command: Sequence[str]) -> subprocess.CompletedProcess[str]:
    """Run git or gh, the only programs this script calls."""
    program, *args = command
    if program == "git":
        return subprocess.run(
            ["git", *args], capture_output=True, text=True, encoding="utf-8", check=False
        )
    if program == "gh":
        return subprocess.run(
            ["gh", *args], capture_output=True, text=True, encoding="utf-8", check=False
        )
    raise ValueError(f"release.py runs only git and gh, not {program!r}")


def pypi_status(name: str, version: str) -> int:
    """HTTP status of PyPI's JSON page for one release: 200 published, 404 absent."""
    url = f"https://pypi.org/pypi/{name}/{version}/json"
    try:
        with urllib.request.urlopen(url, timeout=30) as response:  # nosec B310 - fixed https URL
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


class Releaser:
    def __init__(
        self,
        run: Runner = run_command,
        pypi: Callable[[str, str], int] = pypi_status,
        sleep: Callable[[float], None] = time.sleep,
        out: Callable[[str], None] = print,
    ) -> None:
        self.run, self.pypi, self.sleep, self.out = run, pypi, sleep, out

    def _ok(self, *command: str) -> str:
        result = self.run(command)
        if result.returncode != 0:
            raise Refusal(f"`{' '.join(command)}` failed: {result.stderr.strip()}")
        return result.stdout.strip()

    def _gh(self, endpoint: str) -> str | None:
        """GitHub API body, or None for 404; any other failure refuses."""
        result = self.run(["gh", "api", "-H", API_VERSION, endpoint])
        if result.returncode == 0:
            return result.stdout
        if "HTTP 404" in result.stderr:
            return None
        raise Refusal(f"gh api {endpoint} failed: {result.stderr.strip()}")

    def main_head(self) -> str:
        line = self._ok("git", "ls-remote", "origin", "refs/heads/main")
        if not line:
            raise Refusal("origin has no main branch")
        return line.split()[0]

    def _show(self, sha: str, path: str) -> str | None:
        result = self.run(["git", "show", f"{sha}:{path}"])
        return result.stdout if result.returncode == 0 else None

    def check_required(self, component: str, workflow: str, sha: str) -> None:
        """Every `required-checks` job succeeded in each trusted main run for `sha`.

        A preflight copy of Release Policy's required_checks.py, which still
        decides at release time: refusing here costs nothing, refusing there
        costs the version.
        """
        selectors = required_checks(workflow)
        if not selectors:
            raise Refusal(f"{component}: its release workflow names no required-checks")
        listing = json.loads(
            self._gh(f"repos/{REPO}/actions/runs?head_sha={sha}&per_page=100") or "{}"
        )
        if listing.get("total_count", 0) > 100:
            raise Refusal(f"{component}: more than 100 runs for {sha}; check them by hand")
        runs = [
            run
            for run in listing.get("workflow_runs", [])
            if run["head_sha"] == sha
            and run["event"] in TRUSTED_EVENTS
            and run["head_branch"] == "main"
        ]
        jobs: dict[int, list[dict[str, object]]] = {}
        for path, job_name in selectors:
            matching = [run for run in runs if run["path"] == path]
            if not matching:
                raise Refusal(f"{component}: no push run of {path} on main for {sha}")
            for run in matching:
                if run["id"] not in jobs:
                    endpoint = f"repos/{REPO}/actions/runs/{run['id']}/jobs?per_page=100"
                    jobs[run["id"]] = json.loads(self._gh(endpoint) or "{}").get("jobs", [])
                named = [job for job in jobs[run["id"]] if job["name"] == job_name]
                selector = f"{component}: required check '{path}: {job_name}'"
                if len(named) != 1:
                    raise Refusal(f"{selector} matches {len(named)} jobs in {run['html_url']}")
                if named[0].get("conclusion") != "success":
                    state = named[0].get("conclusion") or named[0].get("status")
                    raise Refusal(f"{selector} is {state} in {run['html_url']}")

    def check(self, component: str, sha: str) -> Plan:
        if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", component):
            raise Refusal(f"{component!r} is not a component name")
        workflow = self._show(sha, f".github/workflows/release-{component}.yml")
        if workflow is None:
            raise Refusal(f"{component}: no .github/workflows/release-{component}.yml on main")
        for root in COMPONENT_ROOTS:
            directory = f"{root}/{component}"
            pyproject = self._show(sha, f"{directory}/pyproject.toml")
            if pyproject is not None:
                break
        else:
            raise Refusal(f"{component}: no pyproject.toml under packages/ or apps/")
        project = tomllib.loads(pyproject)["project"]
        if project["name"] != component:
            raise Refusal(f"{component}: pyproject name is {project['name']!r}")
        if not re.fullmatch(r"(?:0|[1-9][0-9]*)(?:\.(?:0|[1-9][0-9]*)){2}", project["version"]):
            raise Refusal(f"{component}: version {project['version']!r} is not MAJOR.MINOR.PATCH")
        plan = Plan(component, project["version"])
        notes = (self._show(sha, f"{directory}/RELEASE_NOTES.md") or "").splitlines()
        if not notes or notes[0] != f"# v{plan.version}":
            raise Refusal(f"{component}: RELEASE_NOTES.md must start with '# v{plan.version}'")
        if self._ok("git", "ls-remote", "origin", f"refs/tags/{plan.tag}"):
            raise Refusal(
                f"{plan.tag}: already on GitHub. A pushed tag cannot be deleted or moved, "
                "so release this change under the next version"
            )
        if self.run(["git", "rev-parse", "-q", "--verify", f"refs/tags/{plan.tag}"]).returncode == 0:
            raise Refusal(f"{plan.tag}: a local tag that is not on GitHub; inspect it before releasing")
        if self._gh(f"repos/{REPO}/releases/tags/{plan.tag}") is not None:
            raise Refusal(f"{plan.tag}: a GitHub release already exists")
        status = self.pypi(component, plan.version)
        if status == 200:
            raise Refusal(f"{component} {plan.version}: already on PyPI")
        if status != 404:
            raise Refusal(f"{component} {plan.version}: PyPI answered HTTP {status}")
        self.check_required(component, workflow, sha)
        return plan

    def preflight(self, components: Sequence[str]) -> tuple[str, list[Plan]]:
        repeated = sorted({c for c in components if components.count(c) > 1})
        if repeated:
            raise Refusal(f"named more than once: {', '.join(repeated)}")
        immutable = self._gh(f"repos/{REPO}/immutable-releases")
        if immutable is None or json.loads(immutable).get("enabled") is not True:
            raise Refusal("release immutability is not enabled for this repository")
        sha = self.main_head()
        self._ok("git", "fetch", "--quiet", "origin", sha)
        return sha, [self.check(component, sha) for component in components]

    def wait_for_run(self, plan: Plan, sha: str) -> dict[str, object]:
        workflow = f"release-{plan.component}.yml"
        endpoint = (
            f"repos/{REPO}/actions/workflows/{workflow}/runs"
            f"?branch={plan.tag}&event=push&per_page=1"
        )
        for attempt in range(RUN_START_POLLS):
            runs = json.loads(self._gh(endpoint) or "{}").get("workflow_runs", [])
            if runs and runs[0]["head_sha"] == sha:
                return runs[0]
            if attempt + 1 < RUN_START_POLLS:
                self.sleep(RUN_START_SECONDS)
        raise Refusal(
            f"{plan.tag}: no {workflow} run after {RUN_START_POLLS * RUN_START_SECONDS} "
            "seconds. Pushing the same tag again changes no ref and starts nothing"
        )

    def wait_for_success(self, plan: Plan, run: dict[str, object]) -> None:
        for _ in range(RUN_END_POLLS):
            current = json.loads(self._gh(f"repos/{REPO}/actions/runs/{run['id']}") or "{}")
            if current.get("status") == "completed":
                if current.get("conclusion") != "success":
                    raise Refusal(
                        f"{plan.tag}: release run {run['html_url']} concluded "
                        f"{current.get('conclusion')}"
                    )
                return
            self.sleep(RUN_END_SECONDS)
        raise Refusal(f"{plan.tag}: release run {run['html_url']} has not finished; check it")

    def release(self, components: Sequence[str], dry_run: bool = False) -> None:
        sha, plans = self.preflight(components)
        self.out(f"main {sha}")
        for plan in plans:
            self.out(f"  {plan.tag}")
        if dry_run:
            self.out("dry run: nothing tagged")
            return
        for index, plan in enumerate(plans):
            try:
                self.release_one(plan, sha)
            except Refusal as error:
                rest = ", ".join(p.tag for p in plans[index + 1 :]) or "none"
                raise Refusal(f"{error}; not tagged: {rest}") from None
            self.out(f"{plan.tag}: released")

    def release_one(self, plan: Plan, sha: str) -> None:
        head = self.main_head()
        if head != sha:
            raise Refusal(f"main moved from {sha} to {head}, so {plan.tag} was not tagged")
        self._ok("git", "tag", "-a", plan.tag, "-m", f"{plan.component} v{plan.version}", sha)
        self._ok("git", "push", "origin", f"refs/tags/{plan.tag}")
        run = self.wait_for_run(plan, sha)
        self.out(f"{plan.tag}: {run['html_url']} (keep main frozen until it finishes)")
        self.wait_for_success(plan, run)


def main(argv: Sequence[str] | None = None, releaser: Releaser | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("components", nargs="+")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)
    try:
        (releaser or Releaser()).release(args.components, args.dry_run)
    except Refusal as error:
        print(f"release: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
