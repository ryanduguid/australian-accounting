"""Invalid inputs are refused before any date is worked out."""

from datetime import date

import pytest
from lodgmentcalendar import Facts, due_dates
from lodgmentcalendar.due_dates import FACT_MEANINGS, SOURCES


@pytest.mark.parametrize("args,kwargs,message", [
    (("gst_return", "trust", "2026-27", "self"), {"reporting_cycle": "monthly"},
     "obligation must be one of"),
    (("bas", "sole_trader", "2026-27", "self"), {"reporting_cycle": "monthly"},
     "entity_type must be one of"),
    (("bas", "trust", "2026-27", "accountant"), {"reporting_cycle": "monthly"},
     "channel must be one of"),
    (("bas", "trust", "2026/27", "self"), {"reporting_cycle": "monthly"},
     "financial_year must look like 2026-27"),
    (("bas", "trust", "2026-28", "self"), {"reporting_cycle": "monthly"},
     "financial_year must look like 2026-27"),
    (("bas", "trust", 2026, "self"), {"reporting_cycle": "monthly"},
     "financial_year must look like 2026-27"),
    (("bas", "trust", "2026-27", "self"), {}, "reporting_cycle must be one of"),
    (("ias", "trust", "2026-27", "self"), {"reporting_cycle": "annual"},
     "reporting_cycle must be one of"),
    (("income_tax_return", "trust", "2025-26", "self"), {"reporting_cycle": "monthly"},
     "reporting_cycle applies only to bas and ias"),
    (("income_tax_return", "company", "2025-26", "bas_agent"), {},
     "a BAS agent cannot lodge an income tax return"),
    (("bas", "trust", "2026-27", "self"), {"reporting_cycle": "monthly", "facts": {}},
     "facts must be a Facts instance"),
    (("bas", "trust", "2026-27", "self"),
     {"reporting_cycle": "monthly", "facts": Facts(lodges_electronically=1)},
     "facts.lodges_electronically must be true, false or null"),
    (("bas", "trust", "2026-27", "self"),
     {"reporting_cycle": "monthly", "facts": Facts(new_registrant="yes")},
     "facts.new_registrant must be true, false or null"),
])
def test_invalid_inputs_are_refused(args, kwargs, message):
    with pytest.raises(ValueError, match=message):
        due_dates(*args, **kwargs)


def test_the_turn_of_the_century_year_is_accepted():
    result = due_dates("bas", "trust", "2099-00", "self", reporting_cycle="monthly")
    assert result["obligations"][0]["rule_id"] == "YEAR-NOT-RECORDED"


def test_every_fact_has_a_meaning_and_every_source_is_an_ato_page():
    assert set(FACT_MEANINGS) == set(Facts.__dataclass_fields__)
    assert all(url.startswith("https://www.ato.gov.au/") and provision
               for url, provision in SOURCES.values())


def test_no_modelled_date_reaches_the_conflicting_25_september_row():
    # The ATO table gives 28 September 2026 as the first business day after 25
    # September while listing 28 September as a WA holiday. No modelled date may
    # fall on 25 September 2026 until that conflict is settled.
    checks = [("bas", "monthly"), ("bas", "quarterly"), ("ias", "monthly"),
              ("ias", "quarterly")]
    for obligation, cycle in checks:
        for channel in ("self", "tax_agent"):
            result = due_dates(obligation, "trust", "2026-27", channel, reporting_cycle=cycle,
                               facts=Facts(lodges_electronically=True,
                                           previous_statement_lodged_electronically=True))
            for row in result["obligations"]:
                assert row["nominal_due_date"] != date(2026, 9, 25).isoformat()
