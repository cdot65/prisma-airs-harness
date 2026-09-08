"""Legacy publisher behavior against an isolated, mocked registry."""

import contextlib
import hashlib
import io
import json
from pathlib import Path
import subprocess
import shutil
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import publish_airs_github_packages as publisher


def tar_bytes(manifest, provenance=None):
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w") as archive:
        files = {"package/package.json": manifest}
        if provenance is not None:
            files["package/BUILD-INFO.json"] = provenance
        for name, value in files.items():
            data = json.dumps(value, sort_keys=True).encode()
            member = tarfile.TarInfo(name)
            member.size = len(data)
            archive.addfile(member, io.BytesIO(data))
    return buffer.getvalue()


class Registry:
    def __init__(self):
        self.calls, self.versions, self.tags = [], {}, {}
        self.corrupt = False

    def lookup(self, command, **kwargs):
        self.calls.append(command)
        return subprocess.CompletedProcess(
            command, 0 if command[2] in self.versions else 1, "", "E404"
        )

    def npm(self, *args, cwd=None):
        self.calls.append(list(args))
        if args[0] == "pack":
            directory = Path(cwd)
            data = (
                self.versions[args[1]]
                if args[1].startswith("@")
                else tar_bytes(json.loads((directory / "package.json").read_text()))
            )
            (directory / "packed.tgz").write_bytes(data)
            return json.dumps(
                [
                    {
                        "filename": "packed.tgz",
                        "size": len(data),
                        "integrity": hashlib.sha256(data).hexdigest(),
                    }
                ]
            )
        if args[0] == "publish":
            directory = Path(cwd)
            manifest = json.loads((directory / "package.json").read_text())
            self.versions[manifest["name"] + "@" + manifest["version"]] = (
                directory / args[1]
            ).read_bytes()
            self.tags.setdefault(manifest["name"], {"latest": "old-version"})[
                args[args.index("--tag") + 1]
            ] = manifest["version"]
            return ""
        if args[0] == "dist-tag":
            name, version = args[2].rsplit("@", 1)
            self.tags[name][args[3]] = version
            return ""
        if args[0] == "view" and args[2] == "dist-tags":
            return json.dumps(self.tags[args[1]])
        if args[0] == "view":
            return json.dumps(
                {
                    "integrity": "tampered"
                    if self.corrupt
                    else hashlib.sha256(self.versions[args[1]]).hexdigest(),
                    "tarball": "https://npm.pkg.github.com/download/" + args[1],
                }
            )
        raise AssertionError(args)

    def association(self, command, **kwargs):
        assert command[:2] == ["gh", "api"]
        return json.dumps(
            {
                "repository": {"full_name": "cdot65/airs-harness"},
                "html_url": "https://github.com/package",
                "visibility": "private",
            }
        )


class PublisherTests(unittest.TestCase):
    def fixture(
        self, root, *, receipt_marker=None, manifest_marker=None, provenance=None
    ):
        records = []
        for name in [
            "airs-harness-linux-x64",
            "airs-harness-darwin-arm64",
            "airs-harness",
        ]:
            manifest = {
                "name": name,
                "version": "0.1.0-alpha.10",
                "repository": {"url": "git+https://github.com/cdot65/airs-harness.git"},
            }
            if name == "airs-harness":
                manifest["optionalDependencies"] = {
                    native: manifest["version"] for native in publisher.NAMES - {name}
                }
                manifest["dependencies"] = {"@cdot65/prisma-airs-cli": "5.2.0"}
                manifest.update(manifest_marker or {})
            data = tar_bytes(manifest, provenance if name == "airs-harness" else None)
            filename = name + ".tgz"
            (root / filename).write_bytes(data)
            records.append(
                {
                    "name": name,
                    "filename": filename,
                    "sha256": hashlib.sha256(data).hexdigest(),
                }
            )
        receipt = {
            "complete": True,
            "published": True,
            "version": "0.1.0-alpha.10",
            "packages": records,
        }
        receipt.update(receipt_marker or {})
        (root / "receipt.json").write_text(json.dumps(receipt))

    def invoke(self, root, registry, *, tag="auth-review", output="output"):
        args = [
            "publish",
            "--receipt",
            str(root / "receipt.json"),
            "--archives",
            str(root),
            "--output",
            str(root / output),
        ]
        if tag is not None:
            args += ["--dist-tag", tag]
        with (
            patch.object(sys, "argv", args),
            patch.object(publisher, "npm", registry.npm),
            patch.object(publisher.subprocess, "run", registry.lookup),
            patch.object(publisher.subprocess, "check_output", registry.association),
            contextlib.redirect_stdout(io.StringIO()),
            contextlib.redirect_stderr(io.StringIO()),
        ):
            publisher.main()

    @unittest.skipUnless(shutil.which("bash"), "Requires the workflow Bash shell")
    def test_workflow_checks_guards_before_publish_and_pipeline_fails_closed(self):
        workflow = (
            Path(__file__).resolve().parents[1]
            / ".github/workflows/airs-harness-github-packages.yml"
        ).read_text()
        self.assertIn("defaults:\n  run:\n    shell: bash\n", workflow)
        self.assertLess(
            workflow.index("-p test_publish_airs_github_packages.py"),
            workflow.index("gh release download"),
        )
        result = subprocess.run(
            [
                shutil.which("bash"),
                "--noprofile",
                "--norc",
                "-e",
                "-o",
                "pipefail",
                "-c",
                "(exit 7) | cat",
            ],
            capture_output=True,
        )
        self.assertEqual(result.returncode, 7)

    def test_candidate_rejection_including_last_source_has_zero_registry_calls(self):
        markers = [
            {"private_candidate": True},
            {"unsigned_candidate": True},
            {"private": True},
            {"private": "false"},
            {"publishable": False},
            {"release_status": "unsigned-unvalidated-candidate"},
            {"release_ready": False},
            {"passed": False},
            {"published": False},
            {"status": "unvalidated"},
        ]
        for location in ("receipt_marker", "manifest_marker", "provenance"):
            for marker in markers:
                with (
                    self.subTest(location=location, marker=marker),
                    tempfile.TemporaryDirectory() as temporary,
                ):
                    root, registry = Path(temporary), Registry()
                    self.fixture(root, **{location: marker})
                    with self.assertRaises(ValueError):
                        self.invoke(root, registry)
                    self.assertEqual(registry.calls, [])

    def test_invalid_or_missing_tag_has_zero_registry_calls(self):
        for tag in (None, "latest", "stable", "production", "--latest", "review;bad"):
            with self.subTest(tag=tag), tempfile.TemporaryDirectory() as temporary:
                root, registry = Path(temporary), Registry()
                self.fixture(root)
                with self.assertRaises(SystemExit):
                    self.invoke(root, registry, tag=tag)
                self.assertEqual(registry.calls, [])

    def test_dependency_and_checksum_checks_precede_mutation(self):
        for broken in ("dependencies", "checksum"):
            with (
                self.subTest(broken=broken),
                tempfile.TemporaryDirectory() as temporary,
            ):
                root, registry = Path(temporary), Registry()
                self.fixture(
                    root,
                    manifest_marker={"optionalDependencies": {}}
                    if broken == "dependencies"
                    else None,
                )
                if broken == "checksum":
                    (root / "airs-harness.tgz").write_bytes(b"changed")
                with self.assertRaises(ValueError):
                    self.invoke(root, registry)
                self.assertEqual(registry.calls, [])

    def test_explicit_publish_tag_and_immutable_retry_preserve_pin_and_latest(self):
        with tempfile.TemporaryDirectory() as temporary:
            root, registry = Path(temporary), Registry()
            self.fixture(root)
            self.invoke(root, registry)
            publications = [call for call in registry.calls if call[0] == "publish"]
            self.assertEqual(len(publications), 3)
            self.assertTrue(
                all(
                    call[call.index("--tag") + 1] == "auth-review"
                    for call in publications
                )
            )
            self.assertEqual(
                json.loads((root / "output/PUBLICATION.json").read_text())["dist_tag"],
                "auth-review",
            )
            manifest = json.loads(
                (root / "output/airs-harness/package/package.json").read_text()
            )
            self.assertEqual(
                manifest["dependencies"], {"@cdot65/prisma-airs-cli": "5.2.0"}
            )
            originals = dict(registry.versions)
            registry.calls.clear()
            self.invoke(root, registry, tag="preview", output="retry")
            self.assertEqual(registry.versions, originals)
            self.assertFalse(any(call[0] == "publish" for call in registry.calls))
            self.assertEqual(sum(call[0] == "dist-tag" for call in registry.calls), 3)
            self.assertTrue(
                all(
                    tags["latest"] == "old-version"
                    and tags["preview"] == "0.1.0-alpha.10"
                    for tags in registry.tags.values()
                )
            )
            registry.calls.clear()
            registry.corrupt = True
            with self.assertRaises(ValueError):
                self.invoke(root, registry, output="tampered")
            self.assertFalse(
                any(call[0] in ("publish", "dist-tag") for call in registry.calls)
            )


if __name__ == "__main__":
    unittest.main()
