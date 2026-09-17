import argparse, hashlib, json, os, platform, subprocess, sys, threading
from http.server import ThreadingHTTPServer
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("--packages", type=Path, required=True)
p.add_argument("--scripts", type=Path, required=True)
p.add_argument("--output", type=Path, required=True)
a = p.parse_args()
sys.path.insert(0, str(a.scripts.resolve()))
from airs_npm_registry import install_environment, registry_handler

root = a.output.resolve()
root.mkdir(parents=True, exist_ok=False)
prefix = root / "prefix"
prefix.mkdir()
state = root / "state"
tenant_store = root / "tenants.json"
env = install_environment(prefix, "https://npm.cdot.io", False)
env.update(
    AIRS_HARNESS_HOME=str(state),
    PRISMA_AIRS_TENANTS_PATH=str(tenant_store),
    PATH=str(prefix / "bin") + os.pathsep + os.environ["PATH"],
)
checks = []
count = 0


def run(args, environment=None, ok=True):
    global count
    count += 1
    r = subprocess.run(
        [str(x) for x in args],
        env=environment or env,
        text=True,
        capture_output=True,
        timeout=300,
    )
    (root / f"{count:02d}.log").write_text(r.stdout + r.stderr)
    assert (r.returncode == 0) == ok, (args, r.returncode, r.stderr)
    return r.stdout.strip()


def npm(*args, registry="https://npm.cdot.io", environment=None):
    return run(
        [
            "npm",
            *args,
            "--global",
            "--prefix",
            prefix,
            "--registry",
            registry,
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
        ],
        environment,
    )


def version(name, expected, *args):
    assert run([prefix / "bin" / name, *args, "--version"]) == expected


def snapshot():
    return {
        str(f.relative_to(state)): hashlib.sha256(f.read_bytes()).hexdigest()
        for f in state.rglob("*")
        if f.is_file() and f.relative_to(state).parts[:2] != ("tmp", "arg0")
    }


npm("install", "@cdot65/prisma-airs-cli@6.1.1", registry="https://registry.npmjs.org")
npm("install", "airs-harness@0.1.0-alpha.21")
version("airs", "6.1.1")
version("airs-harness", "airs-harness 0.1.0-alpha.21")
run(
    [
        prefix / "bin/airs-harness",
        "env",
        "create",
        "work",
        "--gateway-url",
        "https://gateway.example.com/v1",
    ]
)
reg = json.loads((state / "environments.json").read_text())
original = reg["environments"]["work"]
history = state / "environments" / original["id"] / "history.jsonl"
history.write_text("preserved migration history\n")
before = snapshot()
preflight = json.loads(
    run(["node", a.packages / "airs-harness/bin/airs.js", "--migration-check"])
)
assert (
    preflight["modified"] is False
    and preflight["commands"]["airs"][0]["owner"]["name"] == "@cdot65/prisma-airs-cli"
)
checks.append("old CLI command ownership identified without execution or modification")
npm("install", "@cdot65/prisma-airs-cli@7.0.0", registry="https://registry.npmjs.org")
assert not (prefix / "bin/airs").exists()
version("airs-cli", "7.0.0")
checks.append("CLI 6 to 7 upgrade releases airs and installs airs-cli without force")
records = json.loads((a.packages / "NPM-PACKAGES.json").read_text())["publish_order"]
metadata = {}
archives = {}
requests = []
unexpected = []
redirects = []
server = ThreadingHTTPServer(
    ("127.0.0.1", 0),
    registry_handler(metadata, archives, requests, unexpected, redirects, True),
)
registry = f"http://127.0.0.1:{server.server_port}"
for r in records:
    manifest = json.loads((a.packages / r["name"] / "package.json").read_text())
    path = f"/{r['name']}/-/{r['filename']}"
    archive = a.packages / "tarballs" / r["filename"]
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == r["sha256"]
    manifest["dist"] = {"tarball": registry + path, "integrity": r["integrity"]}
    metadata["/" + r["name"]] = {
        "name": r["name"],
        "dist-tags": {"latest": r["version"]},
        "versions": {r["version"]: manifest},
    }
    archives[path] = archive
threading.Thread(target=server.serve_forever, daemon=True).start()
candidate_env = install_environment(prefix, registry, True)
candidate_env.update(
    {k: env[k] for k in ["AIRS_HARNESS_HOME", "PRISMA_AIRS_TENANTS_PATH", "PATH"]}
)
try:
    npm(
        "install",
        "airs-harness@0.1.0-alpha.22",
        registry=registry,
        environment=candidate_env,
    )
    version("airs", "airs 0.1.0-alpha.22")
    version("airs", "7.0.0", "cli")
    version("airs-cli", "7.0.0")
    version("airs-harness", "airs 0.1.0-alpha.22")
    assert snapshot() == before
    checks.append("paired upgrade preserves named environment ID, settings and history")
    for command in [
        "runtime",
        "redteam",
        "aigateway",
        "model-security",
        "agentguard",
        "tenant",
    ]:
        out = run([prefix / "bin/airs", command], ok=False)
    run([prefix / "bin/airs", "--environment", "missing", "cli", "--version"], ok=False)
    checks.append(
        "legacy product roots and misplaced environment selector fail before starting a session"
    )
    config = root / "development.json"
    config.write_text(
        json.dumps(
            {
                "mgmtTsgId": "migration-fixture",
                "mgmtClientId": "fixture-client",
                "mgmtClientSecret": "not-a-real-secret",
            }
        )
    )
    config.chmod(0o600)
    run(
        [
            prefix / "bin/airs",
            "cli",
            "tenant",
            "create",
            "development",
            "--config",
            config,
        ]
    )
    run([prefix / "bin/airs", "cli", "tenant", "switch", "development"])
    selected = tenant_store.read_bytes()
    run(
        [
            prefix / "bin/airs",
            "env",
            "create",
            "other",
            "--gateway-url",
            "https://other.example.com/v1",
        ]
    )
    run([prefix / "bin/airs", "env", "use", "work"])
    assert (
        tenant_store.read_bytes() == selected
        and history.read_text() == "preserved migration history\n"
    )
    assert "development" in run([prefix / "bin/airs-cli", "tenant", "list"])
    checks.append(
        "environment switching leaves shared standalone/managed CLI tenant selection unchanged"
    )
    shim = root / "shadow"
    shim.mkdir()
    fake = shim / "airs-cli"
    fake.write_text("#!/bin/sh\necho WRONG_GLOBAL_CLI\nexit 90\n")
    fake.chmod(0o755)
    assert (
        run(
            [prefix / "bin/airs", "cli", "--version"],
            dict(env, PATH=str(shim) + os.pathsep + env["PATH"]),
        )
        == "7.0.0"
    )
    checks.append("managed CLI ignores a conflicting global executable earlier on PATH")
    npm("uninstall", "@cdot65/prisma-airs-cli")
    version("airs", "airs 0.1.0-alpha.22")
    version("airs", "7.0.0", "cli")
    npm(
        "install",
        "@cdot65/prisma-airs-cli@7.0.0",
        registry="https://registry.npmjs.org",
    )
    version("airs", "airs 0.1.0-alpha.22")
    npm("uninstall", "airs-harness")
    assert not (prefix / "bin/airs").exists()
    version("airs-cli", "7.0.0")
    checks.append(
        "either package can be uninstalled independently without deleting the other command"
    )
    npm(
        "install",
        "airs-harness@0.1.0-alpha.22",
        registry=registry,
        environment=candidate_env,
    )
    version("airs", "7.0.0", "cli")
    rollback = snapshot()
    npm("uninstall", "airs-harness")
    npm("install", "airs-harness@0.1.0-alpha.21")
    npm(
        "install",
        "@cdot65/prisma-airs-cli@6.1.1",
        registry="https://registry.npmjs.org",
    )
    version("airs-harness", "airs-harness 0.1.0-alpha.21")
    version("airs", "6.1.1")
    assert snapshot() == rollback and tenant_store.read_bytes() == selected
    checks.append(
        "ordered rollback restores alpha.21 and CLI 6 command ownership while preserving state"
    )
    assert not unexpected and not redirects
    receipt = {
        "passed": True,
        "platform": platform.system(),
        "architecture": platform.machine(),
        "harness_version": "0.1.0-alpha.22",
        "cli_version": "7.0.0",
        "checks": checks,
        "external_api_operations": False,
        "force_used": False,
        "isolated_prefix": True,
    }
    (root / "COMMAND-MIGRATION.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))
finally:
    server.shutdown()
