#!/usr/bin/env python3
"""Finite recorded proof actions. No command starts a listener or installs dependencies."""
import argparse
import os
from pathlib import Path
import signal
import subprocess

BLOG = Path(__file__).resolve().parent


def bounded(command, seconds=600):
    """Reap exactly this proof's process group if its external wall deadline expires."""
    process = subprocess.Popen(command, start_new_session=True)
    try:
        return process.wait(timeout=seconds)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
        raise RuntimeError(f'proof deadline exceeded: {command[0]}') from None


def main():
    """Choose a narrow edit-loop suite; the full M6 gate is deferred."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['storage', 'generation'])
    parser.add_argument('--native-proof', action='store_true',
                        help='also compile the existing generated native tree')
    args = parser.parse_args()
    package = 'blog-web' if args.action == 'storage' else 'blog-tool'
    command = ['cargo', 'nextest', 'run', '--manifest-path', str(BLOG / 'Cargo.toml'),
               '-p', package, '--test', 'main', '--locked', '--offline',
               '--test-threads', '2', '--no-fail-fast']
    if args.native_proof:
        if args.action != 'generation':
            parser.error('--native-proof belongs to the generation proof')
        command += ['--features', 'native-proof']
    return bounded(command)


if __name__ == '__main__':
    raise SystemExit(main())
