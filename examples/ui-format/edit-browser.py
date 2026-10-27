#!/usr/bin/env python3
"""Finite editor proof: kill/reap the whole Chromium group after 120 seconds."""
import os
from pathlib import Path
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ.get('UI_FORMAT_OUTPUT_DIR', '/private/tmp/s8-format'))
OUT.mkdir(parents=True, exist_ok=True)
(OUT / 'edit-browser.json').unlink(missing_ok=True)
with (OUT / 'edit-browser.log').open('w') as log:
    child = subprocess.Popen(['node', str(Path(__file__).with_suffix('.cjs'))], cwd=ROOT,
                             stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        code = child.wait(timeout=120)
        if code:
            raise RuntimeError(f'Editor proof failed ({code})')
    except BaseException:
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait()
        raise
print('PASS specialized editor Chromium proof')
