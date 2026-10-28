#!/usr/bin/env python3
"""Finite milestone checks and regeneration, under the outer benchmark recorder."""
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]


def run(*args, **kwargs):
    """Keep every child finite and its Cargo timing stream visible."""
    subprocess.run(args, check=True, timeout=600, **kwargs)


if __name__ == '__main__':
    profiles = ['-p','stackpress-example-profile-ui-shared','-p','stackpress-example-profile-ui-web','-p','stackpress-example-profile-ui-client','-p','stackpress-example-profile-ui-native']
    run('cargo','fmt','--all')
    if '--wasm' not in sys.argv:
        run('cargo','metadata','--offline','--format-version','1', stdout=subprocess.DEVNULL)
        run('cargo','fmt','--all')
        run('cargo','test','--offline','--locked',*profiles,'--all-targets')
        run('cargo','clippy','--offline','--locked',*profiles,'--all-targets','--','-D','warnings')
        run('cargo','check','--offline','--locked','--workspace','--all-targets')
        scopes = ['-p','stackpress-framework-view','-p','stackpress-framework-language','-p','stackpress-framework-ui']
        features='stackpress-framework-view/native-file,stackpress-framework-view/flash,stackpress-framework-ui/stream,stackpress-framework-ui/rich'
        run('cargo','doc','--offline','--locked',*scopes,'--features',features,'--no-deps', env=dict(os.environ,RUSTDOCFLAGS='-D warnings'))
        run('cargo','build','--offline','--locked','-p','stackpress-example-profile-ui-web')
    for mode in ['hydrate','csr']:
        run(sys.executable,'examples/profile-ui/build-client.py',mode)
        run(sys.executable,'examples/profile-ui/asset_manifest.py','check',mode)
        clients=['-p','stackpress-example-profile-ui-client','-p','stackpress-example-ui-preferences-client','-p','stackpress-example-ui-gallery-client']
        run('cargo','clippy','--offline','--locked',*clients,'--target','wasm32-unknown-unknown','--features',mode,'--','-D','warnings')
        run('cargo','check','--offline','--locked','-p','stackpress','--target','wasm32-unknown-unknown','--features',f'framework-view-preferences,framework-view-flash,framework-ui-{mode}')
    run(sys.executable,'examples/ui-gallery/build.py')
    run('cargo','check','--offline','--locked','-p','stackpress','--features','framework-view-native-file,framework-view-flash')
    run('cargo','fmt','--all','--','--check')
    run(sys.executable,'examples/ui-preferences/proofs/contracts.py')
    run(sys.executable,'-m','unittest','discover','-s','examples/ui-preferences/proofs','-p','test_supervise.py')
