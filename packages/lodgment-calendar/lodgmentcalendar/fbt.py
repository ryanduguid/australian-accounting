"""FBT return rules: the 2027 return, for the FBT year ending 31 March 2027."""

from __future__ import annotations

from datetime import date

from .rules import Facts, Rule

_MAY = date(2027, 5, 21)
_JUNE = date(2027, 6, 25)
_FACTS = ("lodges_electronically", "on_fbt_client_list_by_21_may")


def fbt_rule(channel: str, facts: Facts) -> tuple[Rule | None, list[str]]:
    """The rule for this channel, or the facts on its path that were not supplied."""
    if channel == "self":
        return Rule("FBT-21-MAY", "FBT return not lodged by a tax agent: lodge and pay by 21 "
                    "May.", "fbt_self", _MAY), []
    if channel == "bas_agent":
        return Rule("FBT-BAS-AGENT", "The ATO's FBT pages give program dates to tax agents and "
                    "the BAS agent program lists no FBT date.", "fbt_agent", _MAY,
                    unresolved="whether a BAS agent can lodge an FBT return: the ATO's FBT "
                    "pages name only tax agents, and its BAS agent program page lists no FBT "
                    "date", payment="unknown"), []
    # Either fact alone can settle 21 May, so neither is asked before the other.
    if facts.lodges_electronically is False:
        return Rule("FBT-AGENT-PAPER", "FBT return a tax agent lodges by paper: lodge and pay "
                    "by 21 May.", "fbt_agent", _MAY), []
    if facts.on_fbt_client_list_by_21_may is False:
        return Rule("FBT-AGENT-NOT-LISTED", "Client not on the tax agent's FBT client list by "
                    "21 May, so outside the lodgment program: lodge and pay by 21 May.",
                    "fbt_agent", _MAY), []
    if facts.lodges_electronically and facts.on_fbt_client_list_by_21_may:
        return Rule("FBT-AGENT-ELECTRONIC", "FBT return a tax agent lodges electronically, for "
                    "a client on its FBT client list by 21 May: generally lodge and pay by 25 "
                    "June.", "fbt_agent", _JUNE), []
    return None, [name for name in _FACTS if getattr(facts, name) is None]
