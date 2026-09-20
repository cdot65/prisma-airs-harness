"""Synthetic installed /typesafe UI acceptance; never use owner credentials."""

import os
import time

from airs_harness_pty import TerminalSession


def exercise_typesafe_ui(binary, env, root, command):
    with TerminalSession(
        binary,
        env,
        root,
        arguments=["--environment", "judge", "--no-alt-screen"],
    ) as terminal:
        terminal.start()
        terminal.send_line("/typesafe")
        terminal.wait_for(b"Save or replace API key")
        offset = len(terminal.transcript)
        os.write(terminal.master, b"\r")
        terminal.wait_for(b"TypeSafe Jev API key", offset)
        offset = len(terminal.transcript)
        terminal.send_line("synthetic-private-ui-key")
        terminal.wait_for(b"API key saved", offset)
        prefix = [binary, "--environment", "judge", "env", "typesafe"]
        command(
            [
                *prefix,
                "exec",
                "--",
                "node",
                "-e",
                "if(process.env.TYPESAFE_API_KEY!=='synthetic-private-ui-key')process.exit(1)",
            ]
        )
        # Cancel replacement without modifying the existing native-store entry.
        offset = len(terminal.transcript)
        os.write(terminal.master, b"\r")
        terminal.wait_for(b"TypeSafe Jev API key", offset)
        os.write(terminal.master, b"\x1b[200~synthetic-cancelled-key\x1b[201~")
        time.sleep(0.15)
        offset = len(terminal.transcript)
        os.write(terminal.master, b"\x1b")
        terminal.wait_for(b"Cancelled", offset)
        # Removing a key requires the separate confirmation view.
        offset = len(terminal.transcript)
        os.write(terminal.master, b"\x1b[B\x1b[B\r")
        terminal.wait_for(b"Remove this environment's TypeSafe key?", offset)
        offset = len(terminal.transcript)
        os.write(terminal.master, b"\x1b[B\r")
        terminal.wait_for(b"Saved API key removed", offset)
        command(
            [
                *prefix,
                "exec",
                "--",
                "node",
                "-e",
                "if('TYPESAFE_API_KEY' in process.env)process.exit(1)",
            ]
        )
        assert b"synthetic-private-ui-key" not in terminal.transcript
        assert b"synthetic-cancelled-key" not in terminal.transcript
