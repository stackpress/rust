#!/usr/bin/env python3
"""Bounded admin proofs. See README for artifact prerequisites and the complete case matrix."""
import argparse
from proofs.runtime import Context


def main(action=None, suite=None):
    """Dispatch one explicit gate; no action starts a listener or installs a dependency."""
    if action is None:
        parser = argparse.ArgumentParser(description=__doc__)
        parser.add_argument('action', choices=['build','browser','checks','compile','coverage','report','graphs','probe'])
        parser.add_argument('--suite', choices=['roles','crud','format','transfer','plugin','races'], default='transfer')
        args = parser.parse_args()
        action, suite = args.action, args.suite
    # Format scenarios include the disabled-plugin build and the replacement build.
    ctx = Context('format' if suite == 'plugin' else suite)
    if action == 'build':
        from proofs.builds import build
        build(ctx,suite)
    elif action == 'browser':
        from proofs.browser import browser
        browser(ctx,suite)
    elif action == 'checks':
        from proofs.checks import check
        check(ctx)
    elif action == 'compile':
        from proofs.generation import compile_proof
        compile_proof(ctx)
    elif action == 'coverage':
        from proofs.coverage import measure
        measure(ctx)
    elif action == 'report':
        from proofs.coverage import report
        report(ctx)
    elif action == 'graphs':
        from proofs.graphs import compare
        compare(ctx)
    elif action == 'probe':
        from proofs.probe import probe
        probe(ctx)


if __name__ == '__main__':
    main()
