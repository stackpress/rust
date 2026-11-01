#!/usr/bin/env python3
"""Prove the consumer watchdog forcibly kills and reaps a deliberately stuck child."""
import importlib.util
from pathlib import Path
import select
import signal
import subprocess
import sys

script = Path(__file__).resolve().parents[1]/'consumers/run.py'
spec = importlib.util.spec_from_file_location('aggregate_consumers', script)
consumer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(consumer)
child = subprocess.Popen([sys.executable, '-u', '-c',
    'import signal;signal.signal(signal.SIGTERM,signal.SIG_IGN);print("ready",flush=True);signal.pause()'],
    text=True, stdout=subprocess.PIPE, start_new_session=True)
try:
    assert select.select([child.stdout], [], [], 5)[0]
    assert child.stdout.readline().strip() == 'ready'
    try:
        consumer.bounded(child, .2)
    except consumer.WatchdogTimeout as failure:
        assert failure.forced and child.returncode == -signal.SIGKILL
    else:
        raise AssertionError('negative control completed without forced kill')
finally:
    # Even a failed readiness assertion must not leave the intentionally unresponsive child alive.
    if child.poll() is None:
        import os
        os.killpg(child.pid, signal.SIGKILL)
        child.wait(timeout=2)
print('External consumer watchdog negative control: SIGKILL and reap passed.')
