#!/usr/bin/env python3
"""Verify preserved KB inputs, live reference fidelity, skills, and optional source drift."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path


def digest(path: Path) -> str:
    """Return the SHA-256 of a complete file without changing its contents."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_file(path: Path, expected: str, errors: list[str]) -> None:
    """Record missing or changed input evidence rather than refresh its receipt."""
    if not path.is_file():
        errors.append(f"Missing file: {path}")
    elif digest(path) != expected:
        errors.append(f"Hash changed: {path}")


def main() -> int:
    """Check the saved ingestion contract and return nonzero on integrity failure."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check-sources", action="store_true", help="Also compare canonical sibling source files")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    manifest = root / ".agents/resources/provenance/ingestion-manifest.json"
    data = json.loads(manifest.read_text())
    errors: list[str] = []

    # Exact source preservation and live reference fidelity are separate checks.
    for item in data["documents"]:
        check_file(root / item["raw_path"], item["sha256"], errors)
        reference = root / item["reference_path"]
        if not reference.is_file():
            errors.append(f"Missing reference: {reference}")
            continue
        parts = reference.read_text().split("<!-- kb-import:end -->\n\n", 1)
        if len(parts) != 2:
            errors.append(f"Missing provenance boundary: {reference}")
            continue
        body = parts[1]
        actual = hashlib.sha256(body.encode()).hexdigest()
        if actual != item["rendered_body_sha256"]:
            errors.append(f"Imported body changed without reconciled provenance: {reference}")
        headings = re.findall(r"^#{1,6} .+", body, re.M)
        if headings != item["source_headings"] or headings != item["rendered_headings"]:
            errors.append(f"Source heading inventory differs: {reference}")

    for item in data.get("additional_raw_sources", []):
        check_file(root / item["raw_path"], item["sha256"], errors)

    # Validate complete installed packages without treating incidental Python caches as source.
    for skill in data["skills"]:
        folder = root / ".agents/skills" / skill["name"]
        expected = {item["path"] for item in skill["files"]}
        actual = {
            str(path.relative_to(folder))
            for path in folder.rglob("*")
            if path.is_file() and "__pycache__" not in path.parts and path.suffix != ".pyc"
        }
        if actual != expected:
            errors.append(f"Installed skill file inventory changed: {skill['name']}")
        for item in skill["files"]:
            check_file(folder / item["path"], item["sha256"], errors)

    # A migrated note must be safely archived before its old location is removed.
    if data.get("note_migration", {}).get("status") == "moved":
        for item in data["documents"]:
            if item["kind"] == "discussion" and Path(item["original_path"]).exists():
                errors.append(f"Original note still present after recorded move: {item['original_path']}")

    if args.check_sources:
        for item in data["source_files"]:
            check_file(Path(item["absolute_path"]), item["sha256"], errors)
        for item in data["documents"]:
            if item["kind"] == "upstream":
                check_file(Path(item["original_path"]), item["sha256"], errors)

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    print(
        f"PASS: {len(data['documents'])} complete document imports, raw preservation, "
        f"reference heading/body fidelity, {len(data['skills'])} skill packages, and migration state."
    )
    if args.check_sources:
        print(f"PASS: {len(data['source_files'])} source anchors and upstream KB documents match captured hashes.")
    else:
        print("Live sibling source drift was not checked; use --check-sources when those checkouts are available.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
