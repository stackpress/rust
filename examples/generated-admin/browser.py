#!/usr/bin/env python3
"""Parent watchdog for intercepted Chromium; no listener or package installation."""
import os
from verify import run, ROOT

if __name__ == '__main__':
    if not os.environ.get('ADMIN_PLAYWRIGHT'):
        raise RuntimeError('set ADMIN_PLAYWRIGHT to the installed Playwright module')
    run('browser', ['node', str(ROOT / 'examples/generated-admin/browser.cjs')], timeout=120)
