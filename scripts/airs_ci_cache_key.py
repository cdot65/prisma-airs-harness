"""Describe compiler/profile inputs without treating a cache hit as build proof."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

VARIABLES = (
    "RUNNER_OS",
    "RUNNER_ARCH",
    "ImageOS",
    "ImageVersion",
    "AIRS_RELEASE_TARGET",
    "AIRS_CLI_OPT_LEVEL",
    "CARGO_INCREMENTAL",
    "CARGO_BUILD_TARGET",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_PROFILE_RELEASE_DEBUG",
    "CARGO_PROFILE_RELEASE_LTO",
    "CARGO_PROFILE_RELEASE_CODEGEN_UNITS",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_TEST_DEBUG",
    "MACOSX_DEPLOYMENT_TARGET",
    "SDKROOT",
    "DEVELOPER_DIR",
)


def identity(root, environment, output):
    inputs = {
        "schema_version": 1,
        "compiler": output(["rustc", "-vV"]),
        "sccache": output(["sccache", "--version"]),
        "environment": {key: environment.get(key, "") for key in VARIABLES},
        "files": {
            name: hashlib.sha256((root / name).read_bytes()).hexdigest()
            for name in (
                "Cargo.lock",
                "Cargo.toml",
                "rust-toolchain.toml",
                ".cargo/config.toml",
            )
        },
    }
    if environment.get("RUNNER_OS") == "macOS":
        inputs["sdk_version"] = output(
            ["xcrun", "--sdk", "macosx", "--show-sdk-version"]
        )
        inputs["xcode"] = output(["xcodebuild", "-version"])
    encoded = json.dumps(inputs, sort_keys=True, separators=(",", ":")).encode()
    return {"cache_key": hashlib.sha256(encoded).hexdigest(), "inputs": inputs}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    result = identity(
        args.root,
        os.environ,
        lambda cmd: subprocess.check_output(
            cmd, cwd=args.root, text=True, timeout=15
        ).strip(),
    )
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    args.receipt.write_text(json.dumps(result, indent=2) + "\n")
    with Path(os.environ["GITHUB_OUTPUT"]).open("a") as output:
        output.write("cache_key=" + result["cache_key"] + "\n")


if __name__ == "__main__":
    main()
