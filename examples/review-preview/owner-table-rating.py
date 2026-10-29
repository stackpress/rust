#!/usr/bin/env python3
"""M3 table/rating builds/proofs with one capped target and no listeners or installs."""
import argparse
from pathlib import Path
import shlex
import shutil
import review

PACKAGES = ['stackpress-framework-admin', 'stackpress-leptos', 'stackpress-example-ui-gallery-shared', 'stackpress-example-ui-gallery-web']
PREFIXES = ['crates/framework/framework-admin/src/', 'crates/leptos/src/', 'examples/ui-gallery/shared/src/', 'examples/ui-gallery/web/src/']
FEATURES = 'stackpress-framework-admin/ssr,stackpress-framework-admin/codegen,stackpress-leptos/ssr,stackpress-example-ui-gallery-shared/ssr'


def environment(ctx):
    """Point every browser proof at retained matching gallery bytes and the installed browser."""
    root = ctx.logs.parent
    return dict(GALLERY_OUTPUT_DIR=str(root/'gallery'), GALLERY_PLAYWRIGHT=review.PLAYWRIGHT,
                GALLERY_HOST=str(root/'gallery/native-host'),
                GALLERY_COLLECTION_HOST=str(root/'gallery/collections-host'),
                ADMIN_PLAYWRIGHT=review.PLAYWRIGHT)


def build(ctx):
    """Build the ordinary gallery and dedicated fixture, then both generated admin selections."""
    from proofs.runtime import Context, TARGET
    from proofs.builds import build as admin_build
    extra = environment(ctx)
    ctx.run('m3-gallery-build', ['python3', 'examples/ui-gallery/build.py'], extra=extra, timeout=1200)
    ctx.run('m3-collection-host', ['cargo', 'build', '-p', 'stackpress-example-ui-gallery-web', '--example', 'collections',
                                 '--offline', '--locked'])
    shutil.copy2(TARGET/'debug/stackpress-example-ui-gallery-web', ctx.logs.parent/'gallery/native-host')
    shutil.copy2(TARGET/'debug/examples/collections', ctx.logs.parent/'gallery/collections-host')
    admin_build(Context('roles'), 'roles')


def browser(ctx):
    """Preserve the ordinary breadth regression alongside the complete collection contract."""
    from proofs.runtime import Context
    extra = environment(ctx)
    ctx.run('m3-collections-browser', ['node', 'examples/ui-gallery/collections-browser.cjs'], extra=extra, timeout=240)
    ctx.run('m3-breadth-browser', ['node', 'examples/ui-gallery/breadth-browser.cjs'], extra=extra, timeout=300)
    Context('roles').run('m3-admin-browser', ['node', 'examples/generated-admin/browser.cjs'], extra=extra, timeout=240)


def coverage(ctx):
    """Measure native production and replay both real browser fixtures against instrumented hosts."""
    import coverage as coverage_report
    from proofs.runtime import TARGET
    selected = [item for package in PACKAGES for item in ['-p', package]]
    ctx.run('m3-coverage-clean', ['cargo', 'llvm-cov', 'clean', '--workspace', '--offline'])
    ctx.run('m3-coverage-tests', ['cargo', 'llvm-cov', *selected, '--features', FEATURES,
                               '--offline', '--locked', '--no-report'], timeout=900)
    ctx.run('m3-coverage-env', ['cargo', 'llvm-cov', 'show-env', '--sh'])
    extra = environment(ctx)
    for line in (ctx.logs/'m3-coverage-env.log').read_text().splitlines():
        if line.startswith('export '):
            key, value = line[7:].split('=', 1)
            extra[key] = shlex.split(value)[0]
    assert extra['CARGO_LLVM_COV_TARGET_DIR'] == str(TARGET)
    ctx.run('m3-coverage-build', ['cargo', 'build', '-p', 'stackpress-example-ui-gallery-web', '--examples', '--bins',
                               '--offline', '--locked'], extra=extra, timeout=600)
    extra.update(GALLERY_HOST=str(TARGET/'debug/stackpress-example-ui-gallery-web'),
                 GALLERY_COLLECTION_HOST=str(TARGET/'debug/examples/collections'))
    ctx.run('m3-coverage-collections', ['node', 'examples/ui-gallery/collections-browser.cjs'], extra=extra, timeout=240)
    ctx.run('m3-coverage-breadth', ['node', 'examples/ui-gallery/breadth-browser.cjs'], extra=extra, timeout=300)
    ctx.run('m3-coverage-gallery', ['node', 'examples/ui-gallery/browser.cjs'], extra=extra, timeout=300)
    coverage_report.report(ctx, selected, PACKAGES, PREFIXES)


def main():
    """Each action uses the existing disk-budget/process-group watchdog for every child."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['build', 'browser', 'coverage'])
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    ctx = review.context(args.directory)
    globals()[args.action](ctx)


if __name__ == '__main__':
    main()
