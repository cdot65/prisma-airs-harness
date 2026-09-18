#!/usr/bin/env python3
"""Check installed AIRS onboarding through real shells without authenticating.

Run with uv run --with pyte==0.8.2. Uses disposable environment configuration;
does not contact an identity service or read/write native credentials.
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

from validate_airs_onboarding_preview import Preview, write_gallery


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--native-binary", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--shells", nargs="+", default=["bash", "zsh", "fish"])
    args = parser.parse_args()
    binary = args.binary.absolute()
    native = (args.native_binary or binary).resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=True)
    checks, captures, durations = [], {}, {}
    with tempfile.TemporaryDirectory(prefix="airs-terminal-acceptance-") as directory:
        root = Path(directory)
        env = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("AIRS_", "OPENAI_"))
            and key not in ("CODEX_HOME", "CODEX_SQLITE_HOME", "NO_COLOR")
        }
        env["AIRS_HARNESS_HOME"] = str(root / "state")
        subprocess.run(
            [
                str(binary),
                "env",
                "create",
                "work",
                "--gateway-url",
                "https://gateway.invalid.example/v1",
            ],
            env=env,
            cwd=root,
            capture_output=True,
            check=True,
            timeout=30,
        )
        registry_path = root / "state/environments.json"
        registry = registry_path.read_bytes()
        home = (
            root
            / "state/environments"
            / json.loads(registry)["environments"]["work"]["id"]
        )
        config_path = home / "config.toml"
        config = config_path.read_text()

        for name in args.shells:
            shell = shutil.which(name)
            assert shell, f"Required shell is unavailable: {name}"
            if name == "fish":
                arguments = [
                    "--no-config",
                    "-c",
                    '"$argv[1]" login; printf "airs-exit=%s\\n" $status',
                    str(binary),
                ]
            else:
                arguments = (
                    ["--noprofile", "--norc"] if name == "bash" else ["-f"]
                ) + [
                    "-c",
                    '"$1" login; result=$?; printf "airs-exit=%s\\n" "$result"',
                    "airs-fixture",
                    str(binary),
                ]
            terminal = Preview(
                Path(shell), arguments=arguments, environment=env, directory=root
            )
            try:
                terminal.expect("Sign in to continue")
                durations[name] = round(time.monotonic() - terminal.started, 3)
                captures[f"{name} signed-out startup"] = terminal.capture()
                terminal.send(b"\x1b")
                terminal.finish("airs-exit=1")
                checks.append(
                    f"{name}: interactive sign-in cancellation returns to the shell with restored terminal modes"
                )
            finally:
                terminal.close()

        for name, colorless in [("static", False), ("static colorless", True)]:
            config_path.write_text(config + "\n[tui]\nanimations = false\n")
            display_env = dict(env)
            if colorless:
                display_env["NO_COLOR"] = "1"
            terminal = Preview(
                binary, arguments=["login"], environment=display_env, directory=root
            )
            try:
                terminal.expect("Sign in to continue")
                terminal.pump(0.2)
                before = bytes(terminal.transcript)
                terminal.pump(0.5)
                assert bytes(terminal.transcript) == before, (
                    "Static onboarding redrew while idle"
                )
                captures[name] = terminal.capture()
                if colorless:
                    assert all(
                        cell["fg"] == "default" and cell["bg"] == "default"
                        for row in captures[name]["cells"]
                        for cell in row
                    ), "NO_COLOR forced a foreground or background color"
                terminal.send(b"\x1b")
                terminal.finish("", status=1)
                checks.append(
                    f"{name}: selected environment preference applies and the idle screen does not redraw"
                )
            finally:
                terminal.close()

        config_path.write_text(config)
        terminal = Preview(binary, arguments=["login"], environment=env, directory=root)
        try:
            terminal.expect("Sign in to continue")
            for columns, rows in [(120, 40), (48, 18), (30, 10), (80, 24)]:
                terminal.resize(columns, rows)
                captures[f"Installed startup {columns}x{rows}"] = terminal.capture()
            terminal.send(b"\t")
            terminal.send(b"\r")
            terminal.expect("Workspace API key (input hidden")
            terminal.send(b"\x1b")
            terminal.expect("Sign-in needs your attention")
            terminal.send(b"\x1b")
            terminal.finish("", status=1)
            checks.append(
                "resize through large, compact and tiny terminals preserves keyboard selection and cancellation"
            )
        finally:
            terminal.close()

        plain = subprocess.run(
            [str(binary), "env", "list"],
            env=dict(env, TERM="dumb", NO_COLOR="1"),
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            check=True,
            timeout=30,
        )
        assert "work" in plain.stdout and "\x1b" not in plain.stdout + plain.stderr
        checks.append(
            "redirected plain-terminal environment management emits no interactive escapes"
        )
        assert registry_path.read_bytes() == registry
        assert not (home / "credential-binding.json").exists()
        checks.append(
            "terminal acceptance preserves the environment registry and performs no authentication"
        )
    receipt = {
        "passed": True,
        "platform": os.uname().sysname,
        "architecture": os.uname().machine,
        "native_binary_sha256": hashlib.sha256(native.read_bytes()).hexdigest(),
        "launcher": str(binary),
        "first_action_seconds": durations,
        "checks": checks,
        "authentication_performed": False,
    }
    (args.output / "TERMINAL-ACCEPTANCE.json").write_text(
        json.dumps(receipt, indent=2) + "\n"
    )
    write_gallery(
        args.output,
        captures,
        [next(iter(captures.values()))],
        caption="Actual AIRS startup captured through shells and a terminal with disposable synthetic configuration. These checks perform no authentication and do not establish production service access.",
    )
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
