#!/usr/bin/env python3
"""Finite owned builds/checks; no install, listener or shared target directory."""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ.get('UI_FORMAT_OUTPUT_DIR', '/private/tmp/s8-format'))
# An explicit Cargo target root can share native/WASM output under a disk budget.
NATIVE = ROOT / os.environ.get('CARGO_TARGET_DIR', 'target/s8-native')
WASM = ROOT / os.environ.get('CARGO_TARGET_DIR', 'target/s8-wasm')
ENV = dict(os.environ, CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
PACKAGE = 'stackpress-example-ui-format'
HISTORY = []


def run(name, command, target=NATIVE, extra=None, timeout=1200):
    """Retain exact command, deadline and status; fail immediately on nonzero exit."""
    env = dict(ENV, CARGO_TARGET_DIR=str(target), **(extra or {}))
    start = time.monotonic()
    with (OUT / (name + '.log')).open('w') as log:
        log.write('$ ' + ' '.join(map(str, command)) + '\n')
        log.flush()
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                stdout=log, stderr=subprocess.STDOUT, timeout=timeout)
    HISTORY.append(dict(name=name, command=list(map(str, command)), target=str(target),
                        seconds=round(time.monotonic()-start, 3), code=result.returncode))
    (OUT / 'commands.json').write_text(json.dumps(HISTORY, indent=2))
    result.check_returncode()
    print('PASS ' + name, flush=True)


def builds():
    """Measure all optional feature combinations in both real optimized WASM modes."""
    run('native-build', ['cargo', 'build', '--locked', '--offline', '-p', PACKAGE,
                         '--features', 'ssr,rich,format'])
    packet = subprocess.check_output([NATIVE / 'debug/ui-format'], cwd=ROOT, timeout=30)
    (OUT / 'native.json').write_bytes(packet)
    if subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip() != 'wasm-bindgen 0.2.129':
        raise RuntimeError('requires existing matching wasm-bindgen 0.2.129')
    sizes = []
    for mode in ('hydrate', 'csr'):
        for selected in ('base', 'rich', 'format', 'both'):
            features = mode + {'base': '', 'rich': ',rich', 'format': ',format', 'both': ',rich,format'}[selected]
            name = mode + '-' + selected
            # Explicit release settings make size reports reproducible, not production guarantees.
            release = dict(CARGO_PROFILE_RELEASE_OPT_LEVEL='z', CARGO_PROFILE_RELEASE_LTO='thin',
                           CARGO_PROFILE_RELEASE_CODEGEN_UNITS='1', CARGO_PROFILE_RELEASE_DEBUG='0',
                           CARGO_PROFILE_RELEASE_PANIC='abort')
            run(name+'-build', ['cargo', 'build', '--locked', '--offline', '--release', '-p', PACKAGE,
                                '--target', 'wasm32-unknown-unknown', '--features', features], WASM, release)
            assets = OUT / 'assets' / name
            assets.mkdir(parents=True, exist_ok=True)
            run(name+'-bindgen', ['wasm-bindgen', '--target', 'web', '--no-typescript', '--out-name', 'proof',
                                  '--out-dir', assets, WASM / 'wasm32-unknown-unknown/release/stackpress_example_ui_format.wasm'], WASM)
            files = {}
            for path in assets.iterdir():
                data = path.read_bytes()
                files[path.name] = dict(bytes=len(data), gzip=len(gzip.compress(data, mtime=0)), sha256=hashlib.sha256(data).hexdigest())
            sizes.append(dict(mode=mode, selected=selected, files=files))
            run(name+'-tree', ['cargo', 'tree', '--locked', '--offline', '-p', PACKAGE,
                               '--target', 'wasm32-unknown-unknown', '--features', features,
                               '--edges', 'normal,no-proc-macro'], WASM)
            tree = (OUT / (name+'-tree.log')).read_text().splitlines()[1:]
            for line in tree:
                if any(word in line for word in ('stackpress-server', 'stackpress-sql', 'sqlx', 'tokio v', 'tauri v', 'stackpress-cli', 'stackpress-idea')):
                    raise RuntimeError('browser dependency leak: '+line)
                if selected == 'base' and any(word in line for word in ('icu_decimal ', 'icu_datetime ', 'icu_locale ', 'phonenumber ', 'ammonia ', 'pulldown-cmark ')):
                    raise RuntimeError('disabled dependency leak: '+line)
            (OUT / 'sizes.json').write_text(json.dumps(sizes, indent=2))
    # Registry metadata is captured from the exact resolved workspace lockfile.
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--offline', '--format-version', '1',
                                                  '--filter-platform', 'wasm32-unknown-unknown',
                                                  '--features', PACKAGE+'/format,'+PACKAGE+'/rich'], cwd=ROOT))
    before = __import__('tomllib').loads((OUT / 'before.lock').read_text())
    previous = {(p['name'], p['version']) for p in before['package']}
    packages = [dict(name=p['name'], version=p['version'], license=p['license'], rust_version=p['rust_version'])
                for p in metadata['packages']]
    (OUT / 'dependencies.json').write_text(json.dumps(packages, indent=2))
    for package in metadata['packages']:
        if package['name'].startswith('icu_') and package['name'] != 'icu_pattern':
            if package['version'] != '2.4.0' or package['license'] != 'Unicode-3.0':
                raise RuntimeError('ICU version/license drift: '+package['name'])
            if not package['rust_version'] or tuple(map(int, package['rust_version'].split('.'))) > (1,96,0):
                raise RuntimeError('unsupported ICU MSRV: '+package['name'])
    after = __import__('tomllib').loads((ROOT / 'Cargo.lock').read_text())
    current = {(p['name'], p['version']) for p in after['package']}
    (OUT / 'lock-delta.json').write_text(json.dumps(dict(added=sorted(current-previous), removed=sorted(previous-current)), indent=2))
    if previous-current:
        raise RuntimeError('existing package version changed or removed')


def verify():
    """Valid separate render features, MSRV, warning-clean docs/lints and workspace suite."""
    scope = ['-p', 'stackpress-framework-ui', '-p', PACKAGE]
    selected = ['--features', 'ssr,rich,format']
    for name, args in [
        ('fmt', ['fmt', '--all', '--', '--check']),
        ('tests', ['test', '--locked', '--offline', *scope, *selected]),
        ('ssr-tests', ['test', '--locked', '--offline', '-p', 'stackpress-framework-ui', '--features', 'ssr']),
        ('clippy', ['clippy', '--locked', '--offline', *scope, *selected, '--all-targets', '--', '-D', 'warnings']),
        ('msrv', ['+1.96.0', 'check', '--locked', '--offline', *scope, *selected, '--all-targets']),
        ('workspace-tests', ['test', '--workspace', '--all-targets', '--locked', '--offline']),
        ('workspace-docs', ['test', '--workspace', '--doc', '--locked', '--offline']),
    ]:
        run(name, ['cargo', *args])
    run('rustdoc', ['cargo', 'doc', '--locked', '--offline', *scope, *selected, '--no-deps'], extra={'RUSTDOCFLAGS': '-D warnings'})
    for mode in ('hydrate', 'csr'):
        run(mode+'-clippy', ['cargo', 'clippy', '--locked', '--offline', '-p', PACKAGE, '--target', 'wasm32-unknown-unknown',
                             '--features', mode+',rich,format', '--', '-D', 'warnings'], WASM)
    run('kb-after', ['python3', '.agents/scripts/validate-agent-workspace.py'])


def main():
    """Choose an explicit finite stage; cleanup removes only these two owned trees."""
    parser = argparse.ArgumentParser()
    parser.add_argument('stage', choices=['build', 'verify', 'clean'])
    stage = parser.parse_args().stage
    OUT.mkdir(parents=True, exist_ok=True)
    if not (OUT / 'before.lock').exists():
        shutil.copyfile(ROOT / 'Cargo.lock', OUT / 'before.lock')
    if (OUT / 'commands.json').exists():
        HISTORY.extend(json.loads((OUT / 'commands.json').read_text()))
    if stage == 'build':
        builds()
    elif stage == 'verify':
        verify()
    else:
        for target in (NATIVE, WASM):
            run('clean-'+target.name, ['cargo', 'clean'], target)
            if target.exists():
                shutil.rmtree(target)


if __name__ == '__main__':
    main()
