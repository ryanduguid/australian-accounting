"""Payloads for the MCP resources and prompts this server publishes.

A host can read these without calling a tool, so they carry the context a model
needs before it chooses one: what this server refuses, which benchmark years are
shipped, and which engine actually produced a result.

Everything here is read from the installed engines or from this package's own
policy text. Nothing is read from the repository working tree. The published
wheel ships the import package alone, so a resource that read `DISCLAIMER.md` or
`compatibility.json` from disk would be empty for the documented `uvx` install.
The component record is therefore built from installed distribution metadata,
which is the version that produced the result rather than the version a checked
in record says was released; `tests/test_resources.py` holds the two together.

No rate, threshold, law date or source date is written here. Those stay owned by
the engines and appear in their own payloads.
"""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError, version
from typing import Any

from atobenchmark.dataset import available_years, load
from atobenchmark.report import DISCLAIMER as BENCHMARK_DISCLAIMER

from .adapters.div7a import DISCLAIMER as DIV7A_DISCLAIMER
from .adapters.payday import DISCLAIMER as PAYDAY_DISCLAIMER

SERVER_DISTRIBUTION = "aus-accounting-mcp"
ENGINE_DISTRIBUTIONS = (
    "ato-benchmark-compare",
    "div7a-loan-review",
    "payday-super-checker",
)

#: The facade's own boundary statement, paragraph by paragraph. Every one of
#: these is also a paragraph of the public `DISCLAIMER.md`, and
#: `tests/test_resources.py` fails if any of them stops being one, so a host
#: reading the resource and a reader opening the file get the same boundary.
BOUNDARY_PARAGRAPHS = (
    "aus-accounting-mcp is a Model Context Protocol server that exposes reviewed"
    " Australian computational accounting engines. It is not tax, legal, accounting,"
    " financial, investment, BAS-agent, registered-tax-agent, or assurance advice.",
    "Outputs can be wrong, incomplete, stale, or unsuitable for a given set of"
    " facts. Tax law, ATO guidance, rates, thresholds, and administrative practice"
    " change. Verify every mutable fact against current official sources before"
    " relying on an output.",
    "This server does not lodge tax returns, BAS, FBT returns, TPAR, STP reports, or"
    " any other official form. It does not post journals, execute payments, or send"
    " correspondence. Division 7A outputs are experimental reviews of s 109N loan"
    " terms and rates and s 109E minimum yearly repayments for operator-supplied"
    " amalgamated loans; they are not ATO assessments or Division 7A determinations."
    " Unsupported matters, including s 109R repayment classification, unpaid present"
    " entitlements, distributable surplus, interposed entities, debt forgiveness and"
    " Commissioner discretion, remain refused. SBR payloads from this server are"
    " synthetic fixtures marked `synthetic: true`; they are not lodgments.",
    "Payday Super reviews can mark `UNKNOWN` or refuse where the facts do not"
    " establish the statutory test. A remittance date alone cannot produce `ON_TIME`."
    " Experimental SG-charge figures are exposure flags, not an ATO assessment.",
    "`ok: true` means a tool ran, not that a review passed. Retain `UNKNOWN`,"
    " `REFUSED`, `not_supplied` and `null` outcomes as they are returned, and obtain"
    " human review before any consequential accounting action.",
)

BOUNDARY = "\n\n".join(BOUNDARY_PARAGRAPHS)


def _installed(distribution: str) -> str | None:
    try:
        return version(distribution)
    except PackageNotFoundError:  # running from a source tree without installation
        return None


def disclaimer() -> str:
    """The facade boundary followed by each delegated engine's own statement."""
    return "\n\n".join(
        (
            "# Boundary",
            BOUNDARY,
            "# ato-benchmark-compare",
            BENCHMARK_DISCLAIMER,
            "# payday-super-checker",
            PAYDAY_DISCLAIMER,
            "# div7a-loan-review",
            DIV7A_DISCLAIMER,
        )
    )


def benchmark_dataset_years() -> dict[str, Any]:
    """The ATO benchmark years shipped by the installed engine, with provenance.

    Bundled data, not a live ATO lookup. A year outside this list is refused by
    the tools rather than estimated from an adjacent year.
    """
    years = available_years()
    datasets = []
    for year in years:
        data = load(year)
        datasets.append(
            {
                "benchmark_year": data.year,
                "business_types": len(data.business_types),
                "source": dict(data.source),
            }
        )
    return {
        "engine": "ato-benchmark-compare",
        "engine_version": _installed("ato-benchmark-compare"),
        "latest_benchmark_year": years[-1] if years else None,
        "bundled_data": True,
        "live_lookup": False,
        "datasets": datasets,
    }


def component_versions() -> dict[str, Any]:
    """The server and engine versions actually installed in this environment.

    These are the versions that produce results here, so they are read from
    installed metadata rather than from a checked in record. A distribution that
    is not installed reports null rather than a guess.
    """
    return {
        "server": {
            "distribution": SERVER_DISTRIBUTION,
            "version": _installed(SERVER_DISTRIBUTION),
            "registry_identity": "io.github.ryanduguid/aus-accounting",
            "repository": "https://github.com/ryanduguid/australian-accounting",
            "transport": "stdio",
        },
        "engines": [
            {"distribution": name, "version": _installed(name)}
            for name in ENGINE_DISTRIBUTIONS
        ],
    }
