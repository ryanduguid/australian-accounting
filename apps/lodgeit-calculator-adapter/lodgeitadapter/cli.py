"""The command line. Nothing here reaches the network without --enable-network.

    lodgeit-adapter contract                 read the reviewed snapshot (offline)
    lodgeit-adapter discover --enable-network --base-url URL
    lodgeit-adapter drift    --enable-network --base-url URL
    lodgeit-adapter invoke   --enable-network --base-url URL --calculator U --period U --body FILE
    lodgeit-adapter verify   --evidence FILE  check an evidence file (offline)

Exit codes: 0 the command did what it was asked, 1 a refusal or a failure,
2 a usage error. A refusal is not a crash and not a result.
"""

from __future__ import annotations

import argparse
import json
import sys
from decimal import Decimal
from pathlib import Path

from . import evidence as evidence_module
from .client import LodgeitClient, Status
from .config import AdapterConfig, from_environment
from .contract import load as load_contract
from .errors import AdapterError


def _decimalise(value):
    """Rebuild Decimals from a JSON body so money goes out exactly."""
    if isinstance(value, dict):
        return {key: _decimalise(item) for key, item in value.items()}
    if isinstance(value, list):
        return [_decimalise(item) for item in value]
    if isinstance(value, str):
        try:
            return Decimal(value) if value.strip() and _looks_numeric(value) else value
        except Exception:  # noqa: BLE001 - a non-numeric string stays a string
            return value
    return value


def _looks_numeric(text: str) -> bool:
    body = text.strip()
    if body.startswith("-"):
        body = body[1:]
    return body.replace(".", "", 1).isdigit() and body.count(".") <= 1


def build_parser() -> argparse.ArgumentParser:
    # The flags below are attached to the top-level parser and to every
    # subcommand, so `... drift --enable-network` and
    # `... --enable-network drift` both work. argparse otherwise accepts them
    # only before the subcommand, which is not where anyone types them.
    shared = argparse.ArgumentParser(add_help=False)
    shared.add_argument("--contract", default="lodgeit-calculators", help="reviewed snapshot name")
    shared.add_argument("--enable-network", action="store_true",
                        help="permit this one run to contact the configured service")
    shared.add_argument("--base-url", help="the service base URL; must be on the allowlist")
    shared.add_argument("--allow-loopback", action="store_true",
                        help="permit a loopback base URL, for a local stub in tests")
    shared.add_argument("--evidence-out", type=Path, help="write an evidence file for the call")
    shared.add_argument("--not-synthetic", action="store_true",
                        help="record that the input was not fabricated; the flag travels into the "
                             "evidence")

    parser = argparse.ArgumentParser(
        prog="lodgeit-adapter", description=__doc__.split("\n")[0], parents=[shared],
    )
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("contract", parents=[shared], help="print the reviewed snapshot, offline")
    commands.add_parser("discover", parents=[shared], help="fetch the live calculator listing")
    commands.add_parser("drift", parents=[shared],
                        help="compare the live listing with the reviewed snapshot")
    invoke = commands.add_parser("invoke", parents=[shared], help="call one calculator once")
    invoke.add_argument("--calculator", required=True)
    invoke.add_argument("--period", required=True)
    invoke.add_argument("--body", required=True, type=Path, help="JSON request body")
    invoke.add_argument("--label", default="manual invocation")
    verify = commands.add_parser("verify", parents=[shared],
                                 help="check an evidence file, offline")
    verify.add_argument("--evidence", required=True, type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        contract = load_contract(args.contract)
    except AdapterError as exc:
        print(f"lodgeit-adapter: {exc}", file=sys.stderr)
        return 1

    if args.command == "contract":
        print(json.dumps({
            "snapshot_id": contract.snapshot_id,
            "read_at": contract.read_at,
            "read_by": contract.read_by,
            "sha256": contract.sha256,
            "calculators": sorted(contract.calculators),
            "notes": list(contract.notes),
        }, indent=2))
        return 0

    if args.command == "verify":
        try:
            record = evidence_module.read(args.evidence)
        except (OSError, json.JSONDecodeError) as exc:
            print(f"lodgeit-adapter: {args.evidence} could not be read: {exc}", file=sys.stderr)
            return 1
        findings = evidence_module.verify(record)
        for finding in findings:
            print(f"evidence: {finding}")
        if findings:
            return 1
        print(f"evidence: {args.evidence} verifies against its own digest")
        return 0

    base = from_environment()
    config = AdapterConfig(
        enabled=args.enable_network or base.enabled,
        base_url=args.base_url or base.base_url,
        allowed_hosts=base.allowed_hosts,
        allow_loopback=args.allow_loopback or base.allow_loopback,
        connect_timeout=base.connect_timeout,
        read_timeout=base.read_timeout,
        max_response_bytes=base.max_response_bytes,
        max_attempts=base.max_attempts,
    )
    client = LodgeitClient(config, contract)

    if args.command == "discover":
        outcome = client.discover()
    elif args.command == "drift":
        outcome, findings = client.drift()
        if outcome.computed:
            for finding in findings:
                print(f"drift: {finding}")
            if not findings:
                print(f"drift: live discovery agrees with snapshot {contract.snapshot_id}")
            print("drift: a difference is for a person to review. Nothing here updates a snapshot.")
            return 1 if findings else 0
    else:
        try:
            body = _decimalise(json.loads(args.body.read_text(encoding="utf-8")))
        except (OSError, json.JSONDecodeError) as exc:
            print(f"lodgeit-adapter: {args.body} could not be read: {exc}", file=sys.stderr)
            return 2
        outcome = client.invoke(args.calculator, args.period, body)

    print(json.dumps({
        "status": str(outcome.status),
        "http_status": outcome.http_status,
        "findings": list(outcome.findings),
        "result": outcome.result if outcome.computed else None,
    }, indent=2, default=str))

    if args.evidence_out is not None:
        record = evidence_module.build(
            outcome,
            label=getattr(args, "label", args.command),
            synthetic=not args.not_synthetic,
        )
        evidence_module.write(record, args.evidence_out)
        print(f"evidence written to {args.evidence_out} "
              f"(calculation_sha256 {record['calculation_sha256']})")

    if outcome.status is Status.COMPUTED:
        return 0
    print(f"lodgeit-adapter: {outcome.status}. This is not a result and must not be read as one.",
          file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
