"""Build the other benchmark ranges from the ATO's industry pages.

Development-time script. It is not imported by the runtime package. The ATO's
data.gov.au workbook carries only the key ranges; the labour, rent and motor
vehicle ranges (and cost of sales where it is not the key range) are published
only on each industry's "in detail" page. Save those pages as text first, for
example with a browser's batch text export, into a directory holding
``index.json`` (a list of objects with ``url``, ``final_url``, ``file``,
``title`` and ``status``) and one text file per page. Then:

    uv run python tools/build_other_benchmarks.py \
        --pages-dir <saved pages> \
        --dataset atobenchmark/data/benchmarks-2023-24.json \
        --retrieved 2026-09-29 \
        --out atobenchmark/data/other-benchmarks-2023-24.json

Every page must state the dataset's benchmark year, carry a last-updated date
and a QC reference, list the dataset's turnover bands in the same order, and
print every range the dataset publishes, equal to the dataset's own. A page
that disagrees contributes nothing and is named in the output with the reason,
because a range copied from the wrong page, band or year would otherwise sit
beside the right key range and read as published. The build fails unless the
excluded pages are exactly the ones named with --expect-excluded, so a page that
starts disagreeing later stops the next rebuild instead of vanishing quietly.

Each figure is copied from the page as printed; the only conversion is from a
whole-number percentage to a ratio. Where a page prints a single figure for a
band, it is stored as both bounds; no wider interval is inferred.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from atobenchmark.dataset import canonical_digest, normalise  # noqa: E402

HEADINGS = {
    "total expenses": "total_expenses_to_turnover",
    "cost of sales": "cost_of_sales_to_turnover",
    "labour": "labour_to_turnover",
    "rent": "rent_to_turnover",
    "motor vehicle expenses": "motor_vehicle_to_turnover",
}
OTHER_KEYS = (
    "cost_of_sales_to_turnover",
    "labour_to_turnover",
    "rent_to_turnover",
    "motor_vehicle_to_turnover",
)
# Pages print a ratio heading as "'Rent' divided by 'Annual turnover'", sometimes
# with a space inside the quotes or without "by", or as "Rent/turnover".
HEADING_RE = re.compile(
    r"^(?:'?\s*(?P<quoted>[A-Za-z ]+?)\s*'?\s+divided\s+(?:by\s+)?'?\s*Annual turnover\s*'?"
    r"|(?P<slashed>[A-Za-z ]+?)\s*/\s*(?:annual )?turnover)$",
    re.IGNORECASE,
)
AVERAGE_RE = re.compile(r"^\d+(?:\.\d+)?%$")
# A page sometimes prints a single figure ("1%") for a band; it becomes both bounds.
RANGE_RE = re.compile(r"^(\d+(?:\.\d+)?)%(?: to (\d+(?:\.\d+)?)%)?$")
BAND_LABEL_RE = re.compile(r"^(\$[\d,]+\s*[-\u2013\u2014]\s*\$[\d,]+|More than \$[\d,]+)$")
YEAR_RE = re.compile(r"^(Key|Other) benchmarks for (\d{4})[-\u2013](\d{2})$")
UPDATED_RE = re.compile(r"^Last updated (\d{1,2} [A-Z][a-z]+ \d{4})$")
FINANCIAL_YEAR_RE = re.compile(r"tax returns for the (\d{4})\D(\d{2}) financial year")
QC_RE = re.compile(r"^QC\d+$")
TITLE_SUFFIX = " | Australian Taxation Office"

#: Pages whose address and title both differ from the dataset's name. Each is
#: still held to the key-range cross-check, so a wrong entry here fails the build.
ADDRESS_ALIASES = {
    "motor-vehicle-retail-new-and-used": "Motor vehicle retail - new and used car",
    "roofing-services-includes-roof-tiling-guttering-and-metal-roofing": (
        "Roofing services, including roof tiling, guttering and metal roofing"
    ),
}


class BuildError(Exception):
    """Raised when a page cannot be read in the expected layout."""


class PageMismatch(BuildError):
    """Raised when a readable page disagrees with the dataset; the page is left out."""


#: More excluded pages than this means the pages and the dataset do not belong
#: together, not that a few pages carry errors.
MAX_EXCLUDED = 5


def slug(text: str) -> str:
    """The ATO builds each page address from the business type's name."""
    folded = normalise(text).replace("'", "")
    return re.sub(r"[^a-z0-9]+", "-", folded).strip("-")


def label_key(label: str) -> str:
    """Compare band labels however the dash and spacing are printed."""
    return re.sub(r"\s+", "", re.sub("[-\u2013\u2014]", "-", label)).lower()


def ratio(percent: str) -> str:
    return str(Decimal(percent) / 100)


def parse_section(lines: list[str], start: int, where: str) -> tuple[list[str], dict]:
    """Read one benchmark table from its year heading to the next non-ratio heading.

    Returns the band labels and {ratio key: [(min, max) per band, or None]}.
    """
    i = start + 1
    while i < len(lines) and lines[i] != "Annual turnover range":
        if YEAR_RE.match(lines[i]):
            raise BuildError(f"{where}: the next table starts before this one's bands")
        i += 1
    if i == len(lines):
        raise BuildError(f"{where}: no 'Annual turnover range' after the year heading")
    i += 1
    labels = []
    while i < len(lines) and BAND_LABEL_RE.match(lines[i]):
        labels.append(lines[i])
        i += 1
    if not labels:
        raise BuildError(f"{where}: no turnover band labels")
    ranges: dict[str, list[tuple[str, str] | None]] = {}
    while i < len(lines):
        heading = HEADING_RE.match(lines[i])
        if heading is None:
            # A key table prints each range's average ("Average cost of sales",
            # then one figure per band) before the next range. Step over it.
            averages = lines[i + 1 : i + 1 + len(labels)]
            if (
                lines[i].startswith("Average ")
                and len(averages) == len(labels)
                and all(AVERAGE_RE.match(value) for value in averages)
            ):
                i += 1 + len(labels)
                continue
            # A line followed by a full set of ranges is a ratio heading this
            # builder does not recognise, not the end of the table: stop rather
            # than keep the ratios read so far and drop the rest.
            following = lines[i + 1 : i + 1 + len(labels)]
            if len(following) == len(labels) and all(RANGE_RE.match(v) for v in following):
                raise BuildError(f"{where}: unrecognised benchmark heading {lines[i]!r}")
            break
        name = (heading.group("quoted") or heading.group("slashed")).strip().lower()
        key = HEADINGS.get(name)
        if key is None:
            raise BuildError(f"{where}: unrecognised benchmark heading {lines[i]!r}")
        if key in ranges:
            raise BuildError(f"{where}: {lines[i]!r} appears twice")
        i += 1
        values: list[tuple[str, str] | None] = []
        while i < len(lines) and len(values) < len(labels):
            match = RANGE_RE.match(lines[i])
            if match is None:
                raise BuildError(
                    f"{where}: expected a range such as '22% to 38%' under {key}, "
                    f"found {lines[i]!r}"
                )
            low = ratio(match.group(1))
            high = ratio(match.group(2) or match.group(1))
            if Decimal(low) > Decimal(high):
                raise BuildError(f"{where}: {key} range {lines[i]!r} runs backwards")
            values.append((low, high))
            i += 1
        if len(values) != len(labels):
            raise BuildError(f"{where}: {key} has {len(values)} ranges for {len(labels)} bands")
        ranges[key] = values
    if not ranges:
        raise BuildError(f"{where}: no benchmark ranges under the year heading")
    return labels, ranges


def parse_page(text: str, where: str) -> dict:
    # The pages print no-break spaces inside ranges ("54%\xa0to\xa073%"); str
    # patterns treat every Unicode space as \s, so one pass flattens them.
    # Some headings use curly quotes, and one closes a quote with an acute
    # accent ("'Motor vehicle expenses(U+00B4) divided by ...").
    for quote in (0x2018, 0x2019, 0x00B4, 0x0060):
        text = text.replace(chr(quote), "'")
    lines = [re.sub(r"\s+", " ", line).strip() for line in text.splitlines()]
    lines = [line for line in lines if line]
    sections = {}
    updated = qc = financial_year = None
    for index, line in enumerate(lines):
        stated = FINANCIAL_YEAR_RE.search(line)
        if stated and financial_year is None:
            financial_year = f"{stated.group(1)}-{stated.group(2)}"
        year = YEAR_RE.match(line)
        if year:
            kind, value = year.group(1).lower(), f"{year.group(2)}-{year.group(3)}"
            if kind in sections:
                raise BuildError(f"{where}: two {kind} benchmark tables")
            labels, ranges = parse_section(lines, index, f"{where} ({kind})")
            sections[kind] = {"year": value, "labels": labels, "ranges": ranges}
        elif UPDATED_RE.match(line) and updated is None:
            updated = UPDATED_RE.match(line).group(1)
        elif QC_RE.match(line):
            qc = line
    if "key" not in sections:
        raise BuildError(f"{where}: no key benchmark table")
    if financial_year is None:
        raise BuildError(f"{where}: the page does not state which financial year it covers")
    if updated is None or qc is None:
        raise BuildError(f"{where}: the page carries no last-updated date or QC reference")
    return {
        "sections": sections,
        "financial_year": financial_year,
        "last_updated": updated,
        "qc": qc,
    }


def page_entry(item: dict, business_type: dict, page_name: str, raw: bytes, year: str) -> dict:
    """Check one saved page against the dataset and return its other ranges."""
    where = item.get("final_url") or item["url"]
    name = business_type["name"]
    page = parse_page(raw.decode("utf-8"), f"{where}")
    bands = business_type["turnover_bands"]
    if page["financial_year"] != year:
        raise PageMismatch(
            f"{where}: the page covers {page['financial_year']}, dataset is {year}"
        )
    anomalies = []
    for kind, section in page["sections"].items():
        if section["year"] != year:
            # One page heads its key table "2023-23" while the rest of the
            # page says 2023-24. A key table is still usable because its
            # ranges must equal the dataset's below; an other table has no
            # such check, so its heading has to be right.
            if kind != "key":
                raise PageMismatch(
                    f"{where}: {kind} benchmarks are for {section['year']}, "
                    f"dataset is {year}"
                )
            anomalies.append(
                f"key table heading says {section['year']}; the page and its "
                f"ranges are {year}"
            )
        printed = [label_key(label) for label in section["labels"]]
        expected = [label_key(band["label"]) for band in bands]
        if printed != expected:
            raise PageMismatch(
                f"{where}: {kind} bands {section['labels']} do not match the dataset's "
                f"{[band['label'] for band in bands]}"
            )

    # Every range the dataset publishes must be printed on the page too, and
    # every range the dataset also publishes must agree, band by band.
    printed_keys = set()
    for section in page["sections"].values():
        printed_keys.update(section["ranges"])
    for band in bands:
        for key in ("cost_of_sales_to_turnover", "total_expenses_to_turnover"):
            if band.get(key) is not None and key not in printed_keys:
                raise PageMismatch(
                    f"{where}: the page does not print the dataset's {key} range"
                )
    for kind, section in page["sections"].items():
        for key, values in section["ranges"].items():
            for band, value in zip(bands, values):
                published = band.get(key)
                if published is None or value is None:
                    continue
                if (Decimal(published["min"]), Decimal(published["max"])) != (
                    Decimal(value[0]),
                    Decimal(value[1]),
                ):
                    raise PageMismatch(
                        f"{where}: {key} for {band['band']} is {value} on the page but "
                        f"{published} in the dataset"
                    )
    key_ratio = business_type["key_ratio"]
    if key_ratio not in page["sections"]["key"]["ranges"]:
        raise PageMismatch(f"{where}: the key table does not carry {key_ratio}")

    other = page["sections"].get("other", {"ranges": {}})["ranges"]
    out_bands = []
    for position, band in enumerate(bands):
        entry: dict = {"band": band["band"]}
        for key in OTHER_KEYS:
            values = other.get(key)
            if values is None or values[position] is None:
                entry[key] = None
                continue
            if band.get(key) is not None:
                # Already published in the dataset and checked equal above.
                entry[key] = None
                continue
            low, high = values[position]
            entry[key] = {"min": low, "max": high}
        out_bands.append(entry)

    return {
            "name": name,
            "page_title": page_name,
            "page_url": item.get("final_url") or item["url"],
            "page_last_updated": page["last_updated"],
            "page_reference": page["qc"],
            "text_sha256": hashlib.sha256(raw).hexdigest(),
            "page_anomalies": anomalies,
            "turnover_bands": out_bands,
    }


def build(
    pages_dir: Path,
    dataset_path: Path,
    retrieved: str,
    expect_excluded: frozenset[str] = frozenset(),
) -> dict:
    dataset_text = dataset_path.read_text(encoding="utf-8")
    dataset = json.loads(dataset_text)
    year = dataset["benchmark_year"]
    by_name = {normalise(bt["name"]): bt for bt in dataset["business_types"]}
    by_slug = {slug(bt["name"]): bt for bt in dataset["business_types"]}
    if len(by_slug) != len(dataset["business_types"]):
        raise BuildError("two business types share a page address")
    index = json.loads((pages_dir / "index.json").read_text(encoding="utf-8"))

    entries = []
    excluded = []
    seen = set()
    for item in index:
        where = item.get("final_url") or item["url"]
        if item.get("status") != 200 or item.get("error") or item.get("truncated"):
            raise BuildError(f"{where}: page was not saved whole (status {item.get('status')})")
        title = str(item.get("title", ""))
        if not title.endswith(TITLE_SUFFIX):
            raise BuildError(f"{where}: unexpected page title {title!r}")
        page_name = title[: -len(TITLE_SUFFIX)].strip()
        # The address carries the dataset's name; a few page titles reword it
        # ("Alarm installation services" for "Alarm systems installation").
        address = where.rstrip("/").rsplit("/", 1)[-1]
        from_address = by_slug.get(address) or by_name.get(
            normalise(ADDRESS_ALIASES.get(address, ""))
        )
        from_title = by_name.get(normalise(page_name))
        if from_address and from_title and from_address is not from_title:
            raise BuildError(f"{where}: address and title name different business types")
        business_type = from_address or from_title
        if business_type is None:
            raise BuildError(f"{where}: {page_name!r} is not a business type in {year}")
        name = business_type["name"]
        if name in seen:
            raise BuildError(f"{where}: {name!r} has a second page")
        seen.add(name)

        raw = (pages_dir / item["file"]).read_bytes()
        try:
            entries.append(page_entry(item, business_type, page_name, raw, year))
        except PageMismatch as exc:
            excluded.append({"name": name, "page_url": where, "reason": str(exc)})

    missing = sorted(bt["name"] for bt in dataset["business_types"] if bt["name"] not in seen)
    if missing:
        raise BuildError(f"no page for {len(missing)} business types: {missing[:5]}")
    actual = {item["name"] for item in excluded}
    if actual != expect_excluded:
        reasons = "; ".join(item["reason"] for item in excluded if item["name"] not in expect_excluded)
        raise BuildError(
            f"excluded pages {sorted(actual)} differ from --expect-excluded "
            f"{sorted(expect_excluded)}. {reasons}"
        )
    if len(excluded) > MAX_EXCLUDED:
        reasons = "; ".join(item["reason"] for item in excluded[:3])
        raise BuildError(f"{len(excluded)} pages disagree with the dataset: {reasons}")
    entries.sort(key=lambda entry: entry["name"])
    excluded.sort(key=lambda item: item["name"])
    return {
        "schema_version": 1,
        "benchmark_year": year,
        "business_type_count": len(entries),
        "source": {
            "publisher": "Australian Taxation Office",
            "pages": "Small business benchmarks, industry pages (in detail)",
            "index_url": "https://www.ato.gov.au/businesses-and-organisations/"
            "income-deductions-and-concessions/small-business-benchmarks/benchmarks-a-z",
            "retrieved": retrieved,
            "dataset_sha256": dataset["source"]["sha256"],
            "dataset_json_sha256": canonical_digest(dataset_text),
            "licence": "ATO copyright notice: free to copy, adapt, modify, transmit and "
            "distribute, but not in any way that suggests the ATO or the Commonwealth "
            "endorses the user or its products",
            "licence_url": "https://www.ato.gov.au/about-ato/using-our-website/copyright-notice",
            "caution": "Not all expenses are reported by every business. The ATO says to "
            "use these ranges only as a guide where they apply to the business.",
        },
        "business_types": entries,
        # A page that disagrees with the dataset contributes nothing, so its
        # business type keeps "no benchmark in this dataset" for these ratios.
        "excluded_pages": excluded,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pages-dir", required=True, type=Path)
    parser.add_argument("--dataset", required=True, type=Path)
    parser.add_argument("--retrieved", required=True, help="ISO date the pages were saved")
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument(
        "--expect-excluded",
        action="append",
        default=[],
        metavar="BUSINESS_TYPE",
        help="a business type whose page is known to disagree; repeat for each one",
    )
    args = parser.parse_args(argv)
    try:
        built = build(
            args.pages_dir, args.dataset, args.retrieved, frozenset(args.expect_excluded)
        )
    except BuildError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    text = json.dumps(built, indent=1, ensure_ascii=False) + "\n"
    args.out.write_text(text, encoding="utf-8", newline="\n")
    filled = sum(
        1
        for entry in built["business_types"]
        for band in entry["turnover_bands"]
        for key in OTHER_KEYS
        if band[key] is not None
    )
    print(f"wrote {args.out}: {built['business_type_count']} business types, {filled} ranges")
    for item in built["excluded_pages"]:
        print(f"excluded: {item['reason']}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
