#!/usr/bin/env python3
"""One M6-owned target and bounded child processes; never touch another proof's output."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
LOGS = Path('/private/tmp/s8-admin/m6-admin-agent')
TARGET = LOGS / 'target'


def run(name, command, timeout=900, extra=None):
    """Retain exact commands and fail after killing/reaping a timed-out process group."""
    LOGS.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), CARGO_LLVM_COV_TARGET_DIR=str(TARGET),
               CARGO_BUILD_JOBS='4', CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    env.update(extra or {})
    started = time.monotonic()
    with (LOGS / f'{name}.log').open('w') as log:
        log.write('$ ' + ' '.join(command) + '\n')
        log.flush()
        with subprocess.Popen(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT,
                              start_new_session=True) as child:
            expired = False
            try:
                code = child.wait(timeout=timeout)
            except BaseException:
                expired = True
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
                raise
            finally:
                (LOGS / f'{name}-result.json').write_text(json.dumps({
                    'command': command, 'code': child.returncode, 'expired': expired,
                    'seconds': time.monotonic() - started}, indent=2))
    print(f'{name}: {code}', flush=True)
    if code:
        raise SystemExit(code)
    # Rustc may remove a transient file while du traverses it. Retry the owned-tree measure.
    size = 0
    for attempt in range(3):
        measured = subprocess.run(['du', '-sk', str(TARGET)], capture_output=True, text=True, timeout=30) if TARGET.exists() else None
        if measured is None:
            break
        if measured.stdout.split():
            size = int(measured.stdout.split()[0])
        if measured.returncode == 0:
            break
        if attempt == 2:
            raise RuntimeError('Could not measure the owned target after three attempts: '+measured.stderr)
    if size > 8 * 1024 * 1024:
        raise RuntimeError('M6 target exceeded its 8 GiB budget')


if __name__ == '__main__':
    run(sys.argv[1], sys.argv[2:],timeout=120 if sys.argv[1].startswith('browser') else 900)
