"""
Section 99B ITAA 1936 Assessment for receipts from non-resident / foreign trusts.
"""

from dataclasses import dataclass
from decimal import Decimal


@dataclass(frozen=True)
class ForeignTrustReceipt:
    beneficiary_name: str
    gross_amount_received_aud: Decimal
    # Stated, never defaulted: s 99B(1) applies only where the beneficiary was a
    # resident at some time during the year of income, and a default of True
    # would assess a receipt on an assumed residency.
    beneficiary_was_resident_during_year: bool
    # s 99B(2)(a): corpus, EXCEPT to the extent it is attributable to amounts
    # that would have been assessable had a resident derived them. Supply the
    # corpus already net of that carve-out, or state the attributable part.
    corpus_amount_aud: Decimal = Decimal("0.00")
    corpus_attributable_to_notional_assessable_income_aud: Decimal = Decimal("0.00")
    # s 99B(2)(b): amounts that would not have been assessable to a resident.
    not_assessable_to_resident_aud: Decimal = Decimal("0.00")
    already_assessed_under_div6_aud: Decimal = Decimal("0.00")
    source_country: str = "Foreign"


@dataclass(frozen=True)
class Section99BAssessment:
    beneficiary_name: str
    gross_receipt: Decimal
    corpus_exemption: Decimal
    not_assessable_to_resident_exemption: Decimal
    prior_assessed_exemption: Decimal
    assessable_income_under_s99b: Decimal
    statutory_basis: str
    # One caveat naming every exemption amount that came through as nil. The
    # nil defaults are deliberate and conservative, but they make an unsupplied
    # exemption indistinguishable from an established nil, so the result says so
    # rather than presenting the assessable amount as settled.
    caveats: tuple[str, ...] = ()


def evaluate_section99b_liability(receipt: ForeignTrustReceipt) -> Section99BAssessment:
    """
    Calculate the amount included under s 99B(1) less the s 99B(2)(a), (b) and
    (c) reductions, on the compilation in force from 1 July 2026.

    s 99B(1) includes the amount only where the beneficiary was a resident at
    some time during the year of income; a non-resident beneficiary is refused
    rather than assessed. The s 99B(2)(a) corpus reduction excludes corpus
    attributable to amounts that would have been assessable had a resident
    derived them, so that part is added back.

    The 3 exemption amounts default to nil, which is the conservative direction
    but makes an unsupplied figure look like an established one. Each nil is
    named in the result's single caveat.
    """
    for name in (
        "gross_amount_received_aud",
        "corpus_amount_aud",
        "corpus_attributable_to_notional_assessable_income_aud",
        "not_assessable_to_resident_aud",
        "already_assessed_under_div6_aud",
    ):
        value = getattr(receipt, name)
        if not value.is_finite() or value < Decimal("0.00"):
            raise ValueError(f"{name} must be a non-negative finite amount, got {value}")

    if receipt.beneficiary_was_resident_during_year is None:
        raise ValueError(
            "residency during the year of income is not established; s 99B(1) ITAA "
            "1936 turns on it, so state it as True or False rather than assessing "
            "the receipt on an assumption"
        )
    if not receipt.beneficiary_was_resident_during_year:
        raise ValueError(
            "s 99B(1) ITAA 1936 applies only where the beneficiary was a resident "
            "at some time during the year of income; this receipt is outside it"
        )

    gross = receipt.gross_amount_received_aud
    attributable = receipt.corpus_attributable_to_notional_assessable_income_aud
    if attributable > receipt.corpus_amount_aud:
        raise ValueError(
            "corpus attributable to notionally assessable income cannot exceed "
            f"the corpus amount ({attributable} > {receipt.corpus_amount_aud})"
        )
    corpus_exempt = receipt.corpus_amount_aud - attributable
    not_assessable = receipt.not_assessable_to_resident_aud
    prior_taxed = receipt.already_assessed_under_div6_aud

    exemptions = corpus_exempt + not_assessable + prior_taxed
    if exemptions > gross:
        raise ValueError(
            f"exemptions (${exemptions:,.2f}) exceed the gross receipt (${gross:,.2f}); "
            "check the inputs rather than assuming a nil assessable amount"
        )
    assessable = gross - exemptions

    # A nil exemption is the largest assessable amount this section can produce,
    # so a figure nobody supplied and a figure established as nil reach the same
    # result. That is the conservative way round, but it must not read as settled.
    nil_exemptions = [
        name
        for name, value in (
            ("corpus_amount_aud", receipt.corpus_amount_aud),
            ("not_assessable_to_resident_aud", receipt.not_assessable_to_resident_aud),
            ("already_assessed_under_div6_aud", receipt.already_assessed_under_div6_aud),
            ("corpus_attributable_to_notional_assessable_income_aud", attributable),
        )
        if value == Decimal("0.00")
    ]
    caveats: tuple[str, ...] = ()
    if nil_exemptions:
        caveats = (
            "These exemption amounts are nil, and a nil default cannot be told apart "
            f"from a figure that was never supplied: {', '.join(nil_exemptions)}. A nil "
            "exemption gives the largest assessable amount, so establish each figure "
            "from the trust's records before relying on this result.",
        )

    basis = (
        f"s 99B(1) ITAA 1936: gross receipt ${gross:,.2f} less corpus exemption "
        f"${corpus_exempt:,.2f} (s 99B(2)(a), net of ${attributable:,.2f} attributable to "
        f"amounts that would have been assessable to a resident), less ${not_assessable:,.2f} "
        f"(s 99B(2)(b)) and ${prior_taxed:,.2f} already taxed (s 99B(2)(c)). "
        f"Assessable: ${assessable:,.2f}. Review aid only, not advice."
    )

    return Section99BAssessment(
        beneficiary_name=receipt.beneficiary_name,
        gross_receipt=gross,
        corpus_exemption=corpus_exempt,
        not_assessable_to_resident_exemption=not_assessable,
        prior_assessed_exemption=prior_taxed,
        assessable_income_under_s99b=assessable,
        statutory_basis=basis,
        caveats=caveats,
    )
