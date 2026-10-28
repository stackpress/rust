#!/usr/bin/env python3
"""Record development commands and Cargo build stages outside disposable targets."""

import argparse
from collections import Counter
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import time
import uuid


# Cargo reports elapsed build-stage wall time, including linking/cache checks.
# Doctest/fixture compilation after this line remains in command time only.
FINISHED = re.compile(r"Finished `([^`]+)` profile .*? in (?:(\d+)m\s*)?(\d+(?:\.\d+)?)s\s*$")
ACTIVITY = re.compile(r"^\s*(Compiling|Checking|Documenting)\s+")
KINDS = ("build", "test", "check", "clippy", "doc", "coverage", "clean", "other")


def probe(*command):
    """Read optional metadata without failing a benchmark or timing the probe."""
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=5)
        return result.stdout.strip() if result.returncode == 0 else None
    except (OSError, subprocess.TimeoutExpired):
        return None


def default_store():
    """Share records among Git worktrees using the main checkout's resource path."""
    common = probe("git", "rev-parse", "--path-format=absolute", "--git-common-dir")
    root = Path(common).parent if common else Path.cwd()
    return root / ".agents/resources/build-benchmarks"


def save(path, record):
    """Atomically replace one invocation's record; parallel runs own unique files."""
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(record, indent=2) + "\n")
    temporary.replace(path)


def observe(line, record):
    """Extract explicit Cargo evidence without mistaking cache checks for rebuilds."""
    activity = ACTIVITY.match(line)
    if activity:
        key = {"Compiling": "compiling", "Checking": "checking", "Documenting": "documenting"}[activity[1]]
        record["cargo_activity"][key] += 1
    finished = FINISHED.search(line)
    if finished:
        record["cargo_build_stages"].append({
            "profile": finished[1],
            "seconds": int(finished[2] or 0) * 60 + float(finished[3]),
        })


def run(args, store):
    """Stream output and preserve success, failure, interruption, or launch failure."""
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        raise ValueError("run requires a command after --")
    started = datetime.now(timezone.utc)
    run_id = started.strftime("%Y%m%dT%H%M%S.%fZ-") + uuid.uuid4().hex[:12]
    folder = store / "runs"
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / (run_id + ".json")
    record = {
        "schema_version": 1, "id": run_id, "started_at": started.isoformat(),
        "status": "running", "label": args.label, "kind": args.kind,
        "scope": args.scope, "cache": args.cache,
        "command": command, "cwd": str(Path.cwd()),
        "commit": probe("git", "rev-parse", "HEAD"),
        "branch": probe("git", "branch", "--show-current"),
        "dirty": bool(probe("git", "status", "--porcelain")),
        "host": platform.platform(), "machine": platform.machine(),
        "cargo_version": probe("cargo", "--version"),
        "rustc_version": probe("rustc", "--version"),
        "environment": {key: os.environ[key] for key in (
            "CARGO_TARGET_DIR", "CARGO_LLVM_COV_TARGET_DIR", "CARGO_INCREMENTAL",
            "CARGO_PROFILE_DEV_DEBUG", "CARGO_BUILD_JOBS", "RUSTUP_TOOLCHAIN",
        ) if key in os.environ},
        "cargo_activity": {"compiling": 0, "checking": 0, "documenting": 0},
        "cargo_build_stages": [], "log": "runs/" + run_id + ".log",
    }
    # Persist the start first, so an external hard kill leaves a visible attempt.
    save(path, record)
    child = None
    code = 127
    interrupted_signal = signal.SIGINT

    def interrupt(signum, _frame):
        """Handle termination like Ctrl-C so a timeout does not orphan Cargo."""
        nonlocal interrupted_signal
        interrupted_signal = signum
        raise KeyboardInterrupt

    clock_start = time.monotonic()
    with path.with_suffix(".log").open("w") as log:
        previous_term = signal.signal(signal.SIGTERM, interrupt)
        try:
            child = subprocess.Popen(
                command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                text=True, errors="replace", start_new_session=True,
                env={**os.environ, "CARGO_TERM_COLOR": "never"},
            )
            for line in child.stdout:
                log.write(line)
                log.flush()
                sys.stdout.write(line)
                sys.stdout.flush()
                observe(line, record)
            code = child.wait()
            record["status"] = "passed" if code == 0 else "failed"
        except KeyboardInterrupt:
            # Bound shutdown of the whole command tree, including nested fixtures.
            if child is not None:
                try:
                    os.killpg(child.pid, interrupted_signal)
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                except ProcessLookupError:
                    child.wait()
            code = 128 + interrupted_signal
            record["status"] = "interrupted"
        except OSError as error:
            record["status"] = "launch_failed" if child is None else "recorder_failed"
            record["error"] = str(error)
            # Never leave an unobserved command running after a recorder IO failure.
            if child is not None and child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
        finally:
            signal.signal(signal.SIGTERM, previous_term)
            if child is not None and child.stdout is not None:
                child.stdout.close()
    record.update(
        ended_at=datetime.now(timezone.utc).isoformat(),
        command_seconds=round(time.monotonic() - clock_start, 6), exit_code=code,
        cargo_build_seconds=(sum(stage["seconds"] for stage in record["cargo_build_stages"])
                             if record["cargo_build_stages"] else None),
    )
    save(path, record)
    print(f"Benchmark: {path}", file=sys.stderr)
    return code if code >= 0 else 128 - code


def summary(store, as_json):
    """Derive counters from records, avoiding a shared mutable counter or log lock."""
    records = []
    for path in sorted((store / "runs").glob("*.json")):
        # Corrupt evidence must fail visibly instead of silently reducing counts.
        record = json.loads(path.read_text())
        if record["schema_version"] != 1:
            raise ValueError(f"Unsupported benchmark schema: {path}")
        records.append(record)
    stages = [stage for row in records for stage in row["cargo_build_stages"]]
    counts = {
        "store": str(store), "attempts": len(records),
        "by_status": dict(Counter(row["status"] for row in records)),
        "by_kind": dict(Counter(row["kind"] for row in records)),
        "by_scope": dict(Counter(row["scope"] for row in records)),
        "by_cache": dict(Counter(row["cache"] for row in records)),
        "attempts_with_observed_compilation": sum(row["cargo_activity"]["compiling"] > 0 for row in records),
        "successful_declared_cold_workspace_builds": sum(
            row["status"] == "passed" and row["cache"] == "cold"
            and row["scope"] == "workspace" and row["kind"] in ("build", "test", "coverage")
            and row["cargo_activity"]["compiling"] > 0 for row in records
        ),
        "command_seconds_total": round(sum(row.get("command_seconds", 0) for row in records), 6),
        "cargo_build_seconds_total": round(sum(stage["seconds"] for stage in stages), 6),
        "cargo_build_stages": len(stages),
    }
    if as_json:
        print(json.dumps({"summary": counts, "runs": records}, indent=2))
    else:
        for key, value in counts.items():
            print(f"{key}: {value}")
        print("\nid | status | kind | scope | cache | command seconds | Cargo build seconds | label")
        for row in records:
            print(" | ".join(str(row.get(key, "pending")) for key in (
                "id", "status", "kind", "scope", "cache", "command_seconds", "cargo_build_seconds", "label")))
    return 0


def main():
    """Expose run/summary commands without executing a shell or cleaning targets."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--store", type=Path, help="Override the shared resource directory")
    subparsers = parser.add_subparsers(dest="action", required=True)
    runner = subparsers.add_parser("run", help="Record and execute one command")
    runner.add_argument("--label", required=True)
    runner.add_argument("--kind", choices=KINDS, required=True)
    runner.add_argument("--scope", choices=("workspace", "selected", "consumer", "other"), required=True)
    runner.add_argument("--cache", choices=("cold", "warm", "unknown"), default="unknown")
    runner.add_argument("command", nargs=argparse.REMAINDER)
    reporter = subparsers.add_parser("summary", help="Print counters and individual durations")
    reporter.add_argument("--json", action="store_true")
    args = parser.parse_args()
    store = (args.store or default_store()).resolve()
    try:
        return run(args, store) if args.action == "run" else summary(store, args.json)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(2, f"Benchmark error: {error}\n")


if __name__ == "__main__":
    sys.exit(main())
