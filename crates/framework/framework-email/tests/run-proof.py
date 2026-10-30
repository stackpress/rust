#!/usr/bin/env python3
"""Run M0 concurrency proofs with an absolute process-group kill/reap deadline."""

import argparse
import os
import signal
import subprocess
import sys


def main():
    """Supervise one command while forwarding its Cargo output to the recorder."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--timeout", type=int, default=240)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command or args.timeout <= 0:
        parser.error("a command and positive deadline are required")
    child = subprocess.Popen(command, start_new_session=True)

    def terminate(signum, _frame):
        """Do not orphan the separately supervised process group on interruption."""
        raise KeyboardInterrupt(signum)

    signal.signal(signal.SIGTERM, terminate)
    signal.signal(signal.SIGINT, terminate)
    try:
        return child.wait(timeout=args.timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt):
        # Kill and reap the entire proof, including nextest and its test children.
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait()
        print("M0 proof watchdog killed and reaped command", file=sys.stderr)
        return 124


if __name__ == "__main__":
    sys.exit(main())
