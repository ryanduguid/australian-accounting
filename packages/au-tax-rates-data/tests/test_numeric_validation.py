import copy
import json

import pytest

import rates


@pytest.mark.parametrize("unit", ["AUD", "percent", "cents_per_km"])
@pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
def test_non_finite_values_are_refused(unit, value):
    assert rates.check_value(unit, value)


@pytest.mark.parametrize("unit, value", [("AUD", 0), ("AUD", 7483.05),
                                        ("percent", 0), ("percent", 100),
                                        ("cents_per_km", 91), ("AUD", 10 ** 400)])
def test_finite_non_negative_values_remain_valid(unit, value):
    assert rates.check_value(unit, value) == []


def tax_scale():
    return [
        {"from": 0, "to": 100, "base_tax": 0, "marginal_rate": 0},
        {"from": 101, "to": 200, "base_tax": 0, "marginal_rate": 10},
        {"from": 201, "to": None, "base_tax": 10, "marginal_rate": 20},
    ]


@pytest.mark.parametrize("index, field, value", [
    (0, "from", -1), (0, "from", 1), (0, "from", True),
    (0, "to", 100.5), (0, "to", None), (0, "to", "100"),
    (1, "from", None), (1, "from", "101"), (1, "from", float("nan")),
    (1, "to", 99), (1, "to", float("inf")), (1, "to", False),
    (0, "base_tax", "0"), (1, "base_tax", -1), (1, "base_tax", None),
    (1, "base_tax", float("nan")), (1, "base_tax", float("inf")),
    (1, "base_tax", True), (1, "marginal_rate", "10"),
    (2, "marginal_rate", -5), (2, "marginal_rate", 101),
    (2, "marginal_rate", None), (2, "marginal_rate", float("nan")),
    (2, "marginal_rate", float("inf")), (2, "marginal_rate", False),
])
def test_invalid_bracket_values_return_problems(index, field, value):
    scale = tax_scale()
    scale[index][field] = value
    assert rates.check_scale(scale)


def test_valid_scale_uses_independently_calculated_base_amounts():
    # The middle bracket covers 100 dollars at 10%, producing 10 dollars tax.
    assert rates.check_scale(tax_scale()) == []
    assert rates.check_scale([
        {"from": 0, "to": None, "base_tax": 0, "marginal_rate": 100},
    ]) == []


@pytest.mark.parametrize("content", [
    b"", b"\xef\xbb\xbf", b"\xff\xfe\x00garbage", b"{not json", b"[]", b'"x"', b"null",
    b"[" * 100_000 + b"]" * 100_000, b'{"value": ' + b"9" * 5000 + b"}",
], ids=["empty", "bom-only", "binary", "not-json", "array", "string", "null", "deep", "huge-int"])
def test_an_unreadable_record_is_reported_and_the_rest_still_checked(tmp_path, monkeypatch, capsys, content):
    later = copy.deepcopy(next(r for r in rates.load_records() if r["unit"] == "AUD"))
    later["value"] = -1  # a problem only validation finds, so its report proves the check ran
    data = tmp_path / "data"
    data.mkdir()
    (data / f"{later['id']}.json").write_text(json.dumps(later), encoding="utf-8")
    (data / "aaa-broken.json").write_bytes(content)
    monkeypatch.setattr(rates, "DATA", data)
    assert rates.cmd_validate(None) == 1
    out = capsys.readouterr().out
    assert out.startswith("aaa-broken: ")
    assert f"\n{later['id']}: " in out
    assert "2 record(s) with problems" in out


@pytest.mark.parametrize("url", [None, [], {}, True, 1.5])
def test_a_source_url_that_is_not_text_is_a_problem_not_a_crash(tmp_path, monkeypatch, capsys, url):
    record = copy.deepcopy(next(r for r in rates.load_records() if r["unit"] == "AUD"))
    record["source_url"] = url
    data = tmp_path / "data"
    data.mkdir()
    (data / f"{record['id']}.json").write_text(json.dumps(record), encoding="utf-8")
    monkeypatch.setattr(rates, "DATA", data)
    assert rates.cmd_validate(None) == 1
    assert "missing or wrong type: source_url" in capsys.readouterr().out


def test_validation_reports_overflowing_json_number(tmp_path, monkeypatch, capsys):
    record = copy.deepcopy(next(r for r in rates.load_records() if r["unit"] == "AUD"))
    record.pop("pattern", None)
    record["value"] = "OVERFLOW"
    data = tmp_path / "data"
    data.mkdir()
    path = data / f"{record['id']}.json"
    path.write_text(json.dumps(record).replace('"OVERFLOW"', '1e999'), encoding="utf-8")
    monkeypatch.setattr(rates, "DATA", data)
    assert rates.cmd_validate(None) == 1
    assert "1 record(s) with problems" in capsys.readouterr().out
