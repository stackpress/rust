#!/usr/bin/env python3
"""Measure all compiled implementation, including generated stores, without examples exclusions."""
import json
from pathlib import Path
import shlex
import subprocess
from m5_verify import ROOT, LOGS, run

PACKAGES = ['stackpress-framework-admin', 'generated-admin-tool', 'generated-admin-shared', 'generated-admin-web']
PREFIXES = ['crates/framework/framework-admin/src/', 'examples/generated-admin/tool/src/',
            'examples/generated-admin/shared/src/', 'examples/generated-admin/web/src/']


def main():
    """Use conservative production bounds so inline test assertions cannot raise the result."""
    packages = [item for name in PACKAGES for item in ['-p', name]]
    run('coverage-final', ['cargo', 'llvm-cov', *packages, '--features',
        'stackpress-framework-admin/codegen,stackpress-framework-admin/ssr', '--json',
        '--output-path', str(LOGS/'coverage.json'), '--locked'])
    run('coverage-command', ['cargo', 'llvm-cov', 'report', *packages, '--json',
        '--output-path', str(LOGS/'coverage.json'), '-vv'])
    line = next(line for line in (LOGS/'coverage-command.log').read_text().splitlines()
                if 'llvm-cov export' in line)
    command = shlex.split(line.split('`')[1])
    # cargo-llvm-cov's automatic examples exclusion hides these shipped example crates.
    # Export everything, then select only each crate's src tree. Native path inclusions count too.
    index = command.index('-ignore-filename-regex')
    del command[index:index+2]
    raw = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=60, check=True)
    (LOGS/'coverage-raw.json').write_text(raw.stdout)
    files = json.loads(raw.stdout)['data'][0]['files']
    report = {}
    for name, prefix in zip(PACKAGES, PREFIXES):
        measured = [file for file in files if str(ROOT/prefix) in file['filename']]
        covered = sum(file['summary']['lines']['covered'] for file in measured)
        count = sum(file['summary']['lines']['count'] for file in measured)
        ranges = []
        for source in (ROOT/prefix).rglob('*.rs'):
            lines = source.read_text().splitlines()
            for start, line in enumerate(lines):
                if line == '#[cfg(test)]' and lines[start+1] == 'mod tests {':
                    # Rustfmt leaves the top-level module's closing brace at column zero.
                    end = next(index for index in range(start+2, len(lines)) if lines[index] == '}')
                    ranges.append([str(source.relative_to(ROOT)), start+1, end+1])
        # Remove every physical inline-test line from the covered numerator, even comments
        # and braces, but retain the full denominator. This is a lower bound on production
        # coverage, not an inflated claim about the test harness. Generated code is retained.
        subtraction = sum(end-start+1 for _, start, end in ranges)
        lower = max(0, covered-subtraction)
        percent = lower*100/count
        report[name] = dict(covered=covered, executable=count, inline_test_ranges=ranges,
                            conservative_covered=lower, production_lower_bound=percent)
        print(f'{name}: production >= {percent:.2f}% ({lower}/{count}), raw {covered}/{count}', flush=True)
        assert percent >= 70, f'{name} remains below the development floor'
    (LOGS/'coverage-production.json').write_text(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
