"""
Franking account ledger, balance tracking, and Franking Deficit Tax (FDT) calculations
under Part 3-6 (Divisions 205 and 214) of the Income Tax Assessment Act 1997.
"""

from dataclasses import dataclass, field
from datetime import date
from decimal import Decimal, ROUND_HALF_UP
from enum import Enum
from typing import List


class FrankingEntryType(str, Enum):
    # Credits (s 205-15)
    PAYG_INSTALMENT = "PAYG_INSTALMENT"             # Item 1: Payment of PAYG instalment
    COMPANY_TAX_PAYMENT = "COMPANY_TAX_PAYMENT"     # Item 2: Payment of company tax assessment
    FRANKED_DISTRIBUTION_REC = "FRANKED_DIST_REC"   # Item 3: Receipt of franked distribution
    FDT_LIABILITY = "FDT_LIABILITY"                 # s 205-15: Liability to franking deficit tax

    # Debits (s 205-30)
    FRANKED_DISTRIBUTION_PAID = "FRANKED_DIST_PAID" # Item 1: Franked distribution made
    TAX_REFUND = "TAX_REFUND"                       # Item 2: Receipt of tax refund
    # Item 3 is the UNDER-franking debit for a distribution franked below the
    # benchmark in breach of the rule (s 203-50(2) shortfall). Over-franking
    # instead attracts over-franking tax, a tax liability outside this ledger.
    UNDER_FRANKING_DEBIT = "UNDER_FRANKING_DEBIT"


@dataclass(frozen=True)
class FrankingEntry:
    entry_date: date
    entry_type: FrankingEntryType
    amount: Decimal
    description: str
    statutory_reference: str = ""

    @property
    def is_credit(self) -> bool:
        return self.entry_type in {
            FrankingEntryType.PAYG_INSTALMENT,
            FrankingEntryType.COMPANY_TAX_PAYMENT,
            FrankingEntryType.FRANKED_DISTRIBUTION_REC,
            FrankingEntryType.FDT_LIABILITY,
        }

    @property
    def is_debit(self) -> bool:
        return not self.is_credit


@dataclass(frozen=True)
class FrankingDeficitResult:
    closing_balance: Decimal
    has_deficit: bool
    franking_deficit_tax: Decimal
    total_franking_credits_year: Decimal
    fdt_offset_reduction_applies: bool
    allowable_tax_offset: Decimal
    statutory_basis: str


@dataclass
class FrankingAccount:
    financial_year: int
    opening_balance: Decimal = Decimal("0.00")
    entries: List[FrankingEntry] = field(default_factory=list)

    def __post_init__(self) -> None:
        # A deficit opening balance is legitimate; a NaN or infinite one is not,
        # and would otherwise surface as InvalidOperation when the balance quantizes.
        if not self.opening_balance.is_finite():
            raise ValueError(f"opening_balance must be a finite amount, got {self.opening_balance}")

    @staticmethod
    def _validated(amount: Decimal, what: str) -> Decimal:
        if not amount.is_finite() or amount <= Decimal("0.00"):
            raise ValueError(f"{what} must be a positive finite amount, got {amount}")
        return amount.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)

    def record_payg_instalment(self, entry_date: date, amount: Decimal, description: str = "PAYG instalment paid") -> FrankingEntry:
        entry = FrankingEntry(
            entry_date=entry_date,
            entry_type=FrankingEntryType.PAYG_INSTALMENT,
            amount=self._validated(amount, "amount"),
            description=description,
            statutory_reference="s 205-15 Item 1 ITAA 1997",
        )
        self.entries.append(entry)
        return entry

    def record_tax_assessment_paid(self, entry_date: date, amount: Decimal, description: str = "Company tax assessment paid") -> FrankingEntry:
        entry = FrankingEntry(
            entry_date=entry_date,
            entry_type=FrankingEntryType.COMPANY_TAX_PAYMENT,
            amount=self._validated(amount, "amount"),
            description=description,
            statutory_reference="s 205-15 Item 2 ITAA 1997",
        )
        self.entries.append(entry)
        return entry

    def record_franked_distribution_received(self, entry_date: date, franking_credit: Decimal, description: str = "Franked dividend received") -> FrankingEntry:
        entry = FrankingEntry(
            entry_date=entry_date,
            entry_type=FrankingEntryType.FRANKED_DISTRIBUTION_REC,
            amount=self._validated(franking_credit, "franking_credit"),
            description=description,
            statutory_reference="s 205-15 Item 3 ITAA 1997",
        )
        self.entries.append(entry)
        return entry

    def record_franked_distribution_paid(self, entry_date: date, franking_credit_attached: Decimal, description: str = "Franked dividend paid") -> FrankingEntry:
        entry = FrankingEntry(
            entry_date=entry_date,
            entry_type=FrankingEntryType.FRANKED_DISTRIBUTION_PAID,
            amount=self._validated(franking_credit_attached, "franking_credit_attached"),
            description=description,
            statutory_reference="s 205-30 Item 1 ITAA 1997",
        )
        self.entries.append(entry)
        return entry

    def record_tax_refund(self, entry_date: date, refund_amount: Decimal, description: str = "Income tax refund received") -> FrankingEntry:
        entry = FrankingEntry(
            entry_date=entry_date,
            entry_type=FrankingEntryType.TAX_REFUND,
            amount=self._validated(refund_amount, "refund_amount"),
            description=description,
            statutory_reference="s 205-30 Item 2 ITAA 1997",
        )
        self.entries.append(entry)
        return entry

    def record_under_franking_debit(self, entry_date: date, shortfall_amount: Decimal, description: str = "Under-franking debit (benchmark rule breach)") -> FrankingEntry:
        entry = FrankingEntry(
            entry_date=entry_date,
            entry_type=FrankingEntryType.UNDER_FRANKING_DEBIT,
            amount=self._validated(shortfall_amount, "shortfall_amount"),
            description=description,
            statutory_reference="s 205-30 Item 3 / s 203-50(2) ITAA 1997",
        )
        self.entries.append(entry)
        return entry

    def record_fdt_liability(self, entry_date: date, fdt_amount: Decimal, description: str = "Franking deficit tax liability incurred") -> FrankingEntry:
        entry = FrankingEntry(
            entry_date=entry_date,
            entry_type=FrankingEntryType.FDT_LIABILITY,
            amount=self._validated(fdt_amount, "fdt_amount"),
            description=description,
            statutory_reference="s 205-15 ITAA 1997 (liability to franking deficit tax)",
        )
        self.entries.append(entry)
        return entry

    @property
    def total_credits(self) -> Decimal:
        return sum((e.amount for e in self.entries if e.is_credit), Decimal("0.00"))

    @property
    def total_debits(self) -> Decimal:
        return sum((e.amount for e in self.entries if e.is_debit), Decimal("0.00"))

    @property
    def closing_balance(self) -> Decimal:
        return (self.opening_balance + self.total_credits - self.total_debits).quantize(
            Decimal("0.01"), rounding=ROUND_HALF_UP
        )

    def evaluate_franking_deficit(self) -> FrankingDeficitResult:
        """
        Evaluate Franking Deficit Tax (FDT) under s 205-45 and tax offset under s 205-70.
        If the franking deficit exceeds 10% of total franking credits generated in the year,
        the tax offset is reduced by 30% (s 205-70(6)).
        """
        balance = self.closing_balance
        if balance >= Decimal("0.00"):
            return FrankingDeficitResult(
                closing_balance=balance,
                has_deficit=False,
                franking_deficit_tax=Decimal("0.00"),
                total_franking_credits_year=self.total_credits,
                fdt_offset_reduction_applies=False,
                allowable_tax_offset=Decimal("0.00"),
                statutory_basis="s 205-45 ITAA 1997: Surplus franking account balance; no FDT liability.",
            )

        fdt = abs(balance)
        credits_year = self.total_credits
        threshold = credits_year * Decimal("0.10")

        # Check 10% threshold rule (s 205-70(6))
        reduction_applies = fdt > threshold

        if reduction_applies:
            # 30% reduction penalty
            offset = (fdt * Decimal("0.70")).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
            basis = (
                f"s 205-45 ITAA 1997: FDT liability ${fdt:,.2f}. Deficit exceeds 10% of total credits "
                f"(${credits_year:,.2f}); tax offset is reduced by 30% to ${offset:,.2f} under s 205-70(6)."
            )
        else:
            offset = fdt
            basis = f"s 205-45 ITAA 1997: FDT liability ${fdt:,.2f}. 100% allowable as tax offset under s 205-70."

        return FrankingDeficitResult(
            closing_balance=balance,
            has_deficit=True,
            franking_deficit_tax=fdt,
            total_franking_credits_year=credits_year,
            fdt_offset_reduction_applies=reduction_applies,
            allowable_tax_offset=offset,
            statutory_basis=basis,
        )
