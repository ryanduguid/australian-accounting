"""Index local engine reports without merging their conclusions or scope."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def build_index(manifest_path: Path) -> dict:
    content = manifest_path.read_bytes()
    manifest = json.loads(content, object_pairs_hook=unique_object)
    if not isinstance(manifest, dict) or set(manifest) != {"schema_version", "reports"}:
        raise ValueError("Expected schema_version and reports")
    if manifest["schema_version"] != "group-review-input.v1":
        raise ValueError("Unknown manifest schema")
    entries = manifest["reports"]
    if not isinstance(entries, list) or not entries:
        raise ValueError("At least one report is required")
    root = manifest_path.resolve().parent
    records, identities = [], set()
    fields = {"id", "entity", "period", "engine", "engine_version", "scope", "path", "sha256"}
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != fields:
            raise ValueError("Unexpected report metadata fields")
        if any(not isinstance(value, str) or not value.strip() for value in entry.values()):
            raise ValueError("Every report metadata value must be non-empty text")
        if entry["id"] in identities:
            raise ValueError("Duplicate report ID")
        identities.add(entry["id"])
        relative = Path(entry["path"])
        if relative.is_absolute() or any(part.startswith(".") for part in relative.parts):
            raise ValueError("Report path must remain inside the manifest directory")
        path = (root / relative).resolve()
        if not path.is_relative_to(root) or path.suffix not in {".txt", ".json"}:
            raise ValueError("Report must be text or JSON inside the manifest directory")
        if path.stat().st_size > 2_000_000:
            raise ValueError("Report exceeds the 2 MB example limit")
        source = path.read_bytes()
        if hashlib.sha256(source).hexdigest() != entry["sha256"]:
            raise ValueError(f"Evidence digest mismatch: {entry['id']}")
        text = source.decode("utf-8-sig")
        if not text.strip():
            raise ValueError("Engine report is empty")
        records.append({**entry, "source_output": text})
    return {"schema_version": "group-review-index.v1",
            "manifest_sha256": hashlib.sha256(content).hexdigest(), "reports": records,
            "scope": "Index only. Entity, period, engine version and scope labels are supplied "
                     "assertions. Read each preserved source output for its result and caveats. "
                     "No group verdict, tax consolidation, elimination or approval is calculated."}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    try:
        result = build_index(args.manifest)
    except (ValueError, OSError, UnicodeError) as exc:
        parser.exit(1, f"Group review index: {exc}\n")
    print(json.dumps(result, indent=2))
