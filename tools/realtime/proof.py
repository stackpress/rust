#!/usr/bin/env python3
"""Build independent render modes and run finite native/browser transport proofs. Recorder-wrap once."""
import argparse
import os
from pathlib import Path
import subprocess
import tomllib

ROOT=Path(__file__).resolve().parents[2]
# Match the release gate's effort directory so independent targets cannot share outputs.
BASE=Path(os.environ['CARGO_TARGET_DIR']).resolve().parent
OUT=BASE/'browser-artifacts'
MANIFEST=ROOT/'examples/leptos-independent/Cargo.toml'
TARGET=BASE/'browser-build'

def run(command,**kwargs):
    """Each child is finite; all builds use the dedicated consumer target and cached dependencies."""
    print('COMMAND:',*map(str,command),flush=True)
    return subprocess.run(list(map(str,command)),cwd=ROOT,check=True,timeout=600,
                          env=dict(os.environ,CARGO_TARGET_DIR=str(TARGET)),**kwargs)

def build():
    """Keep matching native/hydrate/CSR artifacts while never mixing incompatible feature sets."""
    OUT.mkdir(parents=True,exist_ok=True)
    assert subprocess.check_output(['wasm-bindgen','--version'],text=True).strip()=='wasm-bindgen 0.2.129'
    frozen=(ROOT/'Cargo.lock').read_bytes()
    cargo=['cargo','build','--manifest-path',MANIFEST,'--locked','--offline']
    run(cargo+['--example','realtime_document','--features','ssr,realtime'])
    for mode in ['hydrate','csr']:
        with (OUT/f'{mode}.html').open('w') as output:
            run([TARGET/'debug/examples/realtime_document',mode],stdout=output)
        run(cargo+['--lib','--target','wasm32-unknown-unknown','--features',mode+',realtime'])
        directory=OUT/mode;directory.mkdir(exist_ok=True)
        run(['wasm-bindgen','--target','web','--no-typescript','--out-dir',directory,'--out-name','realtime',TARGET/'wasm32-unknown-unknown/debug/leptos_independent.wasm'])
        (directory/'start.js').write_text("import init,{realtime_start} from './realtime.js'; await init(); window.realtimeDispose=realtime_start(); window.realtimeReady=true;\n")
    assert (ROOT/'Cargo.lock').read_bytes()==frozen
    root={(p['name'],p['version'],p.get('source'),p.get('checksum')) for p in tomllib.loads(frozen.decode())['package'] if p.get('source')}
    nested={(p['name'],p['version'],p.get('source'),p.get('checksum')) for p in tomllib.loads((MANIFEST.parent/'Cargo.lock').read_text())['package'] if p.get('source')}
    assert nested<=root
    print('PASS separate realtime SSR/hydrate/CSR artifacts and nested root-subset lock')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('action',choices=['build']);args=parser.parse_args();build()
