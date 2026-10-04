import pytest

import rates


def record(**changes):
    return {
        "id": "fabricated-date", "unit": "date", "value": "2026-07-01",
        "quote": "Effective date", "pattern": r"Effective date: (.*)",
        "source_url": "https://www.ato.gov.au/fabricated-date", **changes,
    }


@pytest.mark.parametrize("text", ["31 February 2026", "1 August", "1/8/2026"])
def test_unreadable_source_value_is_a_report_finding(text):
    item = record()
    report, findings = rates.build_report(
        [item], {item["source_url"]: f"Effective date: {text}"}, "2026-09-26")
    assert len(findings) == 1
    assert "value pattern could not be read" in findings[0]
    assert "1 finding(s)" in report


@pytest.mark.parametrize("pattern", ["(", "Effective date", r"(Effective) (date)", 42,
                                     r"Effective date: (?:1 August)?(missing)?"])
def test_invalid_pattern_is_a_problem_instead_of_an_exception(pattern):
    assert rates.page_problems(record(pattern=pattern), "Effective date: 1 August 2026")


@pytest.mark.parametrize("pattern", ["", None, False, 0, [], {}])
def test_falsy_supplied_patterns_are_problems(pattern):
    assert rates.page_problems(record(pattern=pattern), "Effective date: 1 August 2026")


def test_absent_pattern_remains_valid():
    item = record()
    del item["pattern"]
    assert rates.page_problems(item, "Effective date: 1 August 2026") == []


def test_malformed_decimal_is_a_report_finding():
    item = record(unit="percent", value=12, pattern=r"Effective date: (.*)")
    assert rates.page_problems(item, "Effective date: 1..2%")


def test_tax_scale_pattern_is_reported_as_unsupported():
    item = record(unit="tax_scale", value=[])
    assert rates.page_problems(item, "Effective date: 1 August 2026")


def test_failed_parse_does_not_hide_later_changed_values():
    bad = record()
    changed = record(id="fabricated-changed", source_url="https://www.ato.gov.au/fabricated-changed")
    pages = {bad["source_url"]: "Effective date: 31 February 2026",
             changed["source_url"]: "Effective date: 1 August 2026"}
    report, findings = rates.build_report([bad, changed], pages, "2026-09-26")
    assert len(findings) == 2
    assert "value changed: 2026-07-01 -> 1 August 2026" in report


def test_check_writes_report_and_returns_failure_when_source_value_is_unreadable(tmp_path, monkeypatch):
    item = record()
    monkeypatch.setattr(rates, "load_records", lambda: [item])
    monkeypatch.setattr(rates, "fetch_all", lambda urls: {item["source_url"]: "Effective date: 31 February 2026"})
    assert rates.main(["check", "--out", str(tmp_path)]) == 1
    reports = list(tmp_path.glob("au-tax-rates-check-*.md"))
    assert len(reports) == 1
    assert "value pattern could not be read" in reports[0].read_text(encoding="utf-8")


def test_empty_pattern_cannot_produce_a_successful_check(tmp_path, monkeypatch):
    item = record(pattern="")
    monkeypatch.setattr(rates, "load_records", lambda: [item])
    monkeypatch.setattr(rates, "fetch_all", lambda urls: {item["source_url"]: "Effective date: 1 August 2026"})
    assert rates.main(["check", "--out", str(tmp_path)]) == 1
