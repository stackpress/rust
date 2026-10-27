#!/usr/bin/env python3
"""Each finite Chromium case owns a SQLite fixture and a 120-second parent watchdog."""
import json
import os
from m6_verify import LOGS, ROOT, run


if __name__=='__main__':
    if not os.environ.get('ADMIN_PLAYWRIGHT'):
        raise RuntimeError('ADMIN_PLAYWRIGHT must name the installed module')
    records=[]
    for mount in ['/','/admin/']:
        for mode in ['hydrate','csr']:
            for plugin in [False]:
                for width,zoom in [(1280,1),(375,2)]:
                    case=dict(mount=mount,mode=mode,width=width,zoom=zoom,plugin=plugin)
                    tag=f"{'root' if mount=='/' else 'admin'}-{mode}-{width}-{zoom}"+('-plugin' if plugin else '')
                    run('browser-'+tag,['node',str(ROOT/'examples/generated-admin/m6_browser.cjs')],timeout=120,
                        extra={'ADMIN_BROWSER_CASE':json.dumps(case),'ADMIN_BROWSER_OUTPUT':str(LOGS),'ADMIN_BROWSER_TAG':tag})
                    records.append(dict(case=case,record=f'browser-{tag}.json'))
    (LOGS/'browser-matrix.json').write_text(json.dumps(records,indent=2))
