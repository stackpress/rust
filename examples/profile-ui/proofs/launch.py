#!/usr/bin/env python3
"""Launch a packaged or instrumented app; collect its proof and kill/reap on timeout."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

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
    if process.returncode != 0 or sorted(report["events"]) != ["denied-ok", "main-ok"] or report["in_flight"] != 0:
        raise SystemExit(json.dumps(report) + "\n" + output)
    print(json.dumps(report))
