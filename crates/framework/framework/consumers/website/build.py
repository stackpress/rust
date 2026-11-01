#!/usr/bin/env python3
"""Build one browser mode in the website consumer target; emit only reusable web artifacts."""
import argparse
import os
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('mode', choices=['hydrate', 'csr'])
parser.add_argument('--out-dir', type=Path, required=True)
args = parser.parse_args()
base = Path(__file__).resolve().parent
target = Path(os.environ['CARGO_TARGET_DIR']).resolve()/'consumer-aggregate-website'
env = {**os.environ, 'CARGO_TARGET_DIR': str(target)}
flags = ['--manifest-path', str(base/'Cargo.toml'), '--target', 'wasm32-unknown-unknown',
         '--features', args.mode, '--lib', '--locked', '--offline']
for command in [['cargo', 'clippy', *flags, '--', '-D', 'warnings'], ['cargo', 'build', *flags]]:
    print('COMMAND:', *command, flush=True)
    subprocess.run(command, check=True, env=env)
args.out_dir.mkdir(parents=True, exist_ok=True)
command = ['wasm-bindgen', '--target', 'web', '--out-dir', str(args.out_dir),
           str(target/'wasm32-unknown-unknown/debug/slice11_website.wasm')]
print('COMMAND:', *command, flush=True)
subprocess.run(command, check=True, env=env)
data = (args.out_dir/'slice11_website_bg.wasm').read_bytes()
for sentinel in [b'SIGNING-SEED-MUST-NOT-REACH-BROWSER', b'DATABASE-SEED-MUST-NOT-REACH-BROWSER', b'SMTP-PASSWORD-MUST-NOT-REACH-BROWSER']:
    assert sentinel not in data
print(f'{args.mode}: facade browser artifact and native-secret sentinel checks passed.', flush=True)
