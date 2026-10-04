"""Snapshot, validate and re-check the AU tax rates dataset.

    python rates.py snapshot   fetch every source page into snapshots/
    python rates.py validate   check data/*.json against the schema and snapshots
    python rates.py check      re-fetch the pages and write a Markdown change report
"""
import argparse
import datetime as dt
import hashlib
import json
import math
import re
import subprocess
import sys
import time
from decimal import Decimal, InvalidOperation
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parent
DATA = ROOT / "data"
SNAPSHOTS = ROOT / "snapshots"
NODRIVER = [r"C:\Tools\nodriver-browser\.venv\Scripts\python.exe", r"C:\Tools\nodriver-browser\server.py"]

FIELDS = {"id": str, "label": str, "unit": str, "period": dict,
          "source_url": str, "retrieved_at": str, "quote": str}
UNITS = {"AUD", "percent", "cents_per_km", "date", "tax_scale"}
PERIOD_KEYS = {"income_year", "effective_from", "effective_to", "as_at"}
INCOME_YEAR = re.compile(r"^\d{4}-\d{2}$")


def plain_spaces(text):
    """Turn non-breaking and other Unicode spaces into ASCII spaces, keeping line breaks and tabs."""
    return re.sub(r"[^\S\n\t]", " ", text)


def norm(text):
    """Collapse whitespace so table cells and line breaks compare as plain text."""
    return re.sub(r"\s+", " ", text).strip()


def parse_text(unit, text):
    """Parse a figure as printed on the page ("$32,500", "8.77%", "91 cents", "1 July 2026")."""
    text = norm(text)
    if unit == "date":
        return dt.datetime.strptime(text, "%d %B %Y").date().isoformat()
    number = {"AUD": r"^\$?([\d,]+(?:\.\d+)?)$",
              "percent": r"^([\d.]+)\s?%?$",
              "cents_per_km": r"^([\d.]+)\s?(?:c|cents)(?: per (?:kilometre|km))?$"}[unit]
    m = re.match(number, text)
    if not m:
        raise ValueError(f"{text!r} is not a {unit} value")
    return Decimal(m.group(1).replace(",", ""))


def check_value(unit, value):
    """Return a list of problems with a record's value for its unit."""
    if unit == "date":
        try:
            dt.date.fromisoformat(value)
            return []
        except (TypeError, ValueError):
            return [f"value {value!r} is not an ISO date"]
    if unit == "tax_scale":
        return check_scale(value)
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return [f"value {value!r} is not a number"]
    if isinstance(value, float) and not math.isfinite(value):
        return [f"value {value!r} is not finite"]
    if value < 0:
        return [f"value {value!r} is negative"]
    if unit == "percent" and value > 100:
        return [f"percent value {value} is above 100"]
    return []


def check_scale(rows):
    """Brackets must be contiguous, and each base amount must equal the tax on the brackets below it."""
    if not isinstance(rows, list) or not rows:
        return ["tax_scale value must be a non-empty list"]
    problems = []
    for i, row in enumerate(rows):
        if not isinstance(row, dict) or set(row) != {"from", "to", "base_tax", "marginal_rate"}:
            return [f"bracket {i} needs exactly from, to, base_tax and marginal_rate"]
        for key in ("from", "to"):
            value = row[key]
            if key == "to" and value is None and i == len(rows) - 1:
                continue
            if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                problems.append(f"bracket {i} {key} must be a non-negative whole dollar amount")
        for key, unit in (("base_tax", "AUD"), ("marginal_rate", "percent")):
            problems += [f"bracket {i} {key}: {p}" for p in check_value(unit, row[key])]
    if problems:
        return problems
    if rows[0]["from"] != 0:
        problems.append("the first bracket must start at zero")
    base = Decimal(0)
    for i, row in enumerate(rows):
        if i and row["from"] != rows[i - 1]["to"] + 1:
            problems.append(f"bracket {i} does not start $1 above the previous bracket")
        if Decimal(str(row["base_tax"])) != base:
            problems.append(f"bracket {i} base_tax {row['base_tax']} should be {base}")
        if row["to"] is not None:
            if row["to"] < row["from"]:
                problems.append(f"bracket {i} upper limit precedes its start")
            width = row["to"] - (row["from"] - 1 if i else row["from"])
            base += Decimal(width) * Decimal(str(row["marginal_rate"])) / 100
    if rows[-1]["to"] is not None:
        problems.append("the top bracket must have no upper limit (to: null)")
    return problems


def scale_renderings(rows):
    """Each taxed bracket as the ATO page prints it: "$4,020 plus 30c for each $1 over $45,000"."""
    def dollars(amount):
        amount = Decimal(str(amount))
        return f"${amount:,.0f}" if amount == amount.to_integral_value() else f"${amount:,.2f}"

    renderings = []
    for row in rows:
        if row["marginal_rate"] == 0:
            continue
        rate = f"{Decimal(str(row['marginal_rate'])).normalize():f}c for each $1 over {dollars(row['from'] - 1)}"
        renderings.append(rate if row["base_tax"] == 0 else f"{dollars(row['base_tax'])} plus {rate}")
    return renderings


def load_records():
    return [json.loads(p.read_text(encoding="utf-8")) for p in sorted(DATA.glob("*.json"))]


def snapshot_path(url):
    return SNAPSHOTS / (urlparse(url).path.rstrip("/").rsplit("/", 1)[-1] + ".txt")


def validate_record(record, page_text=None):
    """Return every problem found in one record, including the quote and pattern checks."""
    problems = [f"missing or wrong type: {k}" for k, t in FIELDS.items() if not isinstance(record.get(k), t)]
    if "value" not in record:
        problems.append("missing: value")
    if problems:
        return problems
    if record["unit"] not in UNITS:
        return [f"unknown unit {record['unit']!r}"]
    value_problems = check_value(record["unit"], record["value"])
    problems += value_problems
    if record["unit"] == "tax_scale" and not value_problems:
        # The arithmetic checks pass for any year's scale. Firecrawl once returned
        # the 2025-26 scale for 2026-27, and only the quote could tell them apart.
        quote = norm(record["quote"])
        problems += [f"quote does not show the bracket {r!r}"
                     for r in scale_renderings(record["value"]) if r not in quote]
    period = record["period"]
    if not period or set(period) - PERIOD_KEYS:
        problems.append(f"period keys must be a non-empty subset of {sorted(PERIOD_KEYS)}")
    if "income_year" in period:
        year = str(period["income_year"])
        if not INCOME_YEAR.fullmatch(year):
            problems.append(f"income_year {period['income_year']!r} is not like 2026-27")
        elif not (1 <= int(year[:4]) < 9999 and int(year[-2:]) == (int(year[:4]) + 1) % 100):
            problems.append(f"income_year {year!r} must describe adjacent calendar years between 1 and 9999")
    dates = {}
    for key in ("effective_from", "effective_to", "as_at"):
        if key in period:
            date_problems = check_value("date", period[key])
            problems += [f"{key}: {p}" for p in date_problems]
            if not date_problems:
                dates[key] = dt.date.fromisoformat(period[key])
    if "effective_from" in dates and "effective_to" in dates and dates["effective_to"] < dates["effective_from"]:
        problems.append("effective_to precedes effective_from")
    if not record["source_url"].startswith("https://www.ato.gov.au/"):
        problems.append("source_url is not a public ato.gov.au page")
    try:
        dt.datetime.fromisoformat(record["retrieved_at"])
    except ValueError:
        problems.append("retrieved_at is not an ISO timestamp")
    if not 0 < len(record["quote"]) <= 400:
        problems.append("quote must be 1 to 400 characters")
    if page_text is not None:
        problems += page_problems(record, page_text)
    return problems


def page_problems(record, page_text):
    """Check the quote and, where a pattern is given, re-read the value from the page text."""
    text = norm(page_text)
    problems = []
    for key in ("quote", "period_quote"):
        if key in record and norm(record[key]) not in text:
            problems.append(f"{key} not found on page")
    if "pattern" in record:
        try:
            pattern = re.compile(record["pattern"])
            if pattern.groups != 1:
                raise ValueError("expected exactly one capture group")
            if record["unit"] == "tax_scale":
                raise ValueError("a tax scale cannot use a single-value pattern")
            m = pattern.search(text)
            if not m:
                problems.append("value pattern not found on page")
            else:
                captured = m.group(1)
                if captured is None:
                    raise ValueError("the capture group did not match")
                found = parse_text(record["unit"], captured)
                expected = record["value"] if record["unit"] == "date" else Decimal(str(record["value"]))
                if found != expected:
                    problems.append(f"value changed: {record['value']} -> {captured}")
        except (re.error, ValueError, TypeError, InvalidOperation) as exc:
            problems.append(f"value pattern could not be read: {exc}")
    return problems


def fetch_text(url):
    """Load a page in the local nodriver browser (headed Chrome) and return its visible text.
    The 3 s wait lets the ATO page finish adding its menu, which lands up to a second after load."""
    run = subprocess.run([*NODRIVER, "text", url, "--json", "--wait", "3"], capture_output=True, text=True,
                         encoding="utf-8", timeout=180)
    if run.returncode not in (0, 1):  # 1 means HTTP 400 or above, reported below from the page details
        raise RuntimeError(run.stderr.strip()[-300:] or f"the browser exited with code {run.returncode}")
    page = json.loads(run.stdout)
    if not isinstance(page, dict) or not isinstance(page.get("text"), str):
        raise RuntimeError("the browser's reply is not a page")
    if run.returncode or page["status"] != 200 or not page["loaded"]:
        raise RuntimeError(f"HTTP {page['status']}, final URL {page['url']}")
    if len(norm(page["text"])) < 200:
        raise RuntimeError("page text is nearly empty")
    return plain_spaces(page["text"])


def fetch_all(urls):
    """Return {url: text or Exception}, pausing between requests to stay polite."""
    pages = {}
    for url in urls:
        try:
            pages[url] = fetch_text(url)
        except (OSError, RuntimeError, KeyError, ValueError, subprocess.SubprocessError) as exc:
            pages[url] = exc
        time.sleep(2)
    return pages


def cmd_snapshot(_args):
    urls = sorted({r["source_url"] for r in load_records()})
    SNAPSHOTS.mkdir(exist_ok=True)
    failed = 0
    for url, text in fetch_all(urls).items():
        if isinstance(text, Exception):
            failed += 1
            print(f"FAILED {url}: {text}")
        else:
            snapshot_path(url).write_text(text, encoding="utf-8")
            print(f"saved {snapshot_path(url).name}")
    return 1 if failed else 0


def cmd_validate(_args):
    failed = 0
    # File by file, so one unreadable record is reported with the others rather
    # than ending the run before they are checked.
    for path in sorted(DATA.glob("*.json")):
        try:
            record = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError, RecursionError) as exc:
            print(f"{path.stem}: cannot be read as a JSON record ({type(exc).__name__})")
            failed += 1
            continue
        if not isinstance(record, dict):
            print(f"{path.stem}: must hold one JSON object")
            failed += 1
            continue
        url = record.get("source_url", "")
        # validate_record reports a source_url that is not text.
        snap = snapshot_path(url) if isinstance(url, str) else None
        text = snap.read_text(encoding="utf-8") if snap is not None and snap.exists() else None
        problems = validate_record(record, text)
        if snap is not None and text is None:
            problems.append(f"no snapshot at {snap.relative_to(ROOT)}")
        for p in problems:
            print(f"{record.get('id')}: {p}")
        failed += bool(problems)
    print(f"{failed} record(s) with problems")
    return 1 if failed else 0


def digest(text):
    return hashlib.sha256(norm(page_body(text)).encode()).hexdigest()


def page_body(text):
    """The text between the ATO site menu and the page's QC reference line.

    A header or footer edit reaches every page at once, which buried the one page
    whose body gained a new year's figure among identical notes. A page without
    either marker keeps that end of its text.
    """
    lines = text.splitlines()
    # One capture joins the menu and search buttons into "MenuSearch".
    start = next((i + 1 for i, line in enumerate(lines) if line.strip() in ("Menu", "MenuSearch")), 0)
    end = next((i for i, line in enumerate(lines) if re.fullmatch(r"QC\s?\d+", line.strip())),
               len(lines))
    return "\n".join(lines[start:end])


def expiring(records, today, days=14):
    """Records that end within `days` of today, or already have, with no successor.

    A successor is a record with the same label starting the day after. Without
    this the SG record would keep passing after 30 June 2027 with no 2027-28 figure.
    """
    today = dt.date.fromisoformat(today)
    starts = {(r.get("label"), r.get("period", {}).get("effective_from")) for r in records}
    findings = []
    for record in records:
        if "effective_to" not in record.get("period", {}):
            continue
        end = dt.date.fromisoformat(record["period"]["effective_to"])
        following = (end + dt.timedelta(days=1)).isoformat()
        if (end - today).days <= days and (record.get("label"), following) not in starts:
            findings.append(f"`{record['id']}`: ends {end.isoformat()} and no record for "
                            f"this figure starts {following}")
    return findings


def build_report(records, pages, today):
    """Markdown change report. Findings: changed value, missing page, failed quote or pattern,
    and a record ending within 14 days with no successor."""
    findings, notes = [], []
    for record in records:
        page = pages[record["source_url"]]
        if isinstance(page, Exception):
            findings.append(f"`{record['id']}`: page missing or unreadable ({page})")
            continue
        findings += [f"`{record['id']}`: {p}" for p in page_problems(record, page)]
    findings += expiring(records, today)
    for url, page in pages.items():
        snap = snapshot_path(url)
        if not isinstance(page, Exception) and snap.exists():
            if digest(page) != digest(snap.read_text(encoding="utf-8")):
                notes.append(url)
    lines = [f"# AU tax rates check {today}", "",
             f"{len(records)} figures on {len(pages)} ATO pages. {len(findings)} finding(s).", "",
             "## Findings", ""]
    lines += [f"- {f}" for f in findings] or ["None."]
    if notes:
        lines += ["", "## Pages with changed wording", "",
                  "Not findings on their own. Review these pages for new or revised figures.", ""]
        lines += [f"- {n}" for n in notes]
    return "\n".join(lines) + "\n", findings


def cmd_check(args):
    records = load_records()
    pages = fetch_all(sorted({r["source_url"] for r in records}))
    today = dt.date.today().isoformat()
    report, findings = build_report(records, pages, today)
    out = Path(args.out) / f"au-tax-rates-check-{today}.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(report, encoding="utf-8")
    print(f"{len(findings)} finding(s); report at {out}")
    return 1 if findings else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("snapshot").set_defaults(func=cmd_snapshot)
    sub.add_parser("validate").set_defaults(func=cmd_validate)
    check = sub.add_parser("check")
    check.add_argument("--out", default=str(ROOT / "reports"), help="folder for the Markdown report")
    check.set_defaults(func=cmd_check)
    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
