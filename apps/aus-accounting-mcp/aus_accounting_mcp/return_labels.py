# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""Individual return destinations from year- and channel-specific ATO instructions."""

from .outputs import ReturnDestination

DESTINATIONS: dict[tuple[str, str, str], ReturnDestination] = {
    ("2025-26", "paper", "employee_union_fees"): {
        "heading": "Other work-related expenses",
        "question": "D5",
        "label": "E",
        "source_url": (
            "https://www.ato.gov.au/forms-and-instructions/individual-tax-return-2026-instructions/"
            "deduction-questions-d1-d10-individual-tax-return-2026/"
            "d5-other-work-related-expenses-2026"
        ),
        "source_quote": (
            "union fees and subscriptions to trade, business or professional associations"
        ),
        "checked_on": "2026-11-23",
    },
    ("2025-26", "mytax", "employee_union_fees"): {
        "heading": "Other work-related expenses",
        "question": None,
        "label": None,
        "source_url": (
            "https://www.ato.gov.au/individuals-and-families/your-tax-return/"
            "instructions-to-complete-your-tax-return/mytax-instructions/2026/deductions/"
            "work-related-expenses/other-work-related-expenses"
        ),
        "source_quote": "show your union and professional fees from the Income statements and "
        "payment summaries section",
        "checked_on": "2026-11-23",
    },
}
