# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Write legislation-provisions.json from a local build of the legislation corpus.

Development-time script, like the crate's other tools, and standard library only.
CI has no corpus, so the root tests/test_dated_rates.py holds each record's
`legislation` citations to this checked-in list, which records where each provision
came from: the corpus retrieval date, each title's compilation, the one corpus row
whose heading is that provision and the digest of the sections.jsonl it was read from.
Run it after changing a citation or the corpus:

    python tools/build_provision_list.py <corpus folder>
"""

import hashlib
import json
import sys
from pathlib import Path

PACKAGE = Path(__file__).resolve().parents[1]


def citations():
    """The distinct (title ID, provision) pairs the records cite."""
    pairs = set()
    for path in sorted((PACKAGE / "data").glob("*.json")):
        record = json.loads(path.read_text(encoding="utf-8"))
        pairs.update((c["title_id"], c["provision"]) for c in record.get("legislation", ()))
    return sorted(pairs)


def heading_number(row):
    """The provision a row's heading starts with, with an ASCII hyphen."""
    words = row.get("heading", "").replace("\u2011", "-").split()
    return words[0] if words else None


def main():
    corpus = Path(sys.argv[1])
    sources = json.loads((corpus / "sources.json").read_text(encoding="utf-8"))
    provisions = []
    for title_id, provision in citations():
        data = (corpus / "markdown" / title_id / "sections.jsonl").read_bytes()
        rows = [json.loads(line) for line in data.splitlines()]
        found = [row for row in rows if heading_number(row) == provision]
        if len(found) != 1:
            raise SystemExit(f"{title_id} has {len(found)} rows for {provision} in this corpus")
        provisions.append(
            {
                "title_id": title_id,
                "provision": provision,
                "act": found[0]["act"],
                "compilation_number": found[0]["compilation_number"],
                "compilation_date": found[0]["compilation_date"],
                "row_id": found[0]["row_id"],
                "heading": found[0]["heading"],
                "sections_sha256": hashlib.sha256(data).hexdigest(),
            }
        )
    listing = {
        "corpus": {"name": sources["corpus"], "retrieved": sources["retrieved"]},
        "provisions": provisions,
    }
    text = json.dumps(listing, ensure_ascii=False, indent=2) + "\n"
    (PACKAGE / "legislation-provisions.json").write_bytes(text.encode("utf-8"))


if __name__ == "__main__":
    main()
