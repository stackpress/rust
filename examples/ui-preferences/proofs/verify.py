#!/usr/bin/env python3
"""Finite M1 native checks; run this outer command through the benchmark recorder."""
import subprocess
import sys


def run(*command):
    """Forward all Cargo stages and stop on the first real failure."""
    subprocess.run(command, check=True)


if __name__ == '__main__':
    scopes = ['-p', 'stackpress-framework-language', '-p', 'stackpress-framework-view',
              '-p', 'stackpress-leptos', '-p', 'stackpress-example-ui-preferences-shared',
              '-p', 'stackpress-example-ui-preferences-web', '-p', 'stackpress-example-profile-ui-shared',
              '-p', 'stackpress-example-profile-ui-web', '-p', 'stackpress-example-profile-ui-client',
              '-p', 'stackpress-example-ui-gallery-shared', '-p', 'stackpress-example-ui-gallery-web']
    features = ['--features', 'stackpress-framework-view/native-file,stackpress-framework-view/flash,stackpress-leptos/stream,stackpress-leptos/rich']
    run('cargo', 'test', '--offline', '--locked', *scopes, *features, '--all-targets')
    run('cargo', 'test', '--offline', '--locked', *scopes, *features, '--doc')
    run('cargo', 'clippy', '--offline', '--locked', *scopes, *features, '--all-targets', '--', '-D', 'warnings')
    run('cargo', 'run', '--offline', '--locked', '-p', 'stackpress-example-profile-codegen', '--', 'emit', 'generate', '-c')
    run('cargo', 'run', '--offline', '--locked', '-p', 'generated-admin-tool', '--', 'emit', 'generate', '--check')
    run('cargo', 'fmt', '--all', '--', '--check')
    run(sys.executable, 'examples/ui-preferences/proofs/contracts.py')
    run(sys.executable, '-m', 'unittest', 'discover', '-s', 'examples/ui-preferences/proofs', '-p', 'test_supervise.py')
