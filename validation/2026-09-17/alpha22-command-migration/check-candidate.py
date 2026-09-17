import argparse, json, os, platform, subprocess, sys
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("--packages", type=Path, required=True)
p.add_argument("--output", type=Path, required=True)
p.add_argument("--scripts", type=Path, required=True)
a = p.parse_args()
root = a.output.resolve()
root.mkdir(parents=True, exist_ok=False)
scripts = a.scripts.resolve()
prefix = root / "prefix"


def run(cmd, log, env=None):
    with (root / log).open("w") as f:
        subprocess.run(
            cmd, env=env, stdout=f, stderr=subprocess.STDOUT, check=True, timeout=600
        )


run(
    [
        sys.executable,
        str(scripts / "validate_airs_npm.py"),
        "--packages",
        str(a.packages.resolve()),
        "--prefix",
        str(prefix),
    ],
    "install.log",
)
command = prefix / "bin/airs"
env = dict(os.environ, AIRS_MANAGED_CLI_ACCEPTANCE="1", AIRS_HARNESS_BIN=str(command))
run(
    [
        sys.executable,
        "-m",
        "unittest",
        "discover",
        "-s",
        str(scripts),
        "-p",
        "test_airs_harness*.py",
        "-v",
    ],
    "installed-tests.log",
    env,
)
arch = {"x86_64": "x64", "arm64": "arm64", "aarch64": "arm64"}[platform.machine()]
osname = {"Linux": "linux", "Darwin": "darwin"}[platform.system()]
native = (
    prefix
    / "lib/node_modules/airs-harness/node_modules"
    / f"airs-harness-{osname}-{arch}"
    / "bin/airs-harness"
)
run(
    [
        sys.executable,
        str(Path(__file__).with_name("check-environments.py")),
        "--binary",
        str(native),
        "--receipt",
        str(root / "ENVIRONMENTS.json"),
    ],
    "environments.log",
)
if platform.system() == "Darwin":
    run(
        [
            sys.executable,
            str(scripts / "validate_airs_macos_keychain.py"),
            "--binary",
            str(command),
            "--receipt",
            str(root / "KEYCHAIN.json"),
        ],
        "keychain.log",
    )
for previous in (
    ["0.1.0-alpha.21"]
    if osname == "linux" and arch == "arm64"
    else ["0.1.0-alpha.21", "0.1.0-alpha.20"]
):
    run(
        [
            sys.executable,
            str(scripts / "validate_airs_npm_upgrade.py"),
            "--packages",
            str(a.packages.resolve()),
            "--previous",
            previous,
            "--output",
            str(root / ("upgrade-" + previous)),
        ],
        "upgrade-" + previous + ".log",
    )
print(
    json.dumps(
        {
            "passed": True,
            "platform": platform.system(),
            "architecture": platform.machine(),
            "scope": "exact candidate npm installation, executable fixtures, lifecycle and upgrades from alpha.21 and alpha.20",
        }
    )
)
