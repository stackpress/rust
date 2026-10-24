"""Drive/reap a native Rust test in a disposable PTY, with no network listener."""

import fcntl
import os
import pty
import select
import signal
import subprocess
import sys
import termios
import time


def own_terminal():
    """Give the child its own controlling terminal for crossterm's /dev/tty access."""
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


def main():
    """Bound the child's life and compare actual termios state after it exits."""
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    child = None
    transcript = bytearray()
    sent = False
    verified = False
    try:
        child = subprocess.Popen(
            [sys.argv[1], "--ignored", "--exact", "tty_native_child", "--nocapture"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            preexec_fn=own_terminal,
        )
        deadline = time.monotonic() + 8
        while child.poll() is None:
            if time.monotonic() >= deadline:
                raise TimeoutError("PTY child exceeded 8 seconds")
            ready, _, _ = select.select([master], [], [], 0.02)
            if ready:
                chunk = os.read(master, 4096)
                transcript.extend(chunk)
                if len(transcript) > 65536:
                    raise RuntimeError("PTY child exceeded bounded transcript")
            if not sent and b"PTY_READY" in transcript:
                os.write(master, b"\x03")
                sent = True
            if not verified and b"PTY_RESTORED" in transcript:
                assert termios.tcgetattr(slave) == before, "native termios was not restored"
                os.write(master, b"verified\n")
                verified = True
        # Drain already-written output without waiting for any additional input.
        while select.select([master], [], [], 0)[0]:
            chunk = os.read(master, 4096)
            if not chunk:
                break
            transcript.extend(chunk)
            if len(transcript) > 65536:
                raise RuntimeError("PTY transcript limit")
        status = child.wait()
        assert status == 0, transcript.decode(errors="replace")
        assert sent and b"PTY_RESTORED" in transcript, transcript.decode(errors="replace")
        assert verified, "native termios was not checked before session leader exit"
        print("PTY proof passed: Ctrl-C, idle stop/join, pending drop, unwind, prior raw mode")
    finally:
        # A failed assertion/deadline still terminates and reaps the exact child process.
        if child is not None and child.poll() is None:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            child.wait()
        os.close(master)
        os.close(slave)


if __name__ == "__main__":
    main()
