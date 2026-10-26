"""Backend selection must rebuild finite hosts before any browser proof can use them."""
import importlib.util
import os
from pathlib import Path
import sys
sys.dont_write_bytecode = True
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("browser_proof", Path(__file__).with_name("browser.py"))
browser = importlib.util.module_from_spec(spec)
spec.loader.exec_module(browser)


class BackendBuildTests(unittest.TestCase):
    """Observe the wrapper's subprocess ordering without launching a browser or listener."""

    def test_backend_switch_rebuilds_both_hosts_before_proofs(self):
        # The same target is deliberately reused while switching in both directions.
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for sqlite in (False, True, False):
                calls = []
                env = {"PROFILE_UI_NATIVE_TARGET": str(root / "native"),
                       "PROFILE_UI_BROWSER_LOGS": str(root / "logs")}
                def record(command, environment, log):
                    calls.append((list(map(str, command)), environment))
                argv = ["browser.py"] + (["--sqlite"] if sqlite else [])
                with patch.dict(os.environ, env), patch.object(sys, "argv", argv), \
                     patch.object(browser, "run", side_effect=record), \
                     patch.object(browser, "startup_failures"):
                    browser.main()
                command, environment = calls[0]
                self.assertEqual(command[:2], ["cargo", "build"], "proof used an unchecked existing backend binary")
                self.assertIn("stackpress-example-profile-ui-web", command)
                self.assertIn("stackpress-example-profile-ui-gateway", command)
                self.assertEqual("sqlite" in command, sqlite)
                self.assertEqual(environment["CARGO_TARGET_DIR"], str((root / "native").resolve()))
                self.assertTrue(any(command[0] == "node" for command, _ in calls))

    def test_failed_rebuild_prevents_proofs_and_stale_success(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "summary.json").write_text('{"result":"passed"}')
            with patch.dict(os.environ, {"PROFILE_UI_BROWSER_LOGS": str(root)}), \
                 patch.object(sys, "argv", ["browser.py"]), \
                 patch.object(browser, "run", side_effect=RuntimeError("build refused")) as run:
                with self.assertRaisesRegex(RuntimeError, "build refused"):
                    browser.main()
            self.assertEqual(run.call_args.args[0][:2], ["cargo", "build"])
            self.assertFalse((root / "summary.json").exists())


if __name__ == "__main__":
    unittest.main()
