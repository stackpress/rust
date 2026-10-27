"""Run the README verbatim, including absent-output bootstrap, under an external watchdog."""

from pathlib import Path
import hashlib
import json
import os
import re
import shutil
import signal
import shlex
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent


def hashes(root):
    """Include the ownership manifest and every emitted file in byte-stability checks."""
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(root.rglob("*")) if p.is_file()}


def supervised(script, environment, log, timeout=600):
    """Bound compilation plus the complete native sequence; kill and reap the process group."""
    with log.open("w") as output:
        child = subprocess.Popen(["bash", "-eu", "-o", "pipefail", "-c", script],
                                 cwd=ROOT, env=environment, stdout=output,
                                 stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            raise AssertionError("README proof timed out; child group killed and reaped")
        finally:
            # Also reap descendants if the supervising interpreter is interrupted.
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
    assert code == 0, f"README command failed ({code}); inspect {log}"


def result(directory, name):
    """Decode only an acknowledged success envelope from the real CLI."""
    envelope = json.loads((directory / f"{name}.json").read_text())
    assert envelope["code"] == 200, envelope
    return envelope["results"]


def verify(directory):
    """Prove row preservation, relation/page semantics and the applied head after refusal."""
    reads = {name: json.loads((directory / f"{name}.json").read_text())
             for name in ["read-001", "read-002", "read-reversed", "read-003", "read-after-refusal"]}
    for name, report in reads.items():
        assert report["profile"] == {"id": "u1", "name": "Ada"}, report
        page = report["projects"]
        assert (page["total"], page["skip"], page["take"]) == (2, 1, 1), page
        assert len(page["rows"]) == 1 and page["rows"][0]["row"]["id"] == "p2", page
        project = page["rows"][0]
        assert project["owner"]["state"] == "loaded", project
        assert project["owner"]["value"]["row"]["name"] == "Ada", project
        tasks = project["tasks"]["value"]
        assert project["tasks"]["state"] == "loaded" and len(tasks) == 2, tasks
        assert {t["row"]["id"] for t in tasks} == {"t2", "t3"}, tasks
        assert all(t["row"]["done"] is False for t in tasks), tasks
        assert report["task_count"] == 3, report
        assert len(report["statements"]) == (7 if "link_count" in report else 6), report
        assert all(q["sql"].startswith("SELECT ") for q in report["statements"]), report
        if name not in ("read-001", "read-reversed"):
            assert project["row"]["heading"] == "Ship UI" and project["row"]["summary"] is None
    assert reads["read-001"] == reads["read-reversed"], "reverse changed rows or query SQL"
    assert reads["read-003"] == reads["read-after-refusal"], "unapproved loss changed rows"
    assert hashlib.sha256((directory / "before-loss.sqlite").read_bytes()).digest() == hashlib.sha256(
        (directory / "data.sqlite").read_bytes()).digest(), "refusal changed complete database bytes"
    assert reads["read-003"]["link_count"] == 0
    unmapped = result(directory, "unmapped")
    assert unmapped["outcome"] == "NeedsExplicitMapping" and "plan" not in unmapped
    assert {"table":"project","from":"title","to":"heading"} in unmapped["suggestions"]
    assert result(directory, "up")["reverse_available"] is True
    assert result(directory, "model")["reverse_available"] is False
    loss = result(directory, "lossy")
    assert loss["reverse_available"] is False
    assert any(f["kind"] == "drop_column" and f["table"] == "task" and f["field"] == "done"
               and f["destructive"] for f in loss["findings"]), loss
    status = result(directory, "status")
    assert status["schema_matches"] and status["applied"]["revision"] == "003", status
    # Exactly install, seed, forward, reverse, forward again and new-model upgrade applied.
    assert status["applied"]["sequence"] == 6, status
    refusal = json.loads((directory / "refused.json").read_text())
    assert refusal["code"] != 200, refusal
    return reads


def snippets(directory, reads):
    """Compare tutorial SQL with immutable artifacts and its Rust with the compiled function."""
    sql = {}
    for name in ("install", "seed", "up", "model", "lossy"):
        plan = result(directory, name)["plan"]
        sql[name] = (directory / "history" / "migrations" / plan / "sqlite-up.sql").read_text().strip()
    plan = result(directory, "up")["plan"]
    sql["down"] = (directory / "history" / "migrations" / plan / "sqlite-down.sql").read_text().strip()
    sql["read"] = "\n".join(f"{q['sql']};\n-- binds: {q['binds']}" for q in reads["read-001"]["statements"])
    source = (HERE / "src/queries.rs").read_text()
    function = source[source.index("pub async fn project_page("):source.index("\n/// Open an existing file")].strip()
    rust = "use data_quickstart::generated::stores::project::{Projects, Query as ProjectQuery, View3};\nuse stackpress_framework_sql::{DataResult, DataSession, stores::Page};\n\n" + function
    expected = {**{f"sql: {key}": value for key, value in sql.items()}, "rust: project_page": rust}
    (directory / "snippets.json").write_text(json.dumps(expected, indent=2) + "\n")
    text = (HERE / "README.md").read_text()
    for marker, body in expected.items():
        language = marker.split(":")[0]
        pattern = rf"<!-- {re.escape(marker)} -->\n```{language}\n(.*?)\n```"
        match = re.search(pattern, text, re.S)
        assert match and match[1] == body, f"README snippet differs: {marker}; actual at {directory / 'snippets.json'}"
    return rust


def compile_rust(source, evidence, environment):
    """Compile the exact tutorial code in an independent consumer using the same target root."""
    crate = evidence / "rust-example"
    (crate / "src").mkdir(parents=True)
    (crate / "src/lib.rs").write_text(source + "\n")
    manifest = ('[package]\nname="quickstart-doc-proof"\nversion="0.0.0"\nedition="2024"\n'
                '[workspace]\n[dependencies]\n'
                f'data-quickstart={{path={json.dumps(str(HERE))},features=["native"]}}\n'
                f'stackpress-framework-sql={{path={json.dumps(str(ROOT / "crates/framework/framework-sql"))}}}\n')
    (crate / "Cargo.toml").write_text(manifest)
    # Reuse the existing dependency versions; this temporary lock may only add its local package.
    shutil.copy2(ROOT / "Cargo.lock", crate / "Cargo.lock")
    command = ["cargo", "check", "--offline", "--manifest-path", str(crate / "Cargo.toml")]
    (evidence / "rust-example-command.json").write_text(json.dumps(command) + "\n")
    supervised(shlex.join(command), environment, evidence / "rust-example.log")


def main():
    """Regenerate absent output, execute every documented shell command and retain evidence."""
    environment = os.environ.copy()
    target = Path(environment.get("CARGO_TARGET_DIR", ROOT / "target")).resolve()
    assert target == ROOT / "target", "use the single owned worktree target"
    target.mkdir(exist_ok=True)
    environment.update(CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL="0", CARGO_PROFILE_DEV_DEBUG="0")
    environment.setdefault("LLVM_PROFILE_FILE", str(target / "quickstart-%p-%m.profraw"))
    for key in ("STACKPRESS_TEST_POSTGRES_URL", "STACKPRESS_TEST_MYSQL_URL"):
        environment.pop(key, None)
    evidence = Path(tempfile.mkdtemp(prefix="quickstart-proof-", dir=target))
    roots = [HERE / f"src/generated/v{i}" for i in (1, 2, 3)]
    originals = {}
    # Validate every original before the first move; a missing later root cannot strand an earlier one.
    for root in roots:
        assert root.is_dir() and not root.is_symlink(), root
        originals[str(root)] = hashes(root)
    moved = []
    try:
        # Restore exactly the acknowledged moves even if a later move or child command fails.
        for root in roots:
            root.rename(evidence / root.name)
            moved.append(root)
        blocks = re.findall(r"^```sh\n(.*?)^```", (HERE / "README.md").read_text(), re.M | re.S)
        assert len(blocks) == 6, "the proof must account for every walkthrough command block"
        bootstrap = "cargo check --offline --locked -p generated-workspace-tool --features database-tools\n"
        # All native owners have closed after block five. Retain the complete known demo file
        # before the refused step, so unchanged selected rows cannot conceal another row mutation.
        before_loss = '\ncp "$QUICKSTART_DIR/data.sqlite" "$QUICKSTART_DIR/before-loss.sqlite"\n'
        script = bootstrap + "\n".join(blocks[:5]) + before_loss + blocks[5]
        (evidence / "commands.sh").write_text(script)
        supervised(script, environment, evidence / "commands.log")
        output = (evidence / "commands.log").read_text()
        paths = re.findall(r"^QUICKSTART_DIR=(.*)$", output, re.M)
        assert len(paths) == 1, "missing walkthrough output directory"
        directory = Path(paths[0])
        assert directory.parent == target and directory.is_dir(), directory
        print(f"Walkthrough evidence: {directory}", flush=True)
        reads = verify(directory)
        rust = snippets(directory, reads)
        compile_rust(rust, evidence, environment)
        for root in reversed(moved):
            assert hashes(root) == originals[str(root)], f"non-repeatable generation: {root}"
        (evidence / "result.json").write_text(json.dumps({"commands": len(blocks), "verified": True,
            "generated_files": sum(map(len, originals.values())), "directory": str(directory)}, indent=2) + "\n")
        print(f"README commands, SQL, compiled Rust, rows, reverse and loss refusal passed; {evidence}")
    finally:
        # Preserve even failed produced output for diagnosis, then restore every original byte.
        for root in reversed(moved):
            backup = evidence / root.name
            if root.exists():
                root.rename(evidence / f"produced-{root.name}")
            backup.rename(root)


if __name__ == "__main__":
    main()
