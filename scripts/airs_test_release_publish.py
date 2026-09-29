"""Publish verified owner-authorized test packages, with immutable resume identity."""

import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile
import time
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import HTTPRedirectHandler, build_opener

from airs_release_receipts import atomic_json, safe_destination
from airs_test_release_spec import (
    PACKAGE_ORDER,
    MAC_TAG,
    package_order,
    STABLE_TAG,
    canonical_digest,
    digest_file,
    load_json,
    regular_file,
    require,
    validate_spec,
)


class _NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        raise ValueError("Registry redirects are prohibited")


class Registry:
    """Anonymous metadata reads and explicitly configured npm publication."""

    def __init__(self, registry, userconfig, output):
        self.registry = registry.rstrip("/")
        self.userconfig = Path(userconfig).absolute()
        self.output = Path(output)
        with regular_file(self.userconfig, 64 * 1024):
            pass
        require(
            os.name == "nt"
            or stat.S_IMODE(self.userconfig.stat().st_mode) & 0o077 == 0,
            "Publication npm userconfig must have private permissions",
        )
        self.opener = build_opener(_NoRedirect())

    def metadata(self, name):
        require(name in PACKAGE_ORDER, "Unexpected package name")
        try:
            with self.opener.open(
                self.registry + "/" + quote(name, safe=""), timeout=30
            ) as response:
                payload = response.read(8 * 1024 * 1024 + 1)
        except HTTPError as error:
            if error.code == 404:
                return {"name": name, "versions": {}, "dist-tags": {}}
            raise ValueError(
                f"Registry metadata request failed (HTTP {error.code})"
            ) from None
        require(len(payload) <= 8 * 1024 * 1024, "Registry metadata exceeds size limit")
        document = json.loads(payload)
        require(
            isinstance(document, dict) and document.get("name") == name,
            "Registry package identity mismatch",
        )
        require(
            isinstance(document.get("versions"), dict), "Registry versions are invalid"
        )
        _tags(document)
        return document

    def publish(self, archive, tag):
        require(
            tag in ("mcp", STABLE_TAG, MAC_TAG), "Only candidate tags can be published"
        )
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.upper().startswith(("NPM", "NODE_AUTH_TOKEN"))
            and key != "NODE_OPTIONS"
        }
        # Neither inherited global/project npm settings nor a retained cache may
        # redirect publication or turn lifecycle scripts back on.
        with tempfile.TemporaryDirectory(
            prefix=".npm-publish-", dir=self.output
        ) as temporary:
            temporary = Path(temporary)
            (temporary / "global.npmrc").write_text("")
            environment.update(
                NPM_CONFIG_USERCONFIG=str(self.userconfig),
                NPM_CONFIG_GLOBALCONFIG=str(temporary / "global.npmrc"),
                NPM_CONFIG_CACHE=str(temporary / "cache"),
                NPM_CONFIG_REGISTRY=self.registry,
                NPM_CONFIG_UPDATE_NOTIFIER="false",
                NPM_CONFIG_LOGLEVEL="error",
                NPM_CONFIG_LOGS_MAX="0",
                NPM_CONFIG_FETCH_RETRIES="0",
                NPM_CONFIG_FETCH_TIMEOUT="180000",
            )
            result = subprocess.run(
                [
                    "npm",
                    "publish",
                    str(Path(archive).absolute()),
                    "--registry",
                    self.registry,
                    "--tag",
                    tag,
                    "--ignore-scripts",
                ],
                cwd=temporary,
                env=environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True,
                timeout=300,
            )
            if result.returncode:
                # npm diagnostics can contain private configuration. Retain only
                # its bounded error code, never raw credential-bearing output.
                code = re.search(
                    r"npm (?:error|ERR!) code ([A-Z0-9_]{1,64})", result.stderr
                )
                suffix = f" ({code[1]})" if code else ""
                raise RuntimeError(
                    f"npm publication failed{suffix}; verify registry credentials and retry"
                )


def _tags(document):
    tags = document.get("dist-tags")
    require(isinstance(tags, dict) and len(tags) <= 128, "Registry tags are invalid")
    require(
        all(
            isinstance(k, str)
            and isinstance(v, str)
            and 0 < len(k) <= 128
            and 0 < len(v) <= 128
            and not any(ord(c) < 32 or ord(c) == 127 for c in k + v)
            for k, v in tags.items()
        ),
        "Registry tags contain invalid values",
    )
    return dict(tags)


def _protected(tags, tag):
    return {key: value for key, value in tags.items() if key != tag}


def _existing(document, record, tag):
    observed = document["versions"].get(record["version"])
    if observed is None:
        return False
    require(isinstance(observed, dict), "Registry version metadata is invalid")
    require(
        observed.get("name") == record["name"]
        and observed.get("version") == record["version"],
        "Registry version identity mismatch",
    )
    require(
        observed.get("dist", {}).get("integrity") == record["integrity"],
        "Existing immutable version has different integrity; publication stopped",
    )
    require(
        _tags(document).get(tag) == record["version"],
        "Existing version is not the candidate tag target; refusing an implicit tag change",
    )
    return True


def _publish(spec, plan, packages, output, registry):
    """Execute a verified plan; injectable transport keeps behavioral tests local."""
    records = plan["publish_order"]
    require(
        [record["name"] for record in records] == package_order(spec),
        "Native packages must publish before the launcher",
    )
    identity = canonical_digest({"spec": spec, "plan": plan})
    output = safe_destination(output)
    output.mkdir(parents=True, exist_ok=True)
    require(
        output.is_dir() and not output.is_symlink(),
        "Publication output must be a real directory",
    )
    receipt_path = output / "PUBLICATION.json"
    receipt = load_json(receipt_path) if receipt_path.exists() else None
    if receipt is not None:
        require(
            receipt.get("schema_version") == 1
            and receipt.get("identity_sha256") == identity,
            "Publication checkpoint identity changed; refusing resume",
        )
        require(
            set(receipt.get("original_tags", {})) == set(PACKAGE_ORDER),
            "Publication checkpoint tags are incomplete",
        )
    current = {name: registry.metadata(name) for name in PACKAGE_ORDER}
    for record in records:
        require(record["version"] == spec["version"], "Publication version mismatch")
        require(
            digest_file(Path(packages) / "tarballs" / record["filename"])
            == record["sha256"],
            "Staged archive changed before publication",
        )
        current[record["name"]] = registry.metadata(record["name"])
        _existing(current[record["name"]], record, spec["tag"])
    if receipt is None:
        receipt = {
            "schema_version": 1,
            "scope": spec["scope"],
            "identity_sha256": identity,
            "spec_sha256": canonical_digest(spec),
            "source_commit": spec["source_commit"],
            "version": spec["version"],
            "registry": spec["registry"],
            "tag": spec["tag"],
            "original_tags": {name: _tags(value) for name, value in current.items()},
            "packages": [],
            "published": False,
            "production_acceptance": False,
        }
        atomic_json(receipt_path, receipt)

    def check_tags(name, document):
        original = _tags({"dist-tags": receipt["original_tags"][name]})
        protected = _protected(_tags(document), spec["tag"])
        # npm creates latest for a package's first stable release, even when
        # publishing with a candidate tag. No established channel is replaced.
        initial_latest = (
            not original
            and spec["registry"] == "https://registry.npmjs.org"
            and spec["scope"] == "owner-authorized-stable"
            and protected == {"latest": spec["version"]}
        )
        if initial_latest:
            created = receipt.setdefault("registry_created_initial_latest", [])
            if name not in created:
                created.append(name)
        require(
            initial_latest or protected == _protected(original, spec["tag"]),
            "Protected registry tags changed; refusing further publication",
        )

    for name, document in current.items():
        check_tags(name, document)
    for record in records:
        name = record["name"]
        # Re-read all tags immediately before each mutation, including packages
        # already verified during a previous interrupted attempt.
        for other in PACKAGE_ORDER:
            check_tags(other, registry.metadata(other))
        document = registry.metadata(name)
        if not _existing(document, record, spec["tag"]):
            archive = Path(packages) / "tarballs" / record["filename"]
            require(
                digest_file(archive) == record["sha256"],
                "Archive changed during publication",
            )
            registry.publish(archive, spec["tag"])
            # A successful upload can precede anonymous metadata visibility.
            # Retry reads only; never repeat the immutable publication blindly.
            for attempt in range(21):
                document = registry.metadata(name)
                if spec["version"] in document["versions"]:
                    break
                if attempt < 20:
                    time.sleep(3)
        require(
            _existing(document, record, spec["tag"]),
            "Published version is absent from the registry",
        )
        check_tags(name, document)
        receipt["packages"] = [
            entry for entry in receipt["packages"] if entry["name"] != name
        ]
        receipt["packages"].append({**record, "registry_integrity_verified": True})
        atomic_json(receipt_path, receipt)
    for record in records:
        document = registry.metadata(record["name"])
        check_tags(record["name"], document)
        require(
            _existing(document, record, spec["tag"]),
            "Registry changed during final verification",
        )
    receipt["published"] = True
    receipt["existing_protected_tags_preserved"] = True
    if spec["tag"] == "mcp":
        receipt["existing_non_mcp_tags_preserved"] = True
    atomic_json(receipt_path, receipt)
    return receipt


def publish_packages(spec, packages, acceptance, output, userconfig):
    from airs_test_release_stage import verify_staged

    output = safe_destination(output)
    spec = validate_spec(spec)
    plan = verify_staged(spec, packages, acceptance)
    for source in (packages, acceptance):
        left, right = Path(output).resolve(), Path(source).resolve()
        require(
            not left.is_relative_to(right) and not right.is_relative_to(left),
            "Publication output must not overlap package or acceptance inputs",
        )
    Path(output).mkdir(parents=True, exist_ok=True)
    return _publish(
        spec, plan, packages, output, Registry(spec["registry"], userconfig, output)
    )
