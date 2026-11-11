"""Regression proofs for escaped paths, Cargo omissions, and copy freshness."""

import tempfile
from pathlib import Path
import unittest

import check_test_boundaries as boundaries
import check_test_copies as copies


class BoundaryTests(unittest.TestCase):
    """Exercise real temporary layouts, including symlinks and nested packages."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.crate = self.root / "crate"
        self.path = self.crate / "tests/main.rs"
        self.path.parent.mkdir(parents=True)

    def scan(self, source):
        self.path.write_text(source)
        return boundaries.scan_file("test", self.crate, self.path)

    def test_includes_paths_and_fixed_reads_escape(self):
        for expression in [
            'include_str!("../../sibling/input.idea");',
            'include_bytes!(r#"../../sibling/input.bin"#);',
            '#[path = "../../sibling/helper.rs"] mod helper;',
            '#[cfg_attr(feature = "special", path = r#"../../sibling/helper.rs"#)] mod helper;',
            'std::fs::read_to_string("../sibling/input.idea");',
            'Path::new(env!("CARGO_MANIFEST_DIR")).join("../sibling");',
            'include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../helper.rs"));',
        ]:
            with self.subTest(expression=expression):
                self.assertEqual(len(self.scan(expression)), 1)

    def test_local_inputs_comments_and_test_data_are_allowed(self):
        self.assertEqual(self.scan('''
            /* nested /* include_str!("../../bad") */ comment */
            // #[path = "../../bad"] mod bad;
            let data = r#"include_str!("../../bad")"#;
            include_str!("../fixtures/local.idea");
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/input"));
            assert!(validate("../invalid").is_err());
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/input");
        '''), [])

    def test_nested_package_is_unavailable_even_inside_crate(self):
        nested = self.crate / "consumers/build-host"
        nested.mkdir(parents=True)
        (nested / "Cargo.toml").touch()
        issues = self.scan('include_str!("../consumers/build-host/source.idea");')
        self.assertEqual(len(issues), 1)
        self.assertIn("nested Cargo package", issues[0].message)

    def test_symlink_cannot_hide_escape(self):
        (self.crate / "fixtures").symlink_to(self.root / "sibling")
        self.assertEqual(len(self.scan('include_str!("../fixtures/input");')), 1)

    def test_unknown_include_expression_fails_closed(self):
        self.assertEqual(len(self.scan('include_str!(some_macro!());')), 1)

    def test_manifest_path_preserves_apostrophes_and_spaces(self):
        crate = Path("/tmp/owner's crate")
        expression = 'concat!(env!("CARGO_MANIFEST_DIR"), "/tests/input")'
        self.assertEqual(boundaries.include_path(expression, crate), str(crate / "tests/input"))

    def test_line_numbers_survive_multiline_comments(self):
        issues = self.scan('/* first\nsecond */\ninclude_str!("../../bad");')
        self.assertEqual(issues[0].line, 3)

    def test_python_repository_anchor_and_local_fixture(self):
        path = self.path.with_suffix(".py")
        path.write_text('from pathlib import Path\nHERE = Path(__file__).resolve().parent\nROOT = HERE.parents[1]\n')
        issues = boundaries.scan_python("test", self.crate, path)
        self.assertEqual(len(issues), 1)
        self.assertEqual(issues[0].line, 3)
        path.write_text('from pathlib import Path\nHERE = Path(__file__).resolve().parent\nINPUT = HERE / "fixtures/input.json"\n')
        self.assertEqual(boundaries.scan_python("test", self.crate, path), [])

    def test_javascript_fixed_reads_and_script_anchors(self):
        path = self.path.with_suffix(".cjs")
        for expression in ["fs.readFileSync('../../sibling/input');", "require('../sibling/helper');", "path.join(__dirname, '../../sibling/input');"]:
            path.write_text(expression)
            self.assertEqual(len(boundaries.scan_file("test", self.crate, path)), 1)
        path.write_text('''
            // fs.readFileSync('../../sibling/input');
            const example = "fs.readFileSync('../../sibling/input')";
            const template = `fs.readFileSync('../../sibling/input')`;
            path.join(__dirname, 'fixtures/input');
        ''')
        self.assertEqual(boundaries.scan_file("test", self.crate, path), [])


class CopyTests(unittest.TestCase):
    """Source changes and copy changes both require an explicit refresh."""

    def test_freshness_and_documented_divergence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source").write_bytes(b"original\r\n")
            (root / "copy").write_bytes(b"// provenance\noriginal\r\n")
            item = dict(source="source", copy="copy", sha256=copies.digest(b"original\r\n"), provenance="// provenance")
            manifest = {"copies": [item]}
            self.assertEqual(copies.check(root, manifest), [])
            (root / "copy").write_bytes(b"diverged\n")
            self.assertEqual(len(copies.check(root, manifest)), 1)
            item.update(divergence_reason="Exercise platform-specific supervision", copy_sha256=copies.digest(b"diverged\n"))
            self.assertEqual(copies.check(root, manifest), [])
            (root / "source").write_bytes(b"changed\n")
            self.assertEqual(len(copies.check(root, manifest)), 1)

    def test_duplicate_destinations_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source").write_bytes(b"same")
            (root / "copy").write_bytes(b"same")
            item = dict(source="source", copy="copy", sha256=copies.digest(b"same"))
            issues = copies.check(root, {"copies": [item, {**item, "copy": "./copy"}]})
            self.assertEqual(len(issues), 1)
            self.assertIn("duplicate", issues[0])


if __name__ == "__main__":
    unittest.main()
