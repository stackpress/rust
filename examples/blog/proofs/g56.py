#!/usr/bin/env python3
"""Measure only the extracted plain-data browser decisions before the post-fix full gate."""
import json
import os
from pathlib import Path
import subprocess

BLOG = Path(__file__).resolve().parents[1]


def main():
    """Reuse the owned coverage target; run native contracts and export their exact binary."""
    target = Path(os.environ['CARGO_TARGET_DIR']).parent / 'coverage'
    output = BLOG / '.build/g56'
    output.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    env['RUSTFLAGS'] = (env.get('RUSTFLAGS', '') + ' -C instrument-coverage').strip()
    env['LLVM_PROFILE_FILE'] = str(target / 'g56-%m-%p.profraw')
    common = ['--manifest-path', str(BLOG / 'Cargo.toml'), '-p', 'blog-shared',
              '--features', 'ssr', '--test', 'main', '--locked', '--offline']
    # Cargo's artifact stream identifies the exact instrumented executable without metadata
    # or stale target discovery. Nextest then runs those contracts with the same flags.
    built = subprocess.run(['cargo', 'test', *common, '--no-run', '--message-format=json'],
                           env=env, text=True, stdout=subprocess.PIPE, check=True)
    artifacts = [json.loads(line) for line in built.stdout.splitlines()]
    binaries = [row['executable'] for row in artifacts
                if row.get('reason') == 'compiler-artifact' and row.get('executable')
                and row['target']['name'] == 'main']
    assert len(binaries) == 1, binaries
    subprocess.run(['cargo', 'nextest', 'run', *common, '--test-threads', '2'], env=env, check=True)
    profiles = sorted(target.glob('g56-*.profraw'))
    assert profiles
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
    host = next(line.split(': ', 1)[1] for line in subprocess.check_output(
        ['rustc', '-vV'], text=True).splitlines() if line.startswith('host:'))
    llvm = sysroot / 'lib/rustlib' / host / 'bin'
    merged = target / 'g56.profdata'
    subprocess.run([str(llvm / 'llvm-profdata'), 'merge', '-sparse', *map(str, profiles),
                    '-o', str(merged)], check=True)
    raw = subprocess.check_output([str(llvm / 'llvm-cov'), 'export', '-format=text',
                                   f'-instr-profile={merged}', '-object', binaries[0]], text=True)
    (output / 'coverage-raw.json').write_text(raw)
    wanted = {BLOG / 'shared/src/browser.rs', BLOG / 'shared/src/form.rs'}
    report = {}
    for row in json.loads(raw)['data'][0]['files']:
        file = Path(row['filename']).resolve()
        if file in wanted:
            lines = row['summary']['lines']
            report[str(file.relative_to(BLOG))] = lines
    assert len(report) == 2, report
    assert all(row['percent'] >= 70 for row in report.values()), report
    (output / 'coverage.json').write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
