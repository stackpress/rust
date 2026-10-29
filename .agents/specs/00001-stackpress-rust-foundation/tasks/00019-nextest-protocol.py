#!/usr/bin/env python3
"""Prove the seven SQL harness protocols without executing any live-DB path."""
import json
import os
import subprocess

PACKAGES = ('stackpress-sql', 'stackpress-sql-postgres', 'stackpress-sql-mysql')
GATES = ('STACKPRESS_TEST_POSTGRES_URL', 'STACKPRESS_TEST_MYSQL_URL')


def run(command, env):
    """Bound every discovery/exact invocation and fail with its actual diagnostics."""
    result = subprocess.run(command, env=env, capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result.stdout


def main():
    """Check lists, enabled-gate discovery, individual skips, exact misses and offline isolation."""
    env = {key: value for key, value in os.environ.items() if key not in GATES}
    command = ['cargo', 'nextest', 'list', '--all-targets', '--locked', '--message-format', 'json']
    for package in PACKAGES:
        command += ['-p', package]
    listing = json.loads(run(command, env))
    count = 0
    exact = 0
    for suite in listing['rust-suites'].values():
        package, binary = suite['package-name'], suite['binary-name']
        if not ((package == 'stackpress-sql' and binary == 'network') or
                (package in PACKAGES[1:] and binary in ('m0', 'm1', 'm2'))):
            continue
        count += 1
        executable = suite['binary-path']
        names = set(suite['testcases'])
        expected_count = 16 if binary == 'network' else (1 if binary == 'm0' else
                         (4 if binary == 'm1' else (3 if package.endswith('postgres') else 2)))
        assert len(names) == expected_count, (package, binary, names)
        ignored = names - {'m2::sqlite_reference'}
        args = [executable, '--list', '--format', 'terse']
        assert set(run(args, env).splitlines()) == {f'{name}: test' for name in names}
        assert set(run(args + ['--ignored'], env).splitlines()) == {f'{name}: test' for name in ignored}
        # A non-URL sentinel is used ONLY for listing: this must neither parse a URL
        # nor open any fixture. No configured-gate execution is performed here.
        enabled = {**env, **dict.fromkeys(GATES, 'discovery-only-not-a-database-url')}
        assert set(run(args, enabled).splitlines()) == {f'{name}: test' for name in names}
        assert run(args + ['--ignored'], enabled) == ''
        for name in sorted(ignored):
            for flags in ([], ['--ignored']):
                output = run([executable, '--exact', name, '--nocapture', *flags], env)
                assert 'ignored (STACKPRESS_TEST_' in output and '_URL unset)' in output, (name, output)
                assert ' ... ok' not in output, (name, output)
                exact += 1
        # Unknown exact filters must not run SQLite or create native fixture authority.
        output = run([executable, '--exact', 'missing::case', '--nocapture'], env)
        assert 'created owned' not in output and ' ... ok' not in output, output
        if 'm2::sqlite_reference' in names:
            output = run([executable, 'm2::sqlite_reference', '--nocapture', '--exact'], env)
            assert 'test m2::sqlite_reference ... ok' in output
            assert 'native_cases' not in output and 'ignored' not in output
            exact += 1
        if package.endswith('mysql') and binary == 'm2':
            output = run([executable, '--g37-ddl', '--list', '--format', 'terse'], env)
            assert output == 'm2::g37-ddl: test\n', output
            output = run([executable, '--g37-ddl', '--exact', 'm2::g37-ddl', '--nocapture'], env)
            assert 'ignored (STACKPRESS_TEST_MYSQL_URL unset)' in output
            exact += 1
        print(f'{package}::{binary}: {len(names)} names; {len(ignored)} runtime ignored; protocol PASS')
    assert count == 7, count
    print(f'PASS: {count} harnesses, 31 discovered cases, {exact} individual unset/offline invocations; no live DB execution')


if __name__ == '__main__':
    main()
