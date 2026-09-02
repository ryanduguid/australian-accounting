"""Keep the documentation honest against the code.

A README figure that has drifted is worse than no README: a reviewer
re-performs the sum, disagrees, and cannot tell which of the two is stale.

Prose assertions run against a whitespace-flattened copy, because these files
are hard-wrapped and a sentence that reads as one line on screen is two in the
source. Structural assertions -- table rows, the banner -- read the raw file.
"""
from __future__ import annotations

from decimal import Decimal
from pathlib import Path

import pytest

from div7aloan.money import ROUNDING
from div7aloan.rates import benchmark_rate, load_table
from div7aloan.register import GATE_COLUMNS, MYR_COLUMNS, OPTIONAL_COLUMNS


def flat(text: str) -> str:
    return " ".join(text.split())


def first_screen(text: str) -> str:
    """Everything above the first horizontal rule.

    Matched as a whole line: the ASCII banner is full of runs of hyphens, and
    a bare index("---") lands inside its top border.
    """
    lines = text.splitlines()
    end = next(n for n, line in enumerate(lines) if line.strip() == "---")
    return "\n".join(lines[:end])


README = Path("README.md").read_text(encoding="utf-8")
DISCLAIMER = Path("DISCLAIMER.md").read_text(encoding="utf-8")
SECURITY = Path("SECURITY.md").read_text(encoding="utf-8")

FLAT_README = flat(README)
FLAT_DISCLAIMER = flat(DISCLAIMER)
FLAT_SECURITY = flat(SECURITY)
FLAT_FIRST_SCREEN = flat(first_screen(README))


# --- the first screen --------------------------------------------------


def test_the_first_screen_says_it_is_not_a_determination():
    assert "Experimental review aid. Not a Division 7A determination." in FLAT_FIRST_SCREEN


@pytest.mark.parametrize(
    "phrase",
    [
        "operator-supplied amalgamated loan",
        "does not form amalgamated loans",
        "does not classify payments",
        "unpaid present entitlements",
        "109RB",
    ],
)
def test_the_first_screen_states_the_v1_scope_and_its_gaps(phrase):
    assert phrase in FLAT_FIRST_SCREEN


def test_the_first_screen_warns_that_a_rising_benchmark_raises_the_repayment():
    assert "rising benchmark rate raises the minimum yearly repayment" in FLAT_FIRST_SCREEN
    assert "not just on new ones" in FLAT_FIRST_SCREEN


def test_the_first_screen_links_the_no_install_explainer():
    assert "https://duguid.com.au/rates/div7a-benchmark-rate/" in FLAT_FIRST_SCREEN


def test_the_banner_is_a_t_account_with_dr_and_cr_sides():
    opening = README.index("```")
    banner = README[opening : README.index("```", opening + 3)]
    assert "DR  what it gives you" in banner
    assert "CR  what it needs" in banner
    rows = [line for line in banner.splitlines() if line.startswith(("+", "|"))]
    assert rows and len({len(line) for line in rows}) == 1, "banner rows are ragged"


# --- rates -------------------------------------------------------------


@pytest.mark.parametrize(
    "year,percent",
    [
        ("2026-27", "8.77%"),
        ("2025-26", "8.37%"),
        ("2024-25", "8.77%"),
        ("2023-24", "8.27%"),
        ("2022-23", "4.77%"),
        ("2021-22", "4.52%"),
        ("2020-21", "4.52%"),
        ("2019-20", "5.37%"),
    ],
)
def test_the_readme_rate_table_matches_the_frozen_table(year, percent):
    assert f"| {year} | {percent} |" in README
    engine = (benchmark_rate(year).rate * Decimal(100)).quantize(Decimal("0.01"))
    assert f"{engine}%" == percent


def test_the_readme_explains_the_may_versus_june_trap():
    assert "May 2025 was 8.37 per cent and June 2025 was 8.27 per cent" in FLAT_README
    assert "8.37 is the benchmark rate" in FLAT_README


def test_the_readme_names_the_rba_series():
    assert "FILRHLBVS" in FLAT_README
    assert "F5" in FLAT_README


def test_the_readme_review_date_matches_the_table_header():
    assert load_table().reviewed_on == "2026-08-28"
    assert "28 August 2026" in FLAT_README


def test_the_readme_documents_the_rounding_mode_the_engine_uses():
    assert ROUNDING in FLAT_README
    assert "the act prescribes no rounding" in FLAT_README.lower()


# --- the register schema ----------------------------------------------


@pytest.mark.parametrize("column", GATE_COLUMNS + MYR_COLUMNS + OPTIONAL_COLUMNS)
def test_every_register_column_is_documented(column):
    assert f"`{column}`" in README, f"{column} is not documented in the README"


def test_the_readme_documents_no_column_the_code_does_not_read():
    known = set(GATE_COLUMNS + MYR_COLUMNS + OPTIONAL_COLUMNS)
    table = README[README.index("## The register CSV") : README.index("## Benchmark rates")]
    documented = {
        line.split("|")[1].strip().strip("`")
        for line in table.splitlines()
        if line.startswith("| `")
    }
    assert documented <= known, f"documented but unread: {documented - known}"


def test_the_readme_warns_about_the_maximum_term_years_name():
    assert "term of the loan" in FLAT_README


# --- refusals ----------------------------------------------------------


@pytest.mark.parametrize(
    "phrase",
    [
        "s 109C payments",
        "s 109F forgiven debts",
        "109XA",
        "PCG 2017/13",
        "TD 2022/11",
        "s 109RB",
        "4 December 1997",
        "Public companies",
        "s 109Y",
        "s 109R",
        "s 109D(6)",
    ],
)
def test_the_readme_lists_each_refusal(phrase):
    assert phrase in FLAT_README


def test_the_readme_refuses_to_predict_an_assessment():
    assert 'does not write "the ATO will assess' in FLAT_README


@pytest.mark.parametrize(
    "banned",
    ["div 7a solver", "division 7a solver", "compliance engine", "deemed dividend calculator"],
)
def test_the_readme_avoids_the_names_this_engine_must_not_go_by(banned):
    assert banned not in FLAT_README.lower()


def test_the_readme_does_not_implement_the_mcp_adapter_here():
    assert "That adapter is not implemented here." in FLAT_README


# --- house files -------------------------------------------------------


@pytest.mark.parametrize(
    "phrase",
    [
        "not tax, legal",
        "not a Division 7A determination and not an ATO assessment",
        "Australian Taxation Office",
        "Commonwealth of Australia",
        "Chartered Accountants Australia and New Zealand",
        "experimental review aid",
        "Do not publish private tax records",
    ],
)
def test_the_disclaimer_covers_the_required_ground(phrase):
    assert phrase in FLAT_DISCLAIMER


def test_the_disclaimer_disclaims_the_ca_anz_endorsement():
    assert "not an endorsement of this software by CA ANZ" in FLAT_DISCLAIMER


def test_the_readme_carries_the_authorship_statement():
    assert "Provisional member of Chartered Accountants Australia and New Zealand" in FLAT_README
    assert "not an endorsement of this software by CA ANZ" in FLAT_README
    assert "own time, on his own equipment" in FLAT_README


def test_security_records_the_no_network_position():
    assert "No network" in FLAT_SECURITY
    assert "never as instructions" in FLAT_SECURITY


def test_the_citation_and_pyproject_agree_on_the_version():
    from div7aloan import __version__

    citation = Path("CITATION.cff").read_text(encoding="utf-8")
    pyproject = Path("pyproject.toml").read_text(encoding="utf-8")
    assert f"version: {__version__}" in citation
    assert f'version = "{__version__}"' in pyproject


def test_the_package_records_the_compilation_it_was_written_against():
    from div7aloan import LAW_COMPILATION

    assert "C1936A00027" in LAW_COMPILATION
    assert "C1936A00027" in FLAT_README


# --- the primary-source review -----------------------------------------

REVIEW_PATH = Path("docs/primary-source-review-2026-08-31.md")
REVIEW = REVIEW_PATH.read_text(encoding="utf-8")
FLAT_REVIEW = flat(REVIEW)


def test_the_readme_links_the_source_review():
    assert str(REVIEW_PATH).replace("\\", "/") in README


def test_the_source_review_names_the_compilation_it_read():
    assert "C1936A00027" in FLAT_REVIEW
    assert "1 July 2026" in FLAT_REVIEW
    assert "volume 2" in FLAT_REVIEW


@pytest.mark.parametrize("section", ["109D", "109E", "109N", "109P", "109R", "109ZD"])
def test_the_source_review_records_which_sections_were_read(section):
    assert section in FLAT_REVIEW


def test_the_source_review_records_how_the_formula_was_recovered():
    """The formula is a bitmap in the compilation. A reader who does not know
    that will text-extract the Act and silently get nothing."""
    assert "image.017.png" in FLAT_REVIEW
    assert "bitmap" in FLAT_REVIEW


def test_the_source_review_records_the_reviewed_position_on_109N_1_b():
    assert "Reviewed position" in FLAT_REVIEW
    assert "Confirmed by Ryan Duguid on 31 August 2026" in FLAT_REVIEW


def test_the_source_review_states_the_rounding_position():
    assert ROUNDING in FLAT_REVIEW
    assert "prescribes none" in FLAT_REVIEW


@pytest.mark.parametrize(
    "year,percent",
    [
        ("2026-27", "8.77%"),
        ("2025-26", "8.37%"),
        ("2024-25", "8.77%"),
        ("2023-24", "8.27%"),
        ("2022-23", "4.77%"),
        ("2021-22", "4.52%"),
        ("2020-21", "4.52%"),
        ("2019-20", "5.37%"),
    ],
)
def test_the_source_review_rate_table_matches_the_frozen_table(year, percent):
    assert f"| {year} | {percent} |" in REVIEW
    engine = (benchmark_rate(year).rate * Decimal(100)).quantize(Decimal("0.01"))
    assert f"{engine}%" == percent


def test_the_source_review_records_the_tables_shelf_life():
    assert load_table().reviewed_until.label in FLAT_REVIEW
    assert "shelf life" in FLAT_REVIEW


def test_the_source_review_says_what_it_does_not_establish():
    for phrase in ("does not establish", "109XA", "PCG 2017/13", "109RB", "public companies"):
        assert phrase in FLAT_REVIEW, phrase


def test_the_source_review_records_the_blocked_sources_honestly():
    """A source trail that omits what could not be read is not a source trail."""
    assert "AustLII" in FLAT_REVIEW
    assert "respected rather than circumvented" in FLAT_REVIEW


def test_the_source_review_disclaims_ca_anz_endorsement():
    assert "not an endorsement of this software or this review by CA ANZ" in FLAT_REVIEW
