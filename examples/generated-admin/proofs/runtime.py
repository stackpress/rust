"""One configurable output root and target; all proof subprocesses have kill/reap watchdogs."""
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]
OUTPUT_ROOT = Path(os.environ.get('ADMIN_PROOF_ROOT', '/private/tmp/s8-admin')).expanduser().resolve()
TARGET = OUTPUT_ROOT / 'target'


class Context:
    """Own a proof's logs/artifacts, sharing only this run's one bounded Cargo target."""
    def __init__(self, suite):
        self.logs = OUTPUT_ROOT / suite
        self.logs.mkdir(parents=True, exist_ok=True)

    def size_kib(self, deadline=None):
        """Retry transient rustc removals while measuring; never ignore a failed measurement."""
        if not TARGET.exists():
            return 0
        deadline = deadline or time.monotonic() + 30
        for attempt in range(3):
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError('Target measurement deadline exceeded')
            result = subprocess.run(['du', '-sk', str(TARGET)], capture_output=True, text=True,
                                    timeout=remaining)
            if result.returncode == 0:
                return int(result.stdout.split()[0])
        raise RuntimeError('Cannot measure owned target: ' + result.stderr)

    def budget(self, deadline=None):
        """Refuse a target above the lane's eight GiB ceiling."""
        size = self.size_kib(deadline)
        if size > 8 * 1024 * 1024:
            raise RuntimeError('Admin proof target exceeded 8 GiB')
        return size

    def run(self, name, command, timeout=900, extra=None, cwd=None):
        """Record exact argv and reap the entire group after timeout, cancellation or budget failure."""
        env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), CARGO_LLVM_COV_TARGET_DIR=str(TARGET),
                   CARGO_BUILD_JOBS='4', CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
        env.update(extra or {})
        start = time.monotonic()
        next_measure = start + 15
        expired = False
        with (self.logs / (name + '.log')).open('w') as log:
            log.write('$ ' + shlex.join(command) + '\n')
            log.flush()
            with subprocess.Popen(command, cwd=cwd or ROOT, env=env, stdout=log, stderr=subprocess.STDOUT,
                                  start_new_session=True) as child:
                try:
                    while child.poll() is None:
                        if time.monotonic() - start >= timeout:
                            expired = True
                            raise TimeoutError(name + ' deadline exceeded')
                        if time.monotonic() >= next_measure:
                            # Measuring disk must never extend this child's outer watchdog.
                            self.budget(deadline=min(start + timeout, time.monotonic() + 30))
                            next_measure = time.monotonic() + 15
                        try:
                            child.wait(timeout=min(1, max(.001, timeout - (time.monotonic()-start))))
                        except subprocess.TimeoutExpired:
                            continue
                except BaseException:
                    expired = expired or time.monotonic() >= start + timeout
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    child.wait()
                    raise
                finally:
                    (self.logs / (name + '-result.json')).write_text(json.dumps({
                        'command': command, 'cwd':str(cwd or ROOT), 'code': child.returncode, 'expired': expired,
                        'seconds': time.monotonic()-start}, indent=2))
        print(f'{name}: {child.returncode}', flush=True)
        if child.returncode:
            raise SystemExit(child.returncode)
        self.budget()
