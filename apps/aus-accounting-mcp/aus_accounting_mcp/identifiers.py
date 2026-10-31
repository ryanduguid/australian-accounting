"""ABN and ACN check digits. Arithmetic only: no lookup, and nothing about registration.

These live in the facade because the engines it can reach are the published
distributions, and none of them holds these rules. They are the published format
tests of 2 registers, with no rate, date or dataset to go stale, so no engine's
review scope is copied here.
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
