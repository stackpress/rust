"""Supervisor boundary tests never launch an emulator or signal another process."""
import importlib.util
import json
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import MagicMock, patch

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('launch_android', Path(__file__).with_name('launch-android.py'))
proof = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proof)


class SupervisorTests(unittest.TestCase):
    """Exercise ownership, loopback rejection and a child timeout cleanup path."""
    def test_listener_inventory_excludes_unowned_sibling(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'listeners.log'
            with patch.object(proof.subprocess, 'check_output', return_value='10 1\n11 10\n22 1\n'), \
                 patch.object(proof.subprocess, 'run', return_value=SimpleNamespace(stdout='p11\ncqemu\nn127.0.0.1:5600\nn[::1]:5601\n')) as inventory:
                self.assertEqual(proof.audit_listeners({10}, output), {5600, 5601})
                command = inventory.call_args.args[0]
                self.assertEqual(command[command.index('-p') + 1], '10,11')

    def test_nonlocal_or_forbidden_listener_fails_closed(self):
        for address in ['*:5600', '0.0.0.0:5600', '127.0.0.1:5037', '127.0.0.1:3001']:
            with self.subTest(address=address), tempfile.TemporaryDirectory() as directory:
                with patch.object(proof.subprocess, 'check_output', return_value='10 1\n'), \
                     patch.object(proof.subprocess, 'run', return_value=SimpleNamespace(stdout='p10\nn' + address + '\n')):
                    with self.assertRaises(AssertionError):
                        proof.audit_listeners({10}, Path(directory) / 'listeners.log')

    def test_timed_out_command_kills_and_reaps_its_own_group(self):
        child = MagicMock(pid=12345)
        child.__enter__.return_value = child
        child.communicate.side_effect = subprocess.TimeoutExpired(['fixture'], 1)
        child.poll.return_value = None
        with patch.object(proof.subprocess, 'Popen', return_value=child), patch.object(proof.os, 'killpg') as kill:
            with self.assertRaises(subprocess.TimeoutExpired):
                proof.run(['fixture'], {}, timeout=1)
            kill.assert_called_once_with(12345, signal.SIGKILL)
            child.wait.assert_called_once()

    def test_daemon_cleanup_targets_only_the_recorded_child(self):
        child = MagicMock()
        child.wait.side_effect = [subprocess.TimeoutExpired(['fixture'], 1),
                                  subprocess.TimeoutExpired(['fixture'], 5), 0]
        with patch.object(proof.os, 'killpg') as group_kill:
            proof.reap(child, timeout=1)
        child.terminate.assert_called_once()
        child.kill.assert_called_once()
        self.assertEqual(child.wait.call_count, 3)
        group_kill.assert_not_called()

    def test_adb_audit_precedes_avd_creation(self):
        server = MagicMock(pid=10)
        server.poll.return_value = None
        order = []
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(proof, 'audit_listeners', side_effect=lambda *a, **k: order.append('audit') or {5609}), \
                 patch.object(proof, 'run', side_effect=lambda *a, **k: order.append('create')):
                ports = proof.create_avd_after_adb_audit(server, 5609, 'avdmanager', {}, 'owned', Path(directory))
        self.assertEqual(order, ['audit', 'create'])
        self.assertEqual(ports, {5609})
        server.terminate.assert_not_called()

    def test_unsafe_adb_binding_is_reaped_without_creating_an_avd(self):
        server = MagicMock(pid=10)
        server.poll.return_value = None
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(proof, 'audit_listeners', side_effect=AssertionError('non-loopback: *:5609')), \
                 patch.object(proof, 'reap') as reap, patch.object(proof, 'run') as create:
                with self.assertRaisesRegex(AssertionError, 'non-loopback'):
                    proof.create_avd_after_adb_audit(server, 5609, 'avdmanager', {}, 'owned', Path(directory))
        server.terminate.assert_called_once()
        reap.assert_called_once_with(server, timeout=2)
        create.assert_not_called()

    def test_unexpected_adb_port_fails_before_avd_creation(self):
        server = MagicMock(pid=10)
        server.poll.return_value = None
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(proof, 'audit_listeners', return_value={5611}), \
                 patch.object(proof, 'reap') as reap, patch.object(proof, 'run') as create:
                with self.assertRaisesRegex(RuntimeError, 'unexpected private adb'):
                    proof.create_avd_after_adb_audit(server, 5609, 'avdmanager', {}, 'owned', Path(directory))
        reap.assert_called_once_with(server, timeout=2)
        create.assert_not_called()

    def test_acceleration_requires_success_and_usable_hypervisor_output(self):
        cases = [(0, 'accel:\n0\nHypervisor.Framework OS X Version 26.6\naccel\n', True),
                 (1, 'Hypervisor.Framework OS X Version 26.6\n', False),
                 (0, 'software emulation only\n', False),
                 (0, 'Hypervisor.Framework is not usable\n', False)]
        for code, output, expected in cases:
            with self.subTest(code=code, output=output), tempfile.TemporaryDirectory() as directory:
                child = MagicMock(returncode=code)
                child.__enter__.return_value = child
                child.poll.return_value = code
                def spawn(*args, **kwargs):
                    kwargs['stdout'].write(output)
                    return child
                with patch.object(proof, 'LOGS', Path(directory)), \
                     patch.object(proof.subprocess, 'Popen', side_effect=spawn) as start:
                    if expected:
                        proof.verify_acceleration('emulator', {})
                    else:
                        with self.assertRaisesRegex(RuntimeError, 'preflight failed'):
                            proof.verify_acceleration('emulator', {})
                self.assertEqual(start.call_args.args[0], ['emulator', '-accel-check'])
                child.wait.assert_any_call(timeout=30)
                report = json.loads((Path(directory) / 'android-acceleration.json').read_text())
                self.assertEqual(report, dict(exit_code=code, timed_out=False, usable=expected, output=output))

    def test_acceleration_timeout_kills_reaps_and_records_failure(self):
        child = MagicMock(pid=12345, returncode=-9)
        child.__enter__.return_value = child
        child.wait.side_effect = [subprocess.TimeoutExpired(['emulator', '-accel-check'], 30), -9]
        child.poll.return_value = None
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(proof, 'LOGS', Path(directory)), \
                 patch.object(proof.subprocess, 'Popen', return_value=child), \
                 patch.object(proof.os, 'killpg') as kill:
                with self.assertRaisesRegex(RuntimeError, 'preflight failed'):
                    proof.verify_acceleration('emulator', {})
            report = json.loads((Path(directory) / 'android-acceleration.json').read_text())
        kill.assert_called_once_with(12345, signal.SIGKILL)
        self.assertEqual(child.wait.call_count, 2)
        self.assertTrue(report['timed_out'])
        self.assertFalse(report['usable'])

    def test_failed_acceleration_prevents_adb_and_avd_access(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path / 'android-build.json').write_text(json.dumps(dict(
                exit_code=0, closed_before_proof=True,
                owned_java_gradle_kotlin_processes_remaining=[], observed_addresses=[])))
            with patch.object(proof, 'LOGS', path), patch.object(proof, 'NATIVE', path), \
                 patch.object(proof, 'free_port', return_value=5609), \
                 patch.object(proof, 'verify_acceleration', side_effect=RuntimeError('preflight failed')), \
                 patch.object(proof.subprocess, 'Popen') as start, \
                 patch.object(proof, 'avd_definitions') as owner_read, \
                 patch.object(proof, 'create_avd_after_adb_audit') as create:
                with self.assertRaisesRegex(RuntimeError, 'preflight failed'):
                    proof.worker()
        start.assert_not_called()
        owner_read.assert_not_called()
        create.assert_not_called()


if __name__ == '__main__':
    unittest.main()
