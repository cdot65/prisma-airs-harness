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
from pathlib import Path


def pin_inline_interface(environment):
    """Keep acceptance on inline scrollback unless a test chose its own settings.

    The harness starts fullscreen by default. Terminal contracts written for inline
    output pin it through the harness-wide settings file; an existing file, even an
    empty one, is left untouched so a test can exercise the real default.
    """
    home = environment.get("AIRS_HARNESS_HOME") or environment.get("AIRS_TERMINAL_HOME")
    if not home:
        return
    settings = Path(home) / "settings.toml"
    if settings.exists():
        return
    settings.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    settings.write_text("[tui]\nfullscreen_transcript = false\n")


class TerminalSession:
    def __init__(
        self,
        binary,
        environment,
        directory,
        *,
        arguments=None,
        terminal_type="xterm-256color",
    ):
        pin_inline_interface(environment)
        self.master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))

        def controlling_terminal():
            os.setsid()
            fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

        try:
            self.process = subprocess.Popen(
                [
                    str(binary),
                    *(arguments if arguments is not None else ["--no-alt-screen"]),
                ],
                env=dict(environment, TERM=terminal_type),
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
        stream_closed = False
        while not predicate() and time.monotonic() < deadline:
            if stream_closed:
                # A PTY can reach EOF just before waitpid observes process exit.
                # Keep polling the requested condition within the same deadline.
                time.sleep(0.01)
                continue
            ready, _, _ = select.select([self.master], [], [], 0.1)
            if ready:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError:
                    chunk = b""
                if not chunk:
                    stream_closed = True
                    continue
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
        selection = "default" if expected == "airs-gateway-default" else expected
        self.send_line("/model " + selection)
        self.wait_for(b"Verify and use this routing?", offset)
        time.sleep(0.25)
        os.write(self.master, b"\r")
        self.wait_for(b"Gateway routing applied", offset, timeout=60)
        time.sleep(0.35)
