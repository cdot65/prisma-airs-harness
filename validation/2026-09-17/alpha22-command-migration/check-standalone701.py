import argparse, json, os, platform, subprocess, sys
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("--prefix", type=Path, required=True)
p.add_argument("--scripts", type=Path, required=True)
p.add_argument("--receipt", type=Path, required=True)
a = p.parse_args()
sys.path.insert(0, str(a.scripts.resolve()))
from airs_npm_registry import install_environment

prefix = a.prefix.resolve()
env = install_environment(prefix, "https://registry.npmjs.org", False)
env["AIRS_HARNESS_HOME"] = str(prefix / "coexistence-probe-state")
with a.receipt.with_suffix(".log").open("w") as log:
    subprocess.run(
        [
            "npm",
            "install",
            "-g",
            "--prefix",
            str(prefix),
            "--registry=https://registry.npmjs.org",
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            "@cdot65/prisma-airs-cli",
        ],
        env=env,
        stdout=log,
        stderr=subprocess.STDOUT,
        check=True,
        timeout=300,
    )

    def check(name, args, expected):
        value = subprocess.check_output(
            [str(prefix / "bin" / name), *args], env=env, text=True
        ).strip()
        assert value == expected, (value, expected)
        return value

    result = {
        "passed": True,
        "platform": platform.system(),
        "architecture": platform.machine(),
        "unversioned_standalone_install": True,
        "harness": check("airs", ["--version"], "airs 0.1.0-alpha.22"),
        "managed_cli": check("airs", ["cli", "--version"], "7.0.0"),
        "standalone_cli": check("airs-cli", ["--version"], "7.0.1"),
    }
    subprocess.run(
        [
            "npm",
            "uninstall",
            "-g",
            "--prefix",
            str(prefix),
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            "@cdot65/prisma-airs-cli",
        ],
        env=env,
        stdout=log,
        stderr=subprocess.STDOUT,
        check=True,
        timeout=180,
    )
    check("airs", ["--version"], "airs 0.1.0-alpha.22")
    check("airs", ["cli", "--version"], "7.0.0")
    assert not (prefix / "bin/airs-cli").exists()
    result["standalone_uninstall_preserves_harness_and_bundle"] = True
    a.receipt.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))
