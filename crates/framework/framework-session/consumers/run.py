#!/usr/bin/env python3
"""Record this outer script once; each standalone application has a distinct target and lock."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tomllib


def run(command, env, expected=None):
    """Retain exact commands and executable markers in the caller's benchmark log."""
    print('COMMAND:', *map(str, command), flush=True)
    result = subprocess.run(command, env=env, check=True, text=True,
                            stdout=subprocess.PIPE if expected else None)
    if expected:
        print(result.stdout, end='', flush=True)
        assert result.stdout.strip() == expected, result.stdout
    return result


def registry(lock):
    """Compare identities/checksums, without treating internal workspace edges as external tuples."""
    return {(p['name'], p['version'], p.get('source'), p.get('checksum'))
            for p in tomllib.loads(lock.decode())['package'] if 'source' in p}


def main():
    """Resolve new locks only under the coordinator's one-time grant; all proofs stay locked/offline."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reconcile-locks', action='store_true')
    parser.add_argument('--refresh-locks', action='store_true', help='approved root-seeded workspace-edge refresh for native consumers')
    parser.add_argument('--only', choices=['direct', 'facade', 'plugin', 'disabled', 'browser'])
    args = parser.parse_args()
    if args.refresh_locks and (not args.reconcile_locks or args.only not in ['direct', 'facade', 'plugin']):
        parser.error('--refresh-locks requires --reconcile-locks and one approved native --only selection')
    base = Path(__file__).resolve().parent
    root = base.parents[3]
    target = Path(os.environ['CARGO_TARGET_DIR']).resolve()
    root_lock = (root/'Cargo.lock').read_bytes()
    external = registry(root_lock)
    for kind in [args.only] if args.only else ['direct', 'facade', 'plugin', 'disabled']:
        assert kind != 'browser' or args.reconcile_locks, 'browser/build.py selects each renderer separately'
        manifest = base/kind/'Cargo.toml'
        lock = manifest.with_name('Cargo.lock')
        env = {**os.environ, 'CARGO_TARGET_DIR': str(target/f'consumer-session-{kind}')}
        if args.reconcile_locks:
            # Exactly one offline resolution is authorized for each new manifest, never root.
            assert not lock.exists() or args.refresh_locks, f'already reconciled: {lock}'
            before = registry(lock.read_bytes()) if lock.exists() else set()
            lock.write_bytes(root_lock)
            run(['cargo', 'tree', '--manifest-path', str(manifest), '--offline', '-e', 'normal', '--depth', '1'], env)
            assert registry(lock.read_bytes()) <= external, 'new registry tuple; stop for coordinator'
            assert (root/'Cargo.lock').read_bytes() == root_lock, 'root lock changed'
            after = registry(lock.read_bytes())
            print(f'LOCK {kind}: {len(after)} registry tuples are a workspace subset; root lock unchanged; delta +{len(after-before)}/-{len(before-after)}', flush=True)
            continue
        flags = ['--manifest-path', str(manifest), '--locked', '--offline']
        graph = subprocess.check_output(['cargo', 'tree', *flags, '-e', 'normal', '--prefix', 'none'], env=env, text=True)
        names = {line.split()[0] for line in graph.splitlines() if line}
        if kind in ['direct', 'facade']:
            assert not names.intersection({'stackpress-server', 'stackpress-sql', 'stackpress-framework-email',
                                           'stackpress-framework-csrf', 'leptos', 'phonenumber'})
            assert ('stackpress' in names) == (kind == 'facade')
            assert 'stackpress-framework-session' in names
        elif kind == 'disabled':
            assert not names.intersection({'stackpress-framework-session', 'stackpress-framework-email',
                                           'stackpress-framework-csrf', 'stackpress-server', 'jsonwebtoken'})
        else:
            assert {'stackpress-framework-session', 'stackpress-framework-api', 'stackpress-framework-email',
                    'stackpress-framework-csrf', 'stackpress-server', 'stackpress-sql-sqlite'} <= names
        assert registry(lock.read_bytes()) <= external
        assert (root/'Cargo.lock').read_bytes() == root_lock
        print(f'GRAPH {kind}: {len(names)} packages; feature boundaries and tuple subset passed', flush=True)
        run(['cargo', 'fmt', '--manifest-path', str(manifest), '--check'], env)
        run(['cargo', 'clippy', *flags, '--all-targets', '--', '-D', 'warnings'], env)
        run(['cargo', 'run', *flags], env, f'slice9a {kind} consumer passed')
        print(json.dumps({'consumer': kind, 'target': env['CARGO_TARGET_DIR']}), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
