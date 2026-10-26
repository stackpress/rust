#!/usr/bin/env python3
"""Parent watchdog for a finite intercepted browser proof; no listener or installation."""
import os
from pathlib import Path
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[2]
LOGS = Path('/private/tmp/s8-m0')

def main():
    """Kill/reap a stuck proof and retain its output, never treating timeout as success."""
    LOGS.mkdir(parents=True, exist_ok=True)
    (LOGS / 'gallery-browser.json').unlink(missing_ok=True)
    with (LOGS / 'gallery-browser.log').open('w') as log:
        with subprocess.Popen(['node',str(ROOT / 'examples/ui-gallery/browser.cjs')], cwd=ROOT,
                              stdout=log, stderr=subprocess.STDOUT, start_new_session=True) as child:
            try:
                code=child.wait(timeout=120)
                if code: raise RuntimeError(f'gallery browser failed ({code}); see {LOGS / "gallery-browser.log"}')
            except BaseException:
                if child.poll() is None: os.killpg(child.pid,signal.SIGKILL)
                child.wait()
                raise
    print('PASS gallery-browser',flush=True)

if __name__=='__main__': main()
