#!/usr/bin/env python3
"""Inspect isolated facade/consumer normal graphs; --check also compiles each selection."""
import argparse
import os
from pathlib import Path
import subprocess
import tomllib

BASE = Path(__file__).resolve().parent
ROOT = BASE.parents[3]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
LOCKED = ['--locked', '--offline']
BUILD_HOST = {'stackpress-idea', 'stackpress-idea-parser', 'stackpress-idea-transformer'}
NATIVE = {'stackpress-server', 'stackpress-server-http', 'stackpress-sql', 'stackpress-sql-sqlite',
          'stackpress-framework-sql', 'stackpress-framework-email', 'stackpress-framework-csrf',
          'ring', 'jsonwebtoken', 'aes', 'cbc', 'sha3', 'lettre', 'tokio', 'qrcode', 'png'}


def inspect(flags, feature=None, target=None, env=None):
    """Normal edges only: neither dev dependency unification nor a workspace graph can mask leaks."""
    extra = (['--features', feature] if feature else []) + (['--target', target] if target else [])
    command = ['cargo', 'tree', *flags, *LOCKED, *extra, '-e', 'normal', '--prefix', 'none']
    print('COMMAND:', *command, flush=True)
    output = subprocess.check_output(command, text=True, env=env)
    names = {line.split()[0] for line in output.splitlines() if line}
    print(f'GRAPH {feature or "default"}/{target or "native"}: {len(names)} packages', flush=True)
    if args.check:
        command = ['cargo', 'check', *flags, *LOCKED, *extra]
        print('COMMAND:', *command, flush=True)
        subprocess.run(command, check=True, env=env)
    return names


facade = tomllib.loads((ROOT/'Cargo.toml').read_text())['features']
names = inspect(['-p', 'stackpress', '--no-default-features'])
assert {name for name in names if name.startswith('stackpress')} == {'stackpress', 'stackpress-lib'}
new = sorted(name for name in facade if name == 'framework' or name.startswith('framework-aggregate-')
             or name.startswith(('framework-api', 'framework-csrf', 'framework-email')))
for feature in new:
    target = 'wasm32-unknown-unknown' if feature.endswith(('hydrate', 'csr')) else None
    names = inspect(['-p', 'stackpress', '--no-default-features'], feature, target)
    if feature in ['framework', 'framework-aggregate-ssr', 'framework-aggregate-hydrate', 'framework-aggregate-csr']:
        assert {name for name in names if name.startswith('stackpress')} == {'stackpress', 'stackpress-lib', 'stackpress-framework'}, names
    if 'codegen' not in feature:
        assert not names.intersection(BUILD_HOST), (feature, names.intersection(BUILD_HOST))

target = Path(os.environ['CARGO_TARGET_DIR']).resolve()
for kind in ['full', 'direct', 'headless', 'api-disabled', 'disabled', 'facade', 'website', 'build-host']:
    flags = ['--manifest-path', str(BASE/kind/'Cargo.toml')]
    # Consumer compilation/execution belongs to run.py and its distinct targets.
    selected = 'ssr' if kind == 'website' else None
    check_before = args.check
    args.check = False
    names = inspect(flags, selected)
    args.check = check_before
    if kind == 'disabled':
        assert {name for name in names if name.startswith('stackpress')} == {'stackpress', 'stackpress-lib'}
    elif kind == 'api-disabled':
        assert 'stackpress-framework-api' not in names
    elif kind == 'build-host':
        assert BUILD_HOST <= names and 'stackpress-sql-sqlite' not in names
    if kind != 'build-host':
        assert not names.intersection(BUILD_HOST), (kind, names.intersection(BUILD_HOST))

for mode in ['hydrate', 'csr']:
    env = {**os.environ, 'CARGO_TARGET_DIR': str(target/'consumer-aggregate-website')}
    names = inspect(['--manifest-path', str(BASE/'website/Cargo.toml')], mode,
                    'wasm32-unknown-unknown', env)
    assert {'stackpress', 'stackpress-framework', 'stackpress-framework-session',
            'stackpress-framework-api', 'stackpress-framework-admin'} <= names
    assert not names.intersection(NATIVE | BUILD_HOST), names.intersection(NATIVE | BUILD_HOST)
print('All aggregate/facade/consumer normal graph boundaries passed.', flush=True)
