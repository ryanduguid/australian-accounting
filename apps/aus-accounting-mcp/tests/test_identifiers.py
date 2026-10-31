"""ABN and ACN check digits through the MCP boundary.

The numbers are the ones ABR and ASIC publish as worked examples and valid
vectors, or deliberately broken copies of them. None is client data.
"""

import asyncio

import pytest
from jsonschema import Draft202012Validator

from aus_accounting_mcp import server
from aus_accounting_mcp.identifiers import ABN_NOT_ESTABLISHED, ACN_NOT_ESTABLISHED

TOOLS = {tool.name: tool for tool in asyncio.run(server.mcp.list_tools())}

# ASIC EDGE Document Messages Specification 5.10, Appendix D, 'Valid ACNs include'.
ASIC_VALID_ACNS = """
    000 000 019  000 250 000  000 500 005  000 750 005
    001 000 004  001 250 004  001 500 009  001 749 999
    001 999 999  002 249 998  002 499 998  002 749 993
    002 999 993  003 249 992  003 499 992  003 749 988
    003 999 988  004 249 987  004 499 987  004 749 982
    004 999 982  005 249 981  005 499 981  005 749 986
    005 999 977  006 249 976  006 499 976  006 749 980
    006 999 980  007 249 989  007 499 989  007 749 975
    007 999 975  008 249 974  008 499 974  008 749 979
    008 999 979  009 249 969  009 499 969  009 749 964
    009 999 964  010 249 966  010 499 966  010 749 961
"""
ACNS = [" ".join(triple) for triple in zip(*[iter(ASIC_VALID_ACNS.split())] * 3)]

# Spaces are the only formatting accepted. Stripping anything else could turn a
# malformed identifier into a different, valid one.
NOT_SPACES = ["\t{}", "{}\n", "{}\r", " {}", " {}", " {}", "{}-", "{}A"]


def call(name, value):
    argument = {"validate_abn": "abn", "validate_acn": "acn"}[name]
    result = asyncio.run(server.mcp.call_tool(name, {argument: value}))
    assert not result.is_error, result.content
    Draft202012Validator(TOOLS[name].output_schema).validate(result.structured_content)
    return result.structured_content


@pytest.mark.parametrize("value", ["51 824 753 556", "51824753556", " 51 824 753 556 "])
def test_abr_worked_example_is_valid_however_it_is_spaced(value):
    result = call("validate_abn", value)

    assert result["result"] == "CHECK_DIGIT_VALID"
    assert result["check_digit_valid"] is True
    assert result["formatted"] == "51 824 753 556"


@pytest.mark.parametrize("value", ["51 824 753 557", "41 824 753 556", "61 824 753 556"])
def test_an_abn_with_a_wrong_digit_fails(value):
    result = call("validate_abn", value)

    assert result["result"] == "CHECK_DIGIT_INVALID"
    assert result["check_digit_valid"] is False
    assert result["formatted"] == value


@pytest.mark.parametrize("value", ["00000090000", "00 000 000 000"])
def test_a_leading_zero_abn_fails_though_its_sum_divides_by_89(value):
    # ABR's first step subtracts 1 from the first digit to give a new 11 digit number,
    # which a leading zero cannot do. The sum alone would pass 00000090000 (99 - 10).
    assert call("validate_abn", value)["result"] == "CHECK_DIGIT_INVALID"


@pytest.mark.parametrize(
    "value",
    [template.format("51824753556") for template in NOT_SPACES]
    + ["51-824-753-556", "5182475355A", "5182475355", "518247535566", "", "   ",
       "٥١824753556"],
)
def test_a_malformed_abn_is_reported_and_never_cleaned_up(value):
    result = call("validate_abn", value)

    assert result["result"] == "MALFORMED"
    assert result["check_digit_valid"] is False
    assert result["formatted"] is None


def test_asic_worked_example_is_valid():
    result = call("validate_acn", "004 085 616")

    assert result["result"] == "CHECK_DIGIT_VALID"
    assert result["formatted"] == "004 085 616"


def test_every_acn_asic_lists_as_valid_passes():
    assert len(ACNS) == 44
    assert {call("validate_acn", acn)["result"] for acn in ACNS} == {"CHECK_DIGIT_VALID"}


def test_a_complement_of_ten_means_a_check_digit_of_zero():
    # 000 250 000: the sum is 30, the remainder 0 and the complement 10, set to 0.
    assert call("validate_acn", "000250000")["check_digit_valid"] is True
    assert call("validate_acn", "000250001")["result"] == "CHECK_DIGIT_INVALID"


@pytest.mark.parametrize("value", ["004 085 617", "004 085 626", "123 456 789"])
def test_an_acn_with_a_wrong_digit_fails(value):
    # 123 456 789 is the illustration on ASIC's ACN page, not a number that passes.
    assert call("validate_acn", value)["result"] == "CHECK_DIGIT_INVALID"


@pytest.mark.parametrize(
    "value",
    [template.format("004085616") for template in NOT_SPACES]
    + ["004-085-616", "00408561", "0040856161", ""],
)
def test_a_malformed_acn_is_reported_and_never_cleaned_up(value):
    result = call("validate_acn", value)

    assert result["result"] == "MALFORMED"
    assert result["formatted"] is None


@pytest.mark.parametrize(
    "name,value",
    [("validate_abn", "51 824 753 556"), ("validate_abn", "51 824 753 557"),
     ("validate_abn", "nonsense"), ("validate_acn", "004 085 616"),
     ("validate_acn", "004 085 617"), ("validate_acn", "nonsense")],
)
def test_every_result_says_a_check_digit_proves_no_registration(name, value):
    result = call(name, value)

    assert result["abr_registration_established"] is False
    assert result["gst_registration_established"] is False
    assert "Australian Business Register" in result["not_established"]
    assert "registered for GST" in result["not_established"]
    assert result["source_url"].startswith("https://")
    assert result["source_checked"] == "2026-10-31"


def test_each_identifier_keeps_its_own_statement_and_source():
    abn = call("validate_abn", "51 824 753 556")
    acn = call("validate_acn", "004 085 616")

    assert abn["not_established"] == ABN_NOT_ESTABLISHED
    assert acn["not_established"] == ACN_NOT_ESTABLISHED
    assert "abr.business.gov.au" in abn["source_url"]
    assert "asic.gov.au" in acn["source_url"]
    assert "ASIC" in acn["not_established"]


@pytest.mark.parametrize("name", ["validate_abn", "validate_acn"])
def test_the_published_description_states_the_limit(name):
    description = " ".join(TOOLS[name].description.split())

    assert "neither" in description and "GST registration" in description


@pytest.mark.parametrize(
    "marker,bad", [("abr_registration_established", True), ("gst_registration_established", True),
                   ("result", "REGISTERED")],
)
def test_the_schema_refuses_a_result_claiming_registration(marker, bad):
    payload = dict(server.validate_abn("51 824 753 556"))
    payload[marker] = bad

    assert list(Draft202012Validator(TOOLS["validate_abn"].output_schema).iter_errors(payload))
