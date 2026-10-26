#!/usr/bin/env python3
"""Launch a packaged or instrumented app; collect its proof and kill/reap on timeout."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from urllib.parse import parse_qs

bundle = Path(sys.argv[1]).resolve()
executable = bundle if bundle.is_file() else bundle / "Contents/MacOS/profile-ui"
with tempfile.TemporaryDirectory(prefix="profile-ui-launch-") as directory:
    receipt = Path(directory) / "launch.json"
    process = subprocess.Popen([str(executable)], env=dict(os.environ, PROFILE_UI_PROOF=str(receipt)), stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        output, _ = process.communicate(timeout=25)
    except subprocess.TimeoutExpired:
        process.kill()
        output, _ = process.communicate()
        raise SystemExit("packaged proof timed out; killed/reaped app\n" + output)
    if not receipt.exists():
        raise SystemExit(f"packaged proof produced no receipt (exit {process.returncode})\n" + output)
    report = json.loads(receipt.read_text())
    Path(os.environ.get("PROFILE_UI_LAUNCH_REPORT", "/private/tmp/stackpress-s5-m0-launch-receipt.json")).write_text(json.dumps(report, indent=2))
    mode = os.environ.get("PROFILE_UI_PROOF_MODE", "navigation")
    expected = ["denied-ok", "main-ok"] if mode == "navigation" else [mode + "-ok"]
    audit = parse_qs(report.get("details") or "")
    valid_audit = audit.get("non_ipc_fetches") == ["0"] and audit.get("csp_image") == ["blocked"] and audit.get("csp_connect") == ["blocked"]
    expected_entries = 2 if mode == "navigation" else 1
    if report.get("customization") != (os.environ.get("PROFILE_UI_CUSTOMIZATION") == "1"):
        raise SystemExit("packaged customization feature does not match the requested proof\n" + json.dumps(report))
    if process.returncode != 0 or sorted(report["events"]) != expected or report["in_flight"] != 0 or not report.get("shutdown_ok") or report.get("entries") != expected_entries or report.get("drops") != expected_entries or not valid_audit:
        raise SystemExit(json.dumps(report) + "\n" + output)
    print(json.dumps(report))
