#!/usr/bin/env python3
"""Final native/workspace and browser-graph gates, all within the sole owned target."""
import re
from m5_verify import LOGS, run


def main():
    """Retain exact bounded commands; require clean normal dependency graphs for both modes."""
    run('fmt', ['cargo','fmt','--all','--','--check'])
    run('workspace-clippy', ['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings'])
    run('workspace-suite', ['cargo','test','--workspace','--locked'],timeout=900)
    run('msrv', ['cargo','+1.96.0','check','--workspace','--locked'])
    run('rustdoc', ['cargo','doc','-p','stackpress-framework-admin','-p','generated-admin-shared',
        '-p','generated-admin-web','-p','generated-admin-tool','--features',
        'stackpress-framework-admin/codegen,stackpress-framework-admin/ssr','--no-deps','--locked'],
        extra={'RUSTDOCFLAGS':'-D warnings'})
    for mode in ['hydrate','csr']:
        run('wasm-clippy-'+mode, ['cargo','clippy','-p','generated-admin-client','--target',
            'wasm32-unknown-unknown','--features',mode,'--locked','--','-D','warnings'])
        run('tree-'+mode, ['cargo','tree','-p','generated-admin-client','--target',
            'wasm32-unknown-unknown','--features',mode,'--edges','normal','--locked'])
        graph=(LOGS/('tree-'+mode+'.log')).read_text()
        assert not re.search(r'\b(?:stackpress-(?:sql|server|idea)|sqlx|tokio)\b',graph), graph


if __name__=='__main__':
    main()
