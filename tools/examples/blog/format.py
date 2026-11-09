#!/usr/bin/env python3
"""Format/check every authored Rust file; generated output is checked by its owner instead."""
import argparse
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
BLOG = ROOT / 'examples/blog'
PROOFS = Path(__file__).resolve().parent


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write',action='store_true')
    args=parser.parse_args()
    files=[str(file) for directory in ('tool/src','tool/tests','web/src','web/tests','plugins/app/src','plugins/app/tests','plugins/store/src','plugins/store/tests','plugins/assets/src','plugins/assets/tests','client/src','xtask/src','xtask/tests')
           for file in sorted((BLOG/directory).rglob('*.rs'))]
    command=['rustfmt','--edition','2024','--config','skip_children=true']
    if not args.write:command.append('--check')
    subprocess.run(command+files,check=True)


if __name__=='__main__':main()
