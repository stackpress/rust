"""Negative controls for first-release ordering and stripped test dependencies."""
from pathlib import Path
import unittest

from check_packages import dependency_policy, stripped_references


class DependencyPolicy(unittest.TestCase):
    """Publish order matters for dev edges as well as normal library edges."""

    def setUp(self):
        self.root = Path('/release-proof')
        self.positions = {self.root / name: i for i, name in enumerate(['early', 'current', 'late'])}

    def audit(self, kind, target, version=None):
        value = {'path': '../' + target}
        if version:
            value['version'] = version
        return dependency_policy(self.root / 'current', {kind: {'other-crate': value}}, self.positions)

    def test_later_dev_edge_is_path_only(self):
        self.assertEqual(self.audit('dev-dependencies', 'late'), ([], {'other_crate'}))
        self.assertTrue(self.audit('dev-dependencies', 'late', '0.2.1')[0])

    def test_earlier_dev_edge_requires_version(self):
        self.assertTrue(self.audit('dev-dependencies', 'early')[0])
        self.assertEqual(self.audit('dev-dependencies', 'early', '0.2.1'), ([], set()))

    def test_private_normal_dependency_fails(self):
        self.assertTrue(self.audit('dependencies', 'private')[0])
        self.assertEqual(self.audit('dev-dependencies', 'private'), ([], {'other_crate'}))

    def test_stripped_dependency_in_packaged_unit_helper_fails(self):
        self.assertTrue(stripped_references('use other_crate::Thing;', 'src/tests/helper.rs', {'other_crate'}))
        self.assertEqual(stripped_references('// other_crate\nconst S: &str = "other_crate";',
                                            'src/tests/helper.rs', {'other_crate'}), [])


if __name__ == '__main__':
    unittest.main()
