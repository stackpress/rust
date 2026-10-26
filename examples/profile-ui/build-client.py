#!/usr/bin/env python3
"""Finite pinned Cargo/wasm-bindgen asset build; never installs or starts a listener."""
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
mode = sys.argv[1] if len(sys.argv) == 2 else "hydrate"
if mode not in ("hydrate", "csr"):
    raise SystemExit("choose hydrate or csr")
version = subprocess.check_output(["wasm-bindgen", "--version"], text=True).strip()
if version != "wasm-bindgen 0.2.129":
    raise SystemExit("requires wasm-bindgen 0.2.129; no implicit install")
target = Path(os.environ.get("PROFILE_UI_TARGET", root / "target" / "profile-ui-wasm"))
environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
subprocess.run(["cargo", "build", "--locked", "-p", "stackpress-example-profile-ui-client", "--target", "wasm32-unknown-unknown", "--features", mode], cwd=root, env=environment, check=True)
output = root / "examples" / "profile-ui" / "dist" / mode
output.mkdir(parents=True, exist_ok=True)
subprocess.run(["wasm-bindgen", "--target", "web", "--out-dir", str(output), "--out-name", "profile_ui", str(target / "wasm32-unknown-unknown/debug/stackpress_example_profile_ui_client.wasm")], check=True)
(output / "start.js").write_text("import init, { start } from './profile_ui.js';\nawait init();\nwindow.disposeProfileUi = start();\nwindow.profileUiReady = true;\n")
if mode == "csr":
    (output / "index.html").write_text('<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><!--HEAD--></head><body><div id="app"></div><script type="module" src="start.js"></script></body></html>')
print(output)
