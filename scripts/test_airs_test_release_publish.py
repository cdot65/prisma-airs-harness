"""Publication ordering, interruption and conflict checks use a local registry."""

import base64
import copy
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import threading
import unittest
from unittest.mock import patch

from airs_npm_registry import install_environment
from airs_test_release_publish import Registry, _publish, publish_packages
from urllib.parse import unquote

from airs_test_release_spec import (
    LAUNCHER,
    PACKAGE_ORDER,
    archive_filename,
    canonical_digest,
    validate_spec,
)

VERSION = "0.1.0-alpha.22.mcp.3"


def fixture(root):
    packages = root / "packages"
    (packages / "tarballs").mkdir(parents=True)
    records = []
    for name in PACKAGE_ORDER:
        manifest = {"name": name, "version": VERSION}
        if name == LAUNCHER:
            manifest.update(
                bin={"airs": "bin/airs.js"},
                optionalDependencies={n: VERSION for n in PACKAGE_ORDER[:-1]},
            )
        else:
            platform, architecture = name.rsplit("-", 2)[-2:]
            manifest.update(os=[platform], cpu=[architecture])
        path = packages / "tarballs" / archive_filename(name, VERSION)
        with tarfile.open(path, "w:gz", format=tarfile.USTAR_FORMAT) as archive:
            contents = {"package/package.json": json.dumps(manifest).encode()}
            if name == LAUNCHER:
                contents["package/bin/airs.js"] = (
                    f'#!/usr/bin/env node\nconsole.log("airs {VERSION}");\n'.encode()
                )
            for relative, payload in contents.items():
                member = tarfile.TarInfo(relative)
                member.size = len(payload)
                member.mode = 0o755 if relative.endswith(".js") else 0o644
                archive.addfile(member, io.BytesIO(payload))
        payload = path.read_bytes()
        records.append(
            {
                "name": name,
                "version": VERSION,
                "filename": path.name,
                "sha256": hashlib.sha256(payload).hexdigest(),
                "integrity": "sha512-"
                + base64.b64encode(hashlib.sha512(payload).digest()).decode(),
            }
        )
    spec = {
        "scope": "owner-authorized-test",
        "source_commit": "a" * 40,
        "version": VERSION,
        "registry": "https://registry.example",
        "tag": "mcp",
    }
    return (
        spec,
        {"publish_order": records, "spec_sha256": canonical_digest(spec)},
        packages,
    )


class MemoryRegistry:
    def __init__(self, plan):
        self.plan = plan
        self.documents = {
            name: {
                "name": name,
                "versions": {},
                "dist-tags": {"latest": "stable", "alpha": "stable", "mcp": "previous"},
            }
            for name in PACKAGE_ORDER
        }
        self.published = []
        self.fail_at = None

    def metadata(self, name):
        return copy.deepcopy(self.documents[name])

    def publish(self, archive, tag):
        record = next(
            record
            for record in self.plan["publish_order"]
            if record["filename"] == archive.name
        )
        if record["name"] == self.fail_at:
            raise RuntimeError("Synthetic interruption")
        self.published.append(record["name"])
        document = self.documents[record["name"]]
        document["versions"][VERSION] = {
            "name": record["name"],
            "version": VERSION,
            "dist": {"integrity": record["integrity"]},
        }
        document["dist-tags"][tag] = VERSION


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.spec, self.plan, self.packages = fixture(self.root)
        self.output = self.root / "publication"
        self.registry = MemoryRegistry(self.plan)

    def run_publish(self):
        return _publish(self.spec, self.plan, self.packages, self.output, self.registry)

    def test_stable_candidate_publication_preserves_latest_and_mcp(self):
        with patch(__name__ + ".VERSION", "0.1.1"):
            spec, plan, packages = fixture(self.root / "stable")
            spec.update(scope="owner-authorized-stable", tag="stable-candidate")
            registry = MemoryRegistry(plan)
            before = copy.deepcopy(registry.documents)
            receipt = _publish(spec, plan, packages, self.output, registry)
            self.assertTrue(receipt["published"])
            for name in PACKAGE_ORDER:
                self.assertEqual(
                    registry.documents[name]["dist-tags"],
                    {**before[name]["dist-tags"], "stable-candidate": "0.1.1"},
                )
            self.assertEqual(registry.published, PACKAGE_ORDER)
            # Resume cannot silently repair a conflicting tag or changed bytes.
            registry.documents[PACKAGE_ORDER[0]]["dist-tags"]["latest"] = "unexpected"
            with self.assertRaisesRegex(ValueError, "Protected"):
                _publish(spec, plan, packages, self.output, registry)

    def test_first_public_stable_publish_records_initial_latest_and_visibility_delay(
        self,
    ):
        with patch(__name__ + ".VERSION", "0.1.3"):
            spec, plan, packages = fixture(self.root / "public")
            spec.update(
                scope="owner-authorized-stable",
                tag="stable-candidate",
                registry="https://registry.npmjs.org",
            )
            registry = MemoryRegistry(plan)
            for document in registry.documents.values():
                document["dist-tags"] = {}
            original_publish = registry.publish
            original_metadata = registry.metadata
            delayed = set()

            def publish(archive, tag):
                original_publish(archive, tag)
                name = registry.published[-1]
                registry.documents[name]["dist-tags"]["latest"] = "0.1.3"
                delayed.add(name)

            def metadata(name):
                if name in delayed:
                    delayed.remove(name)
                    return {"name": name, "versions": {}, "dist-tags": {}}
                return original_metadata(name)

            registry.publish = publish
            registry.metadata = metadata
            with patch("airs_test_release_publish.time.sleep") as sleep:
                receipt = _publish(spec, plan, packages, self.output, registry)
            self.assertTrue(receipt["published"])
            self.assertEqual(receipt["registry_created_initial_latest"], PACKAGE_ORDER)
            self.assertEqual(registry.published, PACKAGE_ORDER)
            self.assertEqual(sleep.call_count, len(PACKAGE_ORDER))
            self.assertEqual(
                _publish(spec, plan, packages, self.output, registry), receipt
            )
            registry.documents[PACKAGE_ORDER[0]]["dist-tags"]["latest"] = "0.1.4"
            with self.assertRaisesRegex(ValueError, "Protected"):
                _publish(spec, plan, packages, self.output, registry)

    def test_interrupted_native_publish_resumes_before_launcher_and_preserves_tags(
        self,
    ):
        self.registry.fail_at = PACKAGE_ORDER[1]
        with self.assertRaisesRegex(RuntimeError, "interruption"):
            self.run_publish()
        self.assertEqual(self.registry.published, PACKAGE_ORDER[:1])
        partial = json.loads((self.output / "PUBLICATION.json").read_text())
        self.assertFalse(partial["published"])
        self.registry.fail_at = None
        receipt = self.run_publish()
        self.assertEqual(self.registry.published, PACKAGE_ORDER)
        self.assertTrue(receipt["published"])
        self.assertTrue(receipt["existing_non_mcp_tags_preserved"])
        self.run_publish()
        self.assertEqual(self.registry.published, PACKAGE_ORDER)

    def test_conflicting_immutable_version_aborts_before_any_mutation(self):
        record = self.plan["publish_order"][-1]
        self.registry.documents[record["name"]]["versions"][VERSION] = {
            "name": record["name"],
            "version": VERSION,
            "dist": {"integrity": "wrong"},
        }
        with self.assertRaisesRegex(ValueError, "different integrity"):
            self.run_publish()
        self.assertEqual(self.registry.published, [])

    def test_linked_parent_blocks_resumed_publication_before_registry_access(self):
        self.registry.fail_at = PACKAGE_ORDER[0]
        with self.assertRaisesRegex(RuntimeError, "interruption"):
            self.run_publish()
        self.registry.fail_at = None
        linked = self.root / "linked"
        linked.symlink_to(self.root, target_is_directory=True)
        self.output = linked / "publication"
        with patch.object(self.registry, "metadata") as metadata:
            with self.assertRaisesRegex(ValueError, "Linked"):
                self.run_publish()
            metadata.assert_not_called()
        self.assertEqual(self.registry.published, [])
        with patch("airs_test_release_publish.Registry") as registry:
            with self.assertRaisesRegex(ValueError, "Linked"):
                publish_packages(
                    self.spec,
                    self.packages,
                    self.root / "acceptance",
                    self.output,
                    self.root / "private.npmrc",
                )
            registry.assert_not_called()

    def test_resume_rejects_changed_spec_or_archive_and_stable_tag_drift(self):
        self.registry.fail_at = PACKAGE_ORDER[1]
        with self.assertRaises(RuntimeError):
            self.run_publish()
        self.registry.fail_at = None
        self.spec["source_commit"] = "b" * 40
        with self.assertRaisesRegex(ValueError, "checkpoint identity"):
            self.run_publish()
        self.spec["source_commit"] = "a" * 40
        archive = self.packages / "tarballs" / self.plan["publish_order"][1]["filename"]
        original = archive.read_bytes()
        archive.write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "archive changed"):
            self.run_publish()
        archive.write_bytes(original)
        self.registry.documents[PACKAGE_ORDER[-1]]["dist-tags"]["latest"] = "unexpected"
        with self.assertRaisesRegex(ValueError, "Protected"):
            self.run_publish()
        self.assertEqual(self.registry.published, PACKAGE_ORDER[:1])

    def test_existing_bytes_under_another_test_tag_are_not_retagged_implicitly(self):
        self.registry.publish(
            self.packages / "tarballs" / self.plan["publish_order"][0]["filename"],
            "mcp",
        )
        self.registry.documents[PACKAGE_ORDER[0]]["dist-tags"]["mcp"] = "newer"
        with self.assertRaisesRegex(ValueError, "implicit tag change"):
            self.run_publish()
        self.assertEqual(self.registry.published, PACKAGE_ORDER[:1])

    def test_optimized_python_keeps_conflict_and_resume_guards(self):
        result = subprocess.run(
            [
                sys.executable,
                "-O",
                "-m",
                "unittest",
                "test_airs_test_release_publish.PublicationTests.test_conflicting_immutable_version_aborts_before_any_mutation",
                "test_airs_test_release_publish.PublicationTests.test_resume_rejects_changed_spec_or_archive_and_stable_tag_drift",
            ],
            cwd=Path(__file__).parent,
            capture_output=True,
            text=True,
            timeout=30,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_stable_tag_is_rejected_by_public_spec_boundary(self):
        from airs_test_release_spec import TARGETS

        document = {
            **self.spec,
            "schema_version": 1,
            "tooling_commit": "b" * 40,
            "packaging_commit": "c" * 40,
            "previous_version": "0.1.0-alpha.22.mcp.2",
            "developer_id_team": "ABCDEFGHIJ",
            "platforms": [
                {"target": target, "binary_sha256": "d" * 64} for target in TARGETS
            ],
        }
        for tag in ["latest", "alpha", "onboarding"]:
            with (
                self.subTest(tag=tag),
                self.assertRaisesRegex(ValueError, "candidate scope"),
            ):
                validate_spec({**document, "tag": tag})

    def test_real_npm_interruption_resume_and_anonymous_install(self):
        documents = self.registry.documents
        archives, mutations, anonymous_reads = {}, [], []
        failed = [False]
        token = "synthetic-token-never-in-release-receipts"

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def respond(self, status, value):
                payload = json.dumps(value).encode()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

            def do_GET(self):
                anonymous_reads.append(self.headers.get("Authorization") is None)
                if unquote(self.path) in archives:
                    payload = archives[unquote(self.path)]
                    self.send_response(200)
                    self.send_header("Content-Length", str(len(payload)))
                    self.end_headers()
                    self.wfile.write(payload)
                elif unquote(self.path[1:]) in documents:
                    self.respond(200, documents[unquote(self.path[1:])])
                else:
                    self.respond(404, {"error": "not_found"})

            def do_PUT(self):
                if self.headers.get("Authorization") != "Bearer " + token:
                    self.respond(401, {"error": "unauthorized"})
                    return
                name = unquote(self.path[1:])
                size = int(self.headers.get("Content-Length", "0"))
                if name not in PACKAGE_ORDER or not 0 < size < 1024 * 1024:
                    self.respond(400, {"error": "invalid"})
                    return
                body = json.loads(self.rfile.read(size))
                if name == PACKAGE_ORDER[1] and not failed[0]:
                    failed[0] = True
                    self.respond(503, {"error": "synthetic_interruption"})
                    return
                documents[name]["versions"].update(body["versions"])
                documents[name]["dist-tags"].update(body["dist-tags"])
                for filename, attachment in body["_attachments"].items():
                    archives[f"/{name}/-/{filename}"] = base64.b64decode(
                        attachment["data"]
                    )
                mutations.append(name)
                self.respond(201, {"ok": True})

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        url = f"http://127.0.0.1:{server.server_port}"
        userconfig = self.root / "publication.npmrc"
        userconfig.write_text(f"//127.0.0.1:{server.server_port}/:_authToken={token}\n")
        userconfig.chmod(0o600)
        self.output.mkdir()
        # The public entrypoint validates HTTPS. This local transport injection
        # exercises npm itself without introducing a production registry bypass.
        self.registry = Registry(url, userconfig, self.output)
        with self.assertRaisesRegex(RuntimeError, "npm publication failed"):
            self.run_publish()
        self.assertEqual(mutations, PACKAGE_ORDER[:1])
        receipt = self.run_publish()
        self.assertTrue(receipt["published"])
        self.assertEqual(mutations, PACKAGE_ORDER)
        self.assertIn(True, anonymous_reads)
        self.assertNotIn(token, (self.output / "PUBLICATION.json").read_text())
        anonymous_start = len(anonymous_reads)
        prefix = self.root / "anonymous"
        prefix.mkdir()
        environment = install_environment(prefix, url, False)
        result = subprocess.run(
            [
                "npm",
                "install",
                "-g",
                "--prefix",
                str(prefix),
                "--ignore-scripts",
                "--no-audit",
                "--no-fund",
                "--registry",
                url,
                LAUNCHER + "@" + VERSION,
            ],
            cwd=prefix,
            env=environment,
            capture_output=True,
            text=True,
            timeout=60,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        command = prefix / ("airs.cmd" if os.name == "nt" else "bin/airs")
        result = subprocess.run(
            [str(command), "--version"],
            env=environment,
            capture_output=True,
            text=True,
            timeout=10,
        )
        self.assertEqual(
            (result.returncode, result.stdout.strip()), (0, "airs " + VERSION)
        )
        self.assertTrue(anonymous_reads[anonymous_start:])
        self.assertTrue(all(anonymous_reads[anonymous_start:]))


if __name__ == "__main__":
    unittest.main()
