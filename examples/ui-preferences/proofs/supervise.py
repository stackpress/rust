#!/usr/bin/env python3
"""Bound the one task target and command lifetime; terminate and reap owned groups."""
import argparse
import os
from pathlib import Path
import signal
import subprocess
import time


def stop(child):
    """Terminate a still-live owned group, then reap its direct child exactly once."""
    # Do not signal a PID/group after its direct child has already been reaped.
    if child.poll() is None:
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    child.wait()


def target_usage(target):
    """Retain du's total when Cargo unlinks a temporary file during the walk."""
    usage = subprocess.run(['du', '-sk', str(target)], text=True, capture_output=True, timeout=10)
    # ENOENT for children that vanished only removes already-deleted bytes. Other errors
    # cannot establish a complete bound and must still stop the supervised command.
    vanished = usage.stderr.splitlines()
    transient = vanished and all(line.startswith('du: ' + str(target) + '/') and
                                 line.endswith(': No such file or directory') for line in vanished)
    if usage.returncode and not (usage.returncode == 1 and transient):
        raise subprocess.CalledProcessError(usage.returncode, usage.args, usage.stdout, usage.stderr)
    total, path = usage.stdout.strip().split('\t', 1)
    if path != str(target) or not total.isdecimal():
        raise RuntimeError('target usage total unavailable')
    return int(total)


def main():
    """Supervise only an explicitly passed command; never start a listener implicitly."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', type=Path, required=True)
    parser.add_argument('--seconds', type=int, default=900)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('command required')
    started = time.monotonic()
    peak = 0
    child = subprocess.Popen(command, start_new_session=True)

    def interrupted(signum, _frame):
        """Recorder termination must also close the supervised child's separate group."""
        raise KeyboardInterrupt(f'signal {signum}')

    signal.signal(signal.SIGTERM, interrupted)
    try:
        while child.poll() is None:
            if args.target.exists():
                # du counts allocated disk blocks; its own read has a finite deadline.
                peak = max(peak, target_usage(args.target))
                if peak > 8 * 1024 * 1024:
                    raise RuntimeError('8 GiB aggregate target ceiling exceeded')
            if time.monotonic() - started > args.seconds:
                raise TimeoutError(f'command exceeded {args.seconds}s')
            time.sleep(1)
        if child.returncode:
            raise subprocess.CalledProcessError(child.returncode, command)
    finally:
        stop(child)
        print(f'supervisor: child {child.pid} reaped; peak target {peak} KiB', flush=True)


if __name__ == '__main__':
    main()
