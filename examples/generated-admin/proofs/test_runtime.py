"""Behavioral watchdog regressions; these children neither build nor bind sockets."""
import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch
from . import runtime


class WatchdogTests(unittest.TestCase):
    """A failed or hung gate must fail, leave metadata and reap its child."""
    def setUp(self):
        runtime.OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=runtime.OUTPUT_ROOT)
        self.ctx = runtime.Context('runtime-tests')
        self.ctx.logs = Path(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def test_timeout_kills_and_reaps(self):
        children, spawn = [], subprocess.Popen
        def capture(*args, **kwargs):
            child = spawn(*args, **kwargs)
            children.append(child)
            return child
        # A loaded host may kill the child before it executes Python. Capture its real PID
        # in the parent, instead of making successful cleanup depend on a child handshake.
        with patch.object(runtime.subprocess,'Popen',side_effect=capture):
            with self.assertRaises(TimeoutError):
                self.ctx.run('hung', [sys.executable,'-c','import time;time.sleep(60)'], timeout=1)
        result = json.loads((self.ctx.logs/'hung-result.json').read_text())
        self.assertTrue(result['expired'])
        self.assertLess(result['seconds'],5)
        self.assertEqual(result['code'],-9)
        self.assertEqual(len(children),1)
        with self.assertRaises(ProcessLookupError):
            os.kill(children[0].pid,0)

    def test_failure_and_environment_are_preserved(self):
        with self.assertRaises(SystemExit) as failure:
            self.ctx.run('failed', [sys.executable,'-c','raise SystemExit(7)'])
        self.assertEqual(failure.exception.code,7)
        self.assertFalse(json.loads((self.ctx.logs/'failed-result.json').read_text())['expired'])
        self.ctx.run('environment', [sys.executable,'-c',"import os;print(os.environ['CARGO_TARGET_DIR']);print(os.environ['CARGO_INCREMENTAL'])"])
        log = (self.ctx.logs/'environment.log').read_text().splitlines()
        self.assertEqual(log[-2:], [str(runtime.TARGET),'0'])

    def test_disposable_workspaces_have_distinct_stable_targets(self):
        first, second = self.ctx.logs/'first', self.ctx.logs/'second'
        first.mkdir()
        second.mkdir()
        command = [sys.executable, '-c', "import os;print(os.environ['CARGO_TARGET_DIR'])"]
        for name, root in [('first', first), ('second', second), ('again', first)]:
            self.ctx.run(name, command, cwd=root)
        targets = [json.loads((self.ctx.logs/(name+'-result.json')).read_text())['target']
                   for name in ['first', 'second', 'again']]
        self.assertEqual(targets[0], targets[2])
        self.assertNotEqual(targets[0], targets[1])
        for name, target in zip(['first', 'second', 'again'], targets):
            self.assertTrue(Path(target).is_relative_to(runtime.TARGET/'consumers'))
            self.assertEqual((self.ctx.logs/(name+'.log')).read_text().splitlines()[-1], target)
        self.assertEqual(self.ctx.target(runtime.ROOT), runtime.TARGET)

    def test_workspace_removes_only_its_target_on_success_and_exception(self):
        target_root = self.ctx.logs/'target'
        target_root.mkdir()
        retained = target_root/'repository-artifact'
        retained.write_text('keep warm')
        for fails in [False, True]:
            with self.subTest(fails=fails), patch.object(runtime, 'TARGET', target_root):
                try:
                    with self.ctx.workspace() as root:
                        target = self.ctx.target(root)
                        target.mkdir(parents=True)
                        (target/'consumer-artifact').write_text('temporary')
                        if fails:
                            raise RuntimeError('fixture failure')
                except RuntimeError as error:
                    self.assertTrue(fails)
                    self.assertEqual(str(error), 'fixture failure')
                self.assertFalse(root.exists())
                self.assertFalse(target.exists())
                self.assertEqual(retained.read_text(), 'keep warm')

    def test_slow_disk_measurement_cannot_extend_the_deadline(self):
        bindir = self.ctx.logs/'bin'
        bindir.mkdir()
        meter = bindir/'du'
        meter.write_text('#!/bin/sh\nexec /bin/sleep 60\n')
        meter.chmod(0o700)
        children, spawn = [], subprocess.Popen
        def capture(*args, **kwargs):
            child = spawn(*args, **kwargs)
            children.append(child)
            return child
        started = time.monotonic()
        with patch.object(runtime,'TARGET',self.ctx.logs), patch.dict(os.environ,
                PATH=str(bindir)+os.pathsep+os.environ['PATH']), patch.object(
                    runtime.subprocess,'Popen',side_effect=capture):
            with self.assertRaises(subprocess.TimeoutExpired):
                self.ctx.size_kib(deadline=started+.5)
        self.assertLess(time.monotonic()-started,3)
        self.assertEqual(len(children),1)
        self.assertIsNotNone(children[0].returncode)
        with self.assertRaises(ProcessLookupError):
            os.kill(children[0].pid,0)


if __name__ == '__main__':
    unittest.main()
