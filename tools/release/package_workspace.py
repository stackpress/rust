#!/usr/bin/env python3
"""Package the full candidate workspace, verify archives, and dry-run the five leaves.

No registry writes are possible: the only publish invocation includes --dry-run.
--allow-dirty is solely for implementer evidence before the coordinator commits.
"""
import argparse
import subprocess

from common import ROOT, EVIDENCE, candidates, manifest, run, write_report


def main():
    """Use Cargo 1.96 workspace-local package resolution in both archive passes."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--allow-dirty', action='store_true')
    parser.add_argument('--pending-tests', action='store_true')
    args = parser.parse_args()
    approved = {name for _, name in candidates()}
    members = manifest(ROOT / 'Cargo.toml')['workspace']['members']
    excluded = [manifest(ROOT / member / 'Cargo.toml')['package']['name'] for member in members
                if manifest(ROOT / member / 'Cargo.toml')['package']['name'] not in approved]
    flags = ['--workspace', '--locked', '--offline']
    if args.allow_dirty:
        flags.append('--allow-dirty')
    for name in excluded:
        flags += ['--exclude', name]
    outcomes = []
    commands = [('workspace-no-verify', ['cargo', 'package', *flags, '--no-verify']),
                ('workspace-verify', ['cargo', 'package', *flags]),
                ('archive-checker', ['python3', str(ROOT / 'tools/release/check_packages.py'), '--archives'])]
    if args.pending_tests:
        commands[-1][1].append('--pending-tests')
    for leaf in ['stackpress-lib', 'stackpress-sql', 'stackpress-leptos', 'stackpress-idea-parser',
                 'stackpress-framework-language']:
        # The initial offline probes required HTTP. The coordinator authorizes
        # anonymous index reads for these dry-runs; no token or upload is used.
        command = ['cargo', 'publish', '--dry-run', '-p', leaf, '--locked']
        if args.allow_dirty:
            command.append('--allow-dirty')
        commands.append(('dry-run-' + leaf, command))
    for phase, command in commands:
        # Continue independent dry-runs when an archive-reference blocker belongs to another lane.
        try:
            run(command)
            code = 0
        except subprocess.CalledProcessError as error:
            code = error.returncode
        outcomes.append({'phase': phase, 'command': command, 'code': code})
        write_report(EVIDENCE / 'packaging-results.json', outcomes)
    return int(any(row['code'] for row in outcomes))


if __name__ == '__main__':
    raise SystemExit(main())
