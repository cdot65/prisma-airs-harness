import hashlib, json, subprocess, sys, tempfile
from pathlib import Path

sys.path.insert(0, "/home/cdot/development/cdot65/prisma-airs-harness/scripts")
from airs_npm_registry import install_environment

root = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix="airs-onboarding-default-") as temp:
    prefix = Path(temp)
    env = install_environment(prefix, "https://npm.cdot.io", False)
    with (root / "default-install.log").open("w") as log:
        subprocess.run(
            [
                "npm",
                "install",
                "-g",
                "--prefix",
                str(prefix),
                "airs-harness",
                "--registry=https://npm.cdot.io",
                "--no-audit",
                "--no-fund",
            ],
            env=env,
            stdout=log,
            stderr=subprocess.STDOUT,
            check=True,
            timeout=300,
        )
    command = prefix / "bin/airs"
    version = subprocess.check_output(
        [str(command), "--version"], env=env, text=True
    ).strip()
    assert version == "airs 0.1.0-alpha.22.onboarding.2"
    cli = subprocess.check_output(
        [str(command), "cli", "--version"], env=env, text=True
    ).strip()
    assert cli == "7.0.0"
    native = (
        prefix
        / "lib/node_modules/airs-harness/node_modules/airs-harness-linux-x64/bin/airs-harness"
    )
    digest = hashlib.sha256(native.read_bytes()).hexdigest()
    assert (
        digest
        == json.loads((root / "linux-REGISTRY-INSTALL.json").read_text())[
            "binary_sha256"
        ]
    )
    receipt = {
        "passed": True,
        "anonymous_fresh_install": True,
        "unpinned_install": True,
        "include_optional_flag_used": False,
        "version_output": version,
        "cli_version": cli,
        "native_sha256": digest,
    }
    (root / "DEFAULT-INSTALL.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))
