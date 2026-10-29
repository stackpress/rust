#!/usr/bin/env python3
"""Audit the tooling-only scope, unchanged safety assertions, lock and output placement."""
import json
from pathlib import Path
import re
import subprocess

ROUNDS = 'crates/sql/sql-sqlite/src/tests/rounds.rs'
WORKFLOW = '.agents/workflows/fast-build-and-test.md'
ALLOWED = {
    WORKFLOW, 'crates/lib/tests/support/watchdog.rs', ROUNDS,
    'crates/sql/sql-sqlite/src/tests/transitions.rs',
    'crates/sql/sql/tests/network.rs', 'crates/sql/sql/tests/harness.rs',
    'crates/sql/test-support/harness.rs',
    *(f'crates/sql/sql-{backend}/tests/m0.rs' for backend in ('postgres', 'mysql')),
    *(f'crates/sql/sql-{backend}/src/tests/m{stage}.rs'
      for backend in ('postgres', 'mysql') for stage in (1, 2)),
}


def git(*arguments):
    """Read this worktree and its own HEAD, never another trial branch."""
    return subprocess.check_output(['git', *arguments], text=True)


def assertions(source):
    """Compare complete unchanged macro spans, retaining messages and expressions."""
    spans = []
    for match in re.finditer(r'\bassert(?:_eq|_ne)?!\(', source):
        depth = 1
        end = match.end()
        while depth:
            depth += (source[end] == '(') - (source[end] == ')')
            end += 1
        spans.append(source[match.start():end])
    return spans


def main():
    """Fail on assertion deletion/change, lock drift, scope drift or wrong output ownership."""
    before = git('show', f'HEAD:{ROUNDS}')
    after = Path(ROUNDS).read_text()
    assert assertions(before) == assertions(after), 'an existing safety assertion changed'
    assert '.chain((1..=64).map(Cut::Pending))' in after
    assert 'Cut::Native' in after and 'Cut::Complete' in after
    assert Path('Cargo.lock').read_bytes() == subprocess.check_output(['git', 'show', 'HEAD:Cargo.lock'])
    old = git('show', f'HEAD:{WORKFLOW}')
    current = Path(WORKFLOW).read_text()
    assert old.split('## 5.')[0] == current.split('## 5.')[0]
    assert old.split('## 6.')[1] == current.split('## 6.')[1]
    paths = set(git('diff', '--name-only').splitlines()) | set(git('ls-files', '--others', '--exclude-standard').splitlines())
    extra = {path for path in paths if path not in ALLOWED and not path.startswith(
        '.agents/specs/00001-stackpress-rust-foundation/tasks/00019-')}
    assert not extra, extra
    subprocess.run(['git', 'diff', '--check'], check=True)
    watchdog = Path('crates/lib/tests/support/watchdog.rs').read_text()
    assert 'option_env!("CARGO_TARGET_TMPDIR")' in watchdog
    assert '.unwrap_or_else(std::env::temp_dir)' in watchdog
    assert '../../target/m0' not in watchdog
    outputs = list(Path('target/nextest-compat/tmp/stackpress-watchdog').glob('*render-unwind.log'))
    assert outputs, 'missing actual render-unwind output under the task target'
    stray = {str(path): [path.stat().st_size, path.stat().st_mtime_ns]
             for path in Path('crates/target').rglob('*render-unwind.log')}
    baseline = Path('/tmp/00019-stray-before.json')
    if baseline.exists():
        assert stray == json.loads(baseline.read_text()), 'stray output changed'
    peak_path = Path('/tmp/00019-size-peak.json')
    peak = json.loads(peak_path.read_text()) if peak_path.exists() else {
        'peak_kib': int(subprocess.check_output(['du', '-sk', 'target/nextest-compat'], text=True).split()[0]),
        'cap_kib': 8 * 1024 * 1024,
    }
    assert peak['peak_kib'] < peak['cap_kib'], peak
    print(f'PASS: {len(assertions(before))} unchanged safety assertions; all 64 original cuts retained')
    print(f'PASS: byte-identical Cargo.lock; {len(paths)} scoped paths; workflow outside section 5 unchanged')
    print(f'PASS: {len(outputs)} render-unwind logs owned by task target; existing crates/target output preserved')
    print(f'PASS: observed target peak {peak["peak_kib"]} KiB below {peak["cap_kib"]} KiB cap')


if __name__ == '__main__':
    main()
