#!/usr/bin/env python3
"""Exercise the real pinned CLI without credentials or live API mutations."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--launcher",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "npm/airs-harness/bin/airs.js",
    )
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    launcher = args.launcher.resolve(strict=True)
    manifest = json.loads((launcher.parent.parent / "package.json").read_text())
    expected = manifest["dependencies"]["@cdot65/prisma-airs-cli"]
    checks = []
    with tempfile.TemporaryDirectory(prefix="airs-cli-contract-") as directory:
        root = Path(directory)
        config = root / "trusted-config.json"
        config.write_text(
            json.dumps(
                {
                    "mgmtTsgId": "fixture-tsg",
                    "mgmtClientId": "fixture-client",
                    "mgmtClientSecret": "fixture-secret",
                    "mgmtEndpoint": "https://trusted.invalid",
                }
            )
        )
        (root / ".env").write_text(
            "PANW_MGMT_ENDPOINT=https://untrusted.invalid\nPANW_AI_SEC_API_KEY=project-injected-fixture\n"
        )
        env = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("PANW_", "PRISMA_AIRS_", "DOTENV_"))
        }
        env.update(
            HOME=directory,
            USERPROFILE=directory,
            PRISMA_AIRS_TENANTS_PATH=str(root / "tenants.json"),
        )

        def run(*arguments, code=0, overrides=None):
            result = subprocess.run(
                ["node", str(launcher), "cli", *arguments],
                cwd=root,
                env=env | (overrides or {}),
                text=True,
                capture_output=True,
                timeout=60,
            )
            if result.returncode != code:
                raise AssertionError(
                    f"{arguments}: exit {result.returncode}; {result.stderr}"
                )
            return result.stdout

        assert run("--version").strip() == expected
        checks.append("exact managed CLI version")
        doctor = json.loads(run("doctor", "--output", "json", code=1))
        assert (
            next(row for row in doctor if row["name"] == "Tenant")["status"] == "fail"
        )
        assert "airs cli tenant create" in next(
            row["hint"] for row in doctor if row["name"] == "Tenant"
        )
        run("tenant", "create", "fixture", "--config", str(config))
        run("tenant", "switch", "fixture")
        # Simulate credentials removed after registration; doctor must not probe.
        config.write_text(
            json.dumps(
                {"mgmtTsgId": "fixture-tsg", "mgmtEndpoint": "https://trusted.invalid"}
            )
        )
        before = config.read_bytes()
        value = json.loads(
            run(
                "tenant",
                "get",
                "fixture",
                "mgmtEndpoint",
                "--output",
                "json",
                overrides={"PANW_MGMT_ENDPOINT": "https://explicit.invalid"},
            )
        )
        assert value == [{"key": "mgmtEndpoint", "value": "https://trusted.invalid"}]
        assert config.read_bytes() == before
        checks.append(
            "selected tenant config preserved; credential env and project dotenv ignored"
        )
        doctor = json.loads(run("doctor", "--output", "json", code=1))
        statuses = {row["name"]: row["status"] for row in doctor}
        assert statuses["Tenant"] == "pass"
        assert statuses["Scanner credentials"] == "skip"
        assert statuses["Management credentials"] == "fail"
        assert (
            statuses["Scanner API"]
            == statuses["Management OAuth"]
            == statuses["AI Gateway API"]
            == "skip"
        )
        assert "airsApiKey" in next(
            row["hint"] for row in doctor if row["name"] == "Scanner credentials"
        )
        for key in ["mgmtClientId", "mgmtClientSecret"]:
            assert key in next(
                row["hint"] for row in doctor if row["name"] == "Management credentials"
            )
        checks.append(
            "doctor distinguishes missing tenant, credentials and skipped network checks"
        )
        commands = [
            ("runtime", "scan"),
            ("runtime", "bulk-scan"),
            ("runtime", "profiles", "update"),
            ("runtime", "topics", "create"),
            ("runtime", "topics", "apply"),
            ("runtime", "topics", "eval"),
            ("runtime", "topics", "revert"),
            ("redteam", "targets"),
            ("redteam", "scan"),
            ("redteam", "prompt-sets"),
            ("aigateway", "workspaces"),
            ("aigateway", "configs"),
            ("aigateway", "admin-guardrails", "list"),
            ("aigateway", "admin-guardrails", "get"),
            ("aigateway", "admin-guardrails", "create"),
            ("aigateway", "admin-guardrails", "update"),
            ("aigateway", "admin-guardrails", "delete"),
            ("aigateway", "admin-guardrails", "mcp-servers", "list"),
            ("aigateway", "admin-guardrails", "mcp-servers", "sync"),
            ("aigateway", "admin-guardrails", "mcp-servers", "upsert"),
            ("aigateway", "inference"),
            ("model-security", "groups"),
            ("model-security", "rules"),
            ("model-security", "scans"),
            ("runtime", "dlp", "filtering-profiles"),
            ("runtime", "dlp", "patterns"),
            ("runtime", "dlp", "dictionaries"),
            ("runtime", "dlp", "generate"),
        ]
        for command in commands:
            assert "Usage:" in run(*command, "--help")
        checks.append(f"{len(commands)} capability command contracts")
        corpus = root / "corpus"
        summary = json.loads(
            run(
                "runtime",
                "dlp",
                "generate",
                "--types",
                "pdf,png,jpeg,svg,docx",
                "--count",
                "1",
                "--out",
                str(corpus),
                "--seed",
                "42",
                "--output",
                "json",
            )
        )
        assert summary["clean"] == 5 and summary["dirty"] > 0
        assert set(summary["byFormat"]) == {"pdf", "png", "jpeg", "svg", "docx"}
        assert (corpus / "manifest.json").is_file()
        signatures = {
            ".pdf": b"%PDF",
            ".png": b"\x89PNG",
            ".jpg": b"\xff\xd8",
            ".docx": b"PK",
        }
        for extension, signature in signatures.items():
            files = list(corpus.rglob("*" + extension))
            assert len(files) > 1 and all(
                p.read_bytes().startswith(signature) for p in files
            )
        assert all("<svg" in p.read_text() for p in corpus.rglob("*.svg"))
        checks.append(
            "real PDF/PNG/JPEG/SVG/DOCX corpus generation including optional native image dependency"
        )
    receipt = {
        "passed": True,
        "cli_version": expected,
        "checks": checks,
        "live_api_operations": False,
        "document_detection_tested": False,
    }
    if args.receipt:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
