#!/usr/bin/env python3
"""Build each renderer separately in this consumer's target and check its native-resource boundary."""
import argparse
import os
from pathlib import Path
import subprocess
import tomllib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('mode', choices=['hydrate', 'csr'])
parser.add_argument('--out-dir', type=Path, required=True)
args = parser.parse_args()
base = Path(__file__).resolve().parent
root = base.parents[4]
target = Path(os.environ['CARGO_TARGET_DIR']).resolve()/'consumer-session-browser'
env = {**os.environ, 'CARGO_TARGET_DIR': str(target)}
flags = ['--manifest-path', str(base/'Cargo.toml'), '--target', 'wasm32-unknown-unknown',
         '--features', args.mode, '--locked', '--offline']


def run(command):
    """The outer benchmark owns deadlines/output; no nested recorder or listener is started."""
    print('COMMAND:', *map(str, command), flush=True)
    subprocess.run(command, env=env, check=True)


run(['cargo', 'fmt', '--manifest-path', str(base/'Cargo.toml'), '--check'])
run(['cargo', 'clippy', *flags, '--', '-D', 'warnings'])
run(['cargo', 'build', *flags])
args.out_dir.mkdir(parents=True, exist_ok=True)
run(['wasm-bindgen', '--target', 'web', '--out-dir', str(args.out_dir),
     str(target/'wasm32-unknown-unknown/debug/slice9_account_browser.wasm')])
graph = subprocess.check_output(['cargo', 'tree', *flags, '-e', 'normal', '--prefix', 'none'], env=env, text=True)
(args.out_dir/'normal-tree.txt').write_text(graph)
names = {line.split()[0] for line in graph.splitlines() if line}
assert 'stackpress' in names and 'stackpress-framework-session' in names
assert not names.intersection({'ring', 'jsonwebtoken', 'aes', 'cbc', 'sha3', 'stackpress-server', 'stackpress-sql',
                               'stackpress-framework-schema', 'stackpress-framework-sql', 'stackpress-framework-csrf',
                               'stackpress-framework-email', 'lettre', 'qrcode', 'png'})
wasm = (args.out_dir/'slice9_account_browser_bg.wasm').read_bytes()
for secret in [b'SIGNING-SEED-MUST-NOT-REACH-BROWSER', b'DATABASE-SEED-MUST-NOT-REACH-BROWSER',
               b'SMTP-PASSWORD-MUST-NOT-REACH-BROWSER']:
    assert secret not in wasm
registry = lambda path: {(p['name'], p['version'], p.get('source'), p.get('checksum'))
                         for p in tomllib.loads(path.read_text())['package'] if 'source' in p}
assert registry(base/'Cargo.lock') <= registry(root/'Cargo.lock')
print(f'{args.mode} facade consumer: consumer-owned target, workspace tuple subset and native-secret boundary passed', flush=True)
