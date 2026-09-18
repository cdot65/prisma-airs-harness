#!/usr/bin/env python3
"""Create a portable, self-contained review bundle from staged AIRS npm packages."""

import argparse
import hashlib
import json
import re
import shutil
import tarfile
from pathlib import Path


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    packages = args.packages.resolve(strict=True)
    receipt = json.loads((packages / "NPM-PACKAGES.json").read_text())
    assert receipt.get("cli_bundle"), "The review bundle must contain the managed CLI"
    records = receipt["publish_order"]
    expected = {
        "airs-harness",
        "airs-harness-linux-x64",
        "airs-harness-linux-arm64",
        "airs-harness-darwin-arm64",
    }
    assert {record["name"] for record in records} == expected
    version = records[0]["version"]
    assert re.fullmatch(r"[0-9A-Za-z.-]+", version)
    assert all(record["version"] == version for record in records)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    bundle = output / f"airs-onboarding-review-{version}"
    bundle.mkdir()
    destination = bundle / "packages"
    (destination / "tarballs").mkdir(parents=True)
    shutil.copy2(packages / "NPM-PACKAGES.json", destination)
    for record in records:
        filename = record["filename"]
        assert Path(filename).name == filename
        archive = packages / "tarballs" / filename
        assert digest(archive) == record["sha256"], "Staged package integrity changed"
        shutil.copy2(archive, destination / "tarballs" / filename)
        target = destination / record["name"]
        target.mkdir()
        shutil.copy2(packages / record["name"] / "package.json", target)
    shutil.copy2(
        packages / "airs-harness/BUNDLE-INVENTORY.json",
        destination / "airs-harness",
    )
    scripts = Path(__file__).resolve().parent
    shutil.copy2(scripts / "install_airs_review.py", bundle / "install.py")
    tools = bundle / "tools"
    tools.mkdir()
    for filename in [
        "validate_airs_npm.py",
        "airs_npm_registry.py",
        "airs_bundle.py",
        "airs_bundle_archive.py",
        "airs_bundle_tree.py",
        "airs_bundle_shims.py",
    ]:
        shutil.copy2(scripts / filename, tools)
    (tools / "fixtures/npm-command-shims").mkdir(parents=True)
    shutil.copy2(
        scripts / "fixtures/npm-command-shims/node.json",
        tools / "fixtures/npm-command-shims/node.json",
    )
    (bundle / "README.md").write_text(
        f"""# AIRS onboarding review {version}

This is an isolated review candidate for Linux x64, Linux ARM64 and Apple Silicon.
It includes the harness, Prisma AIRS CLI and their packaged dependencies.
Apple Silicon executable signing and provenance are inside its native package.
This bundle is not a production SSO or ServiceNow acceptance receipt.

Use Python 3.11+ and Node.js 22.14+ in the 22.x line, or Node.js 24+ with npm.
From this extracted directory, choose a new installation directory:

```sh
python3 install.py --prefix "$HOME/airs-onboarding-review"
"$HOME/airs-onboarding-review/bin/airs" --version
"$HOME/airs-onboarding-review/bin/airs" cli --version
AIRS_HARNESS_HOME="$HOME/airs-onboarding-review/review-home" "$HOME/airs-onboarding-review/bin/airs"
```

Keep using that `AIRS_HARNESS_HOME` when returning to the review environment.
Follow the vault's SSO-to-ServiceNow walkthrough for your company connection values.
The installer verifies the shipped files and exact installed executable and CLI
inventory. It serves the bundled packages on loopback only while npm installs.
No package dependencies are fetched from an external registry, and no PATH or
shell configuration is changed. `INSTALL-VERIFICATION.json` records the result.

To remove the review environment, first use the candidate's `logout` command in
each review environment to clear its OS credentials, then remove the directory
you chose. Your regular `airs` installation continues to use its existing home.

Runtime source: `{receipt["source_commit"]}`.
Packaging source: `{receipt["packaging_commit"]}`.
Review: https://git.cdot.io/cdot/prisma-airs-harness/pulls/47
"""
    )
    contents = {
        "version": version,
        "source_commit": receipt["source_commit"],
        "published": False,
        "files": {
            path.relative_to(bundle).as_posix(): digest(path)
            for path in sorted(bundle.rglob("*"))
            if path.is_file()
        },
    }
    (bundle / "REVIEW-CONTENTS.json").write_text(json.dumps(contents, indent=2) + "\n")
    archive = output / f"{bundle.name}.tar.gz"
    with tarfile.open(archive, "w:gz", compresslevel=1) as tar:
        tar.add(bundle, arcname=bundle.name)
    (output / "SHA256SUMS").write_text(f"{digest(archive)}  {archive.name}\n")
    print(json.dumps({"archive": str(archive), "sha256": digest(archive)}))


if __name__ == "__main__":
    main()
