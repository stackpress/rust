#!/usr/bin/env python3
"""Final native/workspace and browser-graph gates, all within the sole owned target."""
import re


def check(ctx):
    """Keep native checks and each browser feature graph independently bounded."""
    LOGS, run = ctx.logs, ctx.run
    run('fmt', ['cargo','fmt','--all','--','--check'])
    run('workspace-clippy', ['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings'])
    # This lane starts no listeners; the coordinator may separately verify the TCP-only proof.
    run('workspace-suite-no-listener', ['cargo','test','--workspace','--locked','--','--skip','tcp_watchdog'],timeout=900)
    # Browser artifacts are already copied outside the target. Leave room for documentation
    # and the MSRV feature graph rather than accumulating every workspace cache together.
    if ctx.size_kib() > 5 * 1024 * 1024:
        run('release-workspace-cache', ['cargo','clean'])
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
            assert not re.search(r'\b(?:stackpress-(?:framework-sql|sql|server|idea)|sqlx|tokio)\b',graph)
            if not formatted:
                assert not re.search(r'\b(?:icu_(?:decimal|datetime|calendar|time|plurals)|phonenumber|fixed_decimal)\b',graph)

    run('facade', ['cargo','check','-p','stackpress','--no-default-features','--features','framework-admin','--locked'])
    run('tree-tool', ['cargo','tree','-p','generated-admin-tool','--edges','normal','--locked'])
    tool = (LOGS/'tree-tool.log').read_text()
    assert 'generated-admin-shared' not in tool and 'generated-workspace-app' not in tool
