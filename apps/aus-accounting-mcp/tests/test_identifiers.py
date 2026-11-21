# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

"""ABN and ACN check digits and the BSB and account number formats through the MCP boundary.

The ABNs and ACNs are the ones ABR and ASIC publish as worked examples and valid
vectors, or deliberately broken copies of them. The BSBs and account numbers are made
up: a format test needs no real one. None is client data.
"""

import asyncio

import pytest
from jsonschema import Draft202012Validator

from aus_accounting_mcp import server
from aus_accounting_mcp.identifiers import (
    ABN_NOT_ESTABLISHED,
    ACCOUNT_NOT_ESTABLISHED,
    ACN_NOT_ESTABLISHED,
    BSB_NOT_ESTABLISHED,
)

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
    argument = {"validate_abn": "abn", "validate_acn": "acn", "validate_bsb_format": "bsb",
                "validate_account_number_format": "account_number"}[name]
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


@pytest.mark.parametrize("value", ["123-456", "123456", "123 456", " 123 - 456 "])
def test_a_bsb_in_the_published_format_is_valid_however_it_is_spaced(value):
    result = call("validate_bsb_format", value)

    assert result["result"] == "FORMAT_VALID"
    assert result["formatted"] == "123-456"
    # The input need not carry the hyphen, so the reason claims the format only for formatted.
    assert result["reason"] == ("The 6 digits fit AusPayNet's published BSB format, written "
                                "XXX-XXX as formatted shows. No institution or branch was "
                                "looked up.")


@pytest.mark.parametrize(
    "value",
    [template.format("123456") for template in NOT_SPACES]
    + ["12-3456", "1234-56", "123--456", "-123456", "12345", "1234567", "123-4567", "12A-456",
       "123\u2013456", "\u0661\u0662\u0663-456", "", "   ", "-"]
    # Full-width digits and hyphen, other dashes (em dash, minus sign, hyphen,
    # non-breaking hyphen), other whitespace inside, junk at either end, 5 digits
    # and a trailing hyphen. NFKC normalising or folding dashes would accept some.
    + ["\uff11\uff12\uff13-\uff14\uff15\uff16", "123\uff0d456", "123\u2014456", "123\u2212456",
       "123\u2010456", "123\u2011456", "123\t456", "123\u3000456", "No. 123-456", "123-456.",
       "x123456", "123-45", "123-456-"],
)
def test_a_malformed_bsb_is_reported_and_never_cleaned_up(value):
    result = call("validate_bsb_format", value)

    assert result["result"] == "MALFORMED"
    assert result["formatted"] is None
    assert result["reason"] == ("A BSB is 6 ASCII digits, written XXX-XXX or without the "
                                "hyphen, optionally separated by spaces.")


@pytest.mark.parametrize("value", ["123-456", "nonsense"])
def test_every_bsb_result_says_the_format_proves_no_institution_or_branch(value):
    result = call("validate_bsb_format", value)

    assert result["identifier"] == "BSB"
    assert result["institution_established"] is False
    assert result["not_established"] == BSB_NOT_ESTABLISHED
    assert "bsb.auspaynet.com.au" in result["not_established"]
    assert result["source_url"].startswith("https://auspaynet.com.au/")
    # Position 5 of the record, not of the BSB, which fills positions 2 to 8.
    assert "positions 2 to 8" in result["source_provision"]
    assert "character position 5" in result["source_provision"]
    assert result["source_checked"] == "2026-11-15"


def test_the_bsb_description_states_the_limit():
    description = " ".join(TOOLS["validate_bsb_format"].description.split())

    assert "format" in description and "proves neither" in description
    assert "BSB Lookup" in description


@pytest.mark.parametrize("marker,bad", [("institution_established", True),
                                        ("result", "CHECK_DIGIT_VALID")])
def test_the_schema_refuses_a_bsb_result_claiming_more_than_its_format(marker, bad):
    value = "123-456"
    payload = dict(server.validate_bsb_format(value))
    payload[marker] = bad

    schema = TOOLS["validate_bsb_format"].output_schema
    assert list(Draft202012Validator(schema).iter_errors(payload))


def test_no_bsb_tool_name_reads_as_a_valid_bsb():
    # A pass is a format, never a BSB: the name says so, as the result does.
    assert "validate_bsb_format" in TOOLS
    assert "validate_bsb" not in TOOLS


ACCOUNT = "validate_account_number_format"


@pytest.mark.parametrize(
    "value,formatted",
    # BECS Procedures 072, Annexure C.1, Record Type 1, positions 9 to 17: letters, digits,
    # hyphens and blanks, leading zeros shown (its own example, 00-1234), up to 9 characters
    # with hyphens edited out of a longer number, and outer blanks being the field's fill.
    [("12345678", "12345678"), ("1", "1"), ("00-1234", "00-1234"), ("123456789", "123456789"),
     ("12-345-678", "12345678"), ("123-456-789", "123456789"), ("AB-12 34", "AB-12 34"),
     ("abc123", "abc123"), ("  12345678 ", "12345678"), ("0000001", "0000001"),
     ("0-1", "0-1")],
)
def test_an_account_number_in_the_published_format_is_valid(value, formatted):
    result = call(ACCOUNT, value)

    assert result["result"] == "FORMAT_VALID"
    assert result["formatted"] == formatted
    assert result["reason"] == ("The account number fits the account number field AusPayNet "
                                "publishes, as formatted shows. No institution or account was "
                                "looked up.")


@pytest.mark.parametrize(
    "value",
    # All zeros or all blanks, hyphens aside; 10 characters once hyphens are out, or with a
    # blank inside, which the procedures do not edit out; and characters outside the field's
    # set: other dashes and whitespace, full-width and Arabic-Indic digits, accented letters
    # and punctuation. Cleaning any of these up could give a different, valid number.
    ["", "   ", "000000000", "0", "00-0000", "0 0", "-", "--", "1234567890", "12-3456-7890",
     "12345 6789", "123\t456", "123\n", "123–456", "123−456", "123‑456",
     "123　456", "１２３", "١٢٣", "É123", "123.456",
     "123/456", "#123456"],
)
def test_a_malformed_account_number_is_reported_and_never_cleaned_up(value):
    result = call(ACCOUNT, value)

    assert result["result"] == "MALFORMED"
    assert result["formatted"] is None
    assert result["reason"] == ("An account number is up to 9 ASCII letters, digits, hyphens "
                                "or spaces, its hyphens removed when it is longer, and not all "
                                "zeros or blanks.")


@pytest.mark.parametrize("value", ["12345678", "nonsense!"])
def test_every_account_number_result_says_the_format_proves_no_account(value):
    result = call(ACCOUNT, value)

    assert result["identifier"] == "account number"
    assert result["account_established"] is False
    assert result["not_established"] == ACCOUNT_NOT_ESTABLISHED
    assert "belongs to the payee" in result["not_established"]
    assert result["source_url"].startswith("https://auspaynet.com.au/")
    assert "positions 9 to 17" in result["source_provision"]
    assert "Must not contain all blanks or all zeros" in result["source_provision"]
    assert "Edit out hyphens where account number exceeds nine" in result["source_provision"]
    assert result["source_checked"] == "2026-11-16"


def test_the_account_number_description_states_the_limit():
    description = " ".join(TOOLS[ACCOUNT].description.split())

    assert "format" in description and "proves neither" in description
    assert "BSB" in description


@pytest.mark.parametrize("marker,bad", [("account_established", True),
                                        ("result", "CHECK_DIGIT_VALID")])
def test_the_schema_refuses_an_account_number_result_claiming_more_than_its_format(marker, bad):
    payload = dict(server.validate_account_number_format("AB-12 34"))
    payload[marker] = bad

    assert list(Draft202012Validator(TOOLS[ACCOUNT].output_schema).iter_errors(payload))


def test_no_account_number_tool_name_reads_as_a_valid_account():
    assert ACCOUNT in TOOLS
    assert "validate_account_number" not in TOOLS
