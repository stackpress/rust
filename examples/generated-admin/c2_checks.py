#!/usr/bin/env python3
"""Final native/workspace and browser-graph gates, all within the sole owned target."""
import re
from c2_verify import LOGS, run


def main():
    """Retain exact bounded commands; require clean normal dependency graphs for both modes."""
    run('fmt', ['cargo','fmt','--all','--','--check'])
    run('workspace-clippy', ['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings'])
    run('workspace-suite', ['cargo','test','--workspace','--locked'],timeout=900)
    run('format-clippy',['cargo','clippy','-p','stackpress-framework-admin','-p','generated-admin-shared','-p','generated-admin-web','-p','generated-admin-tool','--all-targets','--features','stackpress-framework-admin/codegen,generated-admin-web/format,generated-admin-tool/format','--locked','--','-D','warnings'])
    run('format-suite',['cargo','test','-p','stackpress-framework-admin','-p','generated-admin-shared','-p','generated-admin-web','-p','generated-admin-tool','--features','stackpress-framework-admin/codegen,generated-admin-web/format,generated-admin-tool/format','--locked'])
    run('msrv', ['cargo','+1.96.0','check','--workspace','--features','framework-admin-format,generated-admin-web/format,generated-admin-tool/format','--locked'])
    run('rustdoc', ['cargo','doc','-p','stackpress-framework-admin','-p','generated-admin-shared',
        '-p','generated-admin-web','-p','generated-admin-tool','--features',
        'stackpress-framework-admin/codegen,stackpress-framework-admin/ssr,generated-admin-web/format,generated-admin-tool/format','--no-deps','--locked'],
        extra={'RUSTDOCFLAGS':'-D warnings'})
    for mode in ['hydrate','csr']:
        for formatted in [False,True]:
            tag=mode+('-format' if formatted else '-default')
            features=mode+(',format' if formatted else '')
            run('wasm-clippy-'+tag,['cargo','clippy','-p','generated-admin-client','--target','wasm32-unknown-unknown','--features',features,'--locked','--','-D','warnings'])
            run('tree-'+tag,['cargo','tree','-p','generated-admin-client','--target','wasm32-unknown-unknown','--features',features,'--edges','normal','--locked'])
            graph=(LOGS/('tree-'+tag+'.log')).read_text()
            assert not re.search(r'\b(?:stackpress-(?:sql|server|idea)|sqlx|tokio)\b',graph)
            if not formatted:
                assert not re.search(r'\b(?:icu_(?:decimal|datetime|calendar|time|plurals)|phonenumber|fixed_decimal)\b',graph)


if __name__=='__main__':
    main()
