#!/usr/bin/env python3
"""120-second parent watchdog; kill and reap the full child group on timeout."""
import os
from pathlib import Path
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[2]
LOGS = Path('/private/tmp/s8-lane-b')


def main():
    """Run only finite .invalid request interception, with no listener."""
    LOGS.mkdir(parents=True, exist_ok=True)
    (LOGS / 'browser.json').unlink(missing_ok=True)
    if not os.environ.get('PREFERENCES_PLAYWRIGHT'):
        raise RuntimeError('supply the installed Playwright package via PREFERENCES_PLAYWRIGHT')
    with (LOGS / 'browser.log').open('w') as log:
        with subprocess.Popen(['node', str(ROOT / 'examples/ui-preferences/browser.cjs')], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True) as child:
            try:
                code = child.wait(timeout=120)
                if code:
                    raise RuntimeError(f'proof failed ({code}); see {LOGS / "browser.log"}')
            finally:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                child.wait()
    print('PASS preferences-browser', flush=True)


if __name__ == '__main__':
    main()
