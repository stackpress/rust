#!/usr/bin/env python3
"""Audit the 23 release package lists, file references, metadata, notices and sizes.

Run through the repository benchmark recorder. --archives additionally checks the
actual .crate payload and normalized manifests produced by cargo package. A source
reference failure is reported even in a cfg-disabled module: it must ship safely.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib

from common import ROOT, EVIDENCE, VERSION, candidates, manifest, registry, write_report
from rust_references import check_source, tokens

LIMIT = 10_000_000


def package_metadata(directory, name, package, workspace):
    """Validate the G60/G61 fields without inheriting root README/licence paths."""
    errors = []
    effective = {key: workspace.get(key) if isinstance(value, dict) and value.get('workspace') else value
                 for key, value in package.items()}
    expected_stackpress = name == 'stackpress' or name.startswith('stackpress-framework')
    expected_text = (ROOT / 'LICENSE' if expected_stackpress else
                     ROOT / 'crates/idea/idea-parser/LICENSE').read_bytes()
    license_name = effective.get('license-file', 'LICENSE')
    license_path = directory / license_name
    if not license_path.is_file() or license_path.read_bytes() != expected_text:
        errors.append('licence text missing or differs from approved source')
    if expected_stackpress:
        if effective.get('license-file') != 'LICENSE' or effective.get('license'):
            errors.append('requires license-file = "LICENSE" without an SPDX licence')
    elif effective.get('license') != 'Apache-2.0' or effective.get('license-file'):
        errors.append('requires license = "Apache-2.0" without license-file')
    for field, expected in [('publish', True), ('version', VERSION),
                            ('repository', 'https://github.com/stackpress/rust'), ('readme', 'README.md')]:
        if effective.get(field) != expected:
            errors.append(f'{field}: expected {expected!r}, got {effective.get(field)!r}')
    keywords = effective.get('keywords', [])
    if not keywords or len(keywords) > 5 or any(not re.fullmatch(r'[A-Za-z][A-Za-z0-9_-]{0,19}', k) for k in keywords):
        errors.append('invalid or absent crates.io keywords')
    known_categories = json.loads((EVIDENCE / 'category-sources.json').read_text())
    if any(c not in known_categories for c in effective.get('categories', [])):
        errors.append('category has no cached registry provenance')
    return errors


def dependencies(data):
    """Include normal/build/dev tables and every target-specific counterpart."""
    for owner in [data, *data.get('target', {}).values()]:
        for kind in ['dependencies', 'build-dependencies', 'dev-dependencies']:
            for name, value in owner.get(kind, {}).items():
                if isinstance(value, dict):
                    yield kind, name, value


def dependency_policy(directory, data, positions):
    """Version published prerequisites; strip private and later dev prerequisites."""
    errors, stripped = [], set()
    for kind, key, dep in dependencies(data):
        if 'path' not in dep:
            continue
        target = (directory / dep['path']).resolve()
        later = target in positions and positions[target] > positions[directory.resolve()]
        if kind == 'dev-dependencies' and (target not in positions or later):
            if 'version' in dep:
                errors.append(f'{kind}.{key} points to a later/private crate and must stay path-only')
            stripped.add(key.replace('-', '_'))
        elif target not in positions:
            errors.append(f'unpublished {kind}.{key}')
        elif dep.get('version') != VERSION:
            errors.append(f'{kind}.{key} needs version {VERSION}')
    return errors, stripped


def stripped_references(source, relative, stripped):
    """Catch unit/integration/example support using dependencies Cargo will remove.

    Inspect every packaged Rust file, including dormant test helpers. Strings and
    comments are opaque so generated source snippets do not look like imports.
    """
    return [f'{relative}:{item.line}: references stripped dev-dependency {item.value}'
            for item in tokens(source) if not item.string and item.value in stripped]


def archive_checks(directory, name, packaged, archive):
    """Check actual bytes, path-only dev stripping and shipped third-party attribution."""
    errors = []
    if not archive.is_file():
        return {'archive_bytes': None}, ['archive missing: ' + str(archive)]
    with tarfile.open(archive) as tar:
        prefix = f'{name}-{VERSION}/'
        members = {item.name.removeprefix(prefix): item for item in tar.getmembers() if item.isfile()}
        if set(members) != packaged:
            errors.append('archive file list differs from cargo package --list')
        data = manifest(directory / 'Cargo.toml')
        normalized = tomllib.loads(tar.extractfile(members['Cargo.toml']).read().decode())
        for kind, key, dep in dependencies(normalized):
            if 'path' in dep:
                errors.append('normalized manifest retains a local dependency path: ' + key)
        archived_registry = registry(tar.extractfile(members['Cargo.lock']).read())
        internal_names = {candidate for _, candidate in candidates()}
        archived_external = {row for row in archived_registry if row[0] not in internal_names}
        if not archived_external <= registry((ROOT / 'Cargo.lock').read_bytes()):
            errors.append('archive lock introduces an external registry tuple outside the root lock')
        for kind, key, dep in dependencies(data):
            if kind == 'dev-dependencies' and 'path' in dep and 'version' not in dep:
                if any(other_key == key for _, other_key, _ in dependencies(normalized)):
                    errors.append('path-only unpublished dev-dependency retained: ' + key)
        for relative, member in members.items():
            if relative in ('Cargo.toml', 'Cargo.lock', 'Cargo.toml.orig', '.cargo_vcs_info.json'):
                continue
            original = directory / relative
            if not original.is_file() or tar.extractfile(member).read() != original.read_bytes():
                errors.append('archive/source bytes differ: ' + relative)
        size = sum(item.size for item in members.values())
    if archive.stat().st_size >= LIMIT or size >= LIMIT:
        errors.append('10 MB archive or uncompressed payload limit exceeded')
    return {'archive_bytes': archive.stat().st_size, 'archive_payload_bytes': size}, errors


def main():
    """Collect every candidate's failures before returning a nonzero release gate."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archives', action='store_true')
    parser.add_argument('--pending-tests', action='store_true',
                        help='report only exact coordinator-approved test-boundary omissions as pending')
    parser.add_argument('--output', type=Path, default=EVIDENCE / 'packages.json')
    args = parser.parse_args()
    root_data = manifest(ROOT / 'Cargo.toml')
    assert root_data['workspace']['package']['publish'] is False
    approved = dict((path.resolve(), name) for path, name in candidates())
    positions = {path.resolve(): i for i, (path, _) in enumerate(candidates())}
    baseline = {tuple(row) for row in json.loads((EVIDENCE / 'registry-baseline.json').read_text())}
    assert registry((ROOT / 'Cargo.lock').read_bytes()) == baseline and len(baseline) == 699
    original_notices = json.loads((EVIDENCE / 'notice-baseline.json').read_text())
    for source in ['LICENSE', 'crates/idea/idea-parser/LICENSE']:
        assert hashlib.sha256((ROOT / source).read_bytes()).hexdigest() == original_notices[source], source
    reports = []
    for directory, name in candidates():
        data = manifest(directory / 'Cargo.toml')
        errors = package_metadata(directory, name, data['package'], root_data['workspace']['package'])
        dependency_errors, stripped = dependency_policy(directory, data, positions)
        errors.extend(dependency_errors)
        command = ['cargo', 'package', '--manifest-path', str(directory / 'Cargo.toml'),
                   '--list', '--offline', '--allow-dirty']
        print('COMMAND:', *command, flush=True)
        result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
        if result.stderr:
            print(result.stderr, end='', flush=True)
        packaged = set(result.stdout.splitlines()) if result.returncode == 0 else set()
        if result.returncode:
            errors.append('cargo package --list failed')
        target_roots = {directory / value["path"] for kind in ("lib", "bin", "test", "example", "bench")
                        for value in ([data[kind]] if isinstance(data.get(kind), dict) else data.get(kind, []))
                        if "path" in value}
        target_roots.update(directory.glob("tests/*.rs"))
        target_roots.update(directory.glob("examples/*.rs"))
        for relative in sorted(packaged):
            if relative.endswith('.rs'):
                errors.extend(check_source(directory, relative, packaged, target_roots))
                errors.extend(stripped_references((directory / relative).read_text(), relative, stripped))
        # Attribution and embedded resources are compared by path and byte identity in archives.
        notices = [p.relative_to(directory).as_posix()
                   for base in (directory / 'third-party', directory / 'data')
                   for p in base.rglob('*') if p.is_file()
                   and ('third-party' in p.parts or p.name.startswith('nodemailer-'))]
        original_notices = json.loads((EVIDENCE / 'notice-baseline.json').read_text())
        for relative in notices:
            original = directory / relative
            expected_hash = original_notices.get(original.relative_to(ROOT).as_posix())
            if expected_hash and hashlib.sha256(original.read_bytes()).hexdigest() != expected_hash:
                errors.append('original third-party notice changed: ' + relative)
        for relative in ['README.md', 'LICENSE', *notices]:
            if relative not in packaged:
                errors.append('required file omitted: ' + relative)
        source_bytes = sum((directory / rel).stat().st_size for rel in packaged if (directory / rel).is_file())
        if source_bytes >= LIMIT:
            errors.append('10 MB source payload limit exceeded')
        report = {'name': name, 'directory': str(directory.relative_to(ROOT)),
                  'file_count': len(packaged), 'source_bytes': source_bytes,
                  'files': sorted(packaged), 'errors': errors}
        report['stripped_dev_dependencies'] = sorted(stripped)
        if args.archives:
            target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
            sizes, failures = archive_checks(directory, name, packaged, target / 'package' / f'{name}-{VERSION}.crate')
            report.update(sizes)
            errors.extend(failures)
        pending = []
        if args.pending_tests:
            approved_pending = json.loads((EVIDENCE / 'pending-test-references.json').read_text())
            stripped_pending = json.loads((EVIDENCE / 'pending-stripped-test-references.json').read_text())
            approved_errors = approved_pending.get(name, []) + stripped_pending.get(name, [])
            pending = [error for error in errors if error in approved_errors]
            errors[:] = [error for error in errors if error not in pending]
        report['pending_test_references'] = pending
        reports.append(report)
        print(f'{name}: {len(packaged)} files, {source_bytes} source bytes, {len(errors)} failures', flush=True)
        for error in errors:
            print('  ERROR:', error, flush=True)
        for error in pending:
            print('  PENDING TESTS:', error, flush=True)
    # Non-candidates must retain inherited/explicit publish=false.
    for member in root_data['workspace']['members']:
        path = ROOT / member
        if path.resolve() not in approved:
            value = manifest(path / 'Cargo.toml')['package'].get('publish')
            assert value is False or value == {'workspace': True}, f'private package publish setting: {member}'
    write_report(args.output, reports)
    return int(any(row['errors'] for row in reports))


if __name__ == '__main__':
    raise SystemExit(main())
