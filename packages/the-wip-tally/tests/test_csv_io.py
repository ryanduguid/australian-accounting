"""Reading the contract CSV and the column mapping file, and what each refuses."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from wiptally.csv_io import CsvError, _cell, load_mapping, read_contracts

HEADER = "contract_id,original_contract_sum,costs_incurred,estimated_cost_to_complete,certified_billings"
ROW = "JOB-{n},1000.00,400.00,400.00,450.00"


def _contracts(tmp_path: Path, text: str) -> Path:
    path = tmp_path / "contracts.csv"
    path.write_text(text, encoding="utf-8")
    return path


def _read(path: Path) -> list:
    return read_contracts(path, load_mapping(None))


# --- the mapping file ----------------------------------------------------


def _mapping(tmp_path: Path, payload: object) -> Path:
    path = tmp_path / "map.json"
    path.write_text(json.dumps(payload), encoding="utf-8")
    return path


def test_a_missing_mapping_file_is_refused(tmp_path: Path) -> None:
    with pytest.raises(CsvError, match="^cannot read mapping file "):
        load_mapping(tmp_path / "absent.json")


def test_a_mapping_file_that_is_not_utf8_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "map.json"
    path.write_bytes('{"contract_id": "Numéro"}'.encode("cp1252"))
    with pytest.raises(CsvError, match="is not UTF-8"):
        load_mapping(path)


def test_a_mapping_file_that_is_not_json_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "map.json"
    path.write_text("contract_id = Job", encoding="utf-8")
    with pytest.raises(CsvError, match="is not JSON"):
        load_mapping(path)


@pytest.mark.parametrize(
    ("payload", "message"),
    [
        (["contract_id", "Job"], "must be a JSON object$"),
        ({"job_number": "Job"}, "^mapping file names unknown field 'job_number'$"),
        ({"contract_id": " "}, "^mapping for contract_id must be a non-empty column heading$"),
        ({"contract_id": 7}, "^mapping for contract_id must be a non-empty column heading$"),
    ],
)
def test_a_malformed_mapping_is_refused(tmp_path: Path, payload: object, message: str) -> None:
    with pytest.raises(CsvError, match=message):
        load_mapping(_mapping(tmp_path, payload))


# --- the contract file ---------------------------------------------------


def test_a_missing_contract_file_is_refused(tmp_path: Path) -> None:
    with pytest.raises(CsvError, match="^cannot read "):
        _read(tmp_path / "absent.csv")


def test_an_empty_contract_file_is_refused(tmp_path: Path) -> None:
    with pytest.raises(CsvError, match="contracts.csv is empty$"):
        _read(_contracts(tmp_path, ""))


def test_a_header_without_contract_rows_is_refused(tmp_path: Path) -> None:
    with pytest.raises(CsvError, match="contracts.csv has a header but no contract rows$"):
        _read(_contracts(tmp_path, f"{HEADER}\n\n,,,,\n"))


def test_missing_required_columns_are_named(tmp_path: Path) -> None:
    with pytest.raises(
        CsvError, match=r"is missing required column\(s\): estimated_cost_to_complete, certified_billings$"
    ):
        _read(_contracts(tmp_path, "contract_id,original_contract_sum,costs_incurred\nJOB-1,1,1\n"))


def test_a_duplicated_heading_is_refused(tmp_path: Path) -> None:
    with pytest.raises(CsvError, match="^duplicated column heading 'Customer'$"):
        _read(_contracts(tmp_path, f"{HEADER},customer,Customer\n{ROW.format(n=1)},A,B\n"))


def test_an_unnamed_column_is_passed_over(tmp_path: Path) -> None:
    [contract] = _read(_contracts(tmp_path, f"{HEADER},\n{ROW.format(n=1)},note\n"))
    assert contract.contract_id == "JOB-1"  # nosec B101: pytest outcome assertion


def test_blank_rows_are_passed_over(tmp_path: Path) -> None:
    text = f"{HEADER}\n{ROW.format(n=1)}\n\n,,,,\n{ROW.format(n=2)}\n"
    assert [c.contract_id for c in _read(_contracts(tmp_path, text))] == ["JOB-1", "JOB-2"]  # nosec B101: pytest outcome assertion


def test_a_reused_contract_id_is_refused(tmp_path: Path) -> None:
    text = f"{HEADER}\n{ROW.format(n=1)}\n{ROW.format(n=1)}\n"
    with pytest.raises(CsvError, match="row 3: contract_id 'JOB-1' already used on row 2"):
        _read(_contracts(tmp_path, text))


def test_more_than_twenty_unreadable_rows_stop_the_read_and_say_there_are_more(tmp_path: Path) -> None:
    bad = "".join(f"JOB-{n},abc,400.00,400.00,450.00\n" for n in range(1, 26))
    with pytest.raises(CsvError) as exc_info:
        _read(_contracts(tmp_path, f"{HEADER}\n{bad}"))
    message = str(exc_info.value)
    assert message.count("is not an amount") == 20  # nosec B101: pytest outcome assertion
    assert message.endswith("\n  ... and more")  # nosec B101: pytest outcome assertion


@pytest.mark.parametrize(
    ("header", "row", "message"),
    [
        (HEADER, " ,1000.00,400.00,400.00,450.00", "row 2: contract_id is blank"),
        (HEADER, "JOB-1,,400.00,400.00,450.00", r"row 2 \(JOB-1\): original_contract_sum is blank"),
        (f"{HEADER},progress_method", f"{ROW.format(n=1)},units",
         r"row 2 \(JOB-1\): progress_method 'units' is not one of"),
        (f"{HEADER},retention_classification", f"{ROW.format(n=1)},asset",
         r"row 2 \(JOB-1\): retention_classification 'asset' is not one of"),
    ],
)
def test_an_unusable_row_is_refused_with_its_reason(
    tmp_path: Path, header: str, row: str, message: str
) -> None:
    with pytest.raises(CsvError, match=message):
        _read(_contracts(tmp_path, f"{header}\n{row}\n"))


def test_a_cell_past_the_end_of_its_row_reads_as_absent() -> None:
    # read_contracts refuses short rows first; the helper still answers None.
    assert _cell(["JOB-1"], {"customer": 3}, "Customer") is None  # nosec B101: pytest outcome assertion
