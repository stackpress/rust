"""Contributor lock controls: workspace edges may change; registry checksums may not."""
import importlib.util
from pathlib import Path
import re
import unittest

spec = importlib.util.spec_from_file_location('blog_locks', Path(__file__).resolve().parents[1]/'locks.py')
locks = importlib.util.module_from_spec(spec)
spec.loader.exec_module(locks)


class RootRegistryTests(unittest.TestCase):
    """The baseline is the real 706-entry root; probes change one contract dimension."""

    def setUp(self):
        self.original = locks.ROOT_LOCK.read_bytes()
        self.expected = locks.registry(self.original)

    def test_workspace_edge_only_change_passes(self):
        # Modify an actual local package edge and declare its local target. Neither is registry data.
        edited, count = re.subn(
            rb'(name = "stackpress"\nversion = "[^"\n]+"\ndependencies = \[\n)',
            rb'\1 "workspace-edge-probe",\n', self.original, count=1)
        self.assertEqual(count, 1)
        edited += b'\n[[package]]\nname = "workspace-edge-probe"\nversion = "0.0.0"\n'
        self.assertNotEqual(edited, self.original)
        locks.verify_root_registry(edited, self.expected)

    def test_registry_checksum_change_fails(self):
        edited, count = re.subn(rb'checksum = "[0-9a-f]+"', b'checksum = "' + b'0'*64 + b'"', self.original, count=1)
        self.assertEqual(count, 1)
        with self.assertRaisesRegex(AssertionError, 'root registry tuples/order changed'):
            locks.verify_root_registry(edited, self.expected)

class BlogSubsetTests(unittest.TestCase):
    def test_g58_placement_is_retired(self):
        root = {tuple(row) for row in locks.registry(locks.ROOT_LOCK.read_bytes())}
        blog = {tuple(row) for row in locks.registry(locks.EXAMPLE_LOCK.read_bytes())}
        self.assertLessEqual(blog, root)
        self.assertTrue(locks.G63_ADDITIONS <= root)


if __name__ == '__main__':
    unittest.main()
