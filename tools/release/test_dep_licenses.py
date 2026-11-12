"""Offline controls for frozen dependency licences and nested lock coverage."""
from pathlib import Path
import tempfile
import unittest

from check_dep_licenses import CRATES_IO, audit


class DependencyLicenses(unittest.TestCase):
    """Use fake registry packages so failure proofs need no Cargo or network."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.cache = self.root / '.cargo/registry/src'
        self.lock(self.root)

    def lock(self, directory, name='fake', version='1.0.0', source=True):
        directory.mkdir(parents=True, exist_ok=True)
        text = f'[[package]]\nname = "{name}"\nversion = "{version}"\n'
        if source:
            text += 'source = "registry+https://github.com/rust-lang/crates.io-index"\n'
        (directory / 'Cargo.lock').write_text(text)

    def package(self, declaration='license = "MIT"', registry='cached-registry'):
        directory = self.cache / registry / 'fake-1.0.0'
        directory.mkdir(parents=True, exist_ok=True)
        (directory / 'Cargo.toml').write_text(
            '[package]\nname = "fake"\nversion = "1.0.0"\n' + declaration + '\n')
        return directory

    def check(self, allowed=('MIT',), metadata=None):
        return audit(self.root, self.cache, set(allowed), metadata)

    def test_unlisted_fake_package_fails(self):
        self.package('license = "LicenseRef-Unlisted"')
        result = self.check()
        self.assertEqual(result.counts, {'LicenseRef-Unlisted': 1})
        self.assertIn("unlisted licence expression 'LicenseRef-Unlisted'", result.errors[0])

    def test_missing_source_fails(self):
        self.assertIn('source is not cached', self.check().errors[0])

    def test_nested_consumers_and_fuzz_are_deduplicated(self):
        self.package()
        for path in ['examples/blog', 'crates/sample/consumers/direct', 'fuzz/parser']:
            self.lock(self.root / path)
        result = self.check()
        self.assertEqual((result.locks, result.packages, result.counts, result.errors),
                         (4, 1, {'MIT': 1}, []))

    def test_new_nested_version_requires_its_own_source(self):
        self.package()
        self.lock(self.root / 'examples/blog', version='2.0.0')
        result = self.check()
        self.assertEqual(result.packages, 2)
        self.assertIn('fake 2.0.0', result.errors[0])

    def test_artifacts_and_path_packages_are_excluded(self):
        self.lock(self.root, source=False)
        for path in ['target/consumer', 'node_modules/demo', '.agents/reference']:
            self.lock(self.root / path, name='not-a-source-package')
        result = self.check()
        self.assertEqual((result.locks, result.packages, result.errors), (1, 0, []))

    def test_exact_expression_is_not_normalized(self):
        self.package('license = "MIT/Apache-2.0"')
        self.assertTrue(self.check(('MIT OR Apache-2.0',)).errors)
        self.assertEqual(self.check(('MIT/Apache-2.0',)).errors, [])

    def test_license_file_without_spdx_fails_for_review(self):
        directory = self.package('license-file = "COPYING"')
        (directory / 'COPYING').write_text('Custom licence text')
        self.assertIn("license-file 'COPYING' has no SPDX expression", self.check().errors[0])

    def test_missing_license_file_fails(self):
        self.package('license-file = "COPYING"')
        self.assertIn('license-file is missing', self.check().errors[0])

    def test_conflicting_cached_sources_fail(self):
        self.package()
        self.package('license = "ISC"', registry='other-registry')
        self.assertIn('conflicting cached licence expressions', self.check().errors[0])

    def test_malformed_source_is_a_failure(self):
        directory = self.package()
        (directory / 'Cargo.toml').write_text('not valid TOML')
        self.assertTrue(self.check().errors)

    def test_cached_identity_must_match_the_lock(self):
        directory = self.package()
        manifest = directory / 'Cargo.toml'
        manifest.write_text(manifest.read_text().replace('name = "fake"', 'name = "other"'))
        self.assertIn('cached manifest identity mismatch', self.check().errors[0])

    def test_missing_license_declaration_fails(self):
        self.package('')
        self.assertIn('no license or license-file', self.check().errors[0])

    def test_missing_root_lock_fails(self):
        (self.root / 'Cargo.lock').unlink()
        with self.assertRaisesRegex(ValueError, 'missing root lock'):
            self.check()

    def test_exact_authorized_metadata_fallback(self):
        self.lock(self.root, name='atomic-polyfill', version='1.0.3')
        metadata = {'atomic-polyfill@1.0.3':
                    {'license': 'MIT', 'source': 'crates.io API, 2026-11-12'}}
        result = self.check(metadata=metadata)
        self.assertEqual((result.counts, result.errors), ({'MIT': 1}, []))
        self.assertEqual(result.metadata_sources,
                         ['atomic-polyfill 1.0.3: crates.io API, 2026-11-12'])

    def test_other_uncached_package_cannot_use_metadata(self):
        metadata = {'fake@1.0.0': {'license': 'MIT', 'source': 'unapproved'}}
        self.assertIn('source is not cached', self.check(metadata=metadata).errors[0])

    def test_metadata_requires_provenance(self):
        self.lock(self.root, name='atomic-polyfill', version='1.0.3')
        metadata = {'atomic-polyfill@1.0.3': {'license': 'MIT'}}
        self.assertIn('incomplete registry metadata provenance', self.check(metadata=metadata).errors[0])

    def test_metadata_cannot_cross_versions_or_registries(self):
        metadata = {'atomic-polyfill@1.0.3': {'license': 'MIT', 'source': 'dated'}}
        self.lock(self.root, name='atomic-polyfill', version='1.0.4')
        self.assertTrue(self.check(metadata=metadata).errors)
        self.lock(self.root, name='atomic-polyfill', version='1.0.3')
        lock = self.root / 'Cargo.lock'
        lock.write_text(lock.read_text().replace(CRATES_IO, 'registry+https://example.invalid/index'))
        self.assertTrue(self.check(metadata=metadata).errors)

    def test_metadata_does_not_override_cached_license(self):
        self.lock(self.root, name='atomic-polyfill', version='1.0.3')
        directory = self.cache / 'cached-registry/atomic-polyfill-1.0.3'
        directory.mkdir(parents=True)
        manifest = directory / 'Cargo.toml'
        manifest.write_text('[package]\nname="atomic-polyfill"\nversion="1.0.3"\nlicense="ISC"\n')
        metadata = {'atomic-polyfill@1.0.3': {'license': 'MIT', 'source': 'dated'}}
        result = self.check(metadata=metadata)
        self.assertEqual((result.counts, result.metadata_sources), ({'ISC': 1}, []))
        self.assertTrue(result.errors)
        manifest.write_text('broken TOML')
        self.assertTrue(self.check(metadata=metadata).errors)

    def test_fallback_expression_must_still_be_listed(self):
        self.lock(self.root, name='atomic-polyfill', version='1.0.3')
        metadata = {'atomic-polyfill@1.0.3': {'license': 'ISC', 'source': 'dated'}}
        self.assertIn('unlisted licence expression', self.check(metadata=metadata).errors[0])


if __name__ == '__main__':
    unittest.main()
