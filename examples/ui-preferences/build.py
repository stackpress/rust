#!/usr/bin/env python3
"""Finite builds with installed tools and explicit native/WASM target locations."""
import os
from pathlib import Path
import subprocess
import shutil

ROOT = Path(__file__).resolve().parents[2]
LOGS = Path(os.environ.get('PREFERENCES_LOGS', '/private/tmp/s8-lane-b'))
# Reuse the profile proof's target selectors so one parent can cap both trees.
NATIVE = Path(os.environ.get('PROFILE_UI_NATIVE_TARGET', ROOT / 'target/s8-native'))
WASM = Path(os.environ.get('PROFILE_UI_TARGET', ROOT / 'target/s8-wasm'))


def run(name, command, target):
    """Record a finite child command and fail at its actual nonzero exit."""
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    with (LOGS / f'{name}.log').open('w') as log:
        log.write('$ ' + ' '.join(map(str, command)) + '\n')
        log.flush()
        subprocess.run(list(map(str, command)), cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=600, check=True)
    print('PASS ' + name, flush=True)


def main():
    """Keep hydrate and CSR builds separate; no package/tool installation occurs."""
    if subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip() != 'wasm-bindgen 0.2.129':
        raise RuntimeError('requires already-installed wasm-bindgen 0.2.129')
    LOGS.mkdir(parents=True, exist_ok=True)
    run('preferences-native-build', ['cargo', 'build', '--locked', '-p', 'stackpress-example-ui-preferences-web'], NATIVE)
    for mode in ('hydrate', 'csr'):
        run(f'preferences-{mode}-build', ['cargo', 'build', '--locked', '-p', 'stackpress-example-ui-preferences-client', '--target', 'wasm32-unknown-unknown', '--features', mode], WASM)
        output = LOGS / 'assets' / mode
        if output.exists():
            shutil.rmtree(output)
        output.mkdir(parents=True)
        run(f'preferences-{mode}-bindgen', ['wasm-bindgen', '--target', 'web', '--no-typescript', '--out-dir', output, '--out-name', 'preferences', WASM / 'wasm32-unknown-unknown/debug/stackpress_example_ui_preferences_client.wasm'], WASM)
        shutil.copyfile(ROOT / 'examples/ui-preferences/bootstrap.js', output / 'start.js')
        # Immutable bootstrap filename comes from the same public Rust asset identity.
        import re
        theme = ROOT / 'crates/framework/framework-view/src'
        filename = re.search(r'BOOTSTRAP_FILE: &str = "([^"]+)"', (theme / 'theme.rs').read_text())[1]
        shutil.copyfile(theme / 'initial-theme.js', output / filename)


if __name__ == '__main__':
    main()
