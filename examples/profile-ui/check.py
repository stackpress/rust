#!/usr/bin/env python3
"""One finite locked check sequence; never installs tools or starts any listener."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
import tomllib
import asset_manifest

ROOT = Path(__file__).resolve().parents[2]
NATIVE = Path(os.environ.get("PROFILE_UI_NATIVE_TARGET", ROOT / "target/profile-ui-native")).resolve()
WASM = Path(os.environ.get("PROFILE_UI_TARGET", ROOT / "target/profile-ui-wasm")).resolve()
LOGS = Path(os.environ.get("PROFILE_UI_CHECK_LOGS", ROOT / "examples/profile-ui/dist/check-logs")).resolve()


def run(name, command, target=NATIVE, capture=False, overrides=None):
    """Bound each child, retain exact commands/results, and stop on its first failure."""
    LOGS.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), PROFILE_UI_TARGET=str(WASM))
    # The check's explicit fixture wins over operator proof environment variables.
    env.pop("PROFILE_UI_CONFIG", None)
    env.pop("PROFILE_UI_BASE", None)
    env.update(overrides or {})
    with (LOGS / f"{name}.log").open("wb") as log:
        log.write(("$ " + " ".join(map(str, command)) + "\n").encode()); log.flush()
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                stdout=subprocess.PIPE if capture else log,
                                stderr=log, timeout=600)
        if capture: log.write(result.stdout)
    if result.returncode:
        raise RuntimeError(f"{name} failed ({result.returncode}); see {LOGS / (name + '.log')}")
    print(f"PASS {name}", flush=True)
    return result.stdout.decode() if capture else ""


def preflight():
    """Validate installed tools and the actual locked bindgen version before building."""
    if NATIVE == WASM:
        raise ValueError("native and WASM target directories must differ")
    if subprocess.check_output(["wasm-bindgen", "--version"], text=True).strip() != "wasm-bindgen 0.2.129":
        raise ValueError("install owner-approved wasm-bindgen 0.2.129 explicitly before running checks")
    installed = subprocess.check_output(["rustup", "target", "list", "--installed"], text=True)
    if "wasm32-unknown-unknown" not in installed.splitlines():
        raise ValueError("install the owner-approved wasm32 target explicitly before running checks")
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    if {p["version"] for p in packages if p["name"] == "wasm-bindgen"} != {"0.2.129"}:
        raise ValueError("locked wasm-bindgen differs from the approved CLI")


def main():
    """Freshness → native SSR → separate client modes → manifests → feature/behavior checks."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sqlite", action="store_true", help="check the selected read-only SQLite backend")
    args = parser.parse_args()
    features = ["--features", "sqlite"] if args.sqlite else []
    # A failed rerun must not leave the previous run's successful machine receipt.
    LOGS.mkdir(parents=True, exist_ok=True)
    (LOGS / "summary.json").unlink(missing_ok=True)
    preflight()
    run("proof-wrapper-tests", [sys.executable, "examples/profile-ui/proofs/test_proofs.py"])
    run("manifest-tests", [sys.executable, "examples/profile-ui/proofs/test_assets.py"])
    run("fmt", ["cargo", "fmt", "--all", "--", "--check"])
    run("generated-freshness", ["cargo", "run", "--locked", "-p", "stackpress-example-profile-codegen", "--", "emit", "generate", "--check", "true"])
    run("native-ssr", ["cargo", "build", "--locked", "-p", "stackpress-example-profile-ui-web", "--bins", *features])
    for mode in ("hydrate", "csr"):
        run(mode, [sys.executable, "examples/profile-ui/build-client.py", mode], WASM)
        asset_manifest.check(mode)
    run("gateway-build", ["cargo", "build", "--locked", "-p", "stackpress-example-profile-ui-gateway", "--bins", *features])
    run("native-feature", ["cargo", "check", "--locked", "-p", "stackpress", "--features", "framework-ui-ssr"])
    for mode in ("hydrate", "csr"):
        run(f"facade-{mode}", ["cargo", "check", "--locked", "-p", "stackpress", "--target", "wasm32-unknown-unknown", "--features", f"framework-ui-{mode}"], WASM)
        # Cargo's explicit no-proc-macro edge filter separates host build tooling
        # (Leptos wasm_split_macros uses SHA-256) from the browser runtime graph.
        tree = run(f"runtime-{mode}", ["cargo", "tree", "--locked", "-p", "stackpress-example-profile-ui-client", "--target", "wasm32-unknown-unknown", "--features", mode, "--edges", "normal,no-proc-macro"], WASM, True)
        denied = ("stackpress-server", "stackpress-sql", "stackpress-cli", "stackpress-idea", "sqlx", "libsqlite3-sys", "sql-profile", "tauri v", "tokio v", "sha2 v", "profile-ui-gateway")
        if any(name in tree for name in denied): raise ValueError(f"native dependency entered {mode} runtime graph")
    run("existing-build-hash", ["cargo", "tree", "--locked", "-p", "stackpress-example-profile-ui-client", "--target", "wasm32-unknown-unknown", "--features", "hydrate", "--invert", "sha2@0.10.9", "--edges", "normal,build"], WASM, True)
    default_tree = run("default-facade", ["cargo", "tree", "--locked", "-p", "stackpress", "--edges", "normal,no-proc-macro"], capture=True)
    if any(name in default_tree for name in ("sha2 v", "hyper v", "profile-ui-gateway")): raise ValueError("gateway dependency entered default facade")
    native_tree = run("gateway-runtime", ["cargo", "tree", "--locked", "-p", "stackpress-example-profile-ui-gateway", *features, "--edges", "normal,no-proc-macro"], capture=True)
    if "sha2 v0.10.9" not in native_tree: raise ValueError("gateway SHA-256 runtime edge missing")
    for edge in ("stackpress-example-sql-profile", "stackpress-sql-sqlite", "sqlx v", "libsqlite3-sys"):
        if (edge in native_tree) != args.sqlite:
            raise ValueError(f"unexpected SQLite feature edge: {edge}")
    scopes = ["-p", "stackpress-example-profile-ui-web", "-p", "stackpress-example-profile-ui-gateway"]
    run("tests", ["cargo", "test", "--locked", *scopes, *features])
    customization = "demo-customization,sqlite" if args.sqlite else "demo-customization"
    run("customized-reuse", ["cargo", "test", "--locked", "-p", "stackpress-example-profile-ui-web", "--features", customization, "--test", "reuse"])
    # Integration tests also rebuild package binaries with their selected feature.
    # Restore the selected backend without customization before the finite service proof.
    run("native-ssr-final", ["cargo", "build", "--locked", "-p", "stackpress-example-profile-ui-web", "--bins", *features])
    run("doctests", ["cargo", "test", "--locked", *scopes, *features, "--doc"])
    run("clippy", ["cargo", "clippy", "--locked", *scopes, *features, "--all-targets", "--", "-D", "warnings"])
    # Finite probe calls the real gateway service and exits; it cannot bind a socket.
    probe = NATIVE / "debug/probe"
    with tempfile.TemporaryDirectory(prefix="profile-ui-check-") as directory:
        overrides = {}
        if args.sqlite:
            run("setup-build", ["cargo", "build", "--locked", "-p", "stackpress-example-sql-profile", "--bin", "sql-profile-setup"])
            database = Path(directory) / "profiles.sqlite"
            run("explicit-setup", [NATIVE / "debug/sql-profile-setup", database])
            config = Path(directory) / "config.json"
            config.write_text(json.dumps({"sqlite": {"path": str(database)}}))
            overrides["PROFILE_UI_CONFIG"] = str(config)
        response = run("service-proof", [probe, "GET", "/ui/profiles?id=abc"], capture=True, overrides=overrides)
    meta, html = response.split("\n", 1)
    if json.loads(meta)["status"] != 200 or "Ada Lovelace" not in html: raise ValueError("gateway service proof failed")
    (LOGS / "summary.json").write_text(json.dumps({"backend": "sqlite" if args.sqlite else "memory", "native_target": str(NATIVE), "wasm_target": str(WASM), "build": asset_manifest.contract()[1], "listener_started": False, "result": "passed"}, indent=2) + "\n")
    print(f"All checks passed; logs: {LOGS}")


if __name__ == "__main__":
    try: main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        raise SystemExit(str(error)) from error
