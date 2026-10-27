"""Generate a disposable G38 consumer through the app-owned tool and run real SQLite guards."""
from pathlib import Path
import hashlib
import json
import os
import signal
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]


def supervised(argv, environment, log):
    """Use a parent wall-clock deadline; kill and reap the complete compiler/native process group."""
    with log.open("w") as output:
        child = subprocess.Popen(argv, cwd=ROOT, env=environment, stdout=output,
                                 stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = child.wait(timeout=600)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            raise AssertionError("G38 proof timed out; group killed and reaped")
        finally:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
    assert code == 0, f"G38 command failed ({code}); inspect {log}"


def hashes(directory):
    """Include the ownership manifest in complete freshness comparison."""
    return {str(path.relative_to(directory)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in directory.rglob("*") if path.is_file()}


def main():
    """Keep compilation in the caller's single target and preserve only explicit proof evidence."""
    environment = os.environ.copy()
    assert environment.get("CARGO_TARGET_DIR"), "supply the one capped target directory"
    assert environment.get("G37_PROOF_OUT"), "supply an owned evidence directory"
    evidence = Path(environment["G37_PROOF_OUT"])
    evidence.mkdir(parents=True, exist_ok=True)
    environment.update(CARGO_INCREMENTAL="0", CARGO_PROFILE_DEV_DEBUG="0", CARGO_NET_OFFLINE="true",
                       PYTHONDONTWRITEBYTECODE="1", G37_SNAPSHOTS=str(evidence / "ddl.json"))
    for gate in ["STACKPRESS_TEST_POSTGRES_URL", "STACKPRESS_TEST_MYSQL_URL"]:
        environment.pop(gate, None)
    with tempfile.TemporaryDirectory(prefix="g38-", dir=environment.get("TMPDIR")) as temporary:
        app = Path(temporary)
        generated = app / "src/generated"
        generated.parent.mkdir()
        source = app / "schema.idea"
        source.write_bytes((HERE / "schema.idea").read_bytes())
        command = ["cargo", "run", "-q", "-p", "generated-workspace-tool", "--offline", "--locked", "--",
                   "emit", "generate", "--input", str(source), "--output", str(generated)]
        commands = [command, command + ["--check", "true"]]
        supervised(command, environment, evidence / "generate.log")
        history = app / "history"
        revision = command[:command.index("emit") + 1] + ["revision", "--input", str(source), "--history", str(history), "--name", "one", "--mysql-collation", "utf8mb4_bin", "--mysql-page-bytes", "16384"]
        plan = command[:command.index("emit") + 1] + ["migration-plan", "--history", str(history), "--to", "one", "--dialect", "mysql"]
        commands.extend([revision, plan])
        supervised(revision, environment, evidence / "revision.log")
        supervised(plan, environment, evidence / "mysql-plan.log")
        plan_hash = json.loads((evidence / "mysql-plan.log").read_text())["results"]["plan"]
        (evidence / "revision-format3.json").write_bytes((history / "revisions/one/schema.json").read_bytes())
        (evidence / "mysql-plan-format3.json").write_bytes((history / "migrations" / plan_hash / "plan.json").read_bytes())
        before = hashes(generated)
        # SQL bounds are host-only; record snapshots/UI descriptors contain no storage member.
        for partition in ["contracts", "metadata"]:
            for path in (generated / partition).glob("*.rs"):
                text = path.read_text()
                assert "storage_length" not in text, path
                assert "7usize" not in text and "5usize" not in text, path
        supervised(commands[1], environment, evidence / "freshness.log")
        assert hashes(generated) == before, "check modified output"
        dependencies = {
            "stackpress-framework-schema": ("crates/framework/framework-schema", []),
            "stackpress-framework-sql": ("crates/framework/framework-sql", ["server"]),
            "stackpress-sql": ("crates/sql/sql", []),
            "stackpress-sql-sqlite": ("crates/sql/sql-sqlite", []),
        }
        manifest = '[package]\nname="g38-generated-proof"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
        for name, (path, features) in dependencies.items():
            manifest += f'{name} = {{ path = {json.dumps(str(ROOT / path))}, features = {json.dumps(features)} }}\n'
        manifest += 'serde = { version="1", features=["derive"] }\nserde_json="1"\ntempfile="3"\ntokio = { version="1", features=["rt","macros"] }\n'
        (app / "Cargo.toml").write_text(manifest)
        (app / "src/main.rs").write_bytes((HERE / "main.rs").read_bytes())
        run = ["cargo", "run", "--offline", "--manifest-path", str(app / "Cargo.toml")]
        commands.append(run)
        supervised(run, environment, evidence / "sqlite.log")
        clippy = ["cargo", "clippy", "--offline", "--manifest-path", str(app / "Cargo.toml"), "--", "-D", "warnings"]
        commands.append(clippy)
        supervised(clippy, environment, evidence / "clippy.log")
        expected = HERE.parents[1] / "fixtures/g37-ddl.json"
        if expected.exists():
            assert (evidence / "ddl.json").read_bytes() == expected.read_bytes(), "generated DDL snapshot is stale"
        (evidence / "generated-hashes.json").write_text(json.dumps(before, indent=2) + "\n")
        (evidence / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
    print(f"G38 generation, freshness and native SQLite guards passed; evidence: {evidence}")


if __name__ == "__main__":
    main()
