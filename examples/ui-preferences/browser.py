#!/usr/bin/env python3
"""120-second parent watchdog; kill and reap the full child group on timeout."""
import os
from pathlib import Path
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[2]
LOGS = Path(os.environ.get('PREFERENCES_LOGS', '/private/tmp/s8-lane-b'))


def main():
    """Run only finite .invalid request interception, with no listener."""
    LOGS.mkdir(parents=True, exist_ok=True)
    (LOGS / 'browser.json').unlink(missing_ok=True)
    if not os.environ.get('PREFERENCES_PLAYWRIGHT'):
        raise RuntimeError('supply the installed Playwright package via PREFERENCES_PLAYWRIGHT')
    env = dict(os.environ, CARGO_TARGET_DIR=os.environ.get('PROFILE_UI_NATIVE_TARGET', os.environ.get('CARGO_TARGET_DIR', str(ROOT / 'target/s8-native'))))
    (LOGS / 'screenshots').mkdir(exist_ok=True)
    # Both maintained proofs are part of this standard command, each with a finite child group.
    for name, script in [('browser', 'browser.cjs'), ('first-paint', 'proofs/first-paint.cjs')]:
        (LOGS / f'{name}.json').unlink(missing_ok=True)
        with (LOGS / f'{name}.log').open('w') as log:
            with subprocess.Popen(['node', str(ROOT / 'examples/ui-preferences' / script)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True) as child:
                try:
                    code = child.wait(timeout=120)
                    if code:
                        raise RuntimeError(f'proof failed ({code}); see {LOGS / (name + ".log")}')
                finally:
                    if child.poll() is None:
                        os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
    print('PASS preferences-browser', flush=True)


if __name__ == '__main__':
    main()
