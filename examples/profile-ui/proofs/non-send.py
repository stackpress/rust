#!/usr/bin/env python3
"""Compile the real host registration against a deliberately non-Send page."""
from pathlib import Path
import os
import subprocess
import tempfile

root = Path(__file__).resolve().parents[3]
with tempfile.TemporaryDirectory(prefix="profile-ui-non-send-") as directory:
    crate = Path(directory)
    (crate / "src").mkdir()
    (crate / "src/main.rs").write_text(Path(__file__).with_name("non-send.rs").read_text())
    (crate / "Cargo.toml").write_text(f'''[package]
name = "profile-ui-non-send-proof"
version = "0.0.0"
edition = "2024"
rust-version = "1.96"
[workspace]
[dependencies]
stackpress-lib = {{ path = "{root / 'crates/lib'}" }}
stackpress-server = {{ path = "{root / 'crates/server'}" }}
tokio = {{ version = "1", features = ["rt"] }}
''')
    result = subprocess.run(["cargo", "check", "--offline"], cwd=crate, env=dict(os.environ, CARGO_TARGET_DIR="/private/tmp/stackpress-s5-m0-non-send"), text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=120)
    Path("/private/tmp/stackpress-s5-m0-non-send.log").write_text(result.stdout)
    if result.returncode == 0 or "future cannot be sent between threads safely" not in result.stdout or "Rc<std::string::String>" not in result.stdout:
        raise SystemExit(result.stdout)
    print("PASS: real ServerBuilder rejects Rc<String> across await")
