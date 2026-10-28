"""Exercise persistent accounting without compiling Rust or writing real history."""

import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "build-benchmark.py"


class BuildBenchmarkTests(unittest.TestCase):
    """Verify observable records, exit codes, concurrent writes, and counters."""

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.store = Path(self.temporary.name) / "records"

    def command(self, source, cache="cold", scope="workspace"):
        """Build a recorder invocation around a finite synthetic child."""
        return [sys.executable, str(SCRIPT), "--store", str(self.store), "run",
                "--label", "isolated fixture", "--kind", "test", "--scope", scope,
                "--cache", cache, "--", sys.executable, "-c", source]

    def execute(self, source, **options):
        """Bound the wrapper so a regression cannot hang the test suite."""
        return subprocess.run(self.command(source, **options), capture_output=True,
                              text=True, timeout=20)

    def records(self):
        """Read exactly the records owned by this test's temporary store."""
        return [json.loads(path.read_text()) for path in sorted((self.store / "runs").glob("*.json"))]

    def summary(self):
        """Use the public reporting command rather than reproducing its arithmetic."""
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--store", str(self.store), "summary", "--json"],
            capture_output=True, text=True, timeout=20,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)["summary"]

    def test_multiple_build_stages_and_total_command_time_are_separate(self):
        result = self.execute(
            "import time; print('   Compiling first v0.1.0'); "
            "print('    Finished `test` profile [unoptimized] target(s) in 1m 02.50s'); "
            "print('   Compiling fixture v0.1.0'); "
            "print('    Finished `dev` profile [unoptimized] target(s) in 0.25s'); "
            "time.sleep(0.03)"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        record, = self.records()
        self.assertEqual(record["cargo_build_seconds"], 62.75)
        self.assertEqual(record["cargo_activity"]["compiling"], 2)
        self.assertGreaterEqual(record["command_seconds"], 0.03)
        self.assertLess(record["command_seconds"], record["cargo_build_seconds"])
        self.assertEqual((self.store / record["log"]).read_text(), result.stdout)
        summary = self.summary()
        self.assertEqual(summary["attempts"], 1)
        self.assertEqual(summary["attempts_with_observed_compilation"], 1)
        self.assertEqual(summary["successful_declared_cold_workspace_builds"], 1)
        self.assertEqual(summary["cargo_build_stages"], 2)

    def test_cached_run_is_an_attempt_without_observed_compilation(self):
        self.assertEqual(self.execute(
            "print('    Finished `test` profile [unoptimized] target(s) in 0.02s')",
            cache="warm",
        ).returncode, 0)
        summary = self.summary()
        self.assertEqual(summary["by_cache"], {"warm": 1})
        self.assertEqual(summary["attempts_with_observed_compilation"], 0)
        self.assertEqual(summary["successful_declared_cold_workspace_builds"], 0)

    def test_selected_crate_build_is_not_counted_as_a_workspace_rebuild(self):
        self.assertEqual(self.execute(
            "print('   Compiling selected v0.1.0'); "
            "print('    Finished `test` profile [unoptimized] target(s) in 0.02s')",
            scope="selected",
        ).returncode, 0)
        summary = self.summary()
        self.assertEqual(summary["attempts_with_observed_compilation"], 1)
        self.assertEqual(summary["successful_declared_cold_workspace_builds"], 0)

    def test_linked_worktrees_resolve_the_same_persistent_store(self):
        root = Path(self.temporary.name)
        repo, linked = root / "main", root / "linked"
        commands = [
            ["git", "init", "-q", str(repo)],
            ["git", "-C", str(repo), "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
             "-c", "commit.gpgsign=false", "commit", "--allow-empty", "-qm", "fixture"],
            ["git", "-C", str(repo), "worktree", "add", "--detach", "-q", str(linked), "HEAD"],
        ]
        for command in commands:
            subprocess.run(command, check=True, capture_output=True, timeout=10)
        for directory in (repo, linked):
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "summary", "--json"], cwd=directory,
                capture_output=True, text=True, check=True, timeout=10,
            )
            self.assertEqual(json.loads(result.stdout)["summary"]["store"],
                             str(repo.resolve() / ".agents/resources/build-benchmarks"))

    def test_failed_compile_preserves_exit_code_and_unknown_stage_duration(self):
        result = self.execute("import sys; print('   Compiling broken v0.1.0'); sys.exit(7)")
        self.assertEqual(result.returncode, 7)
        record, = self.records()
        self.assertEqual(record["status"], "failed")
        self.assertIsNone(record["cargo_build_seconds"])
        summary = self.summary()
        self.assertEqual(summary["attempts_with_observed_compilation"], 1)
        self.assertEqual(summary["successful_declared_cold_workspace_builds"], 0)

    def test_parallel_attempts_do_not_overwrite_each_other(self):
        source = "import time; print('   Compiling parallel v0.1.0'); time.sleep(0.1)"
        children = [subprocess.Popen(self.command(source), stdout=subprocess.PIPE,
                                     stderr=subprocess.PIPE, text=True) for _ in range(4)]
        for child in children:
            _, stderr = child.communicate(timeout=20)
            self.assertEqual(child.returncode, 0, stderr)
        self.assertEqual(len({row["id"] for row in self.records()}), 4)
        self.assertEqual(self.summary()["attempts"], 4)
        self.assertEqual(len(list((self.store / "runs").glob("*.log"))), 4)

    def test_failed_launch_is_recorded(self):
        command = self.command("unused")
        command[-3:] = [str(self.store / "missing-executable")]
        result = subprocess.run(command, capture_output=True, text=True, timeout=20)
        self.assertEqual(result.returncode, 127, result.stderr)
        record, = self.records()
        self.assertEqual(record["status"], "launch_failed")
        self.assertIsNone(record["cargo_build_seconds"])

    def interrupt_running_command(self, signum):
        """Wait for a real child, then verify recorder and child both stop."""
        source = "import os,time; print(os.getpid(), flush=True); time.sleep(30)"
        child = subprocess.Popen(self.command(source), stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, text=True)
        fixture_pid = None
        try:
            self.assertTrue(select.select([child.stdout], [], [], 10)[0], "child did not start")
            fixture_pid = int(child.stdout.readline().strip())
            child.send_signal(signum)
            child.communicate(timeout=10)
            self.assertEqual(child.returncode, 128 + signum)
            record, = self.records()
            self.assertEqual(record["status"], "interrupted")
            with self.assertRaises(ProcessLookupError):
                os.kill(fixture_pid, 0)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
            if fixture_pid is not None:
                try:
                    os.kill(fixture_pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def test_ctrl_c_records_interruption_and_reaps_child(self):
        self.interrupt_running_command(signal.SIGINT)

    def test_sigterm_records_interruption_and_reaps_child(self):
        self.interrupt_running_command(signal.SIGTERM)

    def test_empty_store_and_corrupt_evidence_are_distinguished(self):
        self.assertEqual(self.summary()["attempts"], 0)
        folder = self.store / "runs"
        folder.mkdir(parents=True)
        (folder / "broken.json").write_text("not JSON")
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--store", str(self.store), "summary"],
            capture_output=True, text=True, timeout=20,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("Benchmark error", result.stderr)


if __name__ == "__main__":
    unittest.main()
