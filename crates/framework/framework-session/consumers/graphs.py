#!/usr/bin/env python3
"""Inspect isolated normal feature graphs; --check also compiles each compatible native/WASM mode."""
import argparse
from pathlib import Path
import subprocess
import tomllib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
flags = ['--no-default-features', '--locked', '--offline']
features = tomllib.loads(Path('crates/framework/framework-session/Cargo.toml').read_text())['features']
expected = {'default', 'native', 'sql', 'server', 'auth-server', 'leptos', 'leptos-ssr', 'browser', 'leptos-hydrate', 'leptos-csr'}
assert set(features) == expected
native_forbidden = {'ring', 'jsonwebtoken', 'aes', 'cbc', 'sha3', 'stackpress-server', 'stackpress-sql',
                    'stackpress-framework-sql', 'stackpress-framework-schema', 'stackpress-framework-csrf',
                    'stackpress-framework-email', 'lettre', 'qrcode', 'png', 'tokio'}


def graph(package, selected=None, target=None):
    """No dev edges or workspace package aggregation may hide a leaked optional dependency."""
    command = ['cargo', 'tree', '-p', package, *flags, '-e', 'normal', '--prefix', 'none']
    if selected:
        command += ['--features', selected]
    if target:
        command += ['--target', target]
    print('COMMAND:', *command, flush=True)
    output = subprocess.check_output(command, text=True)
    names = {line.split()[0] for line in output.splitlines() if line}
    print(f'GRAPH {package}/{selected or "default"}/{target or "native"}: {sorted(names)}', flush=True)
    return names


def check(package, selected=None, target=None):
    """Renderer modes compile separately; never select renderer --all-features."""
    if not args.check:
        return
    command = ['cargo', 'check', '--lib', '-p', package, *flags]
    if selected:
        command += ['--features', selected]
    if target:
        command += ['--target', target]
    print('COMMAND:', *command, flush=True)
    subprocess.run(command, check=True)


for selected in [None, 'native', 'sql', 'server', 'auth-server', 'leptos', 'leptos-ssr', 'browser']:
    names = graph('stackpress-framework-session', selected)
    if selected in [None, 'browser']:
        assert not names.intersection(native_forbidden)
    if selected in ['native', 'sql', 'server', 'auth-server']:
        assert not names.intersection({'leptos', 'leptos_meta', 'stackpress-leptos', 'web-sys', 'phonenumber'})
    check('stackpress-framework-session', selected)

names = graph('stackpress')
assert 'stackpress-lib' in names and 'stackpress-framework-session' not in names
assert not names.intersection({'stackpress-server', 'stackpress-framework-email', 'stackpress-framework-csrf'})
for selected in [None, *sorted(expected-{'default'})]:
    facade = 'framework-session' + (f'-{selected}' if selected else '')
    # Browser renderer modes are verified on their actual target below.
    target = 'wasm32-unknown-unknown' if selected in ['leptos-hydrate', 'leptos-csr'] else None
    names = graph('stackpress', facade, target)
    assert 'stackpress-framework-session' in names
    check('stackpress', facade, target)

for selected in [None, 'auth-server', 'leptos-hydrate', 'leptos-csr']:
    names = graph('stackpress-framework-session', selected, 'wasm32-unknown-unknown')
    assert not names.intersection(native_forbidden), sorted(names.intersection(native_forbidden))
    check('stackpress-framework-session', selected, 'wasm32-unknown-unknown')
print('All session/facade feature graphs preserve native/browser/default boundaries.', flush=True)
