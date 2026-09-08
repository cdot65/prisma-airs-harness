"""Bounded real-terminal driver for deterministic and live AIRS acceptance."""

import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import termios
import time


class TerminalSession:
    def __init__(self, binary, environment, directory):
        self.master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))

        def controlling_terminal():
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

        try:
            self.process = subprocess.Popen(
                [str(binary), "--no-alt-screen"],
                env=dict(environment, TERM="xterm-256color"),
                cwd=directory,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                preexec_fn=controlling_terminal,
            )
        except BaseException:
            os.close(self.master)
            raise
        finally:
            os.close(slave)
        self.transcript = bytearray()

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        try:
            if self.process.poll() is None:
                self.process.terminate()
                # Keep consuming terminal output during shutdown. In particular,
                # an npm launcher must reap its child before it can exit; a full
                # PTY buffer must not block the child's terminal restoration.
                deadline = time.monotonic() + 10
                while self.process.poll() is None and time.monotonic() < deadline:
                    ready, _, _ = select.select([self.master], [], [], 0.1)
                    if ready:
                        try:
                            chunk = os.read(self.master, 65536)
                        except OSError:
                            break
                        if not chunk:
                            break
                        self.transcript.extend(chunk)
                try:
                    self.process.wait(timeout=max(0.1, deadline - time.monotonic()))
                except subprocess.TimeoutExpired as error:
                    raise AssertionError(
                        self.transcript.decode(errors="replace")[-5000:]
                    ) from error
        finally:
            if self.process.poll() is None:
                # This driver creates a dedicated process group for its fixture.
                # Reap failed fixtures, including the npm launcher's native child.
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait(timeout=5)
            os.close(self.master)

    def wait_until(self, predicate, timeout=30):
        deadline = time.monotonic() + timeout
        while not predicate() and time.monotonic() < deadline:
            ready, _, _ = select.select([self.master], [], [], 0.1)
            if ready:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                self.transcript.extend(chunk)
        if not predicate():
            raise AssertionError(self.transcript.decode(errors="replace")[-5000:])

    def wait_for(self, marker, offset=0, timeout=30):
        self.wait_until(lambda: marker in self.transcript[offset:], timeout)

    def send_line(self, text):
        # Mark synthetic input as paste so rapid key events cannot defer Enter
        # into a newline while the terminal's paste-burst detector settles.
        os.write(self.master, b"\x1b[200~" + text.encode() + b"\x1b[201~")
        time.sleep(0.25)
        os.write(self.master, b"\r")

    def start(self):
        self.wait_for(b"Yes, continue")
        time.sleep(0.35)
        offset = len(self.transcript)
        os.write(self.master, b"\r")
        self.wait_for(b"permissions:", offset)
        time.sleep(0.35)

    def choose_model(self, direction, expected):
        offset = len(self.transcript)
        self.send_line("/model")
        self.wait_for(b"Select Model", offset)
        os.write(self.master, b"\x1b[B" if direction == "down" else b"\x1b[A")
        time.sleep(0.25)
        os.write(self.master, b"\r")
        self.wait_for(f"Model changed to {expected}".encode(), offset)
        time.sleep(0.35)
