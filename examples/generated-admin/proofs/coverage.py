"""Measure all compiled implementation, including generated stores, without examples exclusions."""
import json
import math
from pathlib import Path
import shlex
import subprocess
from .runtime import ROOT, TARGET

PACKAGES = ['stackpress-framework-admin', 'generated-admin-tool', 'generated-admin-shared', 'generated-admin-web']
PREFIXES = ['crates/framework/framework-admin/src/', 'examples/generated-admin/tool/src/',
            'examples/generated-admin/shared/src/', 'examples/generated-admin/web/src/']
PACKAGE_ARGS = [item for name in PACKAGES for item in ['-p', name]]


def relink_executables(ctx, selection):
    """Keep all instrumented objects, but force the exact feature pass's subprocess binaries.

    Cargo can otherwise leave a fresh but differently featured top-level executable in a
    shared target. Cleaning packages would instead lose objects needed by the combined report.
    """
    removed = []
    for name in ['generated-admin-tool', 'generated-admin-web']:
        executable = TARGET/'debug'/name
        if executable.exists():
            executable.unlink()
            removed.append(str(executable))
    (ctx.logs/(selection+'-relink.json')).write_text(json.dumps(removed,indent=2))


def measure(ctx):
    """Exercise both gated native scopes before reporting their combined counters."""
    LOGS = ctx.logs
    run = ctx.run
    relink_executables(ctx, 'default')
    run('coverage-default',['cargo','llvm-cov',*PACKAGE_ARGS,'--features','stackpress-framework-admin/codegen,stackpress-framework-admin/ssr','--no-report','--locked'])
    relink_executables(ctx, 'format')
    run('coverage-final',['cargo','llvm-cov',*PACKAGE_ARGS,'--features','stackpress-framework-admin/codegen,generated-admin-web/format,generated-admin-tool/format','--no-clean','--json','--output-path',str(LOGS/'coverage.json'),'--locked'])
    report(ctx)


def report(ctx):
    """Export all implementation counters and disclose conservative development/publication gaps."""
    LOGS, run = ctx.logs, ctx.run
    run('coverage-command', ['cargo', 'llvm-cov', 'report', *PACKAGE_ARGS, '--json',
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
        measured = [file for file in files if str(ROOT/prefix) in file['filename'] and not file['filename'].endswith('_tests.rs')]
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
                            conservative_covered=lower, production_lower_bound=percent,
                            publication_gap_lines=max(0,math.ceil(count*.9)-lower),
                            file_gaps=sorted([dict(file=str(Path(file['filename']).relative_to(ROOT)),
                                uncovered=file['summary']['lines']['count']-file['summary']['lines']['covered'])
                                for file in measured],key=lambda file:file['uncovered'],reverse=True))
        print(f'{name}: production >= {percent:.2f}% ({lower}/{count}), raw {covered}/{count}', flush=True)
    (LOGS/'coverage-production.json').write_text(json.dumps(report, indent=2))
    below=[name for name,result in report.items() if result['production_lower_bound'] < 70]
    assert not below, f'crates below development floor: {below}'
