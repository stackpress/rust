#!/usr/bin/env python3
"""Run the release-manifest gate once, preserving per-phase results and exact commands.

Wrap this script once in build-benchmark.py. Independent phases continue after a
failure. Nothing uploads, authenticates, installs tools, or starts a listener.
"""
import argparse
import os
from pathlib import Path
import subprocess
import time

from common import ROOT, EVIDENCE, COMPATIBLE, registry, write_report


def main():
    """Separate incompatible renderers/consumers while keeping the core target warm."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pending-tests', action='store_true')
    parser.add_argument('--allow-dirty', action='store_true')
    parser.add_argument('--only', action='append', help='rerun only named failed phases; retain original evidence')
    args = parser.parse_args()
    base = Path(os.environ['CARGO_TARGET_DIR']).resolve()
    records = EVIDENCE / 'gate-logs'
    records.mkdir(parents=True, exist_ok=True)
    previous = EVIDENCE / 'gate-results.json'
    import json
    results = json.loads(previous.read_text()) if args.only and previous.exists() else []
    selected = set()
    frozen = (ROOT / 'Cargo.lock').read_bytes()

    def phase(name, command, *, target=None, extra=None):
        """Retain complete streamed proof output and continue only independent phases."""
        if args.only and name not in args.only:
            return None
        selected.add(name)
        env = dict(os.environ, **(extra or {}))
        if target:
            env['CARGO_TARGET_DIR'] = str(target)
        print('GATE', name, ':', *map(str, command), flush=True)
        start = time.monotonic()
        attempt = sum(row['phase'] == name for row in results)
        log_path = records / (name + (f'-attempt-{attempt + 1}' if attempt else '') + '.log')
        with log_path.open('w') as log:
            child = subprocess.Popen(command, cwd=ROOT, env=env, text=True,
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            for line in child.stdout:
                print(line, end='', flush=True)
                log.write(line)
            code = child.wait()
        assert (ROOT / 'Cargo.lock').read_bytes() == frozen, 'root lock bytes changed during gate'
        results.append({'phase': name, 'command': command, 'code': code,
                        'command_seconds': round(time.monotonic() - start, 6),
                        'target': env['CARGO_TARGET_DIR'], 'log': str(log_path.relative_to(ROOT))})
        write_report(EVIDENCE / 'gate-results.json', results)
        return code

    phase('fmt', ['cargo', 'fmt', '--all', '--', '--check'])
    for name, features in [('default', []), ('compatible', ['--features', COMPATIBLE])]:
        common = ['--workspace', '--locked', '--offline', *features]
        phase('clippy-' + name, ['cargo', 'clippy', *common, '--all-targets', '--', '-D', 'warnings'])
        phase('nextest-' + name, ['cargo', 'nextest', 'run', *common, '--all-targets',
                                  '--test-threads', '2', '--no-fail-fast', '-E', 'not test(tcp_watchdog)'])
        phase('nextest-tcp-' + name, ['cargo', 'nextest', 'run', *common, '--all-targets',
                                      '--test-threads', '1', '-E', 'test(tcp_watchdog)'])
        phase('doctests-' + name, ['cargo', 'test', *common, '--doc'])
        phase('rustdoc-' + name, ['cargo', 'doc', *common, '--no-deps'], extra={'RUSTDOCFLAGS': '-D warnings'})
    aggregate = 'crates/framework/framework/consumers'
    session = 'crates/framework/framework-session/consumers'
    phase('aggregate-graphs', ['python3', aggregate + '/graphs.py', '--check'])
    phase('aggregate-consumers', ['python3', aggregate + '/run.py'])
    phase('session-graphs', ['python3', session + '/graphs.py', '--check'])
    phase('session-consumers', ['python3', session + '/run.py'])
    phase('generated-workspace', ['cargo', 'run', '-p', 'generated-workspace-tool', '--locked', '--offline',
                                   '--', 'emit', 'generate', '--check', 'true'])
    phase('generated-admin', ['cargo', 'run', '-p', 'generated-admin-tool', '--locked', '--offline',
                              '--', 'emit', 'generate', '--check'])
    phase('api-checkpoint', ['cargo', 'run', '--manifest-path', 'examples/api-checkpoint/tool/Cargo.toml',
                             '--locked', '--offline', '--', '--check'], target=base / 'api-checkpoint')
    for package, example, features in [('stackpress-framework-session', 'regenerate_session_models', 'auth-server,leptos-ssr'),
                                       ('stackpress-framework-api', 'regenerate_api_models', 'sql,codegen,ui-ssr'),
                                       ('stackpress-framework', 'regenerate_aggregate_models', 'full,codegen')]:
        phase(example, ['cargo', 'run', '-p', package, '--example', example, '--locked', '--offline',
                         '--features', features, '--', '--check'])
    phase('blog', ['python3', 'tools/examples/blog/gate.py'], target=ROOT / 'target/00050-blog/native')
    phase('checker-controls', ['python3', '-m', 'unittest', 'discover', '-s', 'tools/release', '-p', 'test_*.py'])
    phase('readmes', ['python3', 'tools/release/check_readmes.py'])
    package_flags = (['--pending-tests'] if args.pending_tests else []) + (['--allow-dirty'] if args.allow_dirty else [])
    phase('packages-and-dry-runs', ['python3', 'tools/release/package_workspace.py', *package_flags])
    phase('facade', ['python3', 'tools/release/check_facade.py'])
    assert not args.only or set(args.only) <= selected, f'unknown phase: {set(args.only or []) - selected}'
    latest = {row['phase']: row for row in results}
    print('Release gate:', sum(row['code'] == 0 for row in latest.values()), '/', len(latest), 'phases passed', flush=True)
    return int(any(row['code'] for row in latest.values()))


if __name__ == '__main__':
    raise SystemExit(main())
