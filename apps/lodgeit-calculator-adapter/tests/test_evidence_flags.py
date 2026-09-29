"""Evidence flags keep their stated boolean meaning across writing and reading."""

from copy import deepcopy

import pytest

from lodgeitadapter import evidence
from lodgeitadapter.cli import main
from lodgeitadapter.client import Outcome, Status

INVALID_FLAGS = ["false", "true", 0, 1, 0.0, 1.0, None, [], {}]


def computed_outcome():
    return Outcome(
        status=Status.COMPUTED,
        calculator="urn:example:calculator",
        period="urn:example:period:2026-07",
        manifest={"calculator": "urn:example:calculator"},
        advisory={"notes": ["Fabricated evidence test."]},
    )


@pytest.mark.parametrize("invalid", INVALID_FLAGS)
def test_build_refuses_a_non_boolean_synthetic_flag(invalid):
    with pytest.raises(ValueError, match="synthetic.*bool"):
        evidence.build(computed_outcome(), label="fabricated", synthetic=invalid)


@pytest.mark.parametrize("field", ["synthetic_input", "validation.accepted"])
@pytest.mark.parametrize("invalid", INVALID_FLAGS)
def test_verify_reports_a_present_non_boolean_flag(field, invalid):
    record = evidence.build(computed_outcome(), label="fabricated", synthetic=True)
    target = record["calculation"]
    parts = field.split(".")
    for part in parts[:-1]:
        target = target[part]
    target[parts[-1]] = invalid
    record["calculation_sha256"] = evidence.digest_of(record)
    assert any(field in finding and "bool" in finding for finding in evidence.verify(record))


def test_verify_reports_a_computed_record_the_producer_did_not_accept():
    record = evidence.build(computed_outcome(), label="fabricated", synthetic=True)
    record["calculation"]["validation"]["accepted"] = False
    record["calculation_sha256"] = evidence.digest_of(record)
    assert any("accepted" in finding for finding in evidence.verify(record))


@pytest.mark.parametrize("synthetic", [False, True])
def test_literal_flags_survive_a_file_round_trip(tmp_path, synthetic):
    record = evidence.build(computed_outcome(), label="fabricated", synthetic=synthetic)
    original = deepcopy(record)
    path = evidence.write(record, tmp_path / "evidence.json")
    reloaded = evidence.read(path)
    assert reloaded == original
    assert reloaded["calculation"]["synthetic_input"] is synthetic
    assert evidence.verify(reloaded) == []
    assert record == reloaded == original


@pytest.mark.parametrize("field", ["synthetic_input", "validation.accepted", "validation"])
def test_omitted_legacy_flags_keep_their_verification_result(field):
    record = evidence.build(computed_outcome(), label="fabricated", synthetic=True)
    target = record["calculation"]
    parts = field.split(".")
    for part in parts[:-1]:
        target = target[part]
    del target[parts[-1]]
    record["calculation_sha256"] = evidence.digest_of(record)
    original = deepcopy(record)
    assert evidence.verify(record) == []
    assert record == original


@pytest.mark.parametrize("status", [value for value in Status if value is not Status.COMPUTED])
def test_non_computed_outcomes_keep_their_accepted_false_flag(status):
    record = evidence.build(Outcome(status=status), label="fabricated", synthetic=True)
    assert record["calculation"]["validation"]["accepted"] is False
    assert evidence.verify(record) == []


def test_verify_command_refuses_the_contradictory_record(tmp_path, capsys):
    record = evidence.build(computed_outcome(), label="fabricated", synthetic=True)
    record["calculation"]["validation"]["accepted"] = False
    record["calculation_sha256"] = evidence.digest_of(record)
    path = evidence.write(record, tmp_path / "evidence.json")
    assert main(["verify", "--evidence", str(path)]) == 1
    assert "validation.accepted=false" in capsys.readouterr().out
