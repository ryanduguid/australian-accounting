# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Check the payroll tax boundary advertised by the Python facade."""

from aus_accounting_mcp.resources import scope


def test_payroll_tax_scope_distinguishes_supported_worksheet_from_exclusions():
    advertised = scope()

    assert "payroll_tax" in advertised["calculation_worksheets"]
    assert advertised["unsupported_calculations"]["payroll_tax"] == (
        "Payroll tax grouping, part-year calculations, exemptions and rebates."
    )
