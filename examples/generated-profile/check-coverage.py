#!/usr/bin/env python3
"""Enforce M0's 70% line floor independently for each newly introduced Cargo package."""

import json
from pathlib import Path
import sys


def main():
    """Aggregate only the declared production scope from an explicitly filtered llvm-cov report."""
    report = json.loads(Path(sys.argv[1]).read_text())
    scopes = [
        "crates/idea/idea-parser/", "crates/idea/idea-transformer/", "crates/idea/idea/",
        "examples/generated-profile/app/", "examples/generated-profile/codegen/",
    ]
    for scope in scopes:
        files = [entry for entry in report["data"][0]["files"] if f"/{scope}" in entry["filename"]]
        count = sum(entry["summary"]["lines"]["count"] for entry in files)
        covered = sum(entry["summary"]["lines"]["covered"] for entry in files)
        assert count > 0, f"{scope}: unmeasured implementation"
        percent = 100 * covered / count
        print(f"{scope}: {covered}/{count} ({percent:.2f}%)")
        assert percent >= 70, f"{scope}: below the per-crate development floor"


if __name__ == "__main__":
    main()
