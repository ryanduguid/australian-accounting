"""Keep the one-contribution boundary visible when related paydays change a result."""

import asyncio
from datetime import date
from decimal import Decimal

from paydaysuper.calendar import load_calendar
from paydaysuper.deadlines import ContribLine
from paydaysuper.rates import load_gic
from paydaysuper.report import assess

from aus_accounting_mcp.server import mcp


def test_single_contribution_warns_when_the_engine_can_align_related_paydays():
    related = [
        ContribLine("synthetic", date(2027, 7, 1), Decimal("120.00"),
                    received=date(2027, 7, 5), first_to_fund=True, row=1),
        ContribLine("synthetic", date(2027, 7, 8), Decimal("120.00"),
                    received=date(2027, 7, 20), row=2),
    ]
    together = assess(related, load_calendar(), load_gic(), date(2027, 8, 1))[-1]
    assert together.verdict == "ON_TIME"
    assert together.deadline.pathway == "ITEM4_ALIGNED"
    assert together.deadline.due == date(2027, 7, 29)

    single = asyncio.run(mcp.call_tool("calc_payday_super_deadline", {
        "employee_id": "synthetic", "qe_day": "2027-07-08", "sg_amount": "120.00",
        "received": "2027-07-20", "as_at": "2027-08-01",
    })).structured_content
    assert single["result"]["verdict"] == "LATE"
    assert single["result"]["due"] == "2027-07-19"
    assert single["assessment_scope"] == "single_contribution"
    assert any("s 18C(2) item 4" in caveat for caveat in single["result"]["caveats"])
    assert any("related contributions" in caveat for caveat in single["result"]["caveats"])
