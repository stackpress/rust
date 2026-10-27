#!/usr/bin/env python3
"""Finite SSR/WASM proofs and plugin rebuilds, using one owned Cargo target directory."""
import shutil
import subprocess
from pathlib import Path

from verify import ROOT, LOGS, TARGET, run


def main():
    """Rebuild the compiled plugin selection, finally restoring tracked default output."""
    if subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip() != 'wasm-bindgen 0.2.129':
        raise RuntimeError('requires the already installed wasm-bindgen 0.2.129')
    try:
        for variant, plugin in [('base', False), ('plugin', True), ('disabled', False)]:
            run(f'{variant}-generate', ['cargo', 'run', '-p', 'generated-admin-tool', '--locked', '--', 'emit', 'generate'] + (['--plugin'] if plugin else []))
            run(f'{variant}-web-build', ['cargo', 'build', '-p', 'generated-admin-web', '--locked'])
            binary = LOGS / f'{variant}-web'
            shutil.copy2(TARGET / 'debug/generated-admin-web', binary)
            for mode in ['hydrate', 'csr']:
                run(f'{variant}-{mode}-build', ['cargo', 'build', '-p', 'generated-admin-client', '--target', 'wasm32-unknown-unknown', '--features', mode, '--locked'])
                assets = LOGS / 'assets' / variant / mode
                assets.mkdir(parents=True, exist_ok=True)
                run(f'{variant}-{mode}-bindgen', ['wasm-bindgen', '--target', 'web', '--no-typescript', '--out-dir', str(assets), '--out-name', 'admin', str(TARGET / 'wasm32-unknown-unknown/debug/generated_admin_client.wasm')])
                shutil.copyfile(ROOT / 'examples/generated-admin/bootstrap.js', assets / 'start.js')
    finally:
        run('restore-default-generation', ['cargo', 'run', '-p', 'generated-admin-tool', '--locked', '--', 'emit', 'generate'])


if __name__ == '__main__':
    main()
