#!/usr/bin/env python3
"""Record this outer proof once; each application owns its lock and distinct target."""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tomllib

BASE = Path(__file__).resolve().parent
ROOT = BASE.parents[3]
KINDS = ['build-host', 'full', 'direct', 'headless', 'api-disabled', 'disabled', 'facade', 'website']


class WatchdogTimeout(RuntimeError):
    """A deadline expires only after the owned process group has been terminated and reaped."""
    def __init__(self, forced):
        super().__init__('finite consumer deadline exceeded; process group reaped')
        self.forced = forced


def bounded(child, deadline):
    """Use an external deadline; an executor deadlock cannot stall the watchdog."""
    try:
        output, _ = child.communicate(timeout=deadline)
        return output
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGTERM)
        forced = False
        try:
            child.communicate(timeout=.3)
        except subprocess.TimeoutExpired:
            forced = True
            os.killpg(child.pid, signal.SIGKILL)
            child.communicate(timeout=2)
        assert child.poll() is not None
        raise WatchdogTimeout(forced) from None


def registry(data):
    """Registry identities include source/checksum and do not collapse duplicate versions."""
    return {(p['name'], p['version'], p['source'], p.get('checksum'))
            for p in tomllib.loads(data.decode())['package'] if p.get('source', '').startswith('registry+')}


def run(command, env):
    """The enclosing recorder retains argv/output and owns the absolute process deadline."""
    print('COMMAND:', *map(str, command), flush=True)
    capture = command[:2] == ['cargo', 'run']
    child = subprocess.Popen(command, env=env, text=True, start_new_session=True,
                             stdout=subprocess.PIPE if capture else None)
    output = bounded(child, 45 if capture else 1200)
    if child.returncode:
        if output:
            print(output, flush=True)
        raise subprocess.CalledProcessError(child.returncode, command)
    if capture:
        for line in output.splitlines():
            if line.startswith('INVENTORY '):
                Path(env['AGGREGATE_INVENTORY']).write_text(line.removeprefix('INVENTORY ')+'\n')
                print('INVENTORY:', env['AGGREGATE_INVENTORY'], flush=True)
            else:
                print(line, flush=True)


def main():
    """Resolve each new lock once; all later executions are locked and offline."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--resolve', action='store_true')
    parser.add_argument('--only', choices=KINDS)
    args = parser.parse_args()
    target = Path(os.environ['CARGO_TARGET_DIR']).resolve()
    root_lock = (ROOT/'Cargo.lock').read_bytes()
    for kind in [args.only] if args.only else KINDS:
        manifest = BASE/kind/'Cargo.toml'
        lock = manifest.with_name('Cargo.lock')
        env = {**os.environ, 'CARGO_TARGET_DIR': str(target/f'consumer-aggregate-{kind}')}
        if args.resolve:
            if lock.exists():
                assert registry(lock.read_bytes()) <= registry(root_lock)
                assert (ROOT/'Cargo.lock').read_bytes() == root_lock
                print(f'LOCK {kind}: already resolved, preserved without a Cargo command', flush=True)
                continue
            lock.write_bytes(root_lock)
            run(['cargo', 'tree', '--manifest-path', str(manifest), '--offline', '-e', 'normal', '--depth', '1'], env)
            # This assertion immediately follows the sole unlocked offline command.
            assert registry(lock.read_bytes()) <= registry(root_lock), 'new external tuple: stop'
            assert (ROOT/'Cargo.lock').read_bytes() == root_lock, 'root lock mutated: stop'
            print(f'LOCK {kind}: registry subset; root bytes unchanged', flush=True)
            continue
        flags = ['--manifest-path', str(manifest), '--locked', '--offline']
        artifacts = target/'aggregate-artifacts'
        artifacts.mkdir(exist_ok=True)
        env['AGGREGATE_INVENTORY'] = str(artifacts/f'{kind}-inventory.json')
        if kind == 'website':
            env['AGGREGATE_PAGE_OUT'] = str(artifacts/'pages')
        selected = ['--features', 'ssr'] if kind == 'website' else []
        assert registry(lock.read_bytes()) <= registry(root_lock)
        run(['cargo', 'fmt', '--manifest-path', str(manifest), '--check'], env)
        run(['cargo', 'clippy', *flags, *selected, '--all-targets', '--', '-D', 'warnings'], env)
        # Compile before the short executable deadline, preserving a fast fail for runtime hangs.
        run(['cargo', 'build', *flags, *selected], env)
        run(['cargo', 'run', *flags, *selected], env)
        if kind in ['direct', 'facade']:
            full = artifacts/'full-inventory.json'
            assert full.exists(), 'run full before comparing direct/facade composition'
            assert json.loads(full.read_text()) == json.loads(Path(env['AGGREGATE_INVENTORY']).read_text())
            print(f'PARITY {kind}: exact phase/event/priority and projected response inventory matches full', flush=True)
        if kind == 'build-host':
            run(['cargo', 'run', *flags, '--', '--check'], env)
        assert (ROOT/'Cargo.lock').read_bytes() == root_lock
        print(json.dumps({'consumer': kind, 'target': env['CARGO_TARGET_DIR']}), flush=True)
        # Only this effort's completed consumer outputs are disposable. Root debug deps stay warm.
        size = sum(p.stat().st_size for p in target.rglob('*') if p.is_file())
        if size > 17 * 1024**3:
            owned = target/f'consumer-aggregate-{kind}'
            print(f'CLEAN owned completed consumer output: {owned}', flush=True)
            shutil.rmtree(owned)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
