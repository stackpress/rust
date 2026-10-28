#!/usr/bin/env python3
"""Finite pinned Cargo/wasm-bindgen asset build; never installs or starts a listener."""
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
import asset_manifest
import tomllib

root = Path(__file__).resolve().parents[2]
if len(sys.argv) != 2:
    raise SystemExit("usage: build-client.py <hydrate|csr>")
mode = sys.argv[1]
if mode not in ("hydrate", "csr"):
    raise SystemExit("choose hydrate or csr")
version = subprocess.check_output(["wasm-bindgen", "--version"], text=True).strip()
if version != "wasm-bindgen 0.2.129":
    raise SystemExit("requires wasm-bindgen 0.2.129; no implicit install")
packages = tomllib.loads((root / "Cargo.lock").read_text())["package"]
if {package["version"] for package in packages if package["name"] == "wasm-bindgen"} != {"0.2.129"}:
    raise SystemExit("locked wasm-bindgen differs from approved CLI 0.2.129")
installed = subprocess.check_output(["rustup", "target", "list", "--installed"], text=True)
if "wasm32-unknown-unknown" not in installed.splitlines():
    raise SystemExit("requires an explicitly installed wasm32-unknown-unknown target; no implicit install")
target = Path(os.environ.get("PROFILE_UI_TARGET", root / "target" / "profile-ui-wasm"))
environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
subprocess.run(["cargo", "build", "--locked", "-p", "stackpress-example-profile-ui-client", "--target", "wasm32-unknown-unknown", "--features", mode], cwd=root, env=environment, check=True)
output = root / "examples" / "profile-ui" / "dist" / mode
# Delete only our known generated outputs; stale chunks must not survive a rebuild.
import shutil
if output.exists(): shutil.rmtree(output)
output.mkdir(parents=True)
subprocess.run(["wasm-bindgen", "--target", "web", "--no-typescript", "--out-dir", str(output), "--out-name", "profile_ui", str(target / "wasm32-unknown-unknown/debug/stackpress_example_profile_ui_client.wasm")], check=True)
(output / "start.js").write_text((root / "examples/profile-ui/bootstrap.js").read_text())
if mode == "csr":
    (output / "index.html").write_text('<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><!--HEAD--></head><body><div id="app"></div><script type="module" src="start.js"></script></body></html>')
# Include the same immutable first-paint asset in the checked web/native inventory.
import re
source = root / 'crates/framework/framework-view/src'
filename = re.search(r'BOOTSTRAP_FILE: &str = "([^"]+)"', (source / 'theme.rs').read_text())[1]
shutil.copyfile(source / 'initial-theme.js', output / filename)
asset_manifest.write(mode)
print(output)
