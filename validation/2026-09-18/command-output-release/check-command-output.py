"""Verify public command guidance and normal TUI exit on exact installed bytes."""

import argparse, hashlib, json, os, re, subprocess, sys, tempfile
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("--binary", type=Path, required=True)
p.add_argument("--native", type=Path, required=True)
p.add_argument("--scripts", type=Path, required=True)
p.add_argument("--receipt", type=Path, required=True)
a = p.parse_args()
a.binary = a.binary.absolute()
checks = []
env = {
    k: v
    for k, v in os.environ.items()
    if not k.startswith(("AIRS_", "OPENAI_", "CODEX_"))
}


def clean(text):
    return re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)


def public(text):
    assert not re.search(r"(?<![\w./-])(?:airs-harness|codex)(?:\s|`)", text), text


with tempfile.TemporaryDirectory(prefix="airs-command-guidance-") as temp:
    env["AIRS_HARNESS_HOME"] = str(Path(temp) / "state")
    for args in [
        [],
        ["env"],
        ["env", "create"],
        ["login"],
        ["mcp"],
        ["mcp", "add"],
        ["mcp", "login"],
        ["doctor"],
        ["exec"],
        ["resume"],
        ["fork"],
        ["queue"],
    ]:
        result = subprocess.run(
            [str(a.binary), *args, "--help"],
            env=env,
            cwd=temp,
            text=True,
            capture_output=True,
            check=True,
            timeout=30,
        )
        assert "Usage: airs" in result.stdout
        public(result.stdout)
        checks.append({"command": ["airs", *args, "--help"], "passed": True})
    subprocess.run(
        [
            str(a.binary),
            "env",
            "create",
            "work",
            "--gateway-url",
            "http://127.0.0.1:9/v1",
            "--allow-http-loopback",
        ],
        env=env,
        cwd=temp,
        capture_output=True,
        check=True,
        timeout=30,
    )
    env["AIRS_API_KEY"] = "synthetic-command-output-test"
    for args, hint in [
        (["fork", "--worktree", "--last"], "`airs fork --worktree`"),
        (["exec", "resume", "--worktree", "session"], "`airs exec fork --worktree`"),
    ]:
        args = ["--environment", "work", *args]
        result = subprocess.run(
            [str(a.binary), *args],
            env=env,
            cwd=temp,
            text=True,
            capture_output=True,
            timeout=30,
        )
        output = result.stdout + result.stderr
        assert result.returncode and hint in output, output
        public(output)
        checks.append({"command": ["airs", *args], "passed": True, "output": output})
os.environ["AIRS_HARNESS_BIN"] = str(a.binary)
sys.path.insert(0, str(a.scripts.resolve()))
from test_airs_harness import TerminalIntegration
from airs_harness_pty import TerminalSession

TerminalIntegration.setUpClass()
case = TerminalIntegration("runTest")
try:
    case.setUp()
    case.configure()
    with TerminalSession(
        a.binary,
        case.env,
        case.work,
        arguments=["--environment", "work", "--no-alt-screen"],
    ) as terminal:
        terminal.start()
        terminal.send_line(
            "Create result.txt using a local shell tool and verify its contents."
        )
        terminal.wait_for(b"Local tool complete.")
        terminal.send_line("/quit")
        terminal.wait_until(lambda: terminal.process.poll() is not None)
        output = clean(terminal.transcript.decode(errors="replace"))
        summary = output[output.rfind("To continue this session, run:") :]
        assert re.search(r"airs --environment work resume [0-9a-f-]{36}", summary), (
            summary
        )
        public(summary)
        checks.append(
            {
                "command": ["airs", "--environment", "work"],
                "passed": True,
                "normal_exit_resume_guidance": summary.strip(),
            }
        )
finally:
    case.tearDown()
    case.doCleanups()
receipt = {
    "passed": True,
    "native_sha256": hashlib.sha256(a.native.read_bytes()).hexdigest(),
    "checks": checks,
}
a.receipt.write_text(json.dumps(receipt, indent=2) + "\n")
print(json.dumps(receipt))
