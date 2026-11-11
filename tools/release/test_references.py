"""Adversarial controls for the package boundary checker; no crate tests are modified."""
from pathlib import Path
import tempfile
import unittest

from rust_references import check_source


class ReferenceChecks(unittest.TestCase):
    """Exercise successful and failing resolutions independently of Cargo's inventory."""
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'src').mkdir()

    def check(self, source, files, root=False):
        """Build a source with supplied packaged paths; existence alone never suffices."""
        path = self.root / 'src/entry.rs'
        path.write_text(source)
        for relative in files:
            item = self.root / relative
            item.parent.mkdir(parents=True, exist_ok=True)
            item.touch()
        return check_source(self.root, 'src/entry.rs', set(files), {path} if root else ())

    def test_literal_include_must_be_packaged(self):
        self.assertEqual(self.check('include_str!("data.json");', ['src/data.json']), [])
        self.assertTrue(self.check('include_bytes!("data.json");', []))

    def test_disabled_cfg_reference_is_checked(self):
        self.assertTrue(self.check('#[cfg(any())] #[path = "missing.rs"] mod test;', []))

    def test_comment_raw_string_and_character_are_opaque(self):
        source = '''/* outer /* nested */ mod missing; */
// include_str!("missing")
const X: &str = r###"mod absent; include_bytes!("bad")"###;
const Y: char = '"';'''
        self.assertEqual(self.check(source, []), [])

    def test_implicit_modules_distinguish_entrypoints(self):
        self.assertEqual(self.check('mod child;', ['src/entry/child.rs']), [])
        self.assertEqual(self.check('mod child;', ['src/child/mod.rs'], root=True), [])
        self.assertTrue(self.check('mod child;', ['src/child.rs']))

    def test_inline_module_and_path_attribute(self):
        self.assertEqual(self.check('mod inner { mod child; }', ['src/entry/inner/child.rs']), [])
        self.assertEqual(self.check('#[path = "other.rs"] mod named;', ['src/other.rs']), [])

    def test_manifest_concat_and_dynamic_expression(self):
        source = 'include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/blob",));'
        self.assertEqual(self.check(source, ['data/blob']), [])
        self.assertTrue(self.check('include_str!(chosen_path!());', []))

    def test_escape_outside_package_rejected(self):
        self.assertTrue(self.check('include_str!("../../external.txt");', ['../external.txt']))


if __name__ == '__main__':
    unittest.main()
