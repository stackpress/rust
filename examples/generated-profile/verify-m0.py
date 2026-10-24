#!/usr/bin/env python3
"""Prove clean bootstrap, real consumer compilation, selection failures and model removal.

Only disposable copies under the physical temporary directory are mutated. Cargo dependencies
point at the current checkout. The original app/schema/generated output is never removed.
"""

from pathlib import Path
import json
import os
import shutil
import subprocess
import tempfile


def run(args, *, success=True, env=None):
    """Bound every native compile/CLI command and retain failures for independent review."""
    completed = subprocess.run(args, capture_output=True, text=True, timeout=180, env=env)
    assert (completed.returncode == 0) == success, (args, completed.stdout, completed.stderr)
    return completed


def main():
    """Exercise the M0 contracts using actual Cargo consumers and the app-owned codegen CLI."""
    repository = Path(__file__).resolve().parents[2]
    example = repository / "examples/generated-profile"
    proof = Path(tempfile.mkdtemp(prefix="stackpress-slice3-m0-", dir="/private/tmp"))
    app = proof / "app"
    app.mkdir()
    # Copy only handwritten application inputs; bootstrap must work without generated modules.
    shutil.copytree(example / "app/src", app / "src", ignore=shutil.ignore_patterns("generated"))
    shutil.copy(example / "app/schema.idea", app / "schema.idea")
    shutil.copy(example / "selection.rs", proof / "selection.rs")
    manifest = (example / "app/Cargo.toml").read_text()
    for field, value in [("version", '"0.1.0"'), ("edition", '"2024"'), ("rust-version", '"1.96"')]:
        manifest = manifest.replace(f"\n{field}.workspace = true", f"\n{field} = {value}")
    manifest = manifest.replace('[lints]\nworkspace = true', '[workspace]')
    manifest = manifest.replace('../../../crates/', f'{repository}/crates/')
    manifest = manifest.replace('../../../tools/', f'{repository}/tools/')
    (app / "Cargo.toml").write_text(manifest)
    env = {**os.environ, "CARGO_TARGET_DIR": str(repository / "target/slice-3-proof")}
    tool = ["cargo", "run", "--locked", "-p", "stackpress-example-profile-codegen", "--",
            "emit", "generate", "--input", str(app / "schema.idea"),
            "--output", str(app / "src/generated")]
    # Build the independently selected generator while the consumer cannot yet compile.
    assert not (app / "src/generated").exists()
    run(tool, env=env)
    generated = app / "src/generated"
    first = {str(path.relative_to(generated)): path.read_bytes()
             for path in generated.rglob("*") if path.is_file()}
    run(tool, env=env)
    assert first == {str(path.relative_to(generated)): path.read_bytes()
                     for path in generated.rglob("*") if path.is_file()}
    run(tool + ["--check", "true"], env=env)
    check = ["cargo", "check", "--offline", "--manifest-path", str(app / "Cargo.toml")]
    run(check, env=env)
    check.append("--locked")
    # App-only selection mismatch must fail in generated const assertions.
    mismatch = run(check + ["--no-default-features"], success=False, env=env)
    assert "assertion failed" in mismatch.stderr
    selection = proof / "selection.rs"
    original = selection.read_text()
    selection.write_text(original.replace("REVISION: u32 = 1", "REVISION: u32 = 2"))
    mismatch = run(check, success=False, env=env)
    assert "assertion failed" in mismatch.stderr
    selection.write_text(original)
    run(check, env=env)
    # A standalone data consumer has only Serde as a normal dependency.
    contracts = proof / "contracts"
    (contracts / "src").mkdir(parents=True)
    (contracts / "Cargo.toml").write_text('''[package]
name = "profile-contract-consumer"
version = "0.0.0"
edition = "2024"
[dependencies]
serde = { version = "1", features = ["derive"] }
[workspace]
''')
    shutil.copy(generated / "contracts/profile.rs", contracts / "src/lib.rs")
    run(["cargo", "check", "--offline", "--manifest-path", str(contracts / "Cargo.toml")], env=env)
    tree = run(["cargo", "tree", "--offline", "--locked", "--edges", "normal",
                "--manifest-path", str(contracts / "Cargo.toml")], env=env).stdout
    assert all(name not in tree for name in ["stackpress-server", "idea-parser", "idea-transformer"])
    # Removing the model deletes its files and yields a compiling no-op registry and actual 404.
    (app / "schema.idea").write_text('plugin "profile-demo" { }\n')
    run(tool, env=env)
    assert not (generated / "contracts/profile.rs").exists()
    assert not (generated / "server/profile.rs").exists()
    run(check, env=env)
    missing = run(["cargo", "run", "--offline", "--locked", "--manifest-path",
                   str(app / "Cargo.toml"), "--bin", "profile-app", "--",
                   "emit", "profile-search", "--id", "abc123"], success=False, env=env)
    assert missing.returncode == 1
    assert json.loads(missing.stdout)["code"] == 404
    print(json.dumps({"proof": str(proof), "repeat_bytes": "equal",
                      "selection_mismatch": "compile failure", "revision_mismatch": "compile failure",
                      "contracts": "serde-only compile", "removed_model": "compiled + CLI 404"}))


if __name__ == "__main__":
    main()
