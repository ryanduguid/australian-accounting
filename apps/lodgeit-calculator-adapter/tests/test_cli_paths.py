"""Command-line and MCP paths the regression suites do not reach. All offline."""

from __future__ import annotations

import asyncio
import json
from decimal import Decimal

import pytest

from lodgeitadapter import cli, evidence
from lodgeitadapter.client import Outcome, Status

CALC = "urn:sbrm:calculator:div7a:at"
PERIOD = "urn:sbrm:period:div7a:fy2026"
GOOD = {
    "statutory_myr": "21874.92", "shortfall": "0.00", "total_repayments": "22000.00",
    "benchmark_rate": "0.0837",
    "manifest": {"calculator": CALC, "period": PERIOD,
                 "rate_table_uris": [], "engine": "div7a/1.0.0", "citation": "ITAA 1936 s 109E"},
    "advisory": {"figure_type": "div7a_myr", "notes": ["Not advice."]},
}


def _live_from(contract) -> list[dict]:
    return [
        {"calc_uri": uri, "supported_periods": list(spec.get("supported_periods", [])),
         "input_schema_ref": spec.get("input_schema_ref")}
        for uri, spec in contract.calculators.items()
    ]


def _network(base_url):
    return ["--enable-network", "--allow-loopback", "--base-url", base_url]


@pytest.fixture(autouse=True)
def _no_adapter_environment(monkeypatch):
    for name in ("LODGEIT_ADAPTER_ENABLED", "LODGEIT_ADAPTER_BASE_URL",
                 "LODGEIT_ADAPTER_ALLOWED_HOSTS", "LODGEIT_ADAPTER_ALLOW_LOOPBACK"):
        monkeypatch.delenv(name, raising=False)


# -- the request body -----------------------------------------------------

def test_a_body_that_is_not_an_object_is_refused():
    with pytest.raises(ValueError, match="^a request body must be a JSON object$"):
        cli._decimalise([1], ["amalgamated_base"])


def test_a_list_path_over_something_that_is_not_a_list_is_left_alone():
    body = {"amalgamated_base": "1.00", "repayments": "none"}
    assert cli._decimalise(body, ["repayments[].amount"]) == body  # nosec B101: pytest outcome assertion


def test_a_dotted_path_reaches_a_nested_field():
    converted = cli._decimalise({"loan": {"amount": "1.50"}}, ["loan.amount"])
    assert converted == {"loan": {"amount": Decimal("1.50")}}  # nosec B101: pytest outcome assertion


@pytest.mark.parametrize("value", [True, None])
def test_a_boolean_or_null_where_a_number_belongs_is_refused(value):
    with pytest.raises(ValueError, match=f"^amalgamated_base: {value} is not a number$"):
        cli._decimalise({"amalgamated_base": value}, ["amalgamated_base"])


# -- the offline commands -------------------------------------------------

def test_an_unknown_contract_name_is_a_refusal(capsys):
    assert cli.main(["--contract", "no-such-snapshot", "contract"]) == 1  # nosec B101: pytest outcome assertion
    assert "lodgeit-adapter: contract snapshot " in capsys.readouterr().err  # nosec B101: pytest outcome assertion


def test_the_contract_command_prints_the_reviewed_snapshot(capsys, contract):
    assert cli.main(["contract"]) == 0  # nosec B101: pytest outcome assertion
    printed = json.loads(capsys.readouterr().out)
    assert printed["snapshot_id"] == contract.snapshot_id  # nosec B101: pytest outcome assertion
    assert printed["calculators"] == sorted(contract.calculators)  # nosec B101: pytest outcome assertion


def test_verify_names_an_evidence_file_it_cannot_read(tmp_path, capsys):
    missing = tmp_path / "missing.json"
    assert cli.main(["verify", "--evidence", str(missing)]) == 1  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err.startswith(f"lodgeit-adapter: {missing} could not be read: ")  # nosec B101: pytest outcome assertion


def test_verify_accepts_a_record_that_hangs_together(tmp_path, capsys):
    record = evidence.build(
        Outcome(status=Status.COMPUTED, manifest={"calculator": CALC},
                advisory={"notes": ["Fabricated evidence test."]}),
        label="fabricated", synthetic=True,
    )
    path = evidence.write(record, tmp_path / "evidence.json")
    assert cli.main(["verify", "--evidence", str(path)]) == 0  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().out == f"evidence: {path} verifies against its own digest\n"  # nosec B101: pytest outcome assertion


# -- the network commands, against the loopback stub ----------------------

def test_discover_prints_the_live_listing(stub, capsys, contract):
    base_url, state = stub
    state.respond(_live_from(contract))
    assert cli.main(["discover", *_network(base_url)]) == 0  # nosec B101: pytest outcome assertion
    printed = json.loads(capsys.readouterr().out)
    assert printed["status"] == str(Status.COMPUTED)  # nosec B101: pytest outcome assertion
    assert state.requests[-1]["path"] == "/v1/calculators"  # nosec B101: pytest outcome assertion


def test_drift_that_agrees_says_so_and_exits_zero(stub, capsys, contract):
    base_url, state = stub
    state.respond(_live_from(contract))
    assert cli.main(["drift", *_network(base_url)]) == 0  # nosec B101: pytest outcome assertion
    out = capsys.readouterr().out
    assert f"drift: live discovery agrees with snapshot {contract.snapshot_id}" in out  # nosec B101: pytest outcome assertion
    assert "Nothing here updates a snapshot." in out  # nosec B101: pytest outcome assertion


def test_drift_that_differs_lists_each_finding_and_exits_one(stub, capsys, contract):
    base_url, state = stub
    state.respond(_live_from(contract)[1:])
    assert cli.main(["drift", *_network(base_url)]) == 1  # nosec B101: pytest outcome assertion
    assert "which live no longer lists" in capsys.readouterr().out  # nosec B101: pytest outcome assertion


def test_drift_that_cannot_reach_the_service_reports_the_outcome(stub, capsys):
    base_url, state = stub
    state.respond({"detail": "down"}, status=503)
    assert cli.main(["drift", *_network(base_url)]) == 1  # nosec B101: pytest outcome assertion
    printed = json.loads(capsys.readouterr().out)
    assert printed["status"] == str(Status.UPSTREAM_UNAVAILABLE)  # nosec B101: pytest outcome assertion


def test_invoke_names_a_body_file_it_cannot_read(stub, tmp_path, capsys):
    base_url, state = stub
    missing = tmp_path / "missing.json"
    code = cli.main(["invoke", *_network(base_url), "--calculator", CALC,
                     "--period", PERIOD, "--body", str(missing)])
    assert code == 2  # nosec B101: pytest outcome assertion
    assert capsys.readouterr().err.startswith(f"lodgeit-adapter: {missing} could not be read: ")  # nosec B101: pytest outcome assertion
    assert state.requests == []  # nosec B101: pytest outcome assertion


# -- the MCP surface --------------------------------------------------------

def _tool(server, name, arguments=None):
    result = asyncio.run(server.call_tool(name, arguments or {}))
    return json.loads(result.content[0].text)


@pytest.fixture
def mcp_server(stub_config):
    pytest.importorskip("mcp.server.mcpserver")
    from lodgeitadapter.mcp import build_server

    return build_server(stub_config)


def test_the_mcp_contract_tool_reads_the_snapshot_offline(mcp_server, stub, contract):
    _, state = stub
    payload = _tool(mcp_server, "reviewed_contract")
    assert payload["snapshot_id"] == contract.snapshot_id  # nosec B101: pytest outcome assertion
    assert payload["remote_access_enabled"] is True  # nosec B101: pytest outcome assertion
    assert state.requests == []  # nosec B101: pytest outcome assertion


def test_the_mcp_drift_tool_reports_findings(mcp_server, stub, contract):
    _, state = stub
    state.respond(_live_from(contract)[1:])
    payload = _tool(mcp_server, "contract_drift")
    assert payload["status"] == str(Status.COMPUTED)  # nosec B101: pytest outcome assertion
    assert any("which live no longer lists" in item for item in payload["findings"])  # nosec B101: pytest outcome assertion


def test_the_mcp_invoke_tool_refuses_a_body_it_cannot_read(mcp_server, stub):
    _, state = stub
    payload = _tool(mcp_server, "invoke_calculator", {
        "calculator_uri": CALC, "period_uri": PERIOD,
        "request_json": "{not json", "network_acknowledged": True,
    })
    assert payload["status"] == "REFUSED_TO_SEND"  # nosec B101: pytest outcome assertion
    assert payload["findings"][0].startswith("request_json could not be read: ")  # nosec B101: pytest outcome assertion
    assert state.requests == []  # nosec B101: pytest outcome assertion


def test_the_mcp_invoke_tool_returns_a_computed_outcome(mcp_server, stub):
    _, state = stub
    state.respond(GOOD)
    payload = _tool(mcp_server, "invoke_calculator", {
        "calculator_uri": CALC, "period_uri": PERIOD,
        "request_json": '{"amalgamated_base": 100000.00, "loan_term_years": 7}',
        "network_acknowledged": True,
    })
    assert payload["status"] == str(Status.COMPUTED)  # nosec B101: pytest outcome assertion
    assert payload["result"]["statutory_myr"] == "21874.92"  # nosec B101: pytest outcome assertion
    sent = state.requests[-1]["body"].decode("utf-8")
    assert '"amalgamated_base":100000.00' in sent  # nosec B101: pytest outcome assertion
