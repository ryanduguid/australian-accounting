"""
CLI interface for The Exchequer Tally (import package edwinnixon).
"""

import argparse
import sys
from datetime import date
from decimal import Decimal, InvalidOperation
from .corporate_tax import BaseRateEntityTest, determine_corporate_tax_rate, turnover_threshold_for
from .distribution_statement import generate_distribution_statement

MAX_MONEY_MAGNITUDE = Decimal("1000000000000.00")
MAX_MONEY_DECIMAL_PLACES = 2

def _finite_decimal(value: str) -> Decimal:
    """Parse a finite Decimal for a domain-specific argparse type."""
    try:
        parsed = Decimal(value)
    except (InvalidOperation, ValueError) as exc:
        raise argparse.ArgumentTypeError(f"not a decimal amount: {value!r}") from exc
    if not parsed.is_finite():
        raise argparse.ArgumentTypeError(f"not a finite decimal amount: {value!r}")
    return parsed


def money_type(value: str) -> Decimal:
    """Parse non-negative AUD amounts bounded to cents and $1 trillion."""
    parsed = _finite_decimal(value)
    exponent = parsed.as_tuple().exponent
    if parsed < 0 or parsed > MAX_MONEY_MAGNITUDE:
        raise argparse.ArgumentTypeError(
            f"money must be between 0 and {MAX_MONEY_MAGNITUDE} AUD: {value!r}"
        )
    if isinstance(exponent, int) and exponent < -MAX_MONEY_DECIMAL_PLACES:
        raise argparse.ArgumentTypeError(
            f"money cannot have more than {MAX_MONEY_DECIMAL_PLACES} decimal places: "
            f"{value!r}"
        )
    return parsed


def percentage_type(value: str) -> Decimal:
    """Parse a percentage in the inclusive range 0 to 100."""
    parsed = _finite_decimal(value)
    if not Decimal("0") <= parsed <= Decimal("100"):
        raise argparse.ArgumentTypeError(f"percentage must be between 0 and 100: {value!r}")
    return parsed


def rate_type(value: str) -> Decimal:
    """Parse a rate as a fraction strictly between 0 and 1."""
    parsed = _finite_decimal(value)
    if not Decimal("0") < parsed < Decimal("1"):
        raise argparse.ArgumentTypeError(f"rate must be between 0 and 1: {value!r}")
    return parsed

def main() -> int:
    parser = argparse.ArgumentParser(
        prog="the-exchequer-tally",
        description="The Exchequer Tally: corporate tax rate and franking account engine for Australian companies. Outputs are review aids, not tax advice.",
    )
    subparsers = parser.add_subparsers(dest="command", help="Available subcommands")

    # Command: bre-test
    bre_parser = subparsers.add_parser("bre-test", help="Test Base Rate Entity (BRE) eligibility under s 23AA ITRA 1986")
    bre_parser.add_argument("--fy", type=int, required=True, help="Financial Year ending (e.g. 2025)")
    bre_parser.add_argument("--turnover", type=money_type, required=True, help="Aggregated turnover ($)")
    bre_parser.add_argument("--assessable", type=money_type, required=True, help="Total assessable income ($)")
    bre_parser.add_argument("--passive", type=money_type, required=True, help="Base Rate Entity Passive Income ($)")

    # Command: dist-statement
    dist_parser = subparsers.add_parser("dist-statement", help="Generate distribution statement details")
    dist_parser.add_argument("--entity", type=str, required=True, help="Company name")
    dist_parser.add_argument("--acn", type=str, required=True, help="ACN or ABN")
    dist_parser.add_argument("--recipient", type=str, required=True, help="Shareholder name")
    dist_parser.add_argument("--amount", type=money_type, required=True, help="Total dividend distribution ($)")
    dist_parser.add_argument("--franking-pct", type=percentage_type, default=Decimal("100.00"), help="Franking percentage (e.g. 100)")
    dist_parser.add_argument("--tax-rate", type=rate_type, default=Decimal("0.25"), help="Corporate tax rate (0.25 or 0.30)")

    args = parser.parse_args()

    try:
        return _dispatch(args, parser)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


NOT_ADVICE = "Not advice. Review aid only; confirm against current law before acting."


def _dispatch(args: argparse.Namespace, parser: argparse.ArgumentParser) -> int:
    if args.command == "bre-test":
        test = BaseRateEntityTest(
            financial_year=args.fy,
            aggregated_turnover=args.turnover,
            assessable_income=args.assessable,
            passive_income=args.passive,
        )
        res = determine_corporate_tax_rate(test)
        print("=" * 60)
        print(f"Base Rate Entity (BRE) Evaluation — FY{args.fy}")
        print("=" * 60)
        threshold_m = turnover_threshold_for(args.fy) / Decimal("1000000")
        print(f"Aggregated Turnover:     ${args.turnover:,.2f} (< ${threshold_m:.0f}M: {test.is_aggregated_turnover_eligible})")
        passive_pct = test.passive_income_percentage
        passive_display = "n/a (no assessable income)" if passive_pct is None else f"{passive_pct:.2f}%"
        print(f"Passive Income Ratio:    {passive_display} (<= 80%: {test.is_brepi_eligible})")
        print(f"Base Rate Entity:        {res.is_base_rate_entity}")
        print(f"Applicable Tax Rate:     {res.applicable_rate * 100:.1f}%")
        print(f"Statutory Basis:         {res.statutory_basis}")
        print(NOT_ADVICE)
        print("=" * 60)
        return 0

    elif args.command == "dist-statement":
        stmt = generate_distribution_statement(
            entity_name=args.entity,
            abn_or_acn=args.acn,
            recipient_name=args.recipient,
            payment_date=date.today(),
            total_distribution=args.amount,
            franking_percentage=args.franking_pct,
            corporate_tax_rate=args.tax_rate,
        )
        print("=" * 60)
        print(f"Australian Dividend Distribution Statement — {stmt.entity_name}")
        print("=" * 60)
        # s 202-75(2)(a): the statement must identify the entity making the distribution.
        print(f"ACN/ABN:                 {stmt.abn_or_acn}")
        print(f"Recipient:               {stmt.recipient_name}")
        print(f"Payment Date:            {stmt.payment_date.isoformat()}")
        print(f"Franked Dividend Amount: ${stmt.franked_amount:,.2f}")
        print(f"Unfranked Amount:        ${stmt.unfranked_amount:,.2f}")
        print(f"Franking Credit:         ${stmt.franking_credit:,.2f}")
        print(f"Franking Percentage:     {stmt.franking_percentage:.2f}%")
        print(f"Gross Assessable:        ${stmt.gross_assessable_income:,.2f}")
        print(NOT_ADVICE)
        print("=" * 60)
        return 0

    parser.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main())
