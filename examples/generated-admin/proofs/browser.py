"""One matrix driver; scenario modules retain every milestone's distinct assertions."""
import json
import os
from .runtime import ROOT

SCRIPTS = {'roles': 'browser.cjs', 'crud': 'm5_browser.cjs',
           'format': 'c2_browser.cjs', 'transfer': 'm6_browser.cjs',
           'races': 'proofs/write_races.cjs'}


def browser(ctx, suite):
    """Run all mounts/modes/layouts, with a fresh 120-second parent for each case."""
    if not os.environ.get('ADMIN_PLAYWRIGHT'):
        raise RuntimeError('ADMIN_PLAYWRIGHT must name the installed module')
    script = str(ROOT/'examples/generated-admin'/SCRIPTS[suite])
    common = {'ADMIN_BROWSER_OUTPUT':str(ctx.logs)}
    if suite in ['roles', 'races']:
        if suite == 'races':
            common['ADMIN_REVIEW_ARTIFACTS'] = str(ctx.logs.parent/'crud')
        ctx.run('browser-'+suite, ['node',script], timeout=120, extra=common)
        return
    records = []
    for mount in ['/', '/admin/']:
        for mode in ['hydrate', 'csr']:
            for plugin in ([False, True] if suite == 'format' else [False]):
                for width, zoom in [(1280,1), (375,2)]:
                    case = dict(mount=mount,mode=mode,width=width,zoom=zoom,plugin=plugin)
                    tag = f"{'root' if mount=='/' else 'admin'}-{mode}-{width}-{zoom}"+('-plugin' if plugin else '')
                    ctx.run('browser-'+tag, ['node',script], timeout=120, extra={**common,
                        'ADMIN_BROWSER_CASE':json.dumps(case), 'ADMIN_BROWSER_TAG':tag})
                    records.append(dict(case=case,record='browser-'+tag+'.json'))
    (ctx.logs/'browser-matrix.json').write_text(json.dumps(records,indent=2))
