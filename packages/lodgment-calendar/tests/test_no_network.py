"""README: 'contacts no service'. This watches a real calculation.

A CPython audit hook, installed before the engine is imported, ends the process on
any name lookup, socket connect, bind or send, or any child process. It sees every
call through CPython's socket and process modules, on every platform. It cannot see
a native extension opening its own socket; this engine has no compiled dependency.
"""

import subprocess
import sys

GUARD = """
import os, sys

WATCHED = {
    "socket.bind", "socket.connect", "socket.getaddrinfo", "socket.gethostbyaddr",
    "socket.gethostbyname", "socket.getnameinfo", "socket.sendmsg", "socket.sendto",
    "subprocess.Popen", "os.exec", "os.fork", "os.forkpty", "os.posix_spawn", "os.spawn",
    "os.startfile", "os.system",
}

def guard(event, args):
    if event in WATCHED:
        os.write(2, f"NETWORK ATTEMPT: {event} {args}".encode())
        os._exit(99)

sys.addaudithook(guard)
"""

CALCULATION = """
from lodgmentcalendar import Facts, due_dates, to_ics

result = due_dates(
    "bas", "company", "2026-27", "bas_agent", reporting_cycle="quarterly",
    facts=Facts(consolidated_head_company=False, lodges_electronically=True,
                previous_statement_lodged_electronically=True,
                includes_monthly_payg_obligation=False))
print(result["obligations"][1]["due_date"])
print(to_ics(result)["ics"])
"""


def guarded(code: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, "-c", GUARD + code], capture_output=True, text=True,
                          timeout=60, check=False)


def test_a_real_calculation_makes_no_network_call():
    completed = guarded(CALCULATION)
    assert completed.returncode == 0, completed.stderr
    # Quarter 2 is rolled from Sunday 28 February over Labour Day (WA) to 2 March.
    assert completed.stdout.startswith("2027-03-02")
    assert "BEGIN:VEVENT" in completed.stdout


def test_the_guard_stops_an_attempted_connection():
    completed = guarded('import socket; socket.create_connection(("127.0.0.1", 9), timeout=1)')
    assert completed.returncode == 99, completed.stderr
    assert "NETWORK ATTEMPT: socket." in completed.stderr
