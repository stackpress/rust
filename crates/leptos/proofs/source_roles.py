#!/usr/bin/env python3
"""Record this outer proof once. Cached tools, intercepted Chromium, no listeners/install."""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / 'target/00040/source-roles-proof'
CONSUMER = OUT / 'consumer'
TARGET = ROOT / 'target/00040/consumer-build'
PLAYWRIGHT = '/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright'


def run(command, *, output=None, timeout=1200):
    """Keep the proof finite and forward Cargo stages to its enclosing recorder."""
    print('COMMAND:', *map(str, command), flush=True)
    env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), SOURCE_ROLES_PLAYWRIGHT=PLAYWRIGHT)
    with output.open('w') if output else open(os.devnull, 'w') as stream:
        with subprocess.Popen(list(map(str, command)), cwd=CONSUMER, env=env,
                              start_new_session=True, stdout=stream if output else None) as child:
            try:
                status = child.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                # Reap this proof's entire process group; a hung compiler/browser cannot linger.
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
                raise
            if status:
                raise subprocess.CalledProcessError(status, command)


def registry(lock):
    """No proof may introduce a registry identity outside the unchanged workspace lock."""
    return {(item['name'], item['version'], item['source'], item.get('checksum'))
            for item in tomllib.loads(lock.decode())['package']
            if item.get('source', '').startswith('registry+')}


def prepare():
    """Create an independent application using only already resolved workspace dependencies."""
    (CONSUMER / 'src/bin').mkdir(parents=True, exist_ok=True)
    path = lambda name: str(ROOT / name)
    manifest = f'''[workspace]
[package]
name = "source-roles-consumer"
version = "0.0.0"
edition = "2024"
build = "build.rs"
[lib]
crate-type = ["rlib", "cdylib"]
[[bin]]
name = "document"
path = "src/bin/document.rs"
required-features = ["ssr"]
[features]
default = []
server = []
ssr = ["leptos/ssr", "stackpress-leptos/ssr", "stackpress-framework-admin/source-ssr"]
hydrate = ["leptos/hydrate", "stackpress-leptos/hydrate", "stackpress-framework-admin/hydrate"]
csr = ["leptos/csr", "stackpress-leptos/csr", "stackpress-framework-admin/csr"]
[dependencies]
leptos = {{version = "=0.8.22", default-features = false}}
stackpress-leptos = {{path = "{path('crates/leptos')}"}}
stackpress-framework-admin = {{path = "{path('crates/framework/framework-admin')}", features = ["source"]}}
stackpress-framework-schema = {{path = "{path('crates/framework/framework-schema')}"}}
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
stackpress-framework = {{path = "{path('crates/framework/framework')}", features = ["native"]}}
stackpress-framework-sql = {{path = "{path('crates/framework/framework-sql')}", features = ["source", "server"]}}
stackpress-sql = {{path = "{path('crates/sql/sql')}"}}
stackpress-lib = {{path = "{path('crates/lib')}"}}
stackpress-server = {{path = "{path('crates/server/server')}"}}
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "=0.2.129"
js-sys = "=0.3.106"
web-sys = {{version = "=0.3.106", features = ["Window", "Document", "Element", "HtmlElement"]}}
[build-dependencies]
stackpress-framework = {{path = "{path('crates/framework/framework')}", features = ["codegen"]}}
stackpress-framework-admin = {{path = "{path('crates/framework/framework-admin')}", features = ["codegen"]}}
stackpress-framework-schema = {{path = "{path('crates/framework/framework-schema')}", features = ["codegen"]}}
stackpress-idea-parser = {{path = "{path('crates/idea/idea-parser')}"}}
'''
    (CONSUMER / 'Cargo.toml').write_text(manifest)
    for source, destination in [('consumer.rs', 'src/lib.rs'), ('document.rs', 'src/bin/document.rs'), ('generate.rs', 'build.rs'), ('seed.json', 'src/seed.json'), ('data.rs', 'src/bin/data.rs')]:
        shutil.copy2(HERE / 'source_roles' / source, CONSUMER / destination)
    shutil.copy2(HERE / 'source_roles/schema.idea', CONSUMER / 'schema.idea')
    lock = CONSUMER / 'Cargo.lock'
    if not lock.exists():
        lock.write_bytes((ROOT / 'Cargo.lock').read_bytes())
        # One offline subset resolution for the new local package; no update/generate-lockfile.
    run(['cargo', 'tree', '--offline', '--depth', '0'])
    assert registry(lock.read_bytes()) <= registry((ROOT / 'Cargo.lock').read_bytes())


def build():
    """Compile aggregate-generated native client/roles and separate hydrate/CSR artifacts."""
    prepare()
    run(['cargo', 'build', '--bin', 'document', '--features', 'ssr', '--locked', '--offline'])
    run([TARGET / 'debug/document'], output=OUT / 'page.html', timeout=30)
    run(['cargo', 'build', '--bin', 'data', '--features', 'ssr', '--locked', '--offline'])
    run([TARGET / 'debug/data'], output=OUT / 'data.json', timeout=30)
    actual = json.loads((OUT / 'data.json').read_text())
    source = Path('/Users/cblanquera/server/projects/stackpress/frui/src/data')
    for name in ['countries', 'currencies']:
        raw = (source / (name + '.ts')).read_text()
        assert actual[name] == json.loads(raw.removeprefix('export default ').strip().removesuffix(';')), name
    raw = (source / 'colors.ts').read_text()
    assert dict(actual['colors']) == json.loads(raw[raw.index('{'):raw.index('};') + 1])
    print('PASS compiled data match all original country/currency/color records', flush=True)
    assert subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip() == 'wasm-bindgen 0.2.129'
    for mode in ['hydrate', 'csr']:
        run(['cargo', 'build', '--lib', '--target', 'wasm32-unknown-unknown', '--features', mode, '--locked', '--offline'])
        assets = OUT / mode
        assets.mkdir(exist_ok=True)
        run(['wasm-bindgen', '--target', 'web', '--no-typescript', '--out-dir', assets,
             '--out-name', 'source_roles', TARGET / 'wasm32-unknown-unknown/debug/source_roles_consumer.wasm'])
    print('PASS native client/SSR plus hydrate/CSR generated consumers and freshness', flush=True)


def browser():
    """Every request is answered by page.route using only the pinned cached Playwright."""
    run(['node', HERE / 'source_roles/browser.cjs', OUT], timeout=180)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['build', 'browser', 'all', 'lint'])
    action = parser.parse_args().action
    original = (ROOT / 'Cargo.lock').read_bytes()
    OUT.mkdir(parents=True, exist_ok=True)
    try:
        if action in ['build', 'all']:
            build()
        if action in ['browser', 'all']:
            browser()
        if action == 'lint':
            run(['cargo', 'clippy', '--all-targets', '--features', 'ssr', '--locked', '--offline', '--', '-D', 'warnings'])
    finally:
        assert (ROOT / 'Cargo.lock').read_bytes() == original, 'workspace lock changed: stop'
    print(json.dumps({'action': action, 'root_lock_unchanged': True, 'registry_tuples': len(registry(original))}), flush=True)


if __name__ == '__main__':
    main()
