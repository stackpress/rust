#!/usr/bin/env python3
"""Build default, every single facade feature, and the full compatible SSR set.

The consumer imports every public facade re-export and builds against the path
facade. This is G61's build/import/dependency-tree proof, not a feature powerset.
Hydrate and CSR are checked individually; renderer --all-features is never used.
"""
import argparse
import os
from pathlib import Path
import re

from common import ROOT, EVIDENCE, manifest, run, seed_lock, write_report


def imports(source):
    """Turn the advertised facade re-exports into explicit, feature-scoped imports."""
    rows = []
    # Each alias in the facade is guarded by one direct feature or is unconditional.
    for match in re.finditer(r'(?:(#\[cfg\(feature = "([^"]+)"\)\])\s+)?pub use (stackpress_\w+) as (\w+);', source):
        feature, alias = match[2], match[4]
        rows.append((feature, alias))
    group = re.search(r'pub use stackpress_lib::\{([^}]+)\};', source, re.S)
    assert group, 'foundation re-export group missing'
    rows.extend((None, name.strip()) for name in group[1].split(',') if name.strip())
    assert len(rows) == len(set(rows)), 'duplicate facade export'
    code = ['//! Compile proof for every advertised stackpress namespace and direct import.']
    for feature, alias in rows:
        if feature:
            code.append(f'#[cfg(feature = "{feature}")]')
        code.append(f'use stackpress::{alias} as _;')
    return '\n'.join(code) + '\n', rows


def main():
    """Use one root-seeded scratch lock and warm target for the whole feature matrix."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--only', choices=['default', 'singles', 'full'])
    args = parser.parse_args()
    features = list(manifest(ROOT / 'Cargo.toml')['features'])
    features.remove('default')
    target = Path(os.environ['CARGO_TARGET_DIR']).resolve()
    consumer = target / 'facade-consumer'
    consumer.mkdir(parents=True, exist_ok=True)
    source, paths = imports((ROOT / 'src/lib.rs').read_text())
    (consumer / 'src').mkdir(exist_ok=True)
    # Unused imports are deliberate evidence; deny warnings catches missing feature mappings.
    (consumer / 'src/lib.rs').write_text('#![deny(warnings)]\n#![allow(unused_imports)]\n' + source)
    forwarded = '\n'.join(f'"{feature}" = ["stackpress/{feature}"]' for feature in features)
    (consumer / 'Cargo.toml').write_text(
        '[workspace]\n[package]\nname = "stackpress-release-facade-consumer"\nversion = "0.0.0"\n'
        'edition = "2024"\npublish = false\n[dependencies]\n'
        f'stackpress = {{ path = "{ROOT}", default-features = false }}\n'
        '[features]\ndefault = []\n' + forwarded + '\n')
    env = dict(os.environ, CARGO_TARGET_DIR=str(target / 'facade-build'))
    seed_lock(consumer, env)
    flags = ['--manifest-path', str(consumer / 'Cargo.toml'), '--locked', '--offline', '--lib']
    cases = [('default', [])] if args.only in (None, 'default') else []
    if args.only in (None, 'singles'):
        cases += [(feature, ['--features', feature]) for feature in features]
    # Include every native facade capability while choosing SSR for each renderer.
    full = [feature for feature in features if not any(part in feature for part in ('hydrate', 'csr'))]
    if args.only in (None, 'full'):
        cases.append(('compatible-full-ssr', ['--features', ','.join(full)]))
    passed = []
    for name, selection in cases:
        print('FACADE:', name, flush=True)
        run(['cargo', 'build', *flags, *selection], env=env)
        passed.append(name)
    command = ['cargo', 'tree', '-p', 'stackpress', '--no-default-features', '--locked',
               '--offline', '-e', 'normal', '--prefix', 'none']
    graph = run(command, text=True, capture_output=True).stdout
    print(graph, end='', flush=True)
    names = {line.split()[0] for line in graph.splitlines() if line.startswith('stackpress')}
    assert names == {'stackpress', 'stackpress-lib'}, names
    write_report(EVIDENCE / ('facade' + ('-' + args.only if args.only else '') + '.json'),
                 {'passed': passed, 'single_feature_count': len(features),
                  'compatible_full_features': full, 'imports': paths, 'default_stackpress_tree': sorted(names)})
    print(f'Facade: {len(passed)} builds; {len(paths)} advertised paths; default dependency tree passed')


if __name__ == '__main__':
    main()
