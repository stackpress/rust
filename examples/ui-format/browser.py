#!/usr/bin/env python3
"""120-second parent watchdog; timeout kills and reaps the complete browser group."""
import os
from pathlib import Path
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ.get('UI_FORMAT_OUTPUT_DIR', '/private/tmp/s8-format'))
if not os.environ.get('UI_FORMAT_PLAYWRIGHT'):
    raise RuntimeError('UI_FORMAT_PLAYWRIGHT must name an already installed Playwright package')
with (OUT / 'browser.log').open('w') as log:
    child = subprocess.Popen(['node', str(Path(__file__).with_suffix('.cjs'))], cwd=ROOT,
                             env=os.environ, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        code = child.wait(timeout=120)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGKILL)
        child.wait()
        raise RuntimeError('Chromium proof exceeded the 120 second parent deadline')
    if code:
        # Also stop descendants on a failed driver before leaving the finite wrapper.
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        raise RuntimeError(f'Chromium proof failed ({code}); see {OUT / "browser.log"}')
print('PASS Chromium proof')
