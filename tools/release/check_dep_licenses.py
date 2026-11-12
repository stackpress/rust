#!/usr/bin/env python3
"""Check locked dependency licences offline, with three dated exceptions (G62).

The allowlist records exact existing declarations, including legacy slash syntax;
this tool neither interprets licence compatibility nor changes the frozen list.
"""
from collections import Counter
from dataclasses import dataclass
import os
from pathlib import Path
import tomllib

from common import ROOT

ALLOWLIST = ROOT / 'tools/release/dependency-licenses.toml'
# Build outputs and agent reference material are not source consumer workspaces.
SKIP_DIRECTORIES = {'target', 'node_modules', '__pycache__'}
# Coordinator-authorized, dated metadata covers only these uncached lock entries.
METADATA_PACKAGES = {('atomic-polyfill', '1.0.3'), ('curve25519-dalek-derive', '0.1.1'),
                     ('fiat-crypto', '0.2.9')}
CRATES_IO = 'registry+https://github.com/rust-lang/crates.io-index'


class SourceNotCached(ValueError):
    """Distinguish an absent source from an unreadable or invalid cached source."""


def lockfiles(root):
    """Find root and nested source locks, including newly added fuzz workspaces."""
    required = root / 'Cargo.lock'
    if not required.is_file():
        raise ValueError(f'missing root lock: {required}')
    locks = []
    for directory, children, files in os.walk(root):
        children[:] = sorted(name for name in children
                             if not name.startswith('.') and name not in SKIP_DIRECTORIES)
        if 'Cargo.lock' in files:
            locks.append(Path(directory) / 'Cargo.lock')
    return sorted(locks)


def registry_packages(locks):
    """Count each name/version/registry identity once across all consumers."""
    packages = set()
    for lock in locks:
        data = tomllib.loads(lock.read_text(encoding='utf-8'))
        for package in data['package']:
            source = package.get('source', '')
            if source.startswith('registry+'):
                packages.add((package['name'], package['version'], source))
    return sorted(packages)


def cached_license(registry_src, name, version):
    """Read exact cached declarations; missing or conflicting sources fail closed."""
    manifests = sorted(registry_src.glob(f'*/{name}-{version}/Cargo.toml'))
    if not manifests:
        raise SourceNotCached('source is not cached')
    expressions = set()
    for manifest in manifests:
        package = tomllib.loads(manifest.read_text(encoding='utf-8'))['package']
        if (package.get('name'), package.get('version')) != (name, version):
            raise ValueError(f'cached manifest identity mismatch: {manifest}')
        expression = package.get('license')
        if not isinstance(expression, str) or not expression.strip():
            # A licence filename is not an SPDX expression. Report it for review
            # rather than guessing an identifier from arbitrary licence prose.
            license_file = package.get('license-file')
            if license_file:
                path = manifest.parent / license_file
                if not path.is_file():
                    raise ValueError(f'license-file is missing: {path}')
                path.read_text(encoding='utf-8')
                raise ValueError(f'license-file {license_file!r} has no SPDX expression')
            raise ValueError('cached manifest has no license or license-file')
        expressions.add(expression)
    if len(expressions) != 1:
        raise ValueError(f'conflicting cached licence expressions: {sorted(expressions)}')
    return expressions.pop()


def package_license(registry_src, name, version, source, metadata):
    """Prefer cached source; return provenance only for an approved absent source."""
    try:
        return cached_license(registry_src, name, version), None
    except SourceNotCached:
        # A malformed cached source never reaches this exception handler.
        entry = metadata.get(f'{name}@{version}', {})
        if (name, version) not in METADATA_PACKAGES or source != CRATES_IO or not entry:
            raise
        expression = entry['license']
        if not isinstance(expression, str) or not expression.strip() or not entry.get('source'):
            raise ValueError('incomplete registry metadata provenance')
        return expression, f'{name} {version}: {entry["source"]}'


@dataclass
class Audit:
    """Keep incomplete inventory counts distinct from accepted declarations."""
    locks: int
    packages: int
    counts: Counter
    errors: list[str]
    metadata_sources: list[str]


def audit(root, registry_src, allowed, metadata=None):
    """Audit every registry identity and accumulate independent package failures."""
    locks = lockfiles(root)
    packages = registry_packages(locks)
    counts = Counter()
    errors = []
    metadata_sources = []
    for name, version, source in packages:
        label = f'{name} {version} ({source})'
        try:
            expression, provenance = package_license(registry_src, name, version, source, metadata or {})
        except (OSError, ValueError, KeyError) as error:
            errors.append(f'{label}: {error}')
            continue
        if provenance:
            metadata_sources.append(provenance)
        counts[expression] += 1
        if expression not in allowed:
            errors.append(f'{label}: unlisted licence expression {expression!r}')
    return Audit(len(locks), len(packages), counts, errors, metadata_sources)


def main():
    """Print reproducible expression counts and return nonzero on any failure."""
    try:
        config = tomllib.loads(ALLOWLIST.read_text(encoding='utf-8'))
        allowed = set(config['expressions'])
        cargo_home = Path(os.environ.get('CARGO_HOME', Path.home() / '.cargo')).expanduser()
        result = audit(ROOT, cargo_home / 'registry/src', allowed, config.get('registry-metadata'))
    except (OSError, ValueError, KeyError) as error:
        print(f'Dependency licences: FAIL: {error}')
        return 1
    for expression, count in sorted(result.counts.items()):
        print(f'{count:4d}  {expression}')
    for error in result.errors:
        print('FAIL:', error)
    for source in result.metadata_sources:
        print('METADATA:', source)
    print(f'Dependency licences: {result.packages} unique registry packages in '
          f'{result.locks} locks; {sum(result.counts.values())} declarations read; '
          f'{len(result.metadata_sources)} dated metadata fallbacks; '
          f'{len(result.counts)} expressions; {len(result.errors)} failures')
    return int(bool(result.errors))


if __name__ == '__main__':
    raise SystemExit(main())
