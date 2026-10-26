#!/usr/bin/env python3
"""Run every finite browser proof at both mounts against the selected built backend."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]


def run(command, env, log):
    """Retain exact child output and enforce a deadline without any listener."""
    with log.open("w") as output:
        output.write("$ " + " ".join(map(str, command)) + "\n")
        output.flush()
        subprocess.run(list(map(str, command)), cwd=ROOT, env=env, stdout=output,
                       stderr=subprocess.STDOUT, timeout=120, check=True)
    print(f"PASS {log.stem}", flush=True)


def startup_failures(native, database, env, log):
    """Both finite executable boundaries explain explicit setup and leave the file untouched."""
    commands = ([native / "debug/stackpress-example-profile-ui-web"],
                [native / "debug/probe", "GET", "/ui/profiles?id=abc"])
    with log.open("wb") as output:
        for file_exists in (False, True):
            # An empty file is deliberately uninstalled; neither host may initialize it.
            if file_exists:
                database.write_bytes(b"")
            for command in commands:
                result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                        capture_output=True, timeout=10)
                output.write(result.stderr)
                if result.returncode == 0 or b"sql-profile-setup" not in result.stderr:
                    raise RuntimeError("missing/uninstalled database must fail with the setup remedy")
                if database.exists() != file_exists or (file_exists and database.read_bytes()):
                    raise RuntimeError("startup changed the missing/uninstalled database")
    print("PASS startup-errors-without-setup", flush=True)


def main():
    """Use only explicit temporary setup, then intercept all real Chromium traffic."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sqlite", action="store_true", help="use already built SQLite feature binaries")
    args = parser.parse_args()
    native = Path(os.environ.get("PROFILE_UI_NATIVE_TARGET", ROOT / "target/profile-ui-native")).resolve()
    logs = Path(os.environ.get("PROFILE_UI_BROWSER_LOGS", ROOT / "examples/profile-ui/dist/browser-logs")).resolve()
    logs.mkdir(parents=True, exist_ok=True)
    # A failed run must not retain a previous successful receipt.
    (logs / "summary.json").unlink(missing_ok=True)
    env = dict(os.environ, PROFILE_UI_HOST=str(native / "debug/stackpress-example-profile-ui-web"),
               PROFILE_UI_GATEWAY_PROBE=str(native / "debug/probe"))
    env.pop("PROFILE_UI_CONFIG", None)
    env.pop("PROFILE_UI_DISPLAY", None)
    env.pop("PROFILE_UI_CUSTOMIZED", None)
    with tempfile.TemporaryDirectory(prefix="profile-ui-browser-") as directory:
        if args.sqlite:
            database = Path(directory) / "profiles.sqlite"
            config = Path(directory) / "config.json"
            config.write_text(json.dumps({"sqlite": {"path": str(database)}}))
            env["PROFILE_UI_CONFIG"] = str(config)
            # Verify executable startup before performing the one explicit installation.
            startup_failures(native, database, env, logs / "startup-errors.log")
            run([native / "debug/sql-profile-setup", database], env, logs / "explicit-setup.log")
        for base, mount in (("", "root"), ("/demo/v1", "mounted")):
            for script in ("browser", "browser-m2", "browser-navigation", "browser-gateway"):
                run(["node", f"examples/profile-ui/proofs/{script}.cjs"],
                    dict(env, PROFILE_UI_BASE=base, PROFILE_UI_SCREENSHOT_PREFIX=str(logs / mount)),
                    logs / f"{script}-{mount}.log")
    (logs / "summary.json").write_text(json.dumps({"backend": "sqlite" if args.sqlite else "memory",
        "proofs": 8, "mounts": ["", "/demo/v1"], "listener_started": False, "result": "passed"}, indent=2) + "\n")


if __name__ == "__main__":
    main()
