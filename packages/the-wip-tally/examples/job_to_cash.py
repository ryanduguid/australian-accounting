"""Trace one fabricated contract through the existing WIP engine to cash inputs."""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
from dataclasses import replace
from decimal import Decimal
from pathlib import Path

from wiptally.csv_io import load_mapping, read_contracts
from wiptally.model import Schedule
from wiptally.report import build_review_pack, write_review_pack, write_schedule_csv
from wiptally.schedule import measure

HERE = Path(__file__).resolve().parent


def run(output: Path, *, extra_cost_to_complete: Decimal = Decimal(0)) -> dict:
    if not extra_cost_to_complete.is_finite() or extra_cost_to_complete < 0:
        raise ValueError("Cost increase must be finite and non-negative")
    source = HERE / "job-to-cash.csv"
    source_bytes = source.read_bytes()
    contracts = read_contracts(source, load_mapping(None), source_bytes=source_bytes)
    if len(contracts) != 1:
        raise ValueError("The example requires one contract")
    contract = replace(contracts[0], estimated_cost_to_complete=contracts[0].estimated_cost_to_complete + extra_cost_to_complete)
    position = measure(contract)
    assumptions_path = HERE / "job-cash-assumptions.json"
    assumptions = json.loads(assumptions_path.read_text(encoding="utf-8"))
    receipts = Decimal(assumptions["receipts_to_date"])
    paid = Decimal(assumptions["costs_paid_to_date"])
    certified_gross = contract.certified_billings + position.gst_on_certified_billings
    retention_gross = contract.retention_withheld + position.gst_on_retention
    collectible = certified_gross - retention_gross - receipts
    if collectible < 0 or contract.committed_outstanding > contract.estimated_cost_to_complete:
        raise ValueError("Cash or commitment assumptions do not reconcile")
    # The committed amount is already inside remaining cost; never add it twice.
    future_cost = contract.estimated_cost_to_complete * Decimal(assumptions["future_cost_cash_multiplier"])
    output.mkdir(parents=True, exist_ok=False)
    # Persist the exact scenario source so the existing pack verifier can rebuild it.
    with source.open(encoding="utf-8", newline="") as stream:
        reader = csv.DictReader(stream)
        fields = reader.fieldnames
        rows = list(reader)
    rows[0]["estimated_cost_to_complete"] = str(contract.estimated_cost_to_complete)
    scenario = output / "contracts.csv"
    with scenario.open("x", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fields)
        writer.writeheader()
        writer.writerows(rows)
    schedule = Schedule(as_at="2026-09-30", positions=[position], source_name=scenario.name)
    schedule_path = output / "wip-schedule.csv"
    write_schedule_csv(schedule_path, schedule)
    write_review_pack(output / "wip-review.md", build_review_pack(schedule_path, scenario, schedule, source_bytes=scenario.read_bytes()))
    cash_rows = [
        {"id": "CERTIFIED-RECEIVABLE", "date": assumptions["certified_collection_date"], "receipt": str(collectible), "payment": "0", "evidence": assumptions["certified_collection_evidence"]},
        {"id": "RETENTION", "date": assumptions["retention_collection_date"], "receipt": str(retention_gross), "payment": "0", "evidence": assumptions["retention_release_assumption"]},
        {"id": "REMAINING-COST", "date": assumptions["remaining_cost_payment_date"], "receipt": "0", "payment": str(future_cost), "evidence": assumptions["remaining_cost_evidence"]},
    ]
    with (output / "cash-assumptions.csv").open("x", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, ["id", "date", "receipt", "payment", "evidence"])
        writer.writeheader()
        writer.writerows(cash_rows)
    result = {"contract_id": contract.contract_id, "currency": "AUD", "as_at": "2026-09-30",
              "cash_assumptions_sha256": hashlib.sha256((output / "cash-assumptions.csv").read_bytes()).hexdigest(),
              "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
              "scenario_source_sha256": hashlib.sha256(scenario.read_bytes()).hexdigest(),
              "gross_profit_at_completion": str(position.gross_profit_at_completion),
              "revenue_to_date": str(position.revenue_to_date), "certified_billings": str(contract.certified_billings),
              "contract_asset": str(position.contract_asset), "uncertified_claims": str(contract.uncertified_claims),
              "retention_gross": str(retention_gross), "receipts_to_date": str(receipts),
              "costs_paid_to_date": str(paid), "cash_to_date": str(receipts - paid),
              "collectible_certified_balance": str(collectible), "future_cost_cash": str(future_cost),
              "committed_in_remaining_cost": str(contract.committed_outstanding),
              "scope": "Fabricated bridge under supplied assumptions. Uncertified and future unbilled claims are excluded from cash. Retention date is a planning assumption."}
    (output / "job-to-cash.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--extra-cost-to-complete", type=Decimal, default=Decimal(0))
    args = parser.parse_args()
    print(json.dumps(run(args.output, extra_cost_to_complete=args.extra_cost_to_complete), indent=2))
