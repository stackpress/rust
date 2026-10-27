#!/usr/bin/env python3
"""Measure native implementation in every changed crate, including the pipe executable."""
import argparse
import json
from pathlib import Path
import re
import shlex
import shutil
import review

PACKAGES = [review.PACKAGE, 'stackpress-example-ui-gallery-web',
            'stackpress-example-ui-preferences-web', 'stackpress-example-ui-preferences-shared',
            'stackpress-example-ui-stream-web', 'generated-admin-web']
PREFIXES = ['examples/review-preview/src/', 'examples/ui-gallery/web/src/',
            'examples/ui-preferences/web/src/', 'examples/ui-preferences/shared/src/',
            'examples/ui-stream/web/src/', 'examples/generated-admin/web/src/']


def main():
    """Retain matching owner binaries while temporarily exercising an instrumented pipe host."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--report-only', action='store_true', help='Reuse retained matching native counters/objects')
    args = parser.parse_args(); ctx = review.context(args.directory)
    from proofs.runtime import TARGET
    selection = [value for package in PACKAGES for value in ['-p', package]]
    if args.report_only:
        report(ctx, selection)
        return
    root = ctx.logs.parent
    common = dict(GALLERY_OUTPUT_DIR=str(root/'gallery'), PREFERENCES_LOGS=str(root/'preferences'),
                  UI_STREAM_OUTPUT_DIR=str(root/'stream'), ADMIN_PROOF_ROOT=str(root))
    ctx.run('coverage-tests', ['cargo', 'llvm-cov', *selection, '--features', 'review-preview',
                             '--offline', '--locked', '--no-report'], extra=common, timeout=1800)
    ctx.run('coverage-env', ['cargo', 'llvm-cov', 'show-env', '--sh'])
    environment = {}
    for line in (ctx.logs/'coverage-env.log').read_text().splitlines():
        if line.startswith('export '):
            key, value = line[7:].split('=', 1)
            environment[key] = shlex.split(value)[0]
    assert Path(environment['CARGO_LLVM_COV_TARGET_DIR']) == TARGET
    environment['STACKPRESS_REVIEW_VARIANT'] = 'disabled'
    ctx.run('coverage-pipe-build', ['cargo', 'build', '--offline', '--locked', '-p', review.PACKAGE,
                                  '--features', 'review-preview'], extra=environment, timeout=1800)
    binary = root/'bundle/disabled/slice8-review'
    backup = ctx.logs/'coverage-original-slice8-review'
    shutil.copy2(binary, backup)
    try:
        shutil.copy2(TARGET/'debug/slice8-review', binary)
        ctx.run('coverage-browser', ['python3', 'examples/review-preview/verify.py', 'smoke',
                                    '--directory', str(root)], extra=environment, timeout=180)
        shutil.copy2(ctx.logs/'browser.json', ctx.logs/'coverage-browser.json')
    finally:
        shutil.copy2(backup, binary)
    report(ctx, selection)


def report(ctx, selection):
    """Use LLVM LCOV DA records, matching the existing ui-format coverage convention."""
    from proofs.runtime import TARGET
    ctx.run('coverage-command', ['cargo', 'llvm-cov', 'report', *selection, '--json',
                                '--output-path', str(ctx.logs/'coverage.json'), '-vv'])
    line = next(line for line in (ctx.logs/'coverage-command.log').read_text().splitlines()
                if 'llvm-cov export' in line)
    command = shlex.split(line.split('`')[1])
    # Cargo's automatic examples exclusion hides these executable example crates.
    # Export all counters, then select exact src roots without hiding generated code.
    index = command.index('-ignore-filename-regex')
    del command[index:index+2]
    ctx.run('coverage-raw', command, timeout=60)
    command[command.index('-format=text')] = '-format=lcov'
    ctx.run('coverage-lcov', command, timeout=60)
    lcov = (ctx.logs/'coverage-lcov.log').read_text().split('\n', 1)[1]
    for profile in TARGET.glob('*.profdata'):
        shutil.copy2(profile, ctx.logs/profile.name)
    results = {}
    for package, prefix in zip(PACKAGES, PREFIXES):
        excluded = {}
        for source in (review.ROOT/prefix).rglob('*.rs'):
            lines = source.read_text().splitlines()
            for start, line in enumerate(lines[:-1]):
                if line == '#[cfg(test)]' and lines[start+1] == 'mod tests {':
                    end = next(index for index in range(start+2, len(lines)) if lines[index] == '}')
                    excluded.setdefault(str(source), set()).update(range(start+1, end+2))
        files = {}
        for record in lcov.split('end_of_record'):
            match = re.search(r'^SF:(.+)$', record, re.M)
            if not match or not match[1].startswith(str(review.ROOT/prefix)):
                continue
            path = match[1]
            if Path(path).name.endswith(('tests.rs', '_tests.rs')):
                continue
            counts = files.setdefault(path, {})
            # Count each physical executable source line once, coalescing instantiations.
            # Only exact inline-test module ranges are removed, from both quantities.
            for number, hits in re.findall(r'^DA:(\d+),(\d+)', record, re.M):
                number, hits = int(number), int(hits)
                if number not in excluded.get(path, set()):
                    counts[number] = max(counts.get(number, 0), hits)
        measured = [dict(path=path, covered=sum(hits > 0 for hits in counts.values()),
                         executable=len(counts), uncovered=[line for line, hits in counts.items() if not hits],
                         excluded_test_lines=sorted(excluded.get(path, set())))
                    for path, counts in files.items()]
        covered = sum(file['covered'] for file in measured)
        count = sum(file['executable'] for file in measured)
        assert count, package+' has no measured implementation'
        results[package] = dict(covered=covered, executable=count, percent=covered*100/count,
                               convention='LLVM LCOV DA physical executable source lines', files=measured)
        print(f'{package}: {covered}/{count} ({covered*100/count:.2f}%)', flush=True)
    (ctx.logs/'coverage-production.json').write_text(json.dumps(results, indent=2)+'\n')
    assert all(value['percent'] >= 70 for value in results.values()), 'Development coverage floor'


if __name__ == '__main__':
    main()
