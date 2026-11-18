# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Generate data/business_days.json: the national non-business-day table.

DEV-TIME TOOL ONLY. The runtime never imports python-holidays: compliance
output must not change because a transitive dependency updated, and the raw
package output is wrong for SGAA s 6(1) anyway (it lists Brisbane-area
Royal Queensland Show as a QLD PUBLIC holiday).

Method (SGAA 1992 s 6(1) "business day"):
  a day is NOT a business day if it is Sat/Sun, or a public holiday for the
  WHOLE of any State, the ACT or the NT. So: union of PUBLIC-category
  holidays across the 8 listed jurisdictions, minus entries that are not
  whole-of-jurisdiction, weekends excluded (computed at runtime).

Usage, from the repository root:
    uv run --locked python packages/payday-super-checker/tools/generate_calendar.py > packages/payday-super-checker/data/business_days.json
Then hand-review every line against the 8 state/territory government pages
before shipping, record the review date, and raise `verified_until` only to
the last date every jurisdiction covers. The raw generator deliberately
claims no coverage past regime commencement and marks every later date
provisional until confirmed.
"""
from __future__ import annotations

import json
import sys
from collections import defaultdict
from datetime import date

import holidays  # dev-only, pinned

JURISDICTIONS = ["ACT", "NSW", "NT", "QLD", "SA", "TAS", "VIC", "WA"]
YEARS = [2026, 2027, 2028]
RAW_VERIFIED_UNTIL = date(2026, 7, 1)

# Official dates python-holidays 0.108 gets wrong, applied to its output for
# one jurisdiction before the whole-of-jurisdiction filter. A name adds or
# renames the day; None removes it. Removing a date the package does not list
# raises, so a package update that fixes a date has to drop its entry here.
CORRECTIONS: dict[tuple[str, date], str | None] = {
    # WA Act No. 19 of 2026, amending the Public and Bank Holidays Act 1972,
    # changes WA's holidays from 2028, as the WA page read on 14 November
    # 2026 lists.
    # Labour Day moves to the 2nd Monday in March.
    ("WA", date(2028, 3, 6)): None,
    ("WA", date(2028, 3, 13)): "Labour Day",
    # Easter Saturday becomes a public holiday.
    ("WA", date(2028, 4, 15)): "Easter Saturday",
    # King's Birthday moves to the 2nd Monday in June.
    ("WA", date(2028, 6, 12)): "King's Birthday",
    # Show Day takes King's Birthday's old September date.
    ("WA", date(2028, 9, 25)): "Show Day",
    # Western Australia Day moves to the 2nd Monday in November.
    ("WA", date(2028, 6, 5)): None,
    ("WA", date(2028, 11, 13)): "Western Australia Day",
    # The ACT declared an additional holiday on Sunday 25 April 2027 besides
    # the Monday 26 April substitute (ACT 2027 public holidays PDF, read on
    # 14 November 2026).
    ("ACT", date(2027, 4, 25)): "ANZAC Day",
}

# Entries in python-holidays' PUBLIC category that are NOT public holidays
# for the whole of their jurisdiction in the years listed, so must not remove
# a business day. Match the jurisdiction too: the same holiday name can be
# whole-of-state in one place and locally substituted in another.
NOT_WHOLE_OF_JURISDICTION: dict[tuple[str, str], list[int]] = {
    ("QLD", "The Royal Queensland Show"): YEARS,  # Brisbane area only (Ekka)
    # Regional WA areas substitute another day, so the default celebration
    # date does not apply throughout WA. From 2028 King's Birthday is the
    # 2nd Monday in June throughout WA, and Show Day is the day regions
    # substitute.
    ("WA", "King's Birthday"): [2026, 2027],
    ("WA", "Show Day"): YEARS,
    # Non-metro Victorian councils may arrange an alternate local holiday.
    ("VIC", "Melbourne Cup Day"): YEARS,
}

OFFICIAL_SOURCES = {
    "definition": (
        "https://www.ato.gov.au/businesses-and-organisations/super-for-employers/"
        "paying-super-on-payday/payment-deadlines-for-payday-super"
    ),
    "ACT": (
        "https://www.act.gov.au/living-in-the-act/"
        "public-holidays-school-terms-and-daylight-saving"
    ),
    "NSW": "https://www.nsw.gov.au/about-nsw/public-holidays",
    "NT": "https://nt.gov.au/nt-public-holidays",
    "QLD": "https://www.qld.gov.au/recreation/travel/holidays/public",
    "SA": "https://safework.sa.gov.au/resources/public-holidays",
    "TAS": "https://worksafe.tas.gov.au/topics/laws-and-compliance/public-holidays",
    "VIC": (
        "https://business.vic.gov.au/business-information/public-holidays/"
        "victorian-public-holidays-2027"
    ),
    "WA": (
        "https://www.wa.gov.au/service/employment/workplace-arrangements/"
        "public-holidays-western-australia"
    ),
}


def main() -> None:
    merged: dict[date, dict] = {}
    for jur in JURISDICTIONS:
        cal = holidays.Australia(subdiv=jur, years=YEARS, categories=("public",))
        days = dict(cal.items())
        for (corrected_jur, d), name in CORRECTIONS.items():
            if corrected_jur != jur:
                continue
            if name is None:
                del days[d]
            else:
                days[d] = name
        for d, name in sorted(days.items()):
            name = name.strip()
            if d.year in NOT_WHOLE_OF_JURISDICTION.get((jur, name), []):
                print(f"SKIP (not whole-of-state): {d} {jur} {name}", file=sys.stderr)
                continue
            entry = merged.setdefault(d, {"date": d.isoformat(), "names": defaultdict(list)})
            entry["names"][name].append(jur)

    out = []
    for d in sorted(merged):
        names = merged[d]["names"]
        label = "; ".join(sorted(names))
        jurs = sorted({j for js in names.values() for j in js})
        # Generated dates are reference candidates, not primary-source
        # verification. Keep every date after the raw completeness boundary
        # provisional until a human checks the official jurisdiction pages.
        provisional = d > RAW_VERIFIED_UNTIL
        out.append(
            {
                "date": d.isoformat(),
                "name": label,
                "jurisdictions": jurs,
                "provisional": provisional,
            }
        )

    # Provenance comes from the environment that actually ran, not from
    # literals: a hardcoded pin already went stale once (0.101 survived the
    # bump to 0.102), and a hardcoded date would stamp every future
    # regeneration with a false generation date.
    doc = {
        "_comment": (
            "Non-business days under SGAA 1992 s 6(1): public holidays applying to "
            "the WHOLE of any State/ACT/NT. The runtime excludes weekends separately; "
            "holiday rows can also fall on a weekend. "
            f"Generated by tools/generate_calendar.py (holidays=={holidays.__version__}), "
            "and REQUIRES hand-checking against the official jurisdiction sources "
            "recorded in this file before shipping. Part-day holidays (SA/QLD/NT "
            "Christmas & New Year's Eve evenings), "
            "regional holidays, WA King's Birthday before 2028, WA Show Day from 2028 "
            "and Melbourne Cup Day are excluded "
            "because they do not apply throughout a jurisdiction. Unconfirmed dates "
            "are retained as provisional reference entries but do not extend deadlines."
        ),
        "verified_from": "2026-07-01",
        # Generation is not official verification. A reviewer raises this
        # horizon only after checking every jurisdiction and records that
        # review date below. This makes accidental raw output fail closed.
        "verified_until": RAW_VERIFIED_UNTIL.isoformat(),
        "generated": date.today().isoformat(),
        "official_sources": {"checked": None, **OFFICIAL_SOURCES},
        "non_business_days": out,
    }
    json.dump(doc, sys.stdout, indent=2)
    print()


if __name__ == "__main__":
    main()
