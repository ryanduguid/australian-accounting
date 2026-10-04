import copy
import json
import re
from decimal import Decimal

import pytest

import rates

RECORDS = rates.load_records()
IDS = [r["id"] for r in RECORDS]


def snapshot(record):
    return rates.snapshot_path(record["source_url"]).read_text(encoding="utf-8")


def test_every_file_is_named_after_its_unique_id():
    assert len(set(IDS)) == len(IDS)
    assert sorted(p.stem for p in rates.DATA.glob("*.json")) == sorted(IDS)


@pytest.mark.parametrize("record", RECORDS, ids=IDS)
def test_record_passes_schema_quote_and_pattern_checks(record):
    assert rates.validate_record(record, snapshot(record)) == []


@pytest.mark.parametrize("record", [r for r in RECORDS if r.get("pattern")], ids=lambda r: r["id"])
def test_pattern_reads_the_committed_value_from_the_snapshot(record):
    m = re.search(record["pattern"], rates.norm(snapshot(record)))
    expected = record["value"] if record["unit"] == "date" else Decimal(str(record["value"]))
    assert rates.parse_text(record["unit"], m.group(1)) == expected


@pytest.mark.parametrize("unit, text, expected", [
    ("AUD", "$32,500", Decimal("32500")),
    ("AUD", "$7,483.05", Decimal("7483.05")),
    ("percent", "8.77%", Decimal("8.77")),
    ("percent", "12.00", Decimal("12")),
    ("cents_per_km", "91 cents", Decimal("91")),
    ("cents_per_km", "70c", Decimal("70")),
    ("date", "1\xa0July 2026", "2026-07-01"),
])
def test_parse_text_units(unit, text, expected):
    assert rates.parse_text(unit, text) == expected


@pytest.mark.parametrize("unit, text", [("AUD", "about $30k"), ("percent", "47 per cent"), ("date", "2026-07-01")])
def test_parse_text_rejects_other_formats(unit, text):
    with pytest.raises(ValueError):
        rates.parse_text(unit, text)


@pytest.mark.parametrize("unit, value", [
    ("AUD", -1), ("AUD", "75000"), ("AUD", True), ("percent", 101), ("date", "27 August 2025"),
])
def test_check_value_rejects_bad_values(unit, value):
    assert rates.check_value(unit, value)


def test_tax_scale_base_amounts_must_add_up():
    scale = copy.deepcopy(next(r for r in RECORDS if r["unit"] == "tax_scale")["value"])
    assert rates.check_scale(scale) == []
    scale[2]["base_tax"] += 1
    assert any("base_tax" in p for p in rates.check_scale(scale))


def test_schema_rejects_missing_fields_and_foreign_sources():
    record = dict(RECORDS[0])
    del record["quote"]
    assert "missing or wrong type: quote" in rates.validate_record(record)
    record = dict(RECORDS[0], source_url="https://example.com/rates")
    assert "source_url is not a public ato.gov.au page" in rates.validate_record(record)


@pytest.mark.parametrize("year", ["2026-99", "2026-26", "2026-25", "2026-27\n", "0000-01", "9999-00"])
def test_income_year_must_describe_adjacent_calendar_years(year):
    record = dict(RECORDS[0], period={"income_year": year})
    assert any("income_year" in p for p in rates.validate_record(record))


@pytest.mark.parametrize("period", [
    {"income_year": "2026-27"},
    {"income_year": "1999-00"},
    {"income_year": "2099-00"},
    {"effective_from": "2026-07-01", "effective_to": "2027-06-30"},
    {"effective_from": "2024-02-29", "effective_to": "2024-02-29"},
    {"effective_from": "2026-07-01"},
    {"effective_to": "2026-06-30"},
    {"as_at": "2026-06-01", "effective_from": "2026-07-01"},
])
def test_valid_periods_keep_century_rollover_open_bounds_and_retrieval_dates(period):
    assert rates.validate_record(dict(RECORDS[0], period=period)) == []


def test_effective_period_cannot_end_before_it_starts():
    record = dict(RECORDS[0], period={"effective_from": "2026-07-01", "effective_to": "2026-06-30"})
    assert "effective_to precedes effective_from" in rates.validate_record(record)


@pytest.mark.parametrize("invalid_date", ["2026-02-29", "2024-02-30", "0000-01-01", 20260701, None])
def test_invalid_period_dates_report_a_problem_without_comparing_them(invalid_date):
    record = dict(RECORDS[0], period={"effective_from": invalid_date, "effective_to": "2026-06-30"})
    assert any(p.startswith("effective_from:") for p in rates.validate_record(record))


def test_page_problems_report_changed_value_and_missing_quote():
    record = next(r for r in RECORDS if r["id"] == "concessional-cap-2026-27")
    changed = snapshot(record).replace("$32,500", "$35,000")
    problems = rates.page_problems(record, changed)
    assert "quote not found on page" in problems
    assert "value changed: 32500 -> $35,000" in problems


def test_report_flags_missing_pages_and_passes_unchanged_pages():
    pages = {r["source_url"]: snapshot(r) for r in RECORDS}
    report, findings = rates.build_report(RECORDS, pages, "2026-09-24")
    assert findings == [] and "0 finding(s)" in report

    missing = next(iter(pages))
    pages[missing] = RuntimeError("HTTP 404")
    report, findings = rates.build_report(RECORDS, pages, "2026-09-24")
    affected = [r["id"] for r in RECORDS if r["source_url"] == missing]
    assert len(findings) == len(affected) and "page missing or unreadable (HTTP 404)" in report


def test_report_notes_only_pages_whose_body_changed():
    # A footer edit reaches every page at once; noting all of them buried the
    # one page that gained a new year's figure.
    pages = {r["source_url"]: snapshot(r).replace("Privacy policy", "Privacy notice")
             for r in RECORDS}
    report, findings = rates.build_report(RECORDS, pages, "2026-09-24")
    assert findings == [] and "## Pages with changed wording" not in report
    # A header edit too, including the GIC page whose capture reads "MenuSearch".
    for url, page in pages.items():
        lines = page.splitlines()
        lines.insert(1, "A new header link")
        pages[url] = "\n".join(lines)
    report, findings = rates.build_report(RECORDS, pages, "2026-09-24")
    assert findings == [] and "## Pages with changed wording" not in report

    url = next(r["source_url"] for r in RECORDS if r["id"] == "concessional-cap-2026-27")
    pages[url] = re.sub(r"(?m)^(QC\s?\d+)$", "A new cap applies from 1 July 2027.\n\\1", pages[url])
    report, findings = rates.build_report(RECORDS, pages, "2026-09-24")
    assert findings == [] and report.count("\n- https://") == 1 and f"- {url}" in report


def test_a_record_ending_without_a_successor_is_a_finding():
    # After 30 June 2027 the SG record kept passing with no 2027-28 figure.
    sg = next(r for r in RECORDS if r["id"] == "super-guarantee-rate-2026-27")

    def sg_findings(records, today):
        return [f for f in rates.expiring(records, today) if f.startswith("`super-guarantee")]

    assert rates.expiring(RECORDS, "2026-09-24") == []
    assert sg_findings(RECORDS, "2027-06-15") == []
    assert sg_findings(RECORDS, "2027-06-17") == [
        "`super-guarantee-rate-2026-27`: ends 2027-06-30 and no record for this figure "
        "starts 2027-07-01"]
    successor = dict(sg, id="super-guarantee-rate-2027-28",
                     period={"effective_from": "2027-07-01", "effective_to": "2028-06-30"})
    assert sg_findings([*RECORDS, successor], "2027-06-17") == []
    # Consecutive GIC quarters succeed one another; the last one does not.
    assert [f.split(":")[0] for f in rates.expiring(RECORDS, "2026-12-20")] == [
        "`gic-annual-rate-oct-dec-2026`"]
    pages = {r["source_url"]: snapshot(r) for r in RECORDS}
    _report, findings = rates.build_report(RECORDS, pages, "2027-07-01")
    assert "`super-guarantee-rate-2026-27`: ends 2027-06-30" in " ".join(findings)


def test_a_scale_must_match_the_brackets_its_quote_prints():
    # Firecrawl once returned the 2025-26 scale for 2026-27, and every
    # arithmetic check passed it.
    current = next(r for r in RECORDS if r["id"] == "resident-tax-rates-2026-27")
    previous = next(r for r in RECORDS if r["id"] == "resident-tax-rates-2025-26")
    assert rates.validate_record(current) == [] and rates.validate_record(previous) == []
    assert rates.scale_renderings(current["value"])[:2] == [
        "15c for each $1 over $18,200", "$4,020 plus 30c for each $1 over $45,000"]
    problems = rates.validate_record(dict(current, value=copy.deepcopy(previous["value"])))
    assert "quote does not show the bracket '16c for each $1 over $18,200'" in problems
    assert len(problems) == 4


def test_data_files_are_stable_json():
    for path in rates.DATA.glob("*.json"):
        text = path.read_text(encoding="utf-8")
        assert text == json.dumps(json.loads(text), indent=2, ensure_ascii=False) + "\n", path.name


def browser_reply(monkeypatch, returncode, status=200, loaded=True, text="x" * 250, stderr=""):
    page = {"status": status, "loaded": loaded, "url": "https://www.ato.gov.au/p", "title": "t", "text": text}
    out = json.dumps(page) if returncode != 2 else ""
    monkeypatch.setattr(rates.subprocess, "run",
                        lambda *a, **k: rates.subprocess.CompletedProcess(a, returncode, out, stderr))


def test_fetch_text_returns_the_page_text_with_plain_spaces(monkeypatch):
    browser_reply(monkeypatch, 0, text="Cap $30,000 " + "x" * 250)
    assert rates.fetch_text("https://www.ato.gov.au/p").startswith("Cap $30,000 ")


@pytest.mark.parametrize("returncode, status, loaded, text, message", [
    (1, 404, True, "x" * 250, "HTTP 404"),
    (0, 200, False, "x" * 250, "HTTP 200"),
    (0, 200, True, "Access denied", "nearly empty"),
    (2, None, False, "", "ERR_NAME_NOT_RESOLVED"),
])
def test_fetch_text_rejects_unusable_pages(monkeypatch, returncode, status, loaded, text, message):
    browser_reply(monkeypatch, returncode, status, loaded, text, stderr="ToolError: net::ERR_NAME_NOT_RESOLVED")
    with pytest.raises(RuntimeError, match=message):
        rates.fetch_text("https://www.ato.gov.au/p")


def test_fetch_all_keeps_each_failure_as_that_page_s_value(monkeypatch):
    def fake(url):
        if url.endswith("bad"):
            raise rates.subprocess.TimeoutExpired("browser", 180)
        return "text"

    monkeypatch.setattr(rates, "fetch_text", fake)
    monkeypatch.setattr(rates.time, "sleep", lambda s: None)
    pages = rates.fetch_all(["https://www.ato.gov.au/ok", "https://www.ato.gov.au/bad"])
    assert pages["https://www.ato.gov.au/ok"] == "text"
    assert isinstance(pages["https://www.ato.gov.au/bad"], rates.subprocess.TimeoutExpired)


def test_fetch_text_rejects_a_success_shaped_reply_from_a_failed_browser(monkeypatch):
    browser_reply(monkeypatch, 3)
    with pytest.raises(RuntimeError, match="exited with code 3"):
        rates.fetch_text("https://www.ato.gov.au/p")


def test_a_malformed_reply_fails_only_its_own_page(monkeypatch):
    good = json.dumps({"status": 200, "loaded": True, "url": "u", "title": "t", "text": "x" * 250})

    def run(args, **kwargs):
        return rates.subprocess.CompletedProcess(args, 0, "null" if args[3].endswith("bad") else good, "")

    monkeypatch.setattr(rates.subprocess, "run", run)
    monkeypatch.setattr(rates.time, "sleep", lambda s: None)
    pages = rates.fetch_all(["https://www.ato.gov.au/bad", "https://www.ato.gov.au/ok"])
    assert isinstance(pages["https://www.ato.gov.au/bad"], RuntimeError)
    assert pages["https://www.ato.gov.au/ok"] == "x" * 250


@pytest.mark.parametrize("edit", ["change", "remove"])
def test_the_payg_section_is_checked_not_its_contents_entry(edit):
    record = next(r for r in RECORDS if r["id"] == "payg-tax-tables-effective")
    lines = snapshot(record).splitlines()
    heading = "Tax tables that were updated and apply from 1 July 2026"
    contents, section = [i for i, line in enumerate(lines) if line.strip() == heading]
    lines[section] = heading.replace("2026", "2027") if edit == "change" else ""
    assert lines[contents].strip() == heading
    assert rates.page_problems(record, "\n".join(lines))
