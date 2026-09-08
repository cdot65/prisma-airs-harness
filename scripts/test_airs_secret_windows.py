#!/usr/bin/env python3
"""Exercise the candidate's hidden prompt through real Windows ConPTY input.

CI only: pywinpty==3.0.5, https://github.com/andfoy/pywinpty/tree/v3.0.5.
Ctrl+Enter uses the terminal-to-ConPTY wire protocol, not injected Rust events:
https://github.com/microsoft/terminal/blob/main/doc/specs/%234999%20-%20Improved%20keyboard%20handling%20in%20Conpty.md
This is native console acceptance, not a claim about physical desktop keybindings.
"""

import argparse
import ctypes
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time

PYWINPTY_VERSION = "3.0.5"
PROMPT = "Workspace API key (input hidden; Ctrl+Enter to submit; Esc to cancel):"
RESTORED = "[fixture-terminal-restored]"
PROBE = "terminal-restoration-verified"
# Microsoft's Win32 input mode: CSI Vk;Sc;Uc;Kd;Cs;Rc_. Left Ctrl state is 8.
# These encode the four physical key transitions, including Ctrl+Enter's LF.
CONTROL_ENTER = (
    "\x1b[17;29;0;1;8;1_\x1b[13;28;10;1;8;1_\x1b[13;28;10;0;8;1_\x1b[17;29;0;0;0;1_"
)


def console_api():
    from ctypes import wintypes

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetStdHandle.argtypes = [wintypes.DWORD]
    kernel.GetStdHandle.restype = wintypes.HANDLE
    kernel.GetConsoleMode.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetConsoleMode.restype = wintypes.BOOL
    kernel.SetConsoleMode.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel.SetConsoleMode.restype = wintypes.BOOL
    handle = kernel.GetStdHandle(-10 & 0xFFFFFFFF)

    def get_mode():
        mode = wintypes.DWORD()
        if not kernel.GetConsoleMode(handle, ctypes.byref(mode)):
            raise ctypes.WinError(ctypes.get_last_error())
        return mode.value

    def set_mode(mode):
        if not kernel.SetConsoleMode(handle, mode):
            raise ctypes.WinError(ctypes.get_last_error())

    return get_mode, set_mode


def observe(binary, result_path):
    """Run inside ConPTY, sharing this native console with the real candidate."""
    get_mode, set_mode = console_api()
    original = get_mode()
    # A non-default baseline catches implementations that blindly re-enable
    # PROCESSED_INPUT instead of restoring the original bitmask.
    expected = (original | 0x0002 | 0x0004) & ~0x0001
    result = {"original_mode": original, "expected_restored_mode": expected}
    child = None
    try:
        set_mode(expected)
        child = subprocess.Popen([str(binary), "login", "--with-api-key"])
        observed = set()
        deadline = time.monotonic() + 35
        while child.poll() is None and time.monotonic() < deadline:
            observed.add(get_mode())
            time.sleep(0.01)
        if child.poll() is None:
            child.kill()
            child.wait(timeout=5)
            raise AssertionError("Candidate prompt timed out")
        result.update(
            candidate_exit=child.returncode,
            observed_raw_mode=any(mode & 0x0007 == 0 for mode in observed),
            restored_mode=get_mode(),
        )
        # Leave the restored mode in place for a real line-input/echo check.
        print(RESTORED, flush=True)
        result["restored_line_input"] = sys.stdin.readline().rstrip("\r\n") == PROBE
        result["passed"] = (
            result["observed_raw_mode"]
            and result["restored_mode"] == expected
            and result["restored_line_input"]
        )
    finally:
        set_mode(original)
        result_path.write_text(json.dumps(result, indent=2) + "\n")


class ConsoleSession:
    def __init__(self, arguments, cwd, env):
        from winpty import PtyProcess
        from winpty.enums import Backend

        # 3.0.5 uses `backend or os.environ[...]`: integer zero would allow an
        # ambient backend override. Its coercion also accepts string "0".
        self.process = PtyProcess.spawn(
            arguments,
            cwd=str(cwd),
            env=env,
            dimensions=(32, 140),
            backend=str(Backend.ConPTY),
        )
        self.events = queue.Queue(maxsize=256)
        self.transcript = ""

        def read():
            try:
                while True:
                    self.events.put(self.process.read(4096), timeout=1)
            except (EOFError, OSError, queue.Full):
                pass

        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def wait_for(self, marker, timeout=20):
        deadline = time.monotonic() + timeout
        while marker not in self.transcript:
            if time.monotonic() >= deadline:
                raise AssertionError("ConPTY output marker missing or timed out")
            try:
                self.transcript += self.events.get(timeout=0.05)
            except queue.Empty:
                continue
            if len(self.transcript) > 1_048_576:
                raise AssertionError("ConPTY output exceeded fixture limit")

    def write(self, value):
        self.process.write(value)

    def finish(self):
        deadline = time.monotonic() + 10
        while self.process.isalive() and time.monotonic() < deadline:
            try:
                self.transcript += self.events.get(timeout=0.05)
            except queue.Empty:
                pass
        if self.process.isalive():
            raise AssertionError("ConPTY observer did not exit")
        while not self.events.empty():
            self.transcript += self.events.get_nowait()

    def close(self):
        if self.process.isalive():
            # Bound cleanup to this fixture's process tree, including candidate.
            subprocess.run(
                ["taskkill", "/PID", str(self.process.pid), "/T", "/F"],
                capture_output=True,
                timeout=10,
                check=False,
            )
        self.process.close(force=True)
        self.reader.join(timeout=1)


def acceptance(binary, output):
    if sys.platform != "win32":
        raise SystemExit("This acceptance gate requires native Windows ConPTY")
    if importlib.metadata.version("pywinpty") != PYWINPTY_VERSION:
        raise AssertionError("Install the exact fixture pywinpty version")
    output.mkdir(parents=True, exist_ok=False)
    receipt = {
        "passed": False,
        "backend": "ConPTY",
        "pywinpty": PYWINPTY_VERSION,
        "physical_desktop_keys_tested": False,
        "cases": [],
    }
    cases = [
        ("escape", "\x1b", "Sign-in cancelled", False),
        ("control-c", "\x03", "Sign-in cancelled", False),
        ("multiline-paste", "\r\nfixture-paste-tail", "Use Ctrl+Enter", False),
        ("lf-only-paste", "\nfixture-paste-tail", None, False),
        ("typed-unicode", "é", "printable ASCII", False),
        ("control-enter", CONTROL_ENTER, "Configured workspace credential", True),
    ]
    try:
        with tempfile.TemporaryDirectory(prefix="airs-conpty-") as temporary:
            root = Path(temporary)
            for name, suffix, marker, success in cases:
                receipt["active_case"] = name
                work = root / name
                work.mkdir()
                home = work / "state"
                env = {
                    k: v
                    for k, v in os.environ.items()
                    if not k.upper().startswith(("AIRS_", "PANW_", "PRISMA_AIRS_"))
                }
                env.update(AIRS_HARNESS_HOME=str(home))
                setup = subprocess.run(
                    [
                        str(binary),
                        "setup",
                        "--gateway-url",
                        "https://gateway.invalid/v1",
                    ],
                    env=env,
                    cwd=work,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                if setup.returncode:
                    raise AssertionError("Candidate environment setup failed")
                result_path = work / "console-result.json"
                terminal = ConsoleSession(
                    [
                        sys.executable,
                        str(Path(__file__).resolve()),
                        "--observe",
                        "--binary",
                        str(binary),
                        "--receipt",
                        str(result_path),
                    ],
                    work,
                    env,
                )
                secret = "fixture-hidden-key-" + name
                helper_args = None
                try:
                    terminal.wait_for(PROMPT)
                    if success:
                        # Fail closed if this console did not negotiate the
                        # real Win32 input transport needed for Ctrl+Enter.
                        terminal.wait_for("\x1b[?9001h", timeout=2)
                    terminal.write(secret)
                    time.sleep(0.1)
                    terminal.write(suffix)
                    if marker is not None:
                        terminal.wait_for(marker)
                    terminal.wait_for(RESTORED)
                    terminal.write(PROBE + "\r")
                    terminal.finish()
                    proof = json.loads(result_path.read_text())
                    if not proof.get("passed"):
                        raise AssertionError(
                            "Native console mode/input restoration failed"
                        )
                    if (proof["candidate_exit"] == 0) != success:
                        raise AssertionError("Unexpected candidate exit status")
                    if any(
                        piece in terminal.transcript
                        for piece in [secret, "fixture-hidden", "fixture-paste-tail"]
                    ):
                        raise AssertionError("Secret input was echoed")
                    bindings = list(home.rglob("credential-binding.json"))
                    if success:
                        if len(bindings) != 1:
                            raise AssertionError(
                                "Ctrl+Enter did not create one binding"
                            )
                        binding = json.loads(bindings[0].read_text())
                        if (
                            binding["credential_fingerprint"]
                            != hashlib.sha256(secret.encode()).hexdigest()
                        ):
                            raise AssertionError(
                                "ConPTY submission changed the credential"
                            )
                        helper_args = [
                            str(binary),
                            "credential",
                            "--home",
                            str(bindings[0].parent),
                            "--binding",
                            binding["id"],
                        ]
                        credential = subprocess.run(
                            helper_args,
                            env=env,
                            cwd=work,
                            capture_output=True,
                            text=True,
                            timeout=15,
                        )
                        if credential.returncode or credential.stdout != secret + "\n":
                            raise AssertionError(
                                "Native Credential Manager readback failed"
                            )
                        receipt["native_credential_readback"] = True
                    elif bindings:
                        raise AssertionError(
                            "Rejected input created a credential binding"
                        )
                    for path in home.rglob("*"):
                        if path.is_file() and secret.encode() in path.read_bytes():
                            raise AssertionError(
                                "Fixture key persisted in plaintext state"
                            )
                    receipt["cases"].append({"name": name, "passed": True, **proof})
                finally:
                    transcript = terminal.transcript
                    for private in [secret, "fixture-hidden", "fixture-paste-tail"]:
                        transcript = transcript.replace(
                            private, "[redacted fixture input]"
                        )
                    (output / f"{name}-transcript.txt").write_text(transcript)
                    if result_path.exists():
                        (output / f"{name}-console.json").write_bytes(
                            result_path.read_bytes()
                        )
                    try:
                        terminal.close()
                    finally:
                        # Remove this fixture's native-store entry even after a
                        # failed success assertion; never leave test credentials.
                        if list(home.rglob("credential-binding.json")):
                            logout = subprocess.run(
                                [str(binary), "logout"],
                                env=env,
                                cwd=work,
                                capture_output=True,
                                timeout=15,
                            )
                            if logout.returncode:
                                raise AssertionError(
                                    "Fixture credential cleanup failed"
                                )
                            if helper_args is not None:
                                removed = subprocess.run(
                                    helper_args,
                                    env=env,
                                    cwd=work,
                                    capture_output=True,
                                    text=True,
                                    timeout=15,
                                )
                                if removed.returncode == 0 or removed.stdout:
                                    raise AssertionError(
                                        "Logout did not disable credential helper output"
                                    )
                                receipt["logout_rejects_credential_helper"] = True
        receipt["passed"] = True
        receipt.pop("active_case", None)
    except Exception as error:
        receipt["failure"] = {"type": type(error).__name__, "message": str(error)}
        raise
    finally:
        (output / "conpty-secret-prompt.json").write_text(
            json.dumps(receipt, indent=2) + "\n"
        )
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output-directory", type=Path)
    parser.add_argument("--observe", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--receipt", type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    candidate = args.binary.resolve(strict=True)
    if args.observe:
        observe(candidate, args.receipt)
    else:
        if args.output_directory is None:
            parser.error("--output-directory is required")
        acceptance(candidate, args.output_directory)
