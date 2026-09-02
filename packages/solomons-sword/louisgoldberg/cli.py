"""
CLI interface for Solomon's Sword (import package louisgoldberg).
"""

import argparse
import sys
from decimal import Decimal, InvalidOperation
from .section100a import evaluate_section100a_risk
from .section99b import ForeignTrustReceipt, evaluate_section99b_liability


NOT_ADVICE = "Not advice. Review aid only; confirm against current law and the trust deed before acting."
# Output carries the beneficiary name the operator supplied, because a workpaper
# line item is unusable without it. That makes stdout client data: send it to the
# firm's secure location, never to a path inside a repository.
DATA_BOUNDARY = "Output contains client data. Write it only to the firm's approved secure location."



def decimal_type(value: str) -> Decimal:
    """Fail-closed argparse type for Decimal money."""
    try:
        parsed = Decimal(value)
    except (InvalidOperation, ValueError) as exc:
        raise argparse.ArgumentTypeError(f"not a decimal amount: {value!r}") from exc
    if not parsed.is_finite():
        raise argparse.ArgumentTypeError(f"not a finite decimal amount: {value!r}")
    return parsed

def main() -> int:
    parser = argparse.ArgumentParser(
        prog="solomons-sword",
        description="Solomon's Sword: Division 6 trust allocation with s 100A and s 99B checks. Review aid, not advice.",
    )
    subparsers = parser.add_subparsers(dest="command", help="Available subcommands")

    # Command: s100a-check
    s100a_parser = subparsers.add_parser("s100a-check", help="Evaluate Section 100A reimbursement agreement risk")
    s100a_parser.add_argument("--beneficiary", type=str, required=True, help="Beneficiary name")
    s100a_parser.add_argument("--amount", type=decimal_type, required=True, help="Distribution amount ($)")
    s100a_parser.add_argument("--adult-child", action="store_true", help="Beneficiary is an adult child")
    s100a_parser.add_argument("--retained-by-parents", action="store_true", help="Funds retained by parents without loan")
    s100a_parser.add_argument("--circular", action="store_true", help="Circular flow of funds present")
    received = s100a_parser.add_mutually_exclusive_group()
    received.add_argument("--received-funds", action="store_true", help="Beneficiary received and retained the funds")
    received.add_argument("--funds-not-received", action="store_true", help="Beneficiary did not receive the funds")

    # Command: s99b-check
    s99b_parser = subparsers.add_parser("s99b-check", help="Evaluate Section 99B foreign trust distribution")
    s99b_parser.add_argument("--beneficiary", type=str, required=True, help="Beneficiary name")
    s99b_parser.add_argument("--gross", type=decimal_type, required=True, help="Gross amount received AUD ($)")
    s99b_parser.add_argument("--corpus", type=decimal_type, default=Decimal("0.00"), help="Corpus / capital settlement ($)")
    s99b_parser.add_argument("--corpus-attributable", type=decimal_type, default=Decimal("0.00"), help="Part of the corpus attributable to amounts that would have been assessable to a resident (s 99B(2)(a) proviso)")
    s99b_parser.add_argument("--not-assessable-to-resident", type=decimal_type, default=Decimal("0.00"), help="Amounts that would not have been assessable to a resident (s 99B(2)(b))")
    s99b_parser.add_argument("--prior-taxed", type=decimal_type, default=Decimal("0.00"), help="Amounts already assessed under s 97/98/99/99A (s 99B(2)(c))")

    args = parser.parse_args()

    try:
        return _dispatch(args, parser)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


def _dispatch(args: argparse.Namespace, parser: argparse.ArgumentParser) -> int:

    if args.command == "s100a-check":
        res = evaluate_section100a_risk(
            beneficiary_name=args.beneficiary,
            distribution_amount=args.amount,
            beneficiary_is_adult_child=args.adult_child,
            funds_retained_by_parents_without_loan=args.retained_by_parents,
            circular_flow_of_funds=args.circular,
            beneficiary_actually_received_funds=(
                True if args.received_funds else False if args.funds_not_received else None
            ),
        )
        print("=" * 60)
        print(f"Section 100A Risk Evaluation — {res.beneficiary_name}")
        print("=" * 60)
        print(f"Distribution Amount:     ${res.distribution_amount:,.2f}")
        print(f"Risk Zone:               {res.risk_zone.value}")
        # None is the model's "undecided", which has to read as that on the
        # workpaper rather than as the word None.
        ofd = res.is_ordinary_family_dealing
        print(f"Ordinary Family Dealing: {'Not determined' if ofd is None else ofd}")
        if res.risk_factors_identified:
            print(f"Risk Factors:            {', '.join(res.risk_factors_identified)}")
        print(f"Consequence:             {res.tax_consequence_summary}")
        print(NOT_ADVICE)
        print(DATA_BOUNDARY)
        print("=" * 60)
        return 0

    elif args.command == "s99b-check":
        receipt = ForeignTrustReceipt(
            beneficiary_name=args.beneficiary,
            gross_amount_received_aud=args.gross,
            corpus_amount_aud=args.corpus,
            corpus_attributable_to_notional_assessable_income_aud=args.corpus_attributable,
            not_assessable_to_resident_aud=args.not_assessable_to_resident,
            already_assessed_under_div6_aud=args.prior_taxed,
        )
        s99b = evaluate_section99b_liability(receipt)
        print("=" * 60)
        print(f"Section 99B Assessment — {s99b.beneficiary_name}")
        print("=" * 60)
        print(f"Gross Receipt:           ${s99b.gross_receipt:,.2f}")
        print(f"Corpus Exemption:        ${s99b.corpus_exemption:,.2f}")
        print(f"Assessable under s99B:   ${s99b.assessable_income_under_s99b:,.2f}")
        print(f"Basis:                   {s99b.statutory_basis}")
        print(NOT_ADVICE)
        print(DATA_BOUNDARY)
        print("=" * 60)
        return 0

    parser.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main())
