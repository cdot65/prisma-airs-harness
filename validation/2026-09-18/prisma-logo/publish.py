import argparse, hashlib, json, os, subprocess, time, urllib.request
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("phase", choices=["publish", "tags"])
a = p.parse_args()
root = Path(__file__).resolve().parent
stage = root / "npm-release"
version = "0.1.0-alpha.22.onboarding.2"
registry = "https://npm.cdot.io"
d = json.loads((stage / "NPM-PACKAGES.json").read_text())
records = d["publish_order"]
assert d.get("publishable") is not False
assert [r["name"] for r in records] == [
    "airs-harness-linux-x64",
    "airs-harness-linux-arm64",
    "airs-harness-darwin-arm64",
    "airs-harness",
]


def metadata(name):
    with urllib.request.urlopen(registry + "/" + name, timeout=30) as response:
        return json.load(response)


def run(label, command):
    env = {
        k: v
        for k, v in os.environ.items()
        if not k.upper().startswith(("NPM_", "NODE_AUTH_TOKEN"))
    }
    env.update(
        NPM_CONFIG_USERCONFIG=str(root / "publication.npmrc"),
        NPM_CONFIG_REGISTRY=registry,
        NPM_CONFIG_UPDATE_NOTIFIER="false",
    )
    with (root / (label + ".log")).open("w") as log:
        subprocess.run(
            command,
            cwd=stage,
            env=env,
            stdout=log,
            stderr=subprocess.STDOUT,
            check=True,
            timeout=300,
        )


for r in records:
    assert r["version"] == version and r["runtime_payload_unchanged"]
    assert (
        hashlib.sha256((stage / "tarballs" / r["filename"]).read_bytes()).hexdigest()
        == r["sha256"]
    )
    assert (
        json.loads((stage / r["name"] / "package.json").read_text()).get("private")
        is not True
    )
path = root / "PUBLICATION.json"
receipt = (
    json.loads(path.read_text())
    if path.exists()
    else {
        "version": version,
        "registry": registry,
        "source_commit": d["source_commit"],
        "publication_authorized": True,
        "scope": "owner-requested-onboarding-npm-publication",
        "published": False,
        "default_tags_changed": False,
        "original_tags": {r["name"]: metadata(r["name"])["dist-tags"] for r in records},
        "production_sso_servicenow_acceptance": False,
        "packages": [],
    }
)


def save():
    path.write_text(json.dumps(receipt, indent=2) + "\n")


save()
if a.phase == "publish":
    for record in records:
        name = record["name"]
        current = metadata(name)
        if version not in current["versions"]:
            run(
                "publish-" + name,
                [
                    "npm",
                    "publish",
                    str(stage / "tarballs" / record["filename"]),
                    "--registry",
                    registry,
                    "--tag",
                    "onboarding",
                    "--ignore-scripts",
                ],
            )
        current = metadata(name)
        assert current["versions"][version]["dist"]["integrity"] == record["integrity"]
        assert current["dist-tags"]["onboarding"] == version
        receipt["packages"] = [r for r in receipt["packages"] if r["name"] != name] + [
            {**record, "registry_integrity_verified": True}
        ]
        save()
        print(name + " published", flush=True)
    receipt["published"] = True
    save()
else:
    assert receipt["published"]
    for name in ["linux", "arm64", "mac"]:
        check = json.loads((root / (name + "-REGISTRY-INSTALL.json")).read_text())
        assert (
            check["passed"]
            and check["version"] == version
            and check["anonymous_fresh_install"]
        )
    for r in records:
        name = r["name"]
        assert (
            metadata(name)["versions"][version]["dist"]["integrity"] == r["integrity"]
        )
        for tag in ["alpha", "latest"]:
            run(
                "tag-" + name + "-" + tag,
                [
                    "npm",
                    "dist-tag",
                    "add",
                    name + "@" + version,
                    tag,
                    "--registry",
                    registry,
                ],
            )
        assert all(
            metadata(name)["dist-tags"][tag] == version for tag in ["alpha", "latest"]
        )
    receipt.update(default_tags_changed=True, completed_at=time.time())
    save()
    print("All latest and alpha tags now point to " + version)
