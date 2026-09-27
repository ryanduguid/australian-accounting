"""One search page shape for every paginated tool."""

from __future__ import annotations

from typing import Any

BOUNDARY_NOTICE = (
    "More matches remain, but continuing past 10000 results is not supported: "
    "narrow the query and search again."
)


def page(
    matches: list[dict[str, Any]],
    seen: int,
    *,
    has_more: bool,
    notice: str,
    max_offset: int,
    **extra: Any,
) -> dict[str, Any]:
    """One search page, emitting a continuation offset only when it is accepted back.

    Emitting an offset the tool refuses would strand the caller, so past
    ``max_offset`` the page says what to do instead of returning a value that
    cannot be passed back.
    """
    result: dict[str, Any] = {
        "matches": matches, "has_more": has_more, "next_offset": None, **extra, "notice": notice,
    }
    if has_more:
        if seen > max_offset:
            result["notice"] = notice + " " + BOUNDARY_NOTICE
        else:
            result["next_offset"] = seen
    return result
