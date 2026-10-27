#!/usr/bin/env python3
"""Fresh native coverage, including executed example binaries and pure controllers."""
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ.get('UI_FORMAT_OUTPUT_DIR', '/private/tmp/s8-format'))
TARGET = ROOT / 'target/s8-native'
ENV = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), CARGO_LLVM_COV_TARGET_DIR=str(TARGET),
           CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0', CARGO_BUILD_JOBS='2', RUST_TEST_THREADS='1')


def run(name, command):
    """A timeout fails and kills/reaps the owned process group, including test children."""
    with (OUT / (name + '.log')).open('w') as log:
        log.write('$ ' + ' '.join(map(str, command)) + '\n')
        log.flush()
        child = subprocess.Popen(list(map(str, command)), cwd=ROOT, env=ENV,
                                 stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = child.wait(timeout=1800)
            if code:
                raise subprocess.CalledProcessError(code, command)
        except BaseException:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            raise
    print('PASS ' + name, flush=True)


def test_lines(source):
    """Exclude actual inline test modules, never production functions after mod tests."""
    lines = source.read_text().splitlines()
    excluded = set()
    index = 0
    while index < len(lines):
        if lines[index].startswith('#[cfg(') and re.search(r'\btest\b', lines[index]):
            cursor = index + 1
            while cursor < len(lines) and (lines[cursor].startswith('#[') or not lines[cursor].strip()):
                cursor += 1
            if cursor < len(lines) and lines[cursor] == 'mod tests;':
                excluded.update(range(index + 1, cursor + 2))
            elif cursor < len(lines) and lines[cursor] == 'mod tests {':
                end = next(i for i in range(cursor + 1, len(lines)) if lines[i] == '}')
                excluded.update(range(index + 1, end + 2))
                index = end
        index += 1
    return excluded


def main():
    """Run after normal builds/proofs; merge only fresh profiles with matching objects."""
    OUT.mkdir(parents=True, exist_ok=True)
    if '--run' in sys.argv:
        run('precoverage-clean', ['cargo', 'clean', '--target-dir', TARGET])
        selected = ['--locked', '--offline', '-p', 'stackpress-framework-ui', '-p',
                    'stackpress-example-ui-format', '--features', 'ssr,rich,format']
        run('coverage-tests', ['cargo', 'llvm-cov', '--no-rustc-wrapper', '--no-report', *selected])
        run('coverage-native', ['cargo', 'llvm-cov', 'run', '--no-rustc-wrapper', '--no-report',
                               '--locked', '--offline', '-p', 'stackpress-example-ui-format',
                               '--features', 'ssr,rich,format', '--bin', 'ui-format'])
    # Match the compiler's own LLVM tools, not an unrelated Homebrew LLVM version.
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
    verbose = subprocess.check_output(['rustc', '-vV'], text=True)
    host = next(line.split(': ', 1)[1] for line in verbose.splitlines() if line.startswith('host: '))
    llvm = sysroot / 'lib/rustlib' / host / 'bin'
    profiles = sorted(TARGET.glob('*.profraw'))
    if not profiles:
        raise RuntimeError('fresh instrumented profiles required; use --run')
    profile = OUT / 'coverage.profdata'
    run('coverage-merge', [llvm / 'llvm-profdata', 'merge', '-sparse', *profiles, '-o', profile])
    # Include both test harnesses and the executed finite native entrypoint explicitly.
    objects = []
    seen = set()
    candidates = sorted((TARGET / 'debug/deps').iterdir()) + [TARGET / 'debug/ui-format']
    for candidate in candidates:
        if not candidate.is_file() or not os.access(candidate, os.X_OK):
            continue
        if candidate.name != 'ui-format' and not re.fullmatch(r'[A-Za-z0-9_]+-[a-f0-9]+', candidate.name):
            continue
        stat = candidate.stat()
        identity = stat.st_dev, stat.st_ino
        with candidate.open('rb') as file:
            magic = file.read(4)
        if identity not in seen and magic in (b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\x7fELF'):
            seen.add(identity)
            objects.append(candidate)
    if not objects:
        raise RuntimeError('matching instrumented native objects required')
    (OUT / 'coverage-objects.json').write_text(json.dumps(list(map(str, objects)), indent=2))
    base = [str(llvm / 'llvm-cov'), 'export', '--num-threads=1', '-instr-profile=' + str(profile),
            '-ignore-filename-regex=/.cargo/|/rustc/|/.rustup/', str(objects[0])]
    for object in objects[1:]:
        base += ['-object', str(object)]
    for format, name, extra in [('lcov', 'coverage-full.lcov', []), ('text', 'coverage-full.json', []),
                                ('text', 'coverage-diagnostics.json', ['--dump'])]:
        with (OUT / name).open('w') as output, (OUT / (name + '.log')).open('w') as log:
            subprocess.run(base + ['-format=' + format, *extra], cwd=ROOT, stdout=output,
                           stderr=log, timeout=120, check=True)
    # LCOV's DA records are the executable-line denominator. Coalesce file instances.
    files = {}
    for record in (OUT / 'coverage-full.lcov').read_text().split('end_of_record'):
        source = re.search(r'^SF:(.+)$', record, re.M)
        if not source:
            continue
        path = Path(source[1])
        if not path.is_relative_to(ROOT):
            continue
        relative = str(path.relative_to(ROOT))
        if not relative.startswith(('crates/framework/framework-ui/src/', 'examples/ui-format/src/')):
            continue
        if path.name == 'tests.rs' or path.name.endswith('_tests.rs'):
            continue
        excluded = test_lines(path)
        entry = files.setdefault(relative, dict(counts={}, excluded_test_lines=sorted(excluded)))
        for number, count in re.findall(r'^DA:(\d+),(\d+)', record, re.M):
            number, count = int(number), int(count)
            if number not in excluded:
                entry['counts'][number] = max(entry['counts'].get(number, 0), count)
    summary = {}
    for name, prefix in [('framework-ui', 'crates/framework/framework-ui/src/'),
                         ('ui-format', 'examples/ui-format/src/'),
                         ('controllers', 'crates/framework/framework-ui/src/edit/')]:
        selected = {}
        for path, entry in files.items():
            if not path.startswith(prefix) or name == 'controllers' and Path(path).name in ('dom.rs', 'input.rs'):
                continue
            counts = entry['counts']
            selected[path] = dict(covered=sum(c > 0 for c in counts.values()), executable=len(counts),
                                  uncovered=[n for n, c in counts.items() if not c],
                                  excluded_test_lines=entry['excluded_test_lines'])
        covered = sum(file['covered'] for file in selected.values())
        total = sum(file['executable'] for file in selected.values())
        summary[name] = dict(covered=covered, executable=total, percent=round(100*covered/total, 2), files=selected)
    assert 'examples/ui-format/src/editing.rs' in summary['ui-format']['files'], 'executed editor source omitted'
    assert 'examples/ui-format/src/main.rs' in summary['ui-format']['files'], 'executed entrypoint omitted'
    (OUT / 'coverage-summary.json').write_text(json.dumps(summary, indent=2))
    for name, record in summary.items():
        print(name, record['covered'], record['executable'], record['percent'], flush=True)
        if record['percent'] < 70:
            raise RuntimeError(name + ' below the development gate')


if __name__ == '__main__':
    main()
