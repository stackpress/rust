#!/usr/bin/env python3
"""Verify package-local test copies against their canonical repository bytes.

The manifests pin source hashes as well as copy hashes. Helpers may prepend exactly
the recorded provenance line; fixture data must remain completely byte-identical.
An intentional helper divergence must record a reason and its own copy_sha256.
"""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def digest(data):
    """Hash bytes without newline, encoding, or JSON normalization."""
    return hashlib.sha256(data).hexdigest()


def check(root, manifest):
    """Return every stale source/copy instead of stopping at the first mismatch."""
    errors = []
    destinations = set()
    for item in manifest["copies"]:
        destination = (root / item["copy"]).resolve()
        if destination in destinations:
            errors.append(f'{item["copy"]}: duplicate copy destination')
        destinations.add(destination)
        source, copy = root / item["source"], root / item["copy"]
        try:
            canonical, actual = source.read_bytes(), copy.read_bytes()
        except OSError as error:
            errors.append(str(error))
            continue
        if digest(canonical) != item["sha256"]:
            errors.append(f'{item["source"]}: canonical source changed; refresh its copies')
        if item.get("divergence_reason", "").strip():
            if digest(actual) != item.get("copy_sha256"):
                errors.append(f'{item["copy"]}: intentional divergence hash changed')
            continue
        prefix = (item["provenance"] + "\n").encode() if "provenance" in item else b""
        if actual != prefix + canonical:
            errors.append(f'{item["copy"]}: differs from {item["source"]}')
    return errors


def main():
    """Keep the manifest reviewable and verification read-only."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, action="append", help="explicit manifest(s); default: every tools/release/test-copies*.json")
    args = parser.parse_args()
    paths = args.manifest or sorted((ROOT / "tools/release").glob("test-copies*.json"))
    if not paths:
        parser.error("no test-copy manifests found")
    manifest = {"copies": [item for path in paths for item in json.loads(path.read_text())["copies"]]}
    errors = check(ROOT, manifest)
    for error in errors:
        print(error)
    print(f'Test copies: {len(manifest["copies"])} checked in {len(paths)} manifests; {len(errors)} errors')
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
