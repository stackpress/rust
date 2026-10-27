#!/usr/bin/env python3
"""Reproducible M4 host/WASM checks; each gate has an independent parent watchdog."""
from verify import LOGS, run

PACKAGES = ['-p', 'stackpress-framework-admin', '-p', 'generated-admin-tool',
            '-p', 'generated-admin-shared', '-p', 'generated-admin-web', '-p', 'generated-admin-client']
FEATURES = ['--features', 'stackpress-framework-admin/codegen,generated-admin-shared/ssr']


def main():
    """Keep client dependency graphs separate from host feature unification."""
    run('fmt-final', ['cargo', 'fmt', '--all', '--', '--check'])
    run('scoped-final', ['cargo', 'test', *PACKAGES, *FEATURES, '--locked'])
    run('msrv', ['cargo', '+1.96.0', 'check', *PACKAGES, *FEATURES, '--locked'])
    run('clippy-final', ['cargo', 'clippy', *PACKAGES, *FEATURES, '--all-targets', '--locked', '--', '-D', 'warnings'])
    run('rustdoc', ['cargo', 'doc', *PACKAGES, *FEATURES, '--no-deps', '--locked'], extra={'RUSTDOCFLAGS': '-D warnings'})
    run('facade', ['cargo', 'check', '-p', 'stackpress', '--no-default-features', '--features', 'framework-admin', '--locked'])
    for mode in ['hydrate', 'csr']:
        args = ['-p', 'generated-admin-client', '--target', 'wasm32-unknown-unknown', '--features', mode, '--locked']
        run(f'wasm-{mode}-check', ['cargo', 'check', *args])
        run(f'tree-{mode}', ['cargo', 'tree', *args, '--edges', 'normal'])
        graph = (LOGS / f'tree-{mode}.log').read_text()
        for forbidden in ['stackpress-framework-sql ', 'stackpress-sql ', 'stackpress-server ',
                          'stackpress-idea', 'tokio v', 'sqlx v', 'libsqlite3-sys v', 'stackpress-cli ']:
            assert forbidden not in graph, f'{mode}: forbidden browser edge {forbidden}'
    run('tree-tool', ['cargo', 'tree', '-p', 'generated-admin-tool', '--edges', 'normal', '--locked'])
    graph = (LOGS / 'tree-tool.log').read_text()
    assert 'generated-admin-shared' not in graph and 'generated-workspace-app' not in graph
    run('workspace', ['cargo', 'test', '--workspace', '--all-targets', '--locked'])
    run('workspace-doc', ['cargo', 'test', '--workspace', '--doc', '--locked'])
    run('workspace-clippy', ['cargo', 'clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings'])


if __name__ == '__main__':
    main()
