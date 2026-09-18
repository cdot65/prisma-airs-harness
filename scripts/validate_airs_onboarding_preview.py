#!/usr/bin/env python3
"""Exercise the synthetic Rust onboarding preview through a real Unix PTY.

Run with: uv run --with pyte==0.8.2 scripts/validate_airs_onboarding_preview.py
  --binary PATH --output DIRECTORY

Only synthetic preview data is captured. Never point this tool at an authenticated
production session; the preview executable performs no authentication or writes.
"""

import argparse
import fcntl
import gzip
import hashlib
import html
import json
import os
import platform
import pty
import select
import signal
import struct
import subprocess
import sys
import termios
import time
from pathlib import Path

import pyte


class Preview:
    def __init__(
        self,
        binary,
        screen="welcome",
        columns=80,
        rows=24,
        extra=(),
        *,
        arguments=None,
        environment=None,
        directory=None,
        terminal_stdout=False,
    ):
        self.terminal_stdout = terminal_stdout
        self.master, self.slave = pty.openpty()
        self.columns, self.rows = columns, rows
        fcntl.ioctl(
            self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, columns, 0, 0)
        )
        self.original = termios.tcgetattr(self.slave)
        self.screen = pyte.Screen(columns, rows)
        self.stream = pyte.ByteStream(self.screen)
        self.transcript = bytearray()
        self.started = time.monotonic()

        env = dict(
            os.environ if environment is None else environment,
            TERM="xterm-256color",
            COLORTERM="truecolor",
        )
        if environment is None:
            env.pop("NO_COLOR", None)
        # Acquire the controlling terminal after Python starts in a new session.
        # preexec_fn can deadlock when this driver shares a process with the
        # threaded HTTPS identity fixture.
        attach = (
            "import fcntl,os,sys,termios;"
            "fcntl.ioctl(0,termios.TIOCSCTTY,0);"
            "os.execv(sys.argv[1],sys.argv[1:])"
        )
        self.process = subprocess.Popen(
            [
                sys.executable,
                "-c",
                attach,
                str(binary),
                *(arguments if arguments is not None else [screen, *extra]),
            ],
            stdin=self.slave,
            stdout=self.slave if terminal_stdout else subprocess.PIPE,
            stderr=self.slave,
            env=env,
            cwd=directory,
            start_new_session=True,
        )

    def pump(self, duration=0.15):
        deadline = time.monotonic() + duration
        while time.monotonic() < deadline:
            ready, _, _ = select.select(
                [self.master], [], [], max(0, deadline - time.monotonic())
            )
            if not ready:
                break
            try:
                data = os.read(self.master, 65536)
            except OSError:
                break
            if not data:
                break
            self.transcript.extend(data)
            self.stream.feed(data)

    def expect(self, text, timeout=10):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            self.pump(0.05)
            if text in "\n".join(self.screen.display):
                return
            if self.process.poll() is not None:
                break
        raise AssertionError(
            f"Preview did not display {text!r}:\n" + "\n".join(self.screen.display)
        )

    def send(self, value):
        os.write(self.master, value)
        self.pump()

    def resize(self, columns, rows):
        self.columns, self.rows = columns, rows
        self.screen.resize(lines=rows, columns=columns)
        fcntl.ioctl(
            self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, columns, 0, 0)
        )
        os.kill(self.process.pid, signal.SIGWINCH)
        self.pump(0.25)

    def capture(self):
        return {
            "columns": self.columns,
            "rows": self.rows,
            "text": list(self.screen.display),
            "cells": [
                [dict(self.screen.buffer[y][x]._asdict()) for x in range(self.columns)]
                for y in range(self.rows)
            ],
        }

    def finish(self, expected, status=0):
        try:
            self.process.wait(timeout=5)
            self.pump(0.05)
            output = "" if self.terminal_stdout else self.process.stdout.read().decode()
            assert self.process.returncode == status, output
            assert expected in output, output
            assert termios.tcgetattr(self.slave) == self.original, (
                "Terminal modes were not restored"
            )
            if not self.terminal_stdout or b"\x1b[?1049h" in self.transcript:
                assert b"\x1b[?1049l" in self.transcript, (
                    "Alternate screen was not left"
                )
            assert b"\x1b[?25h" in self.transcript, "Cursor was not restored"
            assert b"\x1b[?2004l" in self.transcript, "Bracketed paste was not disabled"
            if not self.terminal_stdout:
                assert "\x1b" not in output, "Interactive escapes leaked to stdout"
            return output.strip()
        finally:
            self.close()

    def close(self):
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        for fd in (self.master, self.slave):
            try:
                os.close(fd)
            except OSError:
                pass


def css_color(value):
    palette = {
        "default": "inherit",
        "black": "#171a20",
        "red": "#e06c75",
        "green": "#98c379",
        "brown": "#e5c07b",
        "blue": "#61afef",
        "magenta": "#c678dd",
        "cyan": "#56b6c2",
        "white": "#d8dee9",
    }
    if value in palette:
        return palette[value]
    if len(value) == 6 and all(c in "0123456789abcdef" for c in value.lower()):
        return "#" + value
    return "inherit"


def terminal_html(capture):
    rows = []
    for row in capture["cells"]:
        runs, previous, text = [], None, ""
        for cell in row:
            styles = []
            if cell["fg"] != "default":
                styles.append(f"color:{css_color(cell['fg'])}")
            if cell["bg"] != "default":
                styles.append(f"background:{css_color(cell['bg'])}")
            if cell["bold"]:
                styles.append("font-weight:700")
            if cell["italics"]:
                styles.append("font-style:italic")
            if cell["underscore"]:
                styles.append("text-decoration:underline")
            current = ";".join(styles)
            if previous is not None and current != previous:
                escaped = html.escape(text)
                runs.append(
                    f'<span style="{previous}">{escaped}</span>'
                    if previous
                    else escaped
                )
                text = ""
            text += cell["data"]
            previous = current
        escaped = html.escape(text)
        runs.append(
            f'<span style="{previous}">{escaped}</span>' if previous else escaped
        )
        rendered = "".join(runs)
        # Preserve terminal width without trailing source-file whitespace.
        stripped = rendered.rstrip(" ")
        rows.append(stripped + "&#32;" * (len(rendered) - len(stripped)))
    return "\n".join(rows)


def write_gallery(output, captures, animation, *, preview=True, caption=None):
    items = []
    for name, capture in captures.items():
        items.append(
            f"<section><h2>{html.escape(name)}</h2><pre>{terminal_html(capture)}</pre></section>"
        )
    frames = [terminal_html(frame) for frame in animation]
    payload = json.dumps(frames).replace("<", "\\u003c")
    page = """<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AIRS onboarding — actual terminal captures</title>
<style>
:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;padding:32px;background:#0c1015;color:#e5e9f0;font:16px system-ui}
h1{font-size:26px;margin:0 0 10px}p{color:#a4afbe;max-width:760px;line-height:1.6}h2{font-size:15px;color:#a4afbe;font-weight:500}
section{margin:30px 0}pre{background:#141920;color:#e5e9f0;padding:18px;width:max-content;max-width:100%;overflow:auto;border:1px solid #2b3340;border-radius:10px;font:14px/1.35 "DejaVu Sans Mono",monospace;letter-spacing:0;white-space:pre}
button{background:#273140;color:#e5e9f0;border:1px solid #485569;border-radius:5px;padding:8px 12px;cursor:pointer}body.light{background:#f4f5f7;color:#171a20}body.light pre{background:#fff;color:#171a20}body.light p,body.light h2{color:#4c566a}
</style><h1>Prisma AIRS welcome</h1><p>These screens were captured from the actual Rust preview through a terminal. All values are synthetic. The animation replays captured terminal frames; it is not a separately designed web mockup.</p>
<button id="motion">Pause animation</button> <button id="theme">Toggle terminal theme</button>
<section><h2>Animated welcome · 80 × 24</h2><pre id="animation"></pre></section>"""
    if not preview:
        page = page.replace(
            "actual Rust preview",
            "built harness with local HTTPS OIDC and native credential storage",
        )
        page = page.replace(
            "The animation replays captured terminal frames; it is not a separately designed web mockup.",
            "These are fixture results, not production SSO or ServiceNow acceptance.",
        )
    if caption is not None:
        start = page.index("<p>")
        end = page.index("</p>", start)
        page = page[:start] + "<p>" + html.escape(caption) + page[end:]
    if len(frames) < 2:
        page = page.replace('<button id="motion">', '<button id="motion" hidden>')
        page = page.replace(
            "<section><h2>Animated welcome", "<section hidden><h2>Animated welcome"
        )
    page += "".join(items)
    page += (
        "<script>const frames="
        + payload
        + """;let index=0,playing=!matchMedia('(prefers-reduced-motion: reduce)').matches;const el=document.querySelector('#animation');el.innerHTML=frames[0];setInterval(()=>{if(playing){index=(index+1)%frames.length;el.innerHTML=frames[index]}},150);const button=document.querySelector('#motion');function label(){button.textContent=playing?'Pause animation':'Play animation'}label();button.onclick=()=>{playing=!playing;label()};document.querySelector('#theme').onclick=()=>document.body.classList.toggle('light');</script></html>"""
    )
    (output / "gallery.html").write_text(page)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    checks, captures, animation = [], {}, []

    for columns, rows in [(80, 24), (120, 40), (48, 18), (30, 10)]:
        preview = Preview(binary, columns=columns, rows=rows, extra=("--static",))
        try:
            preview.expect("Sign in")
            captures[f"Welcome · {columns} × {rows}"] = preview.capture()
            preview.send(b"\r")
            preview.finish("menu-action:0")
            checks.append(f"{columns}x{rows}: primary action and terminal restoration")
        finally:
            preview.close()

    preview = Preview(binary)
    try:
        preview.expect("Sign in to continue")
        first_frame_seconds = time.monotonic() - preview.started
        for _ in range(24):
            animation.append(preview.capture())
            preview.pump(0.15)
        assert len({"\n".join(frame["text"]) for frame in animation}) >= 8
        preview.send(b"\x1b[B\r")
        preview.finish("menu-action:1")
        checks.append("live prism animation; down-arrow selection during animation")
    finally:
        preview.close()

    for name, keys, expected in [
        ("escape", b"\x1b", "Cancelled"),
        ("ctrl-c", b"\x03", "Cancelled"),
        ("ctrl-d", b"\x04", "Cancelled"),
        ("ctrl-z", b"\x1a", "Cancelled"),
        ("numeric shortcut", b"2", "menu-action:1"),
        ("up wraps", b"\x1b[A\r", "menu-action:2"),
        ("tab navigation", b"\t\r", "menu-action:1"),
    ]:
        preview = Preview(binary, extra=("--static", "--no-color"))
        try:
            preview.expect("Sign in to continue")
            preview.send(keys)
            preview.finish(expected)
            checks.append(name + ": expected result and terminal restoration")
        finally:
            preview.close()

    preview = Preview(binary, "input", extra=("--static",))
    try:
        preview.expect("AI Gateway URL")
        preview.send(b"\x1b[200~https://gateway.example.com/v1\x1b[201~")
        preview.expect("gateway.example.com")
        assert preview.process.poll() is None, "Paste submitted the field"
        captures["Public connection field"] = preview.capture()
        preview.send(b"\r")
        preview.finish("public-input:https://gateway.example.com/v1")
        checks.append("public input: bracketed paste does not submit")
    finally:
        preview.close()

    preview = Preview(binary, "input", extra=("--static",))
    try:
        preview.expect("AI Gateway URL")
        preview.send(b"\x1b[200~one\ntwo\x1b[201~")
        preview.expect("Paste one line")
        captures["Inline paste validation"] = preview.capture()
        preview.send(b"\x1b")
        preview.finish("Cancelled")
        checks.append("multiline paste rejected without submission or terminal damage")
    finally:
        preview.close()

    for screen in ["waiting", "recovery"]:
        preview = Preview(binary, screen)
        try:
            preview.expect("Waiting for" if screen == "waiting" else "Try again")
            captures[screen.title()] = preview.capture()
            preview.send(b"\x1b")
            preview.finish("Cancelled")
            checks.append(screen + ": cancellation and terminal restoration")
        finally:
            preview.close()

    preview = Preview(binary, extra=("--static",))
    try:
        preview.expect("Sign in to continue")
        preview.resize(48, 18)
        preview.expect("Sign in with company SSO")
        preview.send(b"\x1b[B")
        preview.resize(120, 40)
        captures["Resized with selection preserved"] = preview.capture()
        preview.send(b"\r")
        preview.finish("menu-action:1")
        checks.append("resize preserves selection and primary action")
    finally:
        preview.close()

    preview = Preview(binary, "progress", extra=("--static",))
    try:
        preview.expect("Checking connection")
        preview.expect("Saving sign-in")
        captures["Async operation progress"] = preview.capture()
        preview.finish("preview-operation-completed")
        checks.append(
            "async operation: live progress updates and completion restore terminal"
        )
    finally:
        preview.close()

    preview = Preview(binary, extra=("--static", "--no-color"))
    try:
        preview.expect("Sign in to continue")
        initial = preview.capture()
        bytes_before = len(preview.transcript)
        preview.pump(0.5)
        assert initial == preview.capture(), "Static preview changed without input"
        assert len(preview.transcript) == bytes_before, (
            "Static preview redrew without input"
        )
        assert all(
            cell["fg"] == "default" and cell["bg"] == "default"
            for row in initial["cells"]
            for cell in row
        ), "Colorless preview forced colors"
        captures["Static colorless welcome"] = initial
        preview.send(b"\x1b")
        preview.finish("Cancelled")
        checks.append("static colorless display: no idle redraw or forced colors")
    finally:
        preview.close()

    for sig in [signal.SIGTERM, signal.SIGHUP]:
        preview = Preview(binary, "waiting")
        try:
            preview.expect("Waiting for")
            os.kill(preview.process.pid, sig)
            preview.finish("Cancelled")
            checks.append(f"{sig.name}: cancellation and terminal restoration")
        finally:
            preview.close()

    result = {
        "passed": True,
        "platform": platform.system(),
        "architecture": platform.machine(),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "first_action_seconds": round(first_frame_seconds, 3),
        "checks": checks,
        "capture_count": len(captures),
        "animation_frames": len(animation),
        "synthetic_preview_only": True,
        "authentication_performed": False,
    }
    with gzip.open(output / "CAPTURES.json.gz", "wt") as archive:
        json.dump(captures, archive, ensure_ascii=False)
    (output / "PREVIEW-ACCEPTANCE.json").write_text(json.dumps(result, indent=2) + "\n")
    write_gallery(output, captures, animation)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
