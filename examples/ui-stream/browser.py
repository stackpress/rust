#!/usr/bin/env python3
"""Parent 120 second kill/reap watchdog for Chromium and its native host children."""
import os
from pathlib import Path
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = Path('/private/tmp/s8-stream')


def main():
    """Never let a hung browser, service worker or native host escape the proof."""
    if not os.environ.get('UI_STREAM_PLAYWRIGHT'):
        raise RuntimeError('UI_STREAM_PLAYWRIGHT must name the already installed Playwright package')
    OUT.mkdir(exist_ok=True)
    with (OUT / 'browser.log').open('w') as log:
        child = subprocess.Popen(['node', str(ROOT / 'examples/ui-stream/browser.cjs')], cwd=ROOT, env=os.environ, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = child.wait(timeout=120)
            if code: raise RuntimeError(f'browser proof failed ({code}); see {OUT}/browser.log')
        finally:
            # Kill the owned process group even if the leader exited, then reap.
            try: os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            child.wait()
    print('PASS browser proof')


if __name__ == '__main__': main()
