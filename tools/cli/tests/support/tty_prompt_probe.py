"""Drive real production prompts using handshakes; no sleep-based interleavings."""

import fcntl
import os
import pty
import select
import signal
import subprocess
import struct
import sys
import termios
import time


def own_terminal():
    """Give the child its own controlling terminal for crossterm's /dev/tty reader."""
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


def main():
    """Advance scenarios only after actual acquisition/validation markers, then verify termios."""
    master, slave = pty.openpty()
    # A fixed narrow native width makes the wrapping/clear case independently observable.
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 8, 0, 0))
    before = termios.tcgetattr(slave)
    child = None
    transcript = bytearray()
    steps = [
        (b"WRAP_READY", "界\r".encode()),
        (b"BACKPRESSURE_READY", b"x" * 33),
        (b"SUCCESS_READY", "A😀\x7fda\r".encode()),
        (b"VALIDATE_READY", b"answer\r"),
        (b"VALIDATION_PENDING", b"\x03"),
        (b"OUTPUT_READY", b"done\r"),
        (b"EOF_READY", b"\x04"),
        (b"PANIC_READY", b"panic\r"),
    ]
    index = 0
    verified = False
    try:
        child = subprocess.Popen(
            [sys.argv[1], "--ignored", "--exact", "tty_prompt_child", "--nocapture"],
            stdin=slave, stdout=slave, stderr=slave, preexec_fn=own_terminal,
        )
        deadline = time.monotonic() + 8
        while child.poll() is None:
            if time.monotonic() >= deadline:
                raise TimeoutError("PTY prompt child exceeded 8 seconds")
            ready, _, _ = select.select([master], [], [], 0.02)
            if ready:
                transcript.extend(os.read(master, 4096))
                if len(transcript) > 65536:
                    raise RuntimeError("PTY transcript limit")
            if index < len(steps) and steps[index][0] in transcript:
                os.write(master, steps[index][1])
                index += 1
            if not verified and b"PROMPT_RESTORED" in transcript:
                assert termios.tcgetattr(slave) == before, "native termios was not restored"
                os.write(master, b"verified\n")
                verified = True
        status = child.wait()
        assert status == 0, transcript.decode(errors="replace")
        assert index == len(steps) and verified, transcript.decode(errors="replace")
        wrapped = transcript.split(b"WRAP_READY", 1)[1].split(b"WRAP_DONE", 1)[0]
        assert "? 界😀ab\r\ncdefg".encode() in wrapped, wrapped
        assert b"\r\x1b[2K\x1b[1A\r\x1b[2K" in wrapped, wrapped
        print("Production PTY prompt proof passed: input, Unicode, validation cancel, output, EOF, drop, panic, prior raw")
    finally:
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
