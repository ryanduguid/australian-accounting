"""The frozen benchmark rate table, and what happens outside it."""
from __future__ import annotations

import json
from decimal import Decimal

import pytest

from div7aloan.rates import (
    RatesError,
    benchmark_rate,
    load_override,
    load_table,
)
from div7aloan.verdicts import RateVerdict

# The published table at https://duguid.com.au/rates/div7a-benchmark-rate/,
# derived from s 109N(2) and RBA table F5 series FILRHLBVS.
PUBLISHED = {
    "2019-20": "0.0537",
    "2020-21": "0.0452",
    "2021-22": "0.0452",
    "2022-23": "0.0477",
    "2023-24": "0.0827",
    "2024-25": "0.0877",
    "2025-26": "0.0837",
    "2026-27": "0.0877",
}


@pytest.mark.parametrize("year,expected", sorted(PUBLISHED.items()))
def test_every_frozen_year_matches_the_published_table(year, expected):
    result = benchmark_rate(year)
    assert result.verdict is RateVerdict.KNOWN
    assert result.rate == Decimal(expected)
    assert result.rate_text == expected


@pytest.mark.parametrize("year", sorted(PUBLISHED))
def test_every_year_reads_the_may_figure(year):
    """s 109N(2) takes the rate last published before the year of income
    starts on 1 July. The RBA publishes F5 in arrears in the first week of the
    following month, so that is always the May figure."""
    result = benchmark_rate(year)
    assert result.rba_month.endswith("-05"), f"{year} does not read a May figure"
    assert result.rba_month[:4] == year[:4]
    assert result.rba_table == "F5"
    assert result.rba_series == "FILRHLBVS"


def test_2025_26_uses_the_may_figure_not_the_june_one():
    """The trap. May 2025 was 8.37 per cent; June 2025 was 8.27 per cent and
    was published in early July, after 2025-26 had already begun."""
    result = benchmark_rate("2025-26")
    assert result.rate == Decimal("0.0837")
    assert result.rate != Decimal("0.0827"), "2025-26 has taken the June 2025 figure"
    assert result.rba_month == "2025-05"


def test_2023_24_is_the_year_that_carries_0_0827():
    """8.27 per cent is a real benchmark rate, for 2023-24. Confusing it with
    2025-26 is the same mistake from the other direction."""
    assert benchmark_rate("2023-24").rate == Decimal("0.0827")


def test_year_before_coverage_is_unknown_with_a_reason():
    result = benchmark_rate("2018-19")
    assert result.verdict is RateVerdict.UNKNOWN
    assert result.rate is None
    assert result.rate_text is None
    assert "2019-20 to 2026-27" in result.reason


def test_year_after_coverage_is_unknown_with_a_reason():
    result = benchmark_rate("2027-28")
    assert result.verdict is RateVerdict.UNKNOWN
    assert result.rate is None
    assert "override" in result.reason


def test_table_carries_its_own_review_metadata():
    table = load_table()
    assert table.reviewed_until.label == "2026-27"
    assert table.reviewed_on == "2026-08-28"
    assert table.coverage() == "2019-20 to 2026-27"


def test_provenance_reaches_the_result():
    result = benchmark_rate("2026-27")
    assert result.origin == "frozen table"
    assert result.seen == "2026-08-28"
    assert result.table_reviewed_until == "2026-27"
    assert "FILRHLBVS" in result.source


def _write_override(tmp_path, doc):
    path = tmp_path / "override.json"
    path.write_text(json.dumps(doc), encoding="utf-8")
    return path


def test_override_extends_coverage(tmp_path):
    path = _write_override(
        tmp_path,
        {
            "verified_until": "2027-28",
            "citation": "RBA table F5 series FILRHLBVS, May 2027 figure, read 2027-07-02",
            "rates": [{"year_of_income": "2027-28", "rate": "0.0850", "rba_month": "2027-05"}],
        },
    )
    result = benchmark_rate("2027-28", override=load_override(path))
    assert result.verdict is RateVerdict.KNOWN
    assert result.rate == Decimal("0.0850")
    assert result.origin == "reviewed override"


def test_override_without_citation_is_refused(tmp_path):
    path = _write_override(
        tmp_path,
        {"verified_until": "2027-28", "rates": [{"year_of_income": "2027-28", "rate": "0.0850"}]},
    )
    with pytest.raises(RatesError, match="citation"):
        load_override(path)


def test_override_without_verified_until_is_refused(tmp_path):
    path = _write_override(
        tmp_path,
        {"citation": "read from F5", "rates": [{"year_of_income": "2027-28", "rate": "0.0850"}]},
    )
    with pytest.raises(RatesError, match="verified_until"):
        load_override(path)


def test_override_cannot_reach_past_its_own_verified_until(tmp_path):
    path = _write_override(
        tmp_path,
        {
            "verified_until": "2027-28",
            "citation": "read from F5",
            "rates": [{"year_of_income": "2029-30", "rate": "0.0900"}],
        },
    )
    with pytest.raises(RatesError, match="verified_until"):
        load_override(path)


def test_override_with_no_rates_is_refused(tmp_path):
    path = _write_override(tmp_path, {"verified_until": "2027-28", "citation": "x", "rates": []})
    with pytest.raises(RatesError, match="non-empty"):
        load_override(path)


def test_table_refuses_a_row_past_its_reviewed_until(tmp_path):
    """A rate added without moving the header is a coverage claim that has
    drifted away from the rows it describes."""
    path = tmp_path / "rates.csv"
    path.write_text(
        "# reviewed_until: 2020-21\n"
        "# reviewed_on: 2026-08-28\n"
        "year_of_income,rate,rba_table,rba_series,rba_month,source,verify_at,seen\n"
        "2019-20,0.0537,F5,FILRHLBVS,2019-05,src,url,2026-08-28\n"
        "2025-26,0.0837,F5,FILRHLBVS,2025-05,src,url,2026-08-28\n",
        encoding="utf-8",
    )
    with pytest.raises(RatesError, match="reviewed_until"):
        load_table(path)


def test_table_refuses_a_duplicated_year(tmp_path):
    path = tmp_path / "rates.csv"
    path.write_text(
        "# reviewed_until: 2019-20\n"
        "# reviewed_on: 2026-08-28\n"
        "year_of_income,rate,rba_table,rba_series,rba_month,source,verify_at,seen\n"
        "2019-20,0.0537,F5,FILRHLBVS,2019-05,src,url,2026-08-28\n"
        "2019-20,0.0600,F5,FILRHLBVS,2019-05,src,url,2026-08-28\n",
        encoding="utf-8",
    )
    with pytest.raises(RatesError, match="twice"):
        load_table(path)


def test_table_refuses_a_percentage_written_as_a_whole_number(tmp_path):
    """8.77 in the rate column instead of 0.0877 is the hand-edit slip that
    would multiply every repayment by a hundred."""
    path = tmp_path / "rates.csv"
    path.write_text(
        "# reviewed_until: 2026-27\n"
        "# reviewed_on: 2026-08-28\n"
        "year_of_income,rate,rba_table,rba_series,rba_month,source,verify_at,seen\n"
        "2026-27,8.77,F5,FILRHLBVS,2026-05,src,url,2026-08-28\n",
        encoding="utf-8",
    )
    with pytest.raises(RatesError, match="decimal fractions"):
        load_table(path)


def test_rates_are_decimals_not_floats():
    for year in PUBLISHED:
        assert isinstance(benchmark_rate(year).rate, Decimal)
