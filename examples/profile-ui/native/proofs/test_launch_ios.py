"""Proof supervisor safety checks never launch or signal a real display/device."""
import importlib.util
import json
from pathlib import Path
import signal
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('launch_ios', Path(__file__).with_name('launch-ios.py'))
proof = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proof)


class OwnershipTests(unittest.TestCase):
    """Protect unowned processes and cleanup on a failed device command."""
    def test_headless_work_does_not_claim_a_display(self):
        with tempfile.TemporaryDirectory() as directory:
            record = Path(directory) / 'owned.json'
            record.write_text(json.dumps({'display_pids': [], 'display_started': False}))
            with patch.object(proof, 'OWNED', record), patch.object(proof, 'hub_pids') as inventory:
                self.assertEqual(proof.track_hub(), [])
                inventory.assert_not_called()

    def test_terminate_timeout_still_attempts_shutdown_and_delete(self):
        failure = subprocess.TimeoutExpired(['simctl', 'terminate'], 30)
        with patch.object(proof, 'run', side_effect=[failure, '', '']) as run:
            with self.assertRaises(subprocess.TimeoutExpired):
                proof.cleanup('owned-device')
            self.assertEqual([call.args[2] for call in run.call_args_list], ['terminate', 'shutdown', 'delete'])
            self.assertTrue(all(call.args[3] == 'owned-device' for call in run.call_args_list))

    def test_an_unrecorded_display_is_never_signalled(self):
        states = iter([{11, 22}])
        with patch.object(proof, 'hub_pids', side_effect=lambda: next(states, {22})), patch.object(proof.os, 'kill') as kill:
            with self.assertRaises(AssertionError):
                proof.close_hub([11])
            kill.assert_called_once_with(11, signal.SIGTERM)


if __name__ == '__main__':
    unittest.main()
