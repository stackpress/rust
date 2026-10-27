#!/usr/bin/env python3
"""Each CRUD browser case has its own 120-second parent watchdog and owned SQLite fixture."""
import json
import os
from m5_verify import LOGS, ROOT, run


if __name__=='__main__':
    if not os.environ.get('ADMIN_PLAYWRIGHT'):
        raise RuntimeError('ADMIN_PLAYWRIGHT must name the installed module')
    for mount in ['/','/admin/']:
        for mode in ['hydrate','csr']:
            for width,zoom in [(1280,1),(375,2)]:
                case=dict(mount=mount,mode=mode,width=width,zoom=zoom)
                tag=f"{'root' if mount=='/' else 'admin'}-{mode}-{width}-{zoom}"
                run('browser-'+tag,['node',str(ROOT/'examples/generated-admin/m5_browser.cjs')],timeout=120,
                    extra={'ADMIN_BROWSER_CASE':json.dumps(case),'ADMIN_BROWSER_OUTPUT':str(LOGS),'ADMIN_BROWSER_TAG':tag})
