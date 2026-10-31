"""README: the MCP server 'contact[s] no service'. This watches real tool calls.

The real server runs over stdio in its own process. A CPython audit hook, installed
before the server is imported, ends that process on any name lookup, socket connect,
bind, sendto or sendmsg, or any child process, and each delegated engine then does a
calculation.

The hook sees those audited CPython events on every platform. CPython raises no event
for socket.send, so a socket inherited already connected would go unseen, as would a
native extension's own socket. It proves these calls only; the corpus, library and
rulings tools are not called. The one exemption is a loopback bind or connect made
directly by CPython's own socketpair: asyncio's event loop wakes itself with one, which
on Windows is a loopback connection within the process.
"""

import asyncio
import subprocess
import sys
from pathlib import Path

import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

GUARD = """
import os, sys

WATCHED = {
    "socket.bind", "socket.connect", "socket.getaddrinfo", "socket.gethostbyaddr",
    "socket.gethostbyname", "socket.getnameinfo", "socket.sendmsg", "socket.sendto",
    "subprocess.Popen", "os.exec", "os.fork", "os.forkpty", "os.posix_spawn", "os.spawn",
    "os.startfile", "os.system",
}

def socketpair_loopback(event, args):
    if event not in {"socket.bind", "socket.connect"}:
        return False
    if not (isinstance(args[1], tuple) and args[1][0] in {"127.0.0.1", "::1"}):
        return False
    caller = sys._getframe(2)
    return caller.f_globals.get("__name__") == "socket" and "socketpair" in caller.f_code.co_name

def guard(event, args):
    if event in WATCHED and not socketpair_loopback(event, args):
        os.write(2, f"NETWORK ATTEMPT: {event} {args}".encode())
        os._exit(99)

sys.addaudithook(guard)
"""

SERVER = GUARD + "from aus_accounting_mcp.cli import main; main([])\n"

CALCULATIONS = {
    "review_div7a_loan": {
        "year_of_income": "2026-27",
        "year_loan_made": "2025-26",
        "written_agreement": True,
        "terms_in_place_before_lodgment_day": True,
        "maximum_term_years": "7",
        "secured_by_registered_mortgage_over_real_property": False,
        "interest_rate_for_years_after_year_loan_made": "0.0837",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": "1",
        "payments_applied_during_the_year": "108770.00",
        "loan_id": "synthetic-loan-1",
    },
    "get_lodgment_due_dates": {
        "obligation": "bas",
        "entity_type": "company",
        "financial_year": "2026-27",
        "channel": "bas_agent",
        "reporting_cycle": "quarterly",
        "facts": {
            "consolidated_head_company": False,
            "lodges_electronically": True,
            "previous_statement_lodged_electronically": True,
            "includes_monthly_payg_obligation": False,
        },
    },
    "calculate_tax_worksheet": {
        "facts": {
            "kind": "gst",
            "amount": "1100",
            "gst_inclusive": True,
            "scope_confirmed": True,
            "year": "2025-26",
        },
    },
    "calc_payday_super_deadline": {
        "qe_day": "2026-08-06",
        "sg_amount": "800.00",
        "remitted": "2026-08-07",
        "as_at": "2026-08-21",
    },
    "get_ato_benchmarks": {
        "industry": "Bakeries and hot bread shops",
        "turnover": "850000.00",
        "other_income": "0",
        "cost_of_sales": "270000.00",
    },
    "review_gst_coding": {
        "lines": [{
            "id": "1",
            "entity": "Synthetic Alder Pty Ltd",
            "date": "2026-08-03",
            "amount": "-110.00",
            "account": "Repairs",
            "gst_code": "GST",
            "confirmed": True,
            "supplier_registered": False,
        }],
        "registrations": [
            {"entity": "Synthetic Alder Pty Ltd", "registered_from": "2024-07-01"},
        ],
    },
}


async def _call_tools(server: str, calls: dict, stderr_path: Path) -> dict[str, dict]:
    parameters = StdioServerParameters(command=sys.executable, args=["-c", server])
    results = {}
    with stderr_path.open("w", encoding="utf-8") as stderr:
        async with stdio_client(parameters, errlog=stderr) as (read_stream, write_stream):
            async with ClientSession(read_stream, write_stream) as session:
                await session.initialize()
                for name, arguments in calls.items():
                    result = await session.call_tool(name, arguments)
                    assert not result.is_error, f"{name}: {result.content}"
                    results[name] = result.structured_content
    return results


def call_tools(server: str, calls: dict, stderr_path: Path) -> dict[str, dict]:
    # mcp bounds only stdio_client shutdown, so an unset request timeout can wait forever.
    return asyncio.run(asyncio.wait_for(_call_tools(server, calls, stderr_path), timeout=60))


def test_real_tool_calls_make_no_network_call(tmp_path):
    stderr = tmp_path / "stderr.txt"
    results = call_tools(SERVER, CALCULATIONS, stderr)

    assert "NETWORK ATTEMPT" not in stderr.read_text(encoding="utf-8")
    assert results["review_div7a_loan"]["minimum_yearly_repayment"]["verdict"] == "MYR_MET"
    assert results["get_lodgment_due_dates"]["obligations"][1]["due_date"] == "2027-03-02"
    assert results["calculate_tax_worksheet"]["amounts"]["gst"] == "100.00"
    assert results["calc_payday_super_deadline"]["result"]["verdict"] == "AT_RISK"
    assert results["get_ato_benchmarks"]["ratios"]
    assert results["review_gst_coding"]["lines"][0]["state"] == "gst_differs_from_registration"


@pytest.mark.parametrize(
    "body",
    [
        "s = __import__('_socket').socket(); s.connect(('192.0.2.1', 9))",
        "__import__('os').system('exit 0')",
        "helper()",
    ],
    ids=["external connect", "child process", "call from a nested frame"],
)
def test_the_socketpair_exemption_covers_only_its_own_loopback_calls(body):
    module = (
        "def helper():\n"
        "    s = __import__('_socket').socket(); s.connect(('127.0.0.1', 9))\n"
        f"def socketpair():\n    {body}\n"
        "socketpair()\n"
    )
    # The module name makes the frame look like CPython's own.
    source = GUARD + f"exec({module!r}, {{'__name__': 'socket'}})\n"
    completed = subprocess.run([sys.executable, "-c", source], capture_output=True, timeout=30)
    assert completed.returncode == 99, completed.stderr


def test_the_guard_stops_a_connection_made_during_a_tool_call(tmp_path):
    stderr = tmp_path / "stderr.txt"
    server = GUARD + (
        "import socket\n"
        "from aus_accounting_mcp.server import mcp, run_stdio\n"
        "@mcp.tool()\n"
        "def connects() -> None:\n"
        "    socket.create_connection(('127.0.0.1', 9), timeout=1)\n"
        "run_stdio()\n"
    )
    # How the client reports a dead server varies; the stderr line is the evidence.
    with pytest.raises(Exception):
        call_tools(server, {"connects": {}}, stderr)
    assert "NETWORK ATTEMPT: socket." in stderr.read_text(encoding="utf-8")
