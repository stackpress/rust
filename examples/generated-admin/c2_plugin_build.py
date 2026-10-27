#!/usr/bin/env python3
"""Build a compiled formatted-definition replacement, then restore and rebuild its absence."""
import hashlib
import shutil
from c2_verify import LOGS, ROOT, TARGET, run


def hashes(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in root.rglob('*') if p.is_file()}


def build(prefix, destination):
    run(prefix+'-web', ['cargo', 'build', '-p', 'generated-admin-web', '--features', 'format', '--locked'])
    destination.mkdir(parents=True, exist_ok=True)
    shutil.copy2(TARGET/'debug/generated-admin-web', destination/'admin-web')
    for mode in ['hydrate', 'csr']:
        run(prefix+'-'+mode, ['cargo', 'build', '-p', 'generated-admin-client', '--target',
            'wasm32-unknown-unknown', '--features', mode+',format', '--locked'])
        assets = destination/'assets'/mode
        assets.mkdir(parents=True, exist_ok=True)
        run(prefix+'-'+mode+'-bindgen', ['wasm-bindgen', '--target', 'web', '--no-typescript',
            '--out-dir', str(assets), '--out-name', 'admin',
            str(TARGET/'wasm32-unknown-unknown/debug/generated_admin_client.wasm')])
        shutil.copy2(ROOT/'examples/generated-admin/bootstrap.js', assets/'start.js')


def main():
    root = ROOT/'examples/generated-admin/shared/src/generated-format'
    original = hashes(root)
    generate = ['cargo', 'run', '-p', 'generated-admin-tool', '--features', 'format', '--locked', '--',
        '--input', 'examples/generated-admin/schema-format.idea', '--output', str(root)]
    try:
        run('plugin-format-generate', generate+['--plugin'])
        run('plugin-format-check', generate+['--plugin', '--check'])
        build('plugin-format-build', LOGS/'plugin')
    finally:
        run('plugin-format-disable', generate)
        assert original == hashes(root), 'disabling must restore the entire marked output'
    build('plugin-disabled-build', LOGS)


if __name__ == '__main__':
    main()
