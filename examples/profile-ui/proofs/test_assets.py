"""Real file/contract drift is rejected before gateway startup or asset publication."""
import importlib.util
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


if __name__ == "__main__": unittest.main()
