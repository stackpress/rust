#!/usr/bin/env python3
"""Check workspace-only lock movement, capability graphs and immutable asset identity."""
import hashlib
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[3]


def tree(package, features=None, wasm=False):
    """Read only normal runtime edges, excluding build/proc-macro host tooling."""
    command = ['cargo', 'tree', '--offline', '--locked', '-p', package, '--edges', 'normal,no-proc-macro']
    if features:
        command += ['--features', features]
    if wasm:
        command += ['--target', 'wasm32-unknown-unknown']
    return subprocess.check_output(command, cwd=ROOT, text=True)


def main():
    """Fail on external lock drift or a native capability in a browser contract graph."""
    before = tomllib.loads(subprocess.check_output(['git', 'show', 'trial/3:Cargo.lock'], cwd=ROOT, text=True))['package']
    after = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    external = lambda packages: sorted((p['name'], p['version'], p['source'], p.get('checksum')) for p in packages if 'source' in p)
    assert external(before) == external(after), 'external dependency changed'
    assert all(p['name'] != 'stackpress-framework-preferences' for p in after)
    print('PASS workspace-only lock update:', len(external(after)), 'external versions/sources/checksums unchanged')
    assert not (ROOT / 'crates/framework/framework-view/proofs/issue15-m0').exists(), 'parallel prototype retained'
    view = 'stackpress-framework-view'
    default = tree(view)
    assert all(name not in default for name in ['url v', 'libc v', 'stackpress-framework-language', 'stackpress-server', 'leptos v'])
    assert len(tree('stackpress-framework-language').splitlines()) == 1
    for feature in ['preferences', 'flash', 'preferences,flash']:
        browser = tree(view, feature, True)
        assert all(name not in browser for name in ['libc v', 'tokio v', 'stackpress-server', 'stackpress-lib ', 'tauri v', 'sqlx', 'stackpress-sql', 'stackpress-idea', 'stackpress-cli'])
    native = tree(view, 'native-file')
    assert 'libc v' in native and 'stackpress-server' not in native
    for mode in ['hydrate', 'csr']:
        for package in ['stackpress-example-profile-ui-client', 'stackpress-example-ui-preferences-client', 'stackpress-example-ui-gallery-client']:
            graph = tree(package, mode, True)
            assert all(name not in graph for name in ['libc v', 'stackpress-server', 'tokio v', 'tauri v', 'sqlx', 'stackpress-sql', 'stackpress-idea', 'stackpress-cli']), package
    print('PASS pure language, unchanged default view, opt-in native-file and six browser runtime graphs')
    source = ROOT / 'crates/framework/framework-view/src'
    asset = (source / 'initial-theme.js').read_bytes()
    name = re.search(r'BOOTSTRAP_FILE: &str = "([^"]+)"', (source / 'theme.rs').read_text())[1]
    assert name == 'initial-theme-' + hashlib.sha256(asset).hexdigest()[:16] + '.js'
    assert len(asset) <= 512
    text = asset.decode()
    assert text.count('matchMedia(') == 1
    assert all(token not in text for token in ['addEventListener', '.cookie', 'localStorage', 'fetch(', 'XMLHttpRequest'])
    print('PASS immutable one-shot asset:', name, len(asset), 'bytes; no persistence/listeners/fetch')


if __name__ == '__main__':
    main()
