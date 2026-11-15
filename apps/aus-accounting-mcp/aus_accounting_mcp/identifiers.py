# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""ABN and ACN check digits and the BSB and account number formats. Arithmetic and format
only: no lookup, and nothing about registration.

These live in the facade because the engines it can reach are the published
distributions, and none of them holds these rules. They are the published format
tests of 2 registers and of AusPayNet's BSB, with no rate, date or dataset to go
stale, so no engine's review scope is copied here. There is no BSB directory here, and
nothing about accounts.
"""

from __future__ import annotations

from typing import Any

_ABN_WEIGHTS = (10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19)
_ACN_WEIGHTS = (8, 7, 6, 5, 4, 3, 2, 1)
_ALLOWED = frozenset("0123456789 ")

ABN_SOURCE = "https://abr.business.gov.au/Help/AbnFormat"
ABN_PROVISION = "ABN Lookup, Format of the ABN: 'To verify an ABN' (version 9.9.7)"
ACN_SOURCE = ("https://download.asic.gov.au/media/vy0ensam/"
              "els-document-messages-specification-v5-10.pdf")
ACN_PROVISION = ("ASIC EDGE Document Messages Specification version 5.10 (1 December 2024), "
                 "Appendix D, Australian Company Number (ACN) Check Digit")
SOURCE_CHECKED = "2026-10-31"

BSB_SOURCE = ("https://auspaynet.com.au/sites/default/files/2026-10/"
              "BECS%20Procedures%20-%20Version%20E072%20-%20Effective%201%20October%202026"
              "%20-%20public_0.pdf")
BSB_PROVISION = ("AusPayNet BECS Procedures version 072 (effective 1 October 2026), Annexure C.1 "
                 "DE File User Item Specifications, Record Type 1, BSB Number in character "
                 "positions 2 to 8: 'Must be numeric with a hyphen in character position 5'")
BSB_SOURCE_CHECKED = "2026-11-15"

_ACCOUNT_ALLOWED = frozenset("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789- ")
ACCOUNT_PROVISION = ("AusPayNet BECS Procedures version 072 (effective 1 October 2026), Annexure "
                     "C.1 DE File User Item Specifications, Record Type 1, Account Number to be "
                     "Credited/Debited in character positions 9 to 17: 'Alpha (26 letters of the "
                     "alphabet), numeric, hyphens & blanks only are valid. Must not contain all "
                     "blanks or all zeros. ... Edit out hyphens where account number exceeds nine "
                     "characters. Right justified. Blank filled.'")
ACCOUNT_SOURCE_CHECKED = "2026-11-16"

ABN_NOT_ESTABLISHED = (
    "A valid check digit shows only that the digits pass the arithmetic ABN Lookup publishes. "
    "It proves neither that the number was issued to anyone or is active, nor that an entity "
    "is registered in the Australian Business Register or registered for GST. ABN Lookup "
    "(https://abr.business.gov.au) shows an ABN's status and if and when an entity is "
    "registered for GST."
)
ACN_NOT_ESTABLISHED = (
    "A valid check digit shows only that the digits pass the arithmetic ASIC publishes. It "
    "proves neither that ASIC issued the number or that a company holding it exists, nor that "
    "an entity is registered in the Australian Business Register or registered for GST. Search "
    "ASIC's registers for the company and ABN Lookup (https://abr.business.gov.au) for ABN and "
    "GST status."
)
BSB_NOT_ESTABLISHED = (
    "A BSB in the published format shows only that it is 6 digits written XXX-XXX. It proves "
    "neither that its first 2 or 3 digits are an Institution Identifier AusPayNet issued, nor "
    "that the BSB exists, is open or accepts payments, nor anything about an account. "
    "AusPayNet's BSB Lookup (https://bsb.auspaynet.com.au) shows a BSB's institution and branch."
)

ACCOUNT_NOT_ESTABLISHED = (
    "An account number in the published format shows only that it fits the 9 character "
    "account number field of AusPayNet's BECS Procedures. It proves neither that the account "
    "exists, is open or belongs to the payee, nor that it matches a BSB, nor that it carries "
    "the leading zeros its institution's numbering needs. Confirm bank details with the "
    "account holder by a separate, trusted channel."
)


def _digits(value: str, length: int) -> list[int] | None:
    """The digits, or None. Only ASCII spaces are removed: stripping anything else could
    turn a malformed identifier into a different, valid one."""
    if not _ALLOWED.issuperset(value):
        return None
    digits = value.replace(" ", "")
    return [int(digit) for digit in digits] if len(digits) == length else None


def _abn_passes(digits: list[int]) -> bool:
    # ABR's first step subtracts 1 from the first digit to give a new 11 digit number,
    # which a leading zero cannot do, though 00000090000 would still divide by 89.
    if digits[0] == 0:
        return False
    adjusted = [digits[0] - 1, *digits[1:]]
    return sum(digit * weight for digit, weight in zip(adjusted, _ABN_WEIGHTS)) % 89 == 0


def _acn_passes(digits: list[int]) -> bool:
    total = sum(digit * weight for digit, weight in zip(digits, _ACN_WEIGHTS))
    return (10 - total % 10) % 10 == digits[8]


def _result(identifier: str, digits: list[int] | None, passes: bool, groups: tuple[int, ...],
            not_established: str, source: str, provision: str) -> dict[str, Any]:
    formatted = None
    if digits is not None:
        text, start = "".join(map(str, digits)), 0
        parts = []
        for size in groups:
            parts.append(text[start:start + size])
            start += size
        formatted = " ".join(parts)
    length = sum(groups)
    if digits is None:
        result = "MALFORMED"
        reason = f"An {identifier} is {length} ASCII digits, optionally separated by spaces."
    elif passes:
        result = "CHECK_DIGIT_VALID"
        reason = f"The digits pass the published {identifier} check digit test."
    else:
        result = "CHECK_DIGIT_INVALID"
        reason = (f"The digits fail the published {identifier} check digit test, so this is "
                  f"not a valid {identifier} as written. Check it against its source.")
    return {
        "ok": True,
        "identifier": identifier,
        "result": result,
        "check_digit_valid": passes,
        "formatted": formatted,
        "reason": reason,
        "abr_registration_established": False,
        "gst_registration_established": False,
        "not_established": not_established,
        "source_url": source,
        "source_provision": provision,
        "source_checked": SOURCE_CHECKED,
    }


def check_abn(value: str) -> dict[str, Any]:
    digits = _digits(value, 11)
    return _result("ABN", digits, digits is not None and _abn_passes(digits), (2, 3, 3, 3),
                   ABN_NOT_ESTABLISHED, ABN_SOURCE, ABN_PROVISION)


def check_acn(value: str) -> dict[str, Any]:
    digits = _digits(value, 9)
    return _result("ACN", digits, digits is not None and _acn_passes(digits), (3, 3, 3),
                   ACN_NOT_ESTABLISHED, ACN_SOURCE, ACN_PROVISION)


def check_bsb(value: str) -> dict[str, Any]:
    text = value.replace(" ", "")
    if len(text) == 7 and text[3] == "-":
        text = text[:3] + text[4:]
    digits = _digits(text, 6)
    if digits is None:
        result, formatted = "MALFORMED", None
        reason = ("A BSB is 6 ASCII digits, written XXX-XXX or without the hyphen, optionally "
                  "separated by spaces.")
    else:
        result, formatted = "FORMAT_VALID", f"{text[:3]}-{text[3:]}"
        reason = ("The 6 digits fit AusPayNet's published BSB format, written XXX-XXX as "
                  "formatted shows. No institution or branch was looked up.")
    return {
        "ok": True,
        "identifier": "BSB",
        "result": result,
        "formatted": formatted,
        "reason": reason,
        "institution_established": False,
        "not_established": BSB_NOT_ESTABLISHED,
        "source_url": BSB_SOURCE,
        "source_provision": BSB_PROVISION,
        "source_checked": BSB_SOURCE_CHECKED,
    }


def check_account_number(value: str) -> dict[str, Any]:
    # Blanks are valid inside the field, so only the outer ones, the field's fill, go.
    text = value.strip(" ")
    if len(text) > 9:
        text = text.replace("-", "")
    # Hyphens set aside, nothing but zeros and blanks is the all zeros or all blanks
    # the field refuses.
    if (_ACCOUNT_ALLOWED.issuperset(value) and len(text) <= 9
            and text.replace("-", "").strip("0 ")):
        result, formatted = "FORMAT_VALID", text
        reason = ("The account number fits the account number field AusPayNet publishes, as "
                  "formatted shows. No institution or account was looked up.")
    else:
        result, formatted = "MALFORMED", None
        reason = ("An account number is up to 9 ASCII letters, digits, hyphens or spaces, its "
                  "hyphens removed when it is longer, and not all zeros or blanks.")
    return {
        "ok": True,
        "identifier": "account number",
        "result": result,
        "formatted": formatted,
        "reason": reason,
        "account_established": False,
        "not_established": ACCOUNT_NOT_ESTABLISHED,
        "source_url": BSB_SOURCE,
        "source_provision": ACCOUNT_PROVISION,
        "source_checked": ACCOUNT_SOURCE_CHECKED,
    }
