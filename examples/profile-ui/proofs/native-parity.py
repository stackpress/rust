#!/usr/bin/env python3
"""Finite real-adapter comparison; no root dependency edits, installs or listener."""
import os
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

root = Path(__file__).resolve().parents[3]
with tempfile.TemporaryDirectory(prefix="profile-ui-native-parity-") as directory:
    project = Path(directory)
    manifest = '''[package]
name = "profile-ui-native-parity"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
serde_json = "1"
http = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
'''
    for name, path in [("stackpress-example-profile-ui-native", "examples/profile-ui/native"), ("stackpress-example-profile-ui-contracts", "examples/profile-ui/contracts"), ("stackpress-example-generated-profile", "examples/generated-profile/app"), ("stackpress-lib", "crates/lib"), ("stackpress-server", "crates/server/server"), ("stackpress-cli", "tools/cli")]:
        manifest += f'{name} = {{ path = "{root / path}", default-features = false' + (', features = ["http"]' if name == "stackpress-server" else ', features = ["profiles"]' if name == "stackpress-example-generated-profile" else '') + ' }\n'
    (project / "Cargo.toml").write_text(manifest + '\n[[test]]\nname = "native_parity"\npath = "parity.rs"\n')
    shutil.copyfile(root / "Cargo.lock", project / "Cargo.lock")
    shutil.copyfile(Path(__file__).with_suffix(".rs"), project / "parity.rs")
    env = dict(os.environ)
    subprocess.run(["cargo", "generate-lockfile", "--offline", "--manifest-path", str(project / "Cargo.toml")], env=env, check=True)
    # Separate compilation from execution so a lifecycle deadlock kills/reaps the test itself.
    built = subprocess.run(["cargo", "test", "--offline", "--locked", "--manifest-path", str(project / "Cargo.toml"), "--no-run", "--message-format=json"], env=env, check=True, timeout=300, text=True, stdout=subprocess.PIPE)
    executables = [row["executable"] for line in built.stdout.splitlines() if (row := json.loads(line)).get("executable")]
    if len(executables) != 1:
        raise SystemExit("expected one real-adapter parity executable")
    subprocess.run([executables[0], "--nocapture"], env=env, check=True, timeout=10)
