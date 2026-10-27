#!/usr/bin/env python3
"""Finite subprocess runner for M4 gates; retained logs, process-group kill and reap."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
LOGS = Path('/private/tmp/s8-admin')
TARGET = LOGS / 'target'


def run(name, command, timeout=900, extra=None):
    """Keep a bounded child log and make timeout a failing gate, never an ignored result."""
    LOGS.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), CARGO_BUILD_JOBS='4',
               CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    env.update(extra or {})
    start = time.monotonic()
    with (LOGS / f'{name}.log').open('w') as log:
        log.write('$ ' + ' '.join(command) + '\n')
        log.flush()
        with subprocess.Popen(command, cwd=ROOT, env=env, stdout=log,
                              stderr=subprocess.STDOUT, start_new_session=True) as child:
            expired = False
            try:
                code = child.wait(timeout=timeout)
            except BaseException:
                expired = True
                os.killpg(child.pid, signal.SIGKILL)
                code = child.wait()
                raise
            finally:
                (LOGS / f'{name}-result.json').write_text(json.dumps({
                    'command': command, 'code': child.returncode,
                    'expired': expired, 'seconds': time.monotonic() - start,
                }, indent=2))
    print(f'{name}: {code}', flush=True)
    if code:
        raise SystemExit(code)


if __name__ == '__main__':
    run(sys.argv[1], sys.argv[2:], timeout=int(os.environ.get('ADMIN_TIMEOUT', '900')))
