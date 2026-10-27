"""Prove the proof's own timeout, command-failure and absent-output restoration paths."""

import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import prove


class ProofSafety(unittest.TestCase):
    """Watchdog controls use a real process; restoration uses the real owned generated roots."""

    def test_timeout_kills_and_reaps(self):
        """A deliberately stuck child is a failed proof, and its exact PID no longer exists."""
        with tempfile.TemporaryDirectory(dir=prove.ROOT / "target") as directory:
            root = Path(directory)
            script = f'echo $$ > "{root / "pid"}"\nexec python3 -c "import time;time.sleep(60)"'
            with self.assertRaisesRegex(AssertionError, "timed out"):
                prove.supervised(script, os.environ.copy(), root / "log", timeout=0.5)
            pid = int((root / "pid").read_text())
            with self.assertRaises(ProcessLookupError):
                os.kill(pid, 0)

    def test_command_failure_is_not_success(self):
        """A finite nonzero child exit is also a failed proof, with its log retained."""
        with tempfile.TemporaryDirectory(dir=prove.ROOT / "target") as directory:
            with self.assertRaisesRegex(AssertionError, "command failed"):
                prove.supervised("exit 7", os.environ.copy(), Path(directory) / "log", timeout=2)

    def test_failed_bootstrap_restores_all_originals(self):
        """A failure after the output was moved aside restores every source and manifest byte."""
        roots = [prove.HERE / f"src/generated/v{i}" for i in (1, 2, 3)]
        before = [prove.hashes(root) for root in roots]
        with patch.object(prove, "supervised", side_effect=AssertionError("injected child failure")):
            with self.assertRaisesRegex(AssertionError, "injected child failure"):
                prove.main()
        self.assertEqual([prove.hashes(root) for root in roots], before)

    def test_partial_move_failure_restores_only_acknowledged_moves(self):
        """A later rename failure cannot move or overwrite untouched originals during cleanup."""
        with tempfile.TemporaryDirectory(dir=prove.ROOT / "target") as directory:
            fixture = Path(directory)
            roots = [fixture / f"src/generated/v{i}" for i in (1, 2, 3)]
            for root in roots:
                root.mkdir(parents=True)
                (root / "source.rs").write_text(root.name)
            before = [prove.hashes(root) for root in roots]
            rename = Path.rename

            def fail_second(source, destination):
                """Fail before the second move, while leaving all restoration moves operational."""
                if source == roots[1] and Path(destination).name == "v2":
                    raise OSError("injected second move failure")
                return rename(source, destination)

            with patch.object(prove, "HERE", fixture), patch.object(Path, "rename", fail_second):
                with self.assertRaisesRegex(OSError, "injected second move failure"):
                    prove.main()
            self.assertEqual([prove.hashes(root) for root in roots], before)


if __name__ == "__main__":
    unittest.main()
