#!/usr/bin/env python3
"""Bounded M1-only builds/proofs; unrelated static example assets are borrowed explicitly."""
import argparse
import shutil
import shlex
import time
from pathlib import Path
import review

PACKAGES = ['stackpress-framework-ui', 'generated-admin-shared', 'generated-admin-web', review.PACKAGE]
PREFIXES = ['crates/framework/framework-ui/src/', 'examples/generated-admin/shared/src/',
            'examples/generated-admin/web/src/', 'examples/review-preview/src/']


def coverage(ctx):
    """Measure the four M1 native crates, including a real instrumented review pipe."""
    import coverage as coverage_report
    from proofs.runtime import TARGET
    selection = [item for package in PACKAGES for item in ['-p', package]]
    # Ordinary builds share this target. The wrapper's instrumentation flags are
    # invisible to Cargo freshness, so discard reusable workspace objects first.
    ctx.run('m1-coverage-clean', ['cargo', 'llvm-cov', 'clean', '--workspace', '--offline'])
    ctx.run('m1-coverage-tests', ['cargo', 'llvm-cov', *selection, '--features', 'review-preview',
                               '--offline', '--locked', '--no-report'], timeout=900,
            extra={'UI_STREAM_OUTPUT_DIR': str(ctx.logs.parent / 'stream')})
    ctx.run('m1-coverage-env', ['cargo', 'llvm-cov', 'show-env', '--sh'])
    environment = {}
    for line in (ctx.logs / 'm1-coverage-env.log').read_text().splitlines():
        if line.startswith('export '):
            key, value = line[7:].split('=', 1)
            environment[key] = shlex.split(value)[0]
    assert Path(environment['CARGO_LLVM_COV_TARGET_DIR']) == TARGET
    environment['STACKPRESS_REVIEW_VARIANT'] = 'disabled'
    ctx.run('m1-coverage-build', ['cargo', 'build', '--offline', '--locked', '-p', review.PACKAGE,
                               '--features', 'review-preview'], extra=environment, timeout=600)
    binary = ctx.logs.parent / 'bundle/disabled/slice8-review'
    backup = ctx.logs / 'm1-uninstrumented-slice8-review'
    shutil.copy2(binary, backup)
    try:
        shutil.copy2(TARGET / 'debug/slice8-review', binary)
        ctx.run('m1-coverage-browser', ['node', 'examples/review-preview/owner-findings-browser.cjs'],
                extra={**environment, 'REVIEW_DIRECTORY': str(ctx.logs.parent),
                       'REVIEW_PLAYWRIGHT': review.PLAYWRIGHT}, timeout=240)
        shutil.copy2(ctx.logs / 'm1-browser.json', ctx.logs / 'm1-coverage-browser.json')
    finally:
        shutil.copy2(backup, binary)
    coverage_report.report(ctx, selection, PACKAGES, PREFIXES)


def build(ctx, baseline):
    """Fresh matching admin/gateway variants; never regenerate tracked output."""
    directory = ctx.logs.parent
    for relative in ['gallery/gallery-assets', 'preferences/assets', 'stream/assets', 'format/assets']:
        source = baseline / relative
        if not source.is_dir():
            raise RuntimeError('Missing explicitly supplied baseline assets: ' + str(source))
        shutil.copytree(source, directory / relative, dirs_exist_ok=True)
    # Both variants regenerate and compile only inside the existing disposable workspace.
    review.bundles(ctx)
    from proofs.runtime import TARGET
    ctx.run('m1-standalone-build', ['cargo', 'build', '--offline', '--locked', '-p', 'generated-admin-web'])
    shutil.copy2(TARGET / 'debug/generated-admin-web', directory / 'role-catalogue-host')


def main():
    """Every long-running action inherits the one target, cap and process-group watchdog."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['build', 'browser', 'coverage'])
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--baseline', type=Path)
    args = parser.parse_args()
    ctx = review.context(args.directory)
    if args.action == 'build':
        if args.baseline is None:
            parser.error('build requires an explicit --baseline asset directory')
        build(ctx, args.baseline.resolve())
    elif args.action == 'coverage':
        coverage(ctx)
    else:
        for name in ['m1-browser.log', 'm1-browser-result.json', 'm1-browser.json']:
            previous = ctx.logs / name
            if previous.exists():
                shutil.copy2(previous, ctx.logs / (name + '.attempt-' + str(time.time_ns())))
        ctx.run('m1-browser', ['node', 'examples/review-preview/owner-findings-browser.cjs'], timeout=240,
                extra={'REVIEW_DIRECTORY': str(ctx.logs.parent), 'REVIEW_PLAYWRIGHT': review.PLAYWRIGHT})


if __name__ == '__main__':
    main()
