"""Real file/contract drift is rejected before gateway startup or asset publication."""
import importlib.util
import os
from pathlib import Path
import sys
sys.dont_write_bytecode = True
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("asset_manifest", Path(__file__).resolve().parents[1] / "asset_manifest.py")
manifest = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manifest)


class AssetContractTests(unittest.TestCase):
    """Use actual contract source with small real output files, independent of WASM builds."""
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        for name in manifest.CONTRACT_FILES:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes((manifest.ROOT / name).read_bytes())
        self.folder = self.root / "examples/profile-ui/dist/hydrate"
        self.folder.mkdir(parents=True)
        for name, data in {"start.js": b"start", "profile_ui.js": b"module", "profile_ui_bg.wasm": b"wasm"}.items():
            (self.folder / name).write_bytes(data)
        self.patches = [patch.object(manifest, "ROOT", self.root), patch.object(manifest, "PROFILE", self.root / "examples/profile-ui")]
        for item in self.patches: item.start()

    def tearDown(self):
        for item in self.patches: item.stop()
        self.temp.cleanup()

    def test_byte_and_inventory_drift(self):
        original = manifest.write("hydrate")
        self.assertEqual(manifest.check("hydrate"), original)
        (self.folder / "start.js").write_bytes(b"other")
        with self.assertRaisesRegex(ValueError, "mismatch"): manifest.check("hydrate")
        manifest.write("hydrate")
        (self.folder / "extra.js").write_text("extra")
        with self.assertRaisesRegex(ValueError, "mismatch"): manifest.check("hydrate")
        (self.folder / "profile_ui_bg.wasm").unlink()
        with self.assertRaisesRegex(ValueError, "missing"): manifest.check("hydrate")

    def test_contract_and_symlink_drift(self):
        manifest.write("hydrate")
        source = self.root / manifest.CONTRACT_FILES[0]
        source.write_text(source.read_text().replace('pub const REVISION: u32 = 1;', 'pub const REVISION: u32 = 2;'))
        with self.assertRaisesRegex(ValueError, "mismatch"): manifest.check("hydrate")
        (self.folder / "linked.js").symlink_to(self.folder / "start.js")
        with self.assertRaisesRegex(ValueError, "symlink"): manifest.write("hydrate")

    def test_oversized_file_is_rejected_before_reading(self):
        # A sparse file proves admission precedes allocation without retaining 64 MiB.
        large = self.folder / "start.js"
        with large.open("wb") as file:
            file.truncate(64 * 1024 * 1024 + 1)
        original = Path.read_bytes

        def forbid_large_read(path):
            if path == large:
                self.fail("oversized asset was read before its byte limit was checked")
            return original(path)

        with patch.object(Path, "read_bytes", forbid_large_read):
            with self.assertRaisesRegex(ValueError, "64 MiB"):
                manifest.write("hydrate")

    def test_manifest_and_root_symlinks_are_rejected(self):
        manifest.write("hydrate")
        path = self.folder / "manifest.json"
        saved = self.root / "saved-manifest.json"
        path.rename(saved)
        path.symlink_to(saved)
        with self.assertRaisesRegex(ValueError, "symlink"):
            manifest.check("hydrate")
        path.unlink()
        saved.rename(path)
        actual = self.folder.with_name("actual")
        self.folder.rename(actual)
        self.folder.symlink_to(actual, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            manifest.check("hydrate")

    def test_exact_inventory_budgets_and_special_files(self):
        # Existing fixtures total 15 bytes across three files; exact ceilings pass.
        with patch.object(manifest, "MAX_TOTAL_BYTES", 15), patch.object(manifest, "MAX_FILES", 3):
            manifest.write("hydrate")
        with patch.object(manifest, "MAX_TOTAL_BYTES", 14):
            with self.assertRaisesRegex(ValueError, "total byte limit"):
                manifest.write("hydrate")
        (self.folder / "extra.js").write_bytes(b"")
        with patch.object(manifest, "MAX_FILES", 3):
            with self.assertRaisesRegex(ValueError, "count limit"):
                manifest.write("hydrate")
        (self.folder / "extra.js").unlink()
        path = self.folder / "manifest.json"
        path.write_bytes(b" " * (64 * 1024 + 1))
        with self.assertRaisesRegex(ValueError, "64 KiB"):
            manifest.check("hydrate")
        if hasattr(os, "mkfifo"):
            # lstat must reject special files before an open could block the check.
            path.unlink()
            os.mkfifo(path)
            with self.assertRaisesRegex(ValueError, "regular file"):
                manifest.check("hydrate")

    def test_manifest_limit_precedes_publication(self):
        with patch.object(manifest, "MAX_MANIFEST_BYTES", 1):
            with self.assertRaisesRegex(ValueError, "64 KiB"):
                manifest.write("hydrate")
        self.assertFalse((self.folder / "manifest.json").exists())

    def test_failed_recheck_cannot_retain_a_passed_summary(self):
        # Load the real entry point but fail before any external tool/build is launched.
        location = Path(__file__).resolve().parents[1] / "check.py"
        entry_spec = importlib.util.spec_from_file_location("profile_ui_check", location)
        entry = importlib.util.module_from_spec(entry_spec)
        with patch.dict(sys.modules, {"asset_manifest": manifest}):
            entry_spec.loader.exec_module(entry)
        summary = self.root / "summary.json"
        summary.write_text('{"result":"passed"}')
        with patch.object(entry, "LOGS", self.root), patch.object(entry, "preflight", side_effect=ValueError("tool mismatch")):
            with self.assertRaisesRegex(ValueError, "tool mismatch"):
                entry.main()
        self.assertFalse(summary.exists(), "a failed recheck retained a stale success receipt")


if __name__ == "__main__": unittest.main()
