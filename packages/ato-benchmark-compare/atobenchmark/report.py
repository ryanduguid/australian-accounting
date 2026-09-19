"""Compare computed ratios against a benchmark and render the result."""

from __future__ import annotations

from collections.abc import Collection
from dataclasses import dataclass, field
from decimal import Decimal

from .dataset import RATIO_LABELS, Band, BusinessType, Dataset, Range
from .evidence import EvidenceMessage
from .mapping import BUCKETS, EXPENSE_BUCKETS
from .money import money, percent, percent_range
from .ratios import Figures

WITHIN = "within"
BELOW = "below"
ABOVE = "above"
NO_BENCHMARK = "no benchmark in this dataset"
NO_BAND = "no turnover band applies"

DISCLAIMER = (
    "The ATO publishes these benchmarks as ranges, and sitting outside a range is not "
    "of itself a finding that anything is wrong. This tool reports a comparison. It is "
    "not tax advice and it does not decide how any amount should be reported."
)

RATIO_ORDER = (
    "cost_of_sales_to_turnover",
    "total_expenses_to_turnover",
    "labour_to_turnover",
    "rent_to_turnover",
    "motor_vehicle_to_turnover",
)

CALCULATION_FIELDS = tuple(name for name in BUCKETS if name != "excluded")

NOT_SUPPLIED = "not_supplied"


@dataclass(frozen=True)
class _Presence:
    """Which figures a comparison can actually state, given the supplied set.

    Shared by `to_evidenced_dict` and `render_text` so the exported library
    payload and the command-line output cannot drift on which figures count as
    evidenced. `evidenced` is keyed by ratio name over `RATIO_ORDER`.
    """

    income_evidenced: bool
    expense_complete: bool
    labour_evidenced: bool
    # The key ratio reverts to the published one where cost of sales was not
    # supplied: the ATO's fallback to total expenses triggers on a nil, and an
    # omitted bucket reaches the comparison as exactly that nil.
    key_ratio: str
    evidenced: dict[str, bool]


def _ratio_own_buckets(name: str) -> frozenset[str]:
    """The buckets a ratio needs, EXCEPT labour, whose evidence rule also
    depends on w1 (see `_presence`)."""
    return frozenset(
        {
            "total_expenses_to_turnover": set(EXPENSE_BUCKETS),
            "cost_of_sales_to_turnover": {"cost_of_sales"},
            "rent_to_turnover": {"rent"},
            "motor_vehicle_to_turnover": {"motor_vehicle"},
        }.get(name, set())
    )


def _presence(comparison: Comparison, supplied_fields: Collection[str]) -> _Presence:
    known = frozenset(supplied_fields)
    unknown = known - (set(BUCKETS) | {"w1"})
    if unknown:
        raise ValueError(f"unknown supplied field(s): {', '.join(sorted(unknown))}")
    supplied = known - {"w1"}
    w1_supplied = "w1" in known
    income_evidenced = {"turnover", "other_income"} <= supplied
    expense_complete = EXPENSE_BUCKETS <= supplied
    labour_evidenced = (
        {"salary_wages", "contractor_commission", "cost_of_sales_labour"} <= supplied
        and (not w1_supplied or "associated_persons" in supplied)
    )
    key_ratio = (
        comparison.key_ratio
        if "cost_of_sales" in supplied
        else comparison.business_type.key_ratio
    )
    evidenced = {}
    for name in RATIO_ORDER:
        own = labour_evidenced if name == "labour_to_turnover" else (
            _ratio_own_buckets(name) <= supplied
        )
        evidenced[name] = own and income_evidenced
    return _Presence(
        income_evidenced=income_evidenced,
        expense_complete=expense_complete,
        labour_evidenced=labour_evidenced,
        key_ratio=key_ratio,
        evidenced=evidenced,
    )


def omitted_buckets_note(omitted: list[str]) -> str:
    return (
        "These buckets were omitted, not evidenced as zero, so their ratios "
        f"are not_supplied: {', '.join(omitted)}."
    )


def evidenced_ratio(
    comparison: Comparison, supplied_fields: Collection[str], ratio: str
) -> bool:
    """Whether one ratio is evidenced under the same gate as the outputs.

    A caller deciding whether a comparison supports a claim (an exit code, a
    flag) must apply the same presence gate the serialisers apply, or the
    claim can rest on a figure nobody supplied.
    """
    return _presence(comparison, supplied_fields).evidenced.get(ratio, False)


OTHER_INCOME_NOTE = (
    "other_business_income was omitted. The ATO rule divides by sales, or "
    "by total business income once other income exceeds sales, so every "
    "ratio is not_supplied until that figure is established. The turnover "
    "figure, the basis that selected it, the turnover band, the published "
    "ranges and any note quoting that turnover are withheld for the same "
    "reason. Pass 0 only when the operator established the business had "
    "no other income."
)


@dataclass(frozen=True)
class Verdict:
    key: str
    label: str
    ratio: Decimal
    benchmark: Range | None
    status: str
    is_key: bool


@dataclass
class Comparison:
    dataset: Dataset
    business_type: BusinessType
    figures: Figures
    band: Band | None
    verdicts: tuple[Verdict, ...]
    key_ratio: str
    notes: list[str] = field(default_factory=list)
    note_details: list[EvidenceMessage] = field(default_factory=list)

    @property
    def key_verdict(self) -> Verdict | None:
        for verdict in self.verdicts:
            if verdict.is_key:
                return verdict
        return None

    @property
    def outside_key_range(self) -> bool:
        verdict = self.key_verdict
        return verdict is not None and verdict.status in {BELOW, ABOVE}


def compare(dataset: Dataset, business_type: BusinessType, figures: Figures) -> Comparison:
    notes: list[str] = []
    note_details: list[EvidenceMessage] = []

    def add_note(code: str, text: str, required_fields: frozenset[str]) -> None:
        notes.append(text)
        note_details.append(EvidenceMessage(code, text, required_fields))

    band = business_type.band_for(figures.turnover)
    if band is None:
        lowest = business_type.bands[0]
        if figures.turnover < lowest.turnover_from:
            add_note(
                "turnover_below_range",
                f"Turnover of ${money(figures.turnover)} is below the lowest published range "
                f"for this industry ({lowest.label}), so the ATO benchmarks do not apply.",
                frozenset({"turnover", "other_income"}),
            )
        else:
            add_note(
                "turnover_outside_ranges",
                f"Turnover of ${money(figures.turnover)} does not fall in any published range "
                f"for this industry.",
                frozenset({"turnover", "other_income"}),
            )

    key_ratio = business_type.key_ratio
    if key_ratio == "cost_of_sales_to_turnover" and figures.cost_of_sales_for_ratio == 0:
        key_ratio = "total_expenses_to_turnover"
        add_note(
            "cost_of_sales_key_fallback",
            "Cost of sales to turnover is the ATO key range for this industry, but no cost "
            "of sales was mapped. The ATO says to use total expenses to turnover instead "
            "where cost of sales is not reported, so that is the key range used here.",
            frozenset({"cost_of_sales"}),
        )
    elif key_ratio == "cost_of_sales_to_turnover":
        add_note(
            "cost_of_sales_small_judgement",
            "The ATO says to use total expenses to turnover as the key range instead where "
            "cost of sales is only a small amount. It does not put a figure on small, so "
            "both ranges are reported and that judgement is yours.",
            frozenset({"cost_of_sales"}),
        )

    verdicts = []
    for name in RATIO_ORDER:
        ratio = figures.ratios.get(name)
        if ratio is None:
            continue
        benchmark = band.ratios.get(name) if band else None
        if band is None:
            status = NO_BAND
        elif benchmark is None:
            status = NO_BENCHMARK
        elif benchmark.contains(ratio):
            status = WITHIN
        elif ratio < benchmark.minimum:
            status = BELOW
        else:
            status = ABOVE
        verdicts.append(
            Verdict(
                key=name,
                label=RATIO_LABELS.get(name, name),
                ratio=ratio,
                benchmark=benchmark,
                status=status,
                is_key=(name == key_ratio),
            )
        )

    return Comparison(
        dataset=dataset,
        business_type=business_type,
        figures=figures,
        band=band,
        verdicts=tuple(verdicts),
        key_ratio=key_ratio,
        notes=notes,
        note_details=note_details,
    )


def render_text(
    comparison: Comparison,
    unreviewed: int = 0,
    supplied: Collection[str] | None = None,
) -> str:
    """Render the comparison as text.

    `supplied` carries the set of fields the operator evidenced (bucket names,
    plus "w1" where payments to associates were supplied). Given it, a ratio
    or figure nobody supplied reads "not supplied" instead of a computed nil,
    the same gating `to_evidenced_dict` applies, so the command-line output
    and the library payload cannot drift. Left as None, every figure prints,
    which keeps direct library callers rendering as before.
    """
    figures = comparison.figures
    source = comparison.dataset.source
    presence = None if supplied is None else _presence(comparison, supplied)
    supplied_names = None if supplied is None else frozenset(supplied) - {"w1"}
    lines: list[str] = []
    add = lines.append

    add("ATO small business benchmark comparison")
    add("=" * 39)
    add(f"Business type:  {comparison.business_type.name}")
    add(f"Benchmark year: {comparison.dataset.year}")
    if presence is None or presence.income_evidenced:
        add(f"Turnover:       ${money(figures.turnover)} ({figures.turnover_basis})")
        add(f"Turnover band:  {comparison.band.label if comparison.band else 'none applies'}")
    else:
        add("Turnover:       not supplied (other business income was not evidenced)")
        add("Turnover band:  withheld until turnover is established")
    add("")

    width = max(len(v.label) for v in comparison.verdicts) + 6
    add(f"{'Ratio'.ljust(width)}{'This business'.ljust(15)}{'ATO range'.ljust(18)}Result")
    add("-" * (width + 15 + 18 + 8))
    for verdict in comparison.verdicts:
        evidenced = presence is None or presence.evidenced[verdict.key]
        is_key = presence.key_ratio == verdict.key if presence is not None else verdict.is_key
        label = verdict.label + (" (key)" if is_key else "")
        benchmark = (
            percent_range(verdict.benchmark.minimum, verdict.benchmark.maximum)
            if verdict.benchmark
            else "-"
        )
        this_business = percent(verdict.ratio) if evidenced else "not supplied"
        status = verdict.status if evidenced else NOT_SUPPLIED
        add(f"{label.ljust(width)}{this_business.ljust(15)}{benchmark.ljust(18)}{status}")
    add("")

    def figure(value: Decimal, evidenced: bool) -> str:
        return f"${money(value)}" if evidenced else "not supplied"

    add("Figures used")
    add(f"  Sales of goods and services   {figure(figures.trading_sales, presence is None or ('turnover' in (supplied_names or set())))}")
    add(f"  Other business income         {figure(figures.other_income, presence is None or presence.income_evidenced)}")
    add(f"  Total business income         {figure(figures.total_business_income, presence is None or presence.income_evidenced)}")
    add(f"  Total expenses                {figure(figures.total_expenses_reported, presence is None or presence.expense_complete)}")
    add(f"  Less payments to associates   {figure(figures.totals['associated_persons'], presence is None or ('associated_persons' in (supplied_names or set())))}")
    add(f"  Total expenses for the ratio  {figure(figures.total_expenses_for_ratio, presence is None or presence.expense_complete)}")
    add(f"  Cost of sales excluding wages {figure(figures.cost_of_sales_for_ratio, presence is None or ('cost_of_sales' in (supplied_names or set())))}")
    add(f"  Labour                        {figure(figures.labour, presence is None or presence.labour_evidenced)}")
    add("")

    if unreviewed:
        add("Review outstanding")
        add(
            f"  {unreviewed} account(s) still carry the bucket this tool suggested. "
            f"Suggestions are made from account names alone and are not a substitute "
            f"for reading the ledger."
        )
        add("")

    # The same presence qualifications the evidenced payload carries, so the
    # text output states why a figure reads "not supplied".
    rendered_notes = list(comparison.notes)
    if presence is not None:
        omitted = [name for name in CALCULATION_FIELDS if name not in (supplied_names or set())]
        if omitted:
            rendered_notes.append(omitted_buckets_note(omitted))
        if "other_income" not in (supplied_names or set()):
            rendered_notes.append(OTHER_INCOME_NOTE)

    if rendered_notes:
        add("Notes")
        for note in rendered_notes:
            add(f"  - {note}")
        add("")

    if figures.warnings:
        add("Checks to make")
        for warning in figures.warnings:
            add(f"  - {warning}")
        add("")

    add("Source")
    add(f"  {source.get('publisher')}, {source.get('dataset')}, {source.get('resource_name')}")
    add(f"  {source.get('resource_url')}")
    add(f"  Retrieved {source.get('retrieved')}, sha256 {source.get('sha256')}")
    add(f"  Licensed {source.get('licence')} ({source.get('licence_url')})")
    add("")
    add(DISCLAIMER)
    return "\n".join(lines)


def to_dict(comparison: Comparison, unreviewed: int = 0) -> dict:
    figures = comparison.figures
    return {
        "benchmark_year": comparison.dataset.year,
        "business_type": comparison.business_type.name,
        "key_ratio": comparison.key_ratio,
        "turnover": str(figures.turnover),
        "turnover_basis": figures.turnover_basis,
        "turnover_band": None
        if comparison.band is None
        else {"band": comparison.band.band, "label": comparison.band.label},
        "figures": {
            "sales_of_goods_and_services": str(figures.trading_sales),
            "other_business_income": str(figures.other_income),
            "total_business_income": str(figures.total_business_income),
            "total_expenses": str(figures.total_expenses_reported),
            "payments_to_associated_persons": str(figures.totals["associated_persons"]),
            "total_expenses_for_ratio": str(figures.total_expenses_for_ratio),
            "cost_of_sales_for_ratio": str(figures.cost_of_sales_for_ratio),
            "labour": str(figures.labour),
        },
        "bucket_totals": {name: str(value) for name, value in sorted(figures.totals.items())},
        "ratios": [
            {
                "ratio": verdict.key,
                "label": verdict.label,
                "value": str(verdict.ratio),
                "percent": percent(verdict.ratio),
                "benchmark_min": None if verdict.benchmark is None else str(verdict.benchmark.minimum),
                "benchmark_max": None if verdict.benchmark is None else str(verdict.benchmark.maximum),
                "status": verdict.status,
                "is_key_ratio": verdict.is_key,
            }
            for verdict in comparison.verdicts
        ],
        "unreviewed_accounts": unreviewed,
        "notes": list(comparison.notes),
        "checks_to_make": list(figures.warnings),
        "source": dict(comparison.dataset.source),
        "disclaimer": DISCLAIMER,
    }


def to_evidenced_dict(
    comparison: Comparison,
    supplied_fields: Collection[str],
    *,
    unreviewed: int | None = None,
) -> dict:
    """Serialise a comparison without presenting unsupplied amounts as zero."""
    presence = _presence(comparison, supplied_fields)
    payload = to_dict(comparison, unreviewed=unreviewed or 0)
    payload["unreviewed_accounts"] = unreviewed
    figures = comparison.figures
    supplied = frozenset(supplied_fields) - {"w1"}
    key_ratio = presence.key_ratio

    # The ATO's fallback to total expenses is triggered by a nil cost of sales,
    # and an omitted bucket reaches the comparison as exactly that nil. Where the
    # bucket was not supplied the fallback rests on a figure nobody established,
    # so the key ratio reverts to the one the ATO publishes for this industry.
    #
    # is_key_ratio marks that same choice per row, so it reverts with it. A
    # payload naming one ratio at key_ratio while flagging a different row as the
    # key one contradicts itself, and a caller reading either field alone is
    # given a different answer depending on which it happened to read.
    ratios = []
    for row in payload["ratios"]:
        is_key_ratio = row["ratio"] == key_ratio
        if presence.evidenced[row["ratio"]]:
            ratios.append({**row, "is_key_ratio": is_key_ratio})
            continue
        ratios.append(
            {
                "ratio": row["ratio"],
                "label": row["label"],
                "value": None,
                "percent": None,
                "benchmark_min": row["benchmark_min"] if presence.income_evidenced else None,
                "benchmark_max": row["benchmark_max"] if presence.income_evidenced else None,
                "status": NOT_SUPPLIED,
                "is_key_ratio": is_key_ratio,
            }
        )

    for name in CALCULATION_FIELDS:
        if name not in supplied:
            payload["bucket_totals"][name] = None
    if "excluded" not in supplied:
        payload["bucket_totals"]["excluded"] = None
    if "turnover" not in supplied:
        payload["figures"]["sales_of_goods_and_services"] = None
    if not presence.income_evidenced:
        payload["figures"]["other_business_income"] = None
        payload["figures"]["total_business_income"] = None
        payload["turnover"] = None
        payload["turnover_basis"] = None
        payload["turnover_band"] = None
    if not presence.expense_complete:
        payload["figures"]["total_expenses"] = None
        payload["figures"]["total_expenses_for_ratio"] = None
    if "associated_persons" not in supplied:
        payload["figures"]["payments_to_associated_persons"] = None
    if "cost_of_sales" not in supplied:
        payload["figures"]["cost_of_sales_for_ratio"] = None
    payload["key_ratio"] = key_ratio
    if not presence.labour_evidenced:
        payload["figures"]["labour"] = None

    notes = [
        detail.text
        for detail in comparison.note_details
        if detail.required_fields <= frozenset(supplied_fields)
    ]
    checks = [
        detail.text
        for detail in figures.warning_details
        if detail.required_fields <= frozenset(supplied_fields)
    ]
    withheld_checks = len(figures.warning_details) - len(checks)
    omitted = [name for name in CALCULATION_FIELDS if name not in supplied]
    if omitted:
        notes.append(omitted_buckets_note(omitted))
    if "w1" not in supplied_fields:
        omitted.append("w1")
    if "other_income" not in supplied:
        notes.append(OTHER_INCOME_NOTE)
    if withheld_checks:
        notes.append(
            f"{withheld_checks} check(s) to make were withheld because one or more "
            "figures needed to state them were omitted rather than evidenced as zero."
        )

    payload.update(
        {
            "ratios": ratios,
            "notes": notes,
            "checks_to_make": checks,
            "supplied_buckets": sorted(supplied),
            "omitted_buckets": omitted,
            "complete_buckets": presence.expense_complete,
        }
    )
    return payload
