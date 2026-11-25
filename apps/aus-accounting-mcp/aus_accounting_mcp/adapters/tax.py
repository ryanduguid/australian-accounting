# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Validate worksheet facts and delegate all arithmetic to the calculators executable.

The australian-tax-calculators binary is the absolute path in
AUS_TAX_CALCULATORS_BIN, else the first in PATH's absolute folders. It is run only
when its SHA-256 is that of a released executable or the one the operator names in
AUS_TAX_CALCULATORS_SHA256 for a source build, and used only when `--version` names
exactly the release this server was tested against. Without it the tax worksheets
refuse with an install message and every other tool still works.
"""

import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import tempfile
from collections.abc import Iterator
from contextlib import contextmanager
from decimal import Decimal, InvalidOperation
from pathlib import Path
from time import sleep
from typing import Any

from mcp.server.mcpserver.exceptions import ToolError
from pydantic import BaseModel, ConfigDict, Field

from ..errors import InputError

CALCULATORS = "australian-tax-calculators"
#: The only calculators release this server accepts.
CALCULATORS_VERSION = "1.4.0"
#: SHA-256 of the executables in that release's x86_64 Linux and Windows archives,
#: whose own digests its SHA256SUMS lists. Change them with CALCULATORS_VERSION.
RELEASED_DIGESTS: frozenset[str] = frozenset()
BINARY_VARIABLE = "AUS_TAX_CALCULATORS_BIN"
DIGEST_VARIABLE = "AUS_TAX_CALCULATORS_SHA256"
INSTALL = (
    f"The tax worksheets need the {CALCULATORS} {CALCULATORS_VERSION} executable from "
    "https://github.com/ryanduguid/australian-accounting/releases/tag/"
    f"{CALCULATORS}/v{CALCULATORS_VERSION}, verified as that release describes, on PATH "
    f"or at the absolute path in {BINARY_VARIABLE}; for one built from source, also its "
    f"SHA-256 in {DIGEST_VARIABLE}. Tell the user; retrying will not help until it is "
    "installed. The other tools work without it."
)
# A worksheet answers in milliseconds; this bounds a hung or wrong executable.
TIMEOUT_SECONDS = 60
#: The executable's commands that are not one worksheet: its catalogue, its period
#: list, its derivation of another command's result, and 4 that combine several
#: worksheets or checks into results of their own shape. Every other command is a
#: worksheet in its catalogue, which calculate_tax_worksheet serves.
NOT_WORKSHEETS = frozenset({"worksheet-catalogue", "periods", "derive",
                            "personal-tax-estimate", "dgr-check", "charity-check",
                            "fbt-return"})
#: The release executables are under 8 MiB; a source build over this is refused unread.
MAX_BYTES = 256 * 1024 * 1024
COPY_CHUNK_BYTES = 1024 * 1024


class CalculatorsUnavailable(ToolError):
    """The pinned calculators executable is missing, another version or broken."""


def _executable() -> str | None:
    """An absolute path: the operator's AUS_TAX_CALCULATORS_BIN, else PATH's first match.

    Never the working directory: shutil.which and Windows' own search try it first,
    so only PATH's absolute folders are searched, each by an absolute name.
    """
    named = os.environ.get(BINARY_VARIABLE)
    if named:
        if not os.path.isabs(named):
            raise CalculatorsUnavailable(
                f"{BINARY_VARIABLE} must be an absolute path, not {named!r}. {INSTALL}")
        return named
    for folder in os.environ.get("PATH", "").split(os.pathsep):
        if os.path.isabs(folder) and (found := shutil.which(os.path.join(folder, CALCULATORS))):
            return found
    return None


def _run(staged: Path, original: str, arguments: list[str],
         payload: str = "") -> subprocess.CompletedProcess[str]:
    try:
        # Use the exact private file we verified and keep the original directory
        # out of the executable's working directory.
        return subprocess.run(  # NOSONAR nosemgrep
            [str(staged), *arguments], executable=str(staged), cwd=str(staged.parent),  # nosemgrep
            input=payload, capture_output=True, text=True,
            encoding="utf-8", timeout=TIMEOUT_SECONDS, check=False,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise CalculatorsUnavailable(f"{original} did not run: {exc}. {INSTALL}") from exc


def _copy_executable(binary: str, staged: Path) -> str:
    digest = hashlib.sha256()
    # Without O_NONBLOCK, opening a FIFO on PATH can wait for a writer.
    with open(binary, "rb", opener=lambda path, flags: os.open(
            path, flags | getattr(os, "O_NONBLOCK", 0))) as source:
        info = os.fstat(source.fileno())
        if not stat.S_ISREG(info.st_mode):
            raise CalculatorsUnavailable(f"{binary} is not a regular file. {INSTALL}")
        if info.st_size > MAX_BYTES:
            raise CalculatorsUnavailable(
                f"{binary} is {info.st_size} bytes, larger than any {CALCULATORS} "
                f"build this server runs ({MAX_BYTES}). {INSTALL}")
        copied = 0
        with staged.open("xb") as target:
            while chunk := source.read(min(COPY_CHUNK_BYTES, MAX_BYTES - copied + 1)):
                copied += len(chunk)
                if copied > MAX_BYTES:
                    raise CalculatorsUnavailable(
                        f"{binary} exceeds {MAX_BYTES} bytes, larger than any "
                        f"{CALCULATORS} build this server runs. {INSTALL}")
                # The blocking buffered file writes the whole chunk or raises.
                target.write(chunk)
                digest.update(chunk)
    return digest.hexdigest()


@contextmanager
def _staging_directory() -> Iterator[Path]:
    directory = tempfile.TemporaryDirectory(prefix="aus-tax-calculators-")
    try:
        yield Path(directory.name)
    finally:
        # Windows can briefly prevent deletion of an executable after it exits.
        for delay in (0.05, 0.1, 0.2, 0.4, 0.8):
            try:
                directory.cleanup()
            except PermissionError as exc:
                if getattr(exc, "winerror", None) != 32:
                    raise
                sleep(delay)
            else:
                break
        else:
            directory.cleanup()


@contextmanager
def _trusted(binary: str) -> Iterator[Path]:
    """Keep the digest-verified copy alive for the duration of its use."""
    # Either case: sha256sum prints lower case, PowerShell's Get-FileHash upper.
    named = os.environ.get(DIGEST_VARIABLE, "").lower()
    if named and not re.fullmatch(r"[0-9a-f]{64}", named):
        raise CalculatorsUnavailable(
            f"{DIGEST_VARIABLE} must be a SHA-256 of 64 hexadecimal digits. {INSTALL}")
    try:
        with _staging_directory() as folder:
            suffix = ".exe" if os.name == "nt" else ""
            staged = folder / f"{CALCULATORS}{suffix}"
            digest = _copy_executable(binary, staged)
            if digest not in RELEASED_DIGESTS and digest != named:
                raise CalculatorsUnavailable(
                    f"{binary} has SHA-256 {digest}, which is neither a {CALCULATORS} "
                    f"{CALCULATORS_VERSION} release executable nor named in {DIGEST_VARIABLE}. "
                    f"{INSTALL}")
            staged.chmod(0o700)
            yield staged
    except OSError as exc:
        raise CalculatorsUnavailable(f"{binary} did not run: {exc}. {INSTALL}") from exc


def reported_version() -> str | None:
    """The version the executable found reports, or None when there is none to read."""
    try:
        binary = _executable()
        if binary is None:
            return None
        with _trusted(binary) as staged:
            reply = _run(staged, binary, ["--version"])
    except CalculatorsUnavailable:
        return None
    name, _, version = reply.stdout.strip().partition(" ")
    return version if reply.returncode == 0 and name == CALCULATORS and version else None


@contextmanager
def _binary() -> Iterator[tuple[str, Path]]:
    binary = _executable()
    if binary is None:
        raise CalculatorsUnavailable(f"No {CALCULATORS} executable was found. {INSTALL}")
    with _trusted(binary) as staged:
        reply = _run(staged, binary, ["--version"])
        if reply.returncode != 0 or reply.stdout != f"{CALCULATORS} {CALCULATORS_VERSION}\n":
            raise CalculatorsUnavailable(
                f"{binary} --version answered {reply.stdout.strip()!r}, not "
                f"'{CALCULATORS} {CALCULATORS_VERSION}'. {INSTALL}"
            )
        yield binary, staged


def _command(command: str, payload: dict[str, Any]) -> dict[str, Any]:
    """One JSON stdin command: its result, its refusal as InputError, or unavailability."""
    with _binary() as (binary, staged):
        reply = _run(staged, binary, [command, "--refusal-json"], json.dumps(payload))
    try:
        answer = json.loads(reply.stdout)
    except json.JSONDecodeError:
        answer = None
    if reply.returncode == 0 and isinstance(answer, dict):
        return answer
    if (reply.returncode == 2 and isinstance(answer, dict) and answer.get("ok") is False
            and isinstance(answer.get("message"), str)):
        raise InputError(answer["message"])
    raise CalculatorsUnavailable(
        f"{binary} {command} exited {reply.returncode}: {reply.stderr.strip()[:500]}"
    )


class TaxFacts(BaseModel):
    """One worksheet's facts: the kind, its scope confirmation and the inputs it takes.

    Which kinds exist and which inputs each takes come from the executable's own
    worksheet catalogue, so every worksheet it offers is served without an edit
    here. The executable validates and refuses each input.
    """

    model_config = ConfigDict(extra="allow", strict=True)
    scope_confirmed: bool = Field(description="Scope confirmation for this kind; false is refused.")
    kind: str = Field(max_length=80, description="A kind in calculation_worksheets.")


# Plain notation: ASCII digits, an optional leading minus and decimal places,
# with surrounding whitespace ignored. The executable also reads Python's other
# finite decimal syntax (exponents, underscores, a leading plus, ".5", "12."),
# which no fact should use; it refuses NaN and infinities with its own message.
_PLAIN_NUMBER = re.compile(r"\s*-?[0-9]+(?:\.[0-9]+)?\s*")


def _plain(value: Any, path: str) -> Any:
    """`value` with each decimal string, however deep, checked plain and stripped."""
    if isinstance(value, dict):
        return {key: _plain(item, f"{path}.{key}") for key, item in value.items()}
    if isinstance(value, list):
        return [_plain(item, f"{path}[{index}]") for index, item in enumerate(value)]
    if not isinstance(value, str):
        return value
    try:
        finite = Decimal(value).is_finite()
    except InvalidOperation:
        return value
    if finite and _PLAIN_NUMBER.fullmatch(value) is None:
        raise InputError(f"{path}: {value!r} is not a decimal amount")
    return value.strip()


def calculate(facts: TaxFacts) -> dict[str, Any]:
    arguments = facts.model_dump()
    kind = arguments.pop("kind")
    entry = worksheet_catalogue().get(kind)
    if entry is None:
        raise InputError(f"kind {kind!r} is not a worksheet the {CALCULATORS} executable offers; "
                         "calculation_worksheets in aus-accounting://scope lists them.")
    taken = entry["required_inputs"].keys() | (entry.get("optional_inputs") or {}).keys()
    if unknown := sorted(arguments.keys() - taken):
        raise InputError(f"{kind} does not take {', '.join(unknown)}; calculation_worksheets "
                         "lists its required_inputs and optional_inputs.")
    # Each kind is its subcommand, hyphenated.
    return _command(kind.replace("_", "-"),
                    {name: _plain(value, name) for name, value in arguments.items()})


def commands() -> list[str]:
    """The executable's own command list, as its --api-help states it."""
    with _binary() as (binary, staged):
        reply = _run(staged, binary, ["--api-help"])
    listed = reply.stdout.splitlines()[0] if reply.returncode == 0 and reply.stdout else ""
    if not listed.startswith("JSON stdin commands: "):
        raise CalculatorsUnavailable(f"{binary} --api-help answered {listed[:200]!r}. {INSTALL}")
    return listed.removeprefix("JSON stdin commands: ").split(", ")


def worksheet_catalogue() -> dict[str, Any]:
    """The executable's worksheet catalogue: each kind's scope, periods, inputs and example."""
    return _command("worksheet-catalogue", {})


def _table(heading: str, values: dict[str, Any]) -> list[str]:
    if not values:
        return []
    rows = (f"| `{name}` | {value} |" for name, value in values.items())
    return [f"| {heading} | Value |", "| --- | --- |", *rows, ""]


def render_file_note(facts: TaxFacts, result: dict[str, Any]) -> str:
    """Restate one worksheet result as a Markdown file note.

    Every figure, provision, date and caveat is the engine's or the operator's; the
    note adds none. The facts are as supplied, the assumptions are the engine's scope
    statement and the open issues are its warnings.
    """
    entry = worksheet_catalogue()[result["calculation"]]
    # The confirmation, the kind, then the inputs in the catalogue's order.
    order = ["scope_confirmed", "kind", *entry["required_inputs"],
             *(entry.get("optional_inputs") or {})]
    given = facts.model_dump()
    supplied = (
        f"- `{name}`: {value if isinstance(value, str) else json.dumps(value)}"
        for name, value in sorted(given.items(), key=lambda item: order.index(item[0]))
    )
    return "\n".join([
        f"# File note: {result['calculation']} worksheet, {result['period']}",
        "",
        f"Drafted from the {result['engine']} {result['engine_version']} result, which it "
        "restates without adding any figure.",
        "",
        "## Facts relied on",
        "",
        "Established by the operator and not verified by this server.",
        "",
        *supplied,
        "",
        "## Calculation",
        "",
        *_table("Amount (AUD)", result["amounts"]),
        *_table("Rate", result["rates"]),
        "## Assumptions",
        "",
        "The operator confirmed this scope:",
        "",
        result["scope"],
        "",
        "## Provisions and sources",
        "",
        f"The engine's source check covered: {entry['source_passage']}.",
        "",
        *(f"- {url} (checked {result['source_checked']})" for url in result["sources"]),
        "",
        "## Open issues",
        "",
        *(f"- {warning}" for warning in result["warnings"]),
        "",
    ])
