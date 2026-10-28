#!/usr/bin/env python3
"""Exercise success, timeout and recorder-like termination with real owned children."""
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

HERE = Path(__file__).resolve().parent


class SupervisorTests(unittest.TestCase):
    """Never leave a test child alive, even when the assertion itself fails."""

    def run_child(self, mode):
        """Launch a real child, then verify the supervisor's exit and direct-child reap."""
        with tempfile.TemporaryDirectory(prefix='issue15-watchdog-') as folder:
            root = Path(folder)
            marker = root / 'child.pid'
            # The marker identifies only this test's process; no global process scan is used.
            source = 'import os,time; from pathlib import Path; Path(' + repr(str(marker)) + ').write_text(str(os.getpid())); time.sleep(' + ('0.1' if mode == 'success' else '30') + ')'
            command = [sys.executable, str(HERE / 'supervise.py'), '--target', str(root / 'target'),
                       '--seconds', '1' if mode == 'timeout' else '10', '--', sys.executable, '-c', source]
            child_pid = None
            child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            try:
                deadline = time.monotonic() + 5
                while not marker.exists() and time.monotonic() < deadline:
                    time.sleep(0.02)
                self.assertTrue(marker.exists(), 'supervised child never started')
                child_pid = int(marker.read_text())
                if mode == 'terminate':
                    child.terminate()
                output, _ = child.communicate(timeout=10)
                self.assertEqual(child.returncode == 0, mode == 'success', output)
                self.assertIn(f'child {child_pid} reaped', output)
                with self.assertRaises(ProcessLookupError):
                    os.kill(child_pid, 0)
            finally:
                # A broken supervisor must not turn a failed test into a leaked command.
                if child_pid and child.poll() is None:
                    try:
                        os.killpg(child_pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                if child.poll() is None:
                    child.kill()
                child.wait()

    def test_success_reaps_child(self):
        """A normal exit leaves no live process behind."""
        self.run_child('success')

    def test_timeout_kills_and_reaps_child(self):
        """An overlong command fails within its deadline and is reaped."""
        self.run_child('timeout')

    def test_termination_kills_and_reaps_child(self):
        """SIGTERM from the benchmark recorder also closes the nested child group."""
        self.run_child('terminate')


if __name__ == '__main__':
    unittest.main()
