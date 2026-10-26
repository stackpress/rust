#!/usr/bin/env python3
"""Compile the real host registration against a deliberately non-Send page."""
from pathlib import Path
import os
import subprocess
import tempfile
import sys

root = Path(__file__).resolve().parents[3]
page = len(sys.argv) > 1 and sys.argv[1] == "page"
target = os.environ.get("PROFILE_UI_TARGET", str(root / "target" / "profile-ui-native"))
with tempfile.TemporaryDirectory(prefix="profile-ui-non-send-") as directory:
    crate = Path(directory)
    (crate / "src").mkdir()
    (crate / "src/main.rs").write_text(Path(__file__).with_name("non-send-page.rs" if page else "non-send.rs").read_text())
    (crate / "Cargo.toml").write_text(f'''[package]
name = "profile-ui-non-send-proof"
version = "0.0.0"
edition = "2024"
rust-version = "1.96"
[workspace]
[dependencies]
stackpress-lib = {{ path = "{root / 'crates/lib'}" }}
stackpress-server = {{ path = "{root / 'crates/server'}" }}
stackpress-framework-view = {{ path = "{root / 'crates/framework/framework-view'}", features = ["server"] }}
tokio = {{ version = "1", features = ["rt"] }}
''')
    result = subprocess.run(["cargo", "check", "--offline"], cwd=crate, env=dict(os.environ, CARGO_TARGET_DIR=target), text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=120)
    Path(os.environ.get("PROFILE_UI_PROOF_LOG", "/private/tmp/profile-ui-non-send.log")).write_text(result.stdout)
    if result.returncode == 0 or "future cannot be sent between threads safely" not in result.stdout or "Rc<std::string::String>" not in result.stdout:
        raise SystemExit(result.stdout)
    print(f"PASS: real {'PageSpec' if page else 'ServerBuilder'} rejects Rc<String> across await")
