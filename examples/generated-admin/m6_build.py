#!/usr/bin/env python3
"""Build the finite real-data host and matching browser modes in the sole M6 target."""
from pathlib import Path
import shutil
import subprocess
from m6_verify import LOGS, ROOT, TARGET, run


def main():
    """Keep copied finite artifacts after target cleanup; never change another proof's assets."""
    assert subprocess.check_output(['wasm-bindgen','--version'],text=True).strip()=='wasm-bindgen 0.2.129'
    run('generation-check',['cargo','run','-p','generated-admin-tool','--locked','--','emit','generate','--check'])
    run('format-generation-check',['cargo','run','-p','generated-admin-tool','--features','format','--locked','--','--input','examples/generated-admin/schema-format.idea','--output','examples/generated-admin/shared/src/generated-format','--check'])
    run('web-build',['cargo','build','-p','generated-admin-web','--locked'])
    shutil.copy2(TARGET/'debug/generated-admin-web',LOGS/'admin-web')
    for mode in ['hydrate','csr']:
        run(f'{mode}-build',['cargo','build','-p','generated-admin-client','--target','wasm32-unknown-unknown','--features',mode,'--locked'])
        assets=LOGS/'assets'/mode
        assets.mkdir(parents=True,exist_ok=True)
        run(f'{mode}-bindgen',['wasm-bindgen','--target','web','--no-typescript','--out-dir',str(assets),'--out-name','admin',str(TARGET/'wasm32-unknown-unknown/debug/generated_admin_client.wasm')])
        shutil.copyfile(ROOT/'examples/generated-admin/bootstrap.js',assets/'start.js')


if __name__=='__main__':
    main()
