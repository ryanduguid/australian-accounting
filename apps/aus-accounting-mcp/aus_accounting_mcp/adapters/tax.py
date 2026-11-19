# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

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
from decimal import Decimal, InvalidOperation
from typing import Annotated, Any, Literal

from mcp.server.mcpserver.exceptions import ToolError
from pydantic import BaseModel, ConfigDict, Field, TypeAdapter, ValidationError

from ..errors import InputError
from ..money import MONEY_SCHEMA, parse_amount

CALCULATORS = "australian-tax-calculators"
#: The only calculators release this server accepts.
CALCULATORS_VERSION = "1.0.0"
#: SHA-256 of the executables in that release's x86_64 Linux and Windows archives,
#: whose own digests its SHA256SUMS lists. Change them with CALCULATORS_VERSION.
RELEASED_DIGESTS = frozenset({
    "d45d89ed81d1537735e9599e30dc4052b539477c0764d4c55bc32a8b52c87fcc",  # linux-musl
    "7a7e2de825e4c8f97da365184c7a23e4951c40a4629ce24394b91434179591c2",  # windows-msvc
})
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


def _run(binary: str, arguments: list[str], payload: str = "") -> subprocess.CompletedProcess[str]:
    try:
        # An argument list and no shell; the executable is the absolute path the
        # operator named or put on PATH, and the arguments are fixed command names.
        return subprocess.run(  # NOSONAR nosemgrep
            [binary, *arguments], input=payload, capture_output=True, text=True,  # nosemgrep
            encoding="utf-8", timeout=TIMEOUT_SECONDS, check=False,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise CalculatorsUnavailable(f"{binary} did not run: {exc}. {INSTALL}") from exc


def _trusted(binary: str) -> str:
    """The executable, once its SHA-256 is a released one or the operator's; else refused.

    Checked before it runs, so another program on PATH that prints the pinned
    version is never run.

    shortcut: the file is checked by path and then run by path, so whoever can write
    it, its folder or a link on its path can swap it in between; the README tells
    operators to keep it where only they can write. Closing that needs running the
    checked bytes themselves, from a private copy or the descriptor that was hashed.
    """
    # Either case: sha256sum prints lower case, PowerShell's Get-FileHash upper.
    named = os.environ.get(DIGEST_VARIABLE, "").lower()
    if named and not re.fullmatch(r"[0-9a-f]{64}", named):
        raise CalculatorsUnavailable(
            f"{DIGEST_VARIABLE} must be a SHA-256 of 64 hexadecimal digits. {INSTALL}")
    try:
        # Without O_NONBLOCK, opening a FIFO planted on PATH waits for a writer.
        with open(binary, "rb", opener=lambda path, flags: os.open(
                path, flags | getattr(os, "O_NONBLOCK", 0))) as file:
            if not stat.S_ISREG(os.fstat(file.fileno()).st_mode):
                raise CalculatorsUnavailable(f"{binary} is not a regular file. {INSTALL}")
            digest = hashlib.file_digest(file, "sha256").hexdigest()
    except OSError as exc:
        raise CalculatorsUnavailable(f"{binary} did not run: {exc}. {INSTALL}") from exc
    if digest not in RELEASED_DIGESTS | {named}:
        raise CalculatorsUnavailable(
            f"{binary} has SHA-256 {digest}, which is neither a {CALCULATORS} "
            f"{CALCULATORS_VERSION} release executable nor named in {DIGEST_VARIABLE}. "
            f"{INSTALL}")
    return binary


def reported_version() -> str | None:
    """The version the executable found reports, or None when there is none to read."""
    try:
        binary = _executable()
        if binary is None:
            return None
        reply = _run(_trusted(binary), ["--version"])
    except CalculatorsUnavailable:
        return None
    name, _, version = reply.stdout.strip().partition(" ")
    return version if reply.returncode == 0 and name == CALCULATORS and version else None


def _binary() -> str:
    binary = _executable()
    if binary is None:
        raise CalculatorsUnavailable(f"No {CALCULATORS} executable was found. {INSTALL}")
    reply = _run(_trusted(binary), ["--version"])
    if reply.returncode != 0 or reply.stdout != f"{CALCULATORS} {CALCULATORS_VERSION}\n":
        raise CalculatorsUnavailable(
            f"{binary} --version answered {reply.stdout.strip()!r}, not "
            f"'{CALCULATORS} {CALCULATORS_VERSION}'. {INSTALL}"
        )
    return binary


def _command(command: str, payload: dict[str, Any]) -> dict[str, Any]:
    """One JSON stdin command: its result, its refusal as InputError, or unavailability."""
    binary = _binary()
    reply = _run(binary, [command, "--refusal-json"], json.dumps(payload))
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

# The money and scope rules are stated once in the calculate_tax_worksheet description;
# a sentence here is repeated for every field of every worksheet kind in the schema.
Money = Annotated[str, Field(max_length=60, description="AUD decimal string.",
                             json_schema_extra=MONEY_SCHEMA)]
Ratio = Annotated[str, Field(max_length=30, description="Finite decimal string, not a percentage.")]


class Worksheet(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)
    scope_confirmed: bool = Field(description="Scope confirmation for this kind; false is refused.")


class GstFacts(Worksheet):
    kind: Literal["gst"]
    year: Literal["2025-26"]
    amount: Money
    gst_inclusive: bool = Field(description="Whether the established taxable price includes GST.")


class ResidentTaxFacts(Worksheet):
    kind: Literal["resident_tax"]
    year: Literal["2024-25", "2025-26", "2026-27"]
    taxable_income: Money


class CapitalGainsFacts(Worksheet):
    kind: Literal["capital_gains"]
    year: Literal["2025-26"]
    other_gains: Money
    discount_gains: Money
    current_losses: Money
    prior_losses: Money


class FbtFacts(Worksheet):
    kind: Literal["fbt"]
    year_ended: Annotated[int, Field(ge=2026, le=2026,
        description="FBT year ended 31 March 2026, not an income year.")]
    type_one_value: Money
    type_two_value: Money


class DepreciationFacts(Worksheet):
    kind: Literal["depreciation"]
    year: Literal["2025-26"]
    cost: Money
    effective_life: Ratio
    days: Annotated[int, Field(ge=0, le=365,
        description="Days from first use or installation ready for use in 2025-26.")]
    taxable_use: Ratio
    method: Literal["prime_cost", "diminishing_value"]


class SgFacts(Worksheet):
    kind: Literal["quarterly_sg"]
    year: Literal["2025-26"]
    quarter: Annotated[int, Field(ge=1, le=4,
        description="1 is Jul-Sep, 2 Oct-Dec, 3 Jan-Mar, 4 Apr-Jun.")]
    ordinary_time_earnings: Money
    qualifying_contributions: Money


class PaygWithholdingFacts(Worksheet):
    kind: Literal["payg_withholding"]
    year: Literal["2026-27"]
    earnings: Money
    pay_period: Literal["weekly", "fortnightly", "monthly"]
    scale: Literal[1, 2, 3, 5, 6] = Field(
        description="Schedule 1 scale from the payee's declarations; scale 4 is not supported.")


class ContributionCapsFacts(Worksheet):
    kind: Literal["contribution_caps"]
    year: Literal["2024-25", "2025-26", "2026-27"]
    total_super_balance: Money = Field(
        description="Total super balance at the 30 June before the year.")
    concessional_contributions: Money
    unused_concessional_cap: Money = Field(
        description="Unexpired unused concessional cap from the previous 5 years; 0.00 if none.")
    non_concessional_contributions: Money
    under_75_in_year: bool = Field(description="Whether the person is under 75 at any time "
                                               "in the income year.")


class PensionMinimumFacts(Worksheet):
    kind: Literal["pension_minimum"]
    year: Literal["2024-25", "2025-26", "2026-27"]
    account_balance: Money = Field(
        description="Balance on 1 July, or on the commencement day in the first year.")
    age: Annotated[int, Field(ge=0, le=150,
        description="Age on the day the account balance is taken.")]
    days: Annotated[int, Field(ge=1, le=365,
        description="Days in the year from and including the commencement day; every day "
                    "of the year for a pension running on 1 July.")]


class StudyLoanRepaymentFacts(Worksheet):
    kind: Literal["study_loan_repayment"]
    year: Literal["2025-26", "2026-27"]
    repayment_income: Money = Field(
        description="Established repayment income for the year in whole dollars: taxable "
                    "income excluding assessable FHSS released amounts, plus reportable "
                    "fringe benefits, total net investment loss, reportable super "
                    "contributions and exempt foreign employment income.")


class StudyLoanWithholdingFacts(Worksheet):
    kind: Literal["study_loan_withholding"]
    year: Literal["2026-27"]
    earnings: Money
    pay_period: Literal["weekly", "fortnightly", "monthly"]
    scale: Literal[1, 2, 3, 5, 6] = Field(
        description="Schedule 1 scale from the payee's declarations; scale 4 is not supported.")


class MedicareLevyFacts(Worksheet):
    kind: Literal["medicare_levy"]
    year: Literal["2024-25", "2025-26"]
    taxable_income: Money
    section_9a_adjustment_applies: bool = Field(
        description="Whether Medicare Levy Act 1986 s 9A takes lump sums in arrears out of "
                    "taxable income; true is refused.")


class LowIncomeTaxOffsetFacts(Worksheet):
    kind: Literal["low_income_tax_offset"]
    year: Literal["2024-25", "2025-26", "2026-27"]
    taxable_income: Money


class Division293Facts(Worksheet):
    kind: Literal["division_293"]
    year: Literal["2024-25", "2025-26", "2026-27"]
    division_293_income: Money = Field(
        description="Income for surcharge purposes, disregarding reportable super contributions.")
    low_tax_contributions: Money = Field(
        description="Low tax contributed amounts less excess concessional contributions.")


TaxFacts = Annotated[
    GstFacts | ResidentTaxFacts | CapitalGainsFacts | FbtFacts | DepreciationFacts | SgFacts
    | PaygWithholdingFacts | ContributionCapsFacts | PensionMinimumFacts
    | StudyLoanRepaymentFacts | StudyLoanWithholdingFacts | MedicareLevyFacts
    | LowIncomeTaxOffsetFacts | Division293Facts,
    Field(discriminator="kind"),
]
_FACTS: TypeAdapter[TaxFacts] = TypeAdapter(TaxFacts)
MONEY_FIELDS = {
    "amount", "taxable_income", "other_gains", "discount_gains", "current_losses",
    "prior_losses", "type_one_value", "type_two_value", "cost", "ordinary_time_earnings",
    "qualifying_contributions", "earnings", "total_super_balance", "concessional_contributions",
    "unused_concessional_cap", "non_concessional_contributions", "account_balance",
    "repayment_income", "division_293_income", "low_tax_contributions",
}


# The executable describes more for these kinds than TaxFacts takes: quarterly pays,
# scale 4, offsets and Medicare adjustments, later-year depreciation, the FBT year
# ended 31 March 2027, Medicare family, SAPTO and exemption inputs, and earlier years'
# contributions. Their catalogue entries and results state what this tool accepts
# instead, as aus-accounting-mcp 0.9.0 did. Widening TaxFacts would retire them.
ACCEPTED_SCOPES = {
    "contribution_caps": (
        "One individual's contributions received by all their funds in the income year, already"
        " classified by the operator as concessional or non-concessional, with no bring-forward"
        " period started in either of the 2 previous years. Total super balance is at the 30 "
        "June before the year; unused concessional cap is the established total of unexpired "
        "amounts from the previous 5 years. Excludes classification, fund acceptance and "
        "work-test rules, deductibility, the CGT cap, downsizer and other excluded "
        "contributions, excess concessional amounts counting as non-concessional, "
        "determinations, release elections, Division 293 tax and the transfer balance cap."
    ),
    "depreciation": (
        "First year only, ordinary tangible Division 40 asset first held on or after 10 May "
        "2006. Established cost, effective life and taxable-use proportion. Days run from first"
        " use or installation ready for use. No second-element costs, disposals, changed life, "
        "pooling, immediate write-offs, special assets or Division 43 capital works. Does not "
        "determine an effective life from an asset description."
    ),
    "fbt": (
        "Ordinary taxable employer, established type 1 and type 2 taxable values for the year "
        "ended 31 March 2026. Excludes benefit valuation, exemptions, rebates and "
        "not-for-profit caps. Retains gross-up precision for the estimate, then presents the "
        "amounts to cents. The return_item figures follow the FBT return instead: items 14A and"
        " 14B rounded to the nearest dollar, as the ATO's own worked example does, item 15 as "
        "their sum and item 16 at 47%, before any optional rounding down to 5 cents."
    ),
    "medicare_levy": (
        "Australian resident individual for the full income year, single with no spouse or "
        "dependants at any time in the year, not entitled to the seniors and pensioners tax "
        "offset, and not entitled to any full or half Medicare levy exemption. The operator "
        "must also establish whether s 9A applies: from 2024-25 it takes eligible lump sums in "
        "arrears out of taxable income when they are at least 10% of normal taxable income and "
        "its accrual-year tests are met. A person to whom s 9A applies is refused. Supply "
        "established whole-dollar taxable income. Applies the 2% rate and the low-income "
        "reduction: no levy at or below the threshold amount, and up to the phase-in limit no "
        "more than 10% of the excess over it. Excludes the family income reduction, SAPTO "
        "thresholds, exemptions, the Medicare levy surcharge, trustees and part-year residence."
    ),
    "payg_withholding": (
        "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July"
        " 2026 to a payee who gave a TFN, on scale 1, 2, 3, 5 or 6 as the operator established "
        "from the payee's declarations. Earnings include allowances subject to withholding. "
        "Excludes scale 4, tax offsets, Medicare levy adjustments, study and training support "
        "loans, extra amounts for 53 or 27 pays, quarterly and bi-monthly pays, back payments, "
        "bonuses, termination and other schedules."
    ),
    "study_loan_withholding": (
        "One regular weekly, fortnightly or monthly payment of salary or wages made from 1 July"
        " 2026 to a payee who gave a TFN declaration or withholding declaration saying they "
        "have a study and training support loan debt, on scale 1, 2, 3, 5 or 6 as the operator "
        "established. Scale 1 uses the Schedule 8 component for a payee who did not claim the "
        "tax-free threshold; scales 2, 3, 5 and 6 use the one for a payee who claimed it or is "
        "a foreign resident. Earnings include allowances subject to withholding. The result is "
        "the Schedule 1 amount plus the Schedule 8 component, each rounded to the dollar; the "
        "ATO's combined coefficients can differ by a dollar and it accepts either method. "
        "Excludes scale 4, a Medicare levy variation for a spouse or dependants, tax offsets, "
        "extra amounts for 53 or 27 pays, quarterly pays, lump sum termination payments, back "
        "payments, bonuses and other schedules."
    ),
}
ACCEPTED_INPUTS = {
    "payg_withholding": {"pay_period": "weekly, fortnightly or monthly",
                         "scale": "integer 1, 2, 3, 5 or 6"},
    "study_loan_withholding": {"pay_period": "weekly, fortnightly or monthly"},
}


def calculate(facts: TaxFacts) -> dict[str, Any]:
    arguments = facts.model_dump()
    kind = arguments.pop("kind")
    # The executable reads Python's decimal syntax, so each value goes as the
    # str() of the Decimal the Python engine was given.
    for name in arguments.keys() & MONEY_FIELDS:
        arguments[name] = str(parse_amount(arguments[name], name))
    for name in ("effective_life", "taxable_use"):
        if name in arguments:
            try:
                arguments[name] = str(Decimal(arguments[name]))
            except InvalidOperation as exc:
                raise InputError(str(exc)) from exc
    # Each kind is its subcommand, hyphenated.
    result = _command(kind.replace("_", "-"), arguments)
    if kind in ACCEPTED_SCOPES:
        result["scope"] = ACCEPTED_SCOPES[kind]
    return result


def worksheet_catalogue() -> dict[str, Any]:
    """The executable's catalogue, narrowed to what calculate_tax_worksheet accepts.

    The executable has more worksheets, periods and inputs than TaxFacts takes;
    advertising those would invite calls the tool refuses. Kinds and periods are kept
    only where their example validates, optional inputs are dropped, and the kinds in
    ACCEPTED_SCOPES state that scope and ACCEPTED_INPUTS' descriptions.
    """
    narrowed: dict[str, Any] = {}
    for kind, entry in _command("worksheet-catalogue", {}).items():
        periods = []
        for period in entry["supported_periods"]:
            try:
                _FACTS.validate_python({**entry["example"]["facts"], **period["arguments"]})
            except ValidationError:
                continue
            periods.append(period)
        if periods:
            kept = {key: value for key, value in entry.items() if key != "optional_inputs"}
            narrowed[kind] = {
                **kept, "supported_periods": periods,
                "scope": ACCEPTED_SCOPES.get(kind, entry["scope"]),
                "required_inputs": {**entry["required_inputs"], **ACCEPTED_INPUTS.get(kind, {})},
            }
    return narrowed


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
    passage = worksheet_catalogue()[result["calculation"]]["source_passage"]
    supplied = (
        f"- `{name}`: {str(value).lower() if isinstance(value, bool) else value}"
        for name, value in facts.model_dump().items()
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
        f"The engine's source check covered: {passage}.",
        "",
        *(f"- {url} (checked {result['source_checked']})" for url in result["sources"]),
        "",
        "## Open issues",
        "",
        *(f"- {warning}" for warning in result["warnings"]),
        "",
    ])
