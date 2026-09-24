#!/usr/bin/env python3
"""Inspect, accept, stage, publish and verify owner-authorized test channels."""

import argparse
import json
from pathlib import Path
import sys

from airs_release_acceptance import (
    observed_target,
    run_acceptance,
    verify_acceptance_set,
)
from airs_release_receipts import atomic_json, snapshot_tooling
from airs_test_release_publish import publish_packages
from airs_test_release_spec import canonical_digest, load_spec
from airs_test_release_stage import inspect_packages, stage_packages, verify_staged


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    commands = result.add_subparsers(dest="command", required=True)
    snapshot = commands.add_parser(
        "snapshot-tooling",
        help="Bind copied validator files to their exact committed source",
    )
    snapshot.add_argument("--scripts", type=Path, required=True)
    snapshot.add_argument("--commit", required=True)
    snapshot.add_argument("--output", type=Path, required=True)
    for name in ("inspect", "accept", "stage", "publish", "verify", "verify-all"):
        command = commands.add_parser(name)
        command.add_argument("--spec", type=Path, required=True)
        if name != "verify-all":
            command.add_argument("--packages", type=Path, required=True)
        if name in ("accept", "verify"):
            command.add_argument("--scripts", type=Path, required=True)
            command.add_argument("--resume", action="store_true")
        if name in ("stage", "publish", "verify", "verify-all"):
            command.add_argument("--acceptance", type=Path, required=True)
        if name in ("accept", "stage", "publish", "verify"):
            command.add_argument("--output", type=Path, required=True)
        if name in ("verify", "verify-all"):
            command.add_argument(
                "--verification-tooling-commit",
                help="Explicit committed registry validator revision; original candidate provenance is preserved",
            )
        if name == "publish":
            command.add_argument(
                "--userconfig",
                type=Path,
                required=True,
                help="Existing private npm credentials file; token values never belong in arguments",
            )
        if name == "verify-all":
            command.add_argument(
                "--installation", choices=("candidate", "registry"), default="candidate"
            )
    return result


def execute(args):
    if args.command == "snapshot-tooling":
        value = snapshot_tooling(args.scripts, args.commit)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        atomic_json(args.output, value)
        return {
            "tooling_commit": args.commit,
            "files": len(value["files"]),
            "sha256": canonical_digest(value),
        }
    spec = load_spec(args.spec)
    if args.command == "inspect":
        value = inspect_packages(spec, args.packages)
        return {
            "spec_sha256": canonical_digest(spec),
            "scope": spec["scope"],
            "source_commit": spec["source_commit"],
            "version": spec["version"],
            "publish_order": value["publish_order"],
            "production_acceptance": False,
        }
    if args.command == "accept":
        inspect_packages(spec, args.packages)
        path = run_acceptance(
            spec,
            args.packages,
            args.scripts,
            args.output / observed_target(),
            resume=args.resume,
        )
        return {"acceptance": str(path), "installation": "candidate"}
    if args.command == "stage":
        return stage_packages(spec, args.packages, args.acceptance, args.output)
    if args.command == "publish":
        return publish_packages(
            spec, args.packages, args.acceptance, args.output, args.userconfig
        )
    if args.command == "verify":
        verify_staged(spec, args.packages, args.acceptance)
        path = run_acceptance(
            spec,
            args.packages,
            args.scripts,
            args.output / observed_target(),
            resume=args.resume,
            installation="registry",
            verification_tooling_commit=args.verification_tooling_commit,
        )
        return {"acceptance": str(path), "installation": "registry"}
    return verify_acceptance_set(
        spec,
        args.acceptance,
        installation=args.installation,
        verification_tooling_commit=args.verification_tooling_commit,
    )


def main():
    try:
        print(json.dumps(execute(parser().parse_args()), indent=2))
    except (ValueError, RuntimeError, OSError) as error:
        print(f"Release check failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
