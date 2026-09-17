#!/usr/bin/env python3
"""Exercise opt-in inline questions through AIRS using disposable local state."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tomllib

from airs_harness_pty import TerminalSession


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--gateway-url", required=True)
    parser.add_argument("--credential-file", type=Path, required=True)
    parser.add_argument("--output-directory", type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    root = args.output_directory.resolve()
    root.mkdir(parents=True, exist_ok=False)
    work = root / "work"
    work.mkdir()
    binary = args.binary.resolve(strict=True)
    env = dict(os.environ, AIRS_HARNESS_HOME=str(root / "state"))
    env.pop("AIRS_API_KEY", None)

    def cli(*arguments):
        result = subprocess.run(
            [str(binary), *arguments],
            env=env,
            cwd=work,
            text=True,
            capture_output=True,
            timeout=60,
        )
        if result.returncode:
            raise RuntimeError(f"Fixture setup failed: {result.stderr}")

    cli("env", "create", "inline", "--gateway-url", args.gateway_url)
    registry = json.loads((root / "state/environments.json").read_text())
    identifier = registry["environments"][registry["active"]]["id"]
    home = root / "state/environments" / identifier
    config = tomllib.loads((home / "config.toml").read_text())
    catalog_path = Path(config["model_catalog_json"])
    catalog = json.loads(catalog_path.read_text())
    for model in catalog["models"]:
        model["experimental_supported_tools"] = ["request_user_input_async"]
    catalog_path.write_text(json.dumps(catalog))
    # Bind credentials only after establishing this fixture's opt-in catalog.
    cli("login", "--credential-file", str(args.credential_file.resolve(strict=True)))
    receipt = {
        "passed": False,
        "gateway": args.gateway_url,
        "scope": "Opt-in default-route inline question and preserved draft",
        "default_enablement": False,
        "model_field_wire_inspected": False,
    }
    with binary.open("rb") as stream:
        receipt["binary_sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
    draft = "Write exactly DRAFT_RETAINED into draft-retained.txt using a local tool."
    with TerminalSession(binary, env, work) as terminal:
        try:
            terminal.start()
            terminal.send_line(
                "Use request_user_input_async to ask one question titled 'Choose the fixture color' "
                "with options Blue and Green. While waiting, use a local tool to write WAITING "
                "into question-waiting.txt. After I answer, write the chosen color into "
                "chosen-color.txt. Do not guess an answer, ask another question or change configuration."
            )
            terminal.wait_until(
                lambda: (work / "question-waiting.txt").exists(), timeout=180
            )
            terminal.wait_for(b"to answer", timeout=180)
            os.write(terminal.master, b"\x1b[200~" + draft.encode() + b"\x1b[201~")
            terminal.wait_for(draft.encode())
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\x1b[1;3A")
            terminal.wait_for(b"enter submit", offset)
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\r")
            terminal.wait_for(draft.encode(), offset)
            terminal.wait_until(
                lambda: (work / "chosen-color.txt").exists(), timeout=180
            )
            assert (work / "chosen-color.txt").read_text().strip() == "Blue"
            os.write(terminal.master, b"\r")
            terminal.wait_until(
                lambda: (work / "draft-retained.txt").exists(), timeout=180
            )
            assert (work / "draft-retained.txt").read_text().strip() == "DRAFT_RETAINED"
            receipt.update(
                passed=True,
                async_local_work=True,
                selected_answer="Blue",
                draft_preserved=True,
                local_effects_verified=True,
            )
        finally:
            (root / "transcript.log").write_bytes(terminal.transcript)
            (root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
