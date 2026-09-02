from datetime import date
from decimal import Decimal

import pytest

from edwinnixon.distribution_statement import generate_distribution_statement


NON_FINITE = [Decimal("NaN"), Decimal("Infinity"), Decimal("-Infinity")]


def _generate(**overrides: Decimal) -> None:
    values = {
        "total_distribution": Decimal("100.00"),
        "franking_percentage": Decimal("100.00"),
        "corporate_tax_rate": Decimal("0.25"),
    }
    values.update(overrides)
    generate_distribution_statement(
        entity_name="Example Pty Ltd",
        abn_or_acn="000000000",
        recipient_name="Example Recipient",
        payment_date=date(2026, 9, 2),
        **values,
    )


@pytest.mark.parametrize("value", NON_FINITE)
def test_rejects_non_finite_franking_percentage(value: Decimal) -> None:
    with pytest.raises(ValueError, match="finite"):
        _generate(franking_percentage=value)


@pytest.mark.parametrize("value", NON_FINITE)
def test_rejects_non_finite_corporate_tax_rate(value: Decimal) -> None:
    with pytest.raises(ValueError, match="finite"):
        _generate(corporate_tax_rate=value)
