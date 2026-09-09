"""Mock registry mutation controls over real immutable local archive bytes."""

import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import publish_airs_review as publisher
from plan_airs_review_publication import LAUNCHER, NATIVES, REGISTRY, regular_digest


class FakeRegistry:
    def __init__(self, records, archives):
        self.records = {row["name"] + "@" + row["version"]: row for row in records}
        self.archives = archives
        self.existing = set()
        self.mutations = []
        self.corrupt_download = None
        self.wrong_integrity = None
        self.wrong_association = False
        self.tag_values = {}

    def view(self, spec):
        if spec not in self.existing:
            return None
        row = self.records[spec]
        return {
            "integrity": "sha512-wrong"
            if self.wrong_integrity == spec
            else row["integrity"],
            "tarball": REGISTRY
            + "/download/"
            + row["name"]
            + "/"
            + row["version"]
            + "/archive",
        }

    def download(self, spec, directory):
        row = self.records[spec]
        path = directory / row["filename"]
        shutil.copyfile(self.archives / row["filename"], path)
        if self.corrupt_download == spec:
            path.write_bytes(b"registry bytes differ")
        return path

    def publish(self, archive, tag):
        row = next(
            row for row in self.records.values() if row["filename"] == archive.name
        )
        spec = row["name"] + "@" + row["version"]
        self.mutations.append(("publish", spec, tag))
        self.existing.add(spec)
        self.tag_values[row["name"]] = {tag: row["version"]}

    def tag(self, spec, tag):
        self.mutations.append(("tag", spec, tag))
        row = self.records[spec]
        self.tag_values[row["name"]] = {tag: row["version"]}

    def tags(self, name):
        return self.tag_values.get(name, {})

    def association(self, name):
        return {
            "repository": None if self.wrong_association else "cdot65/airs-harness",
            "visibility": "private",
        }


class ReviewPublisher(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.archives = self.root / "tarballs"
        self.archives.mkdir()
        records = []
        for index, name in enumerate([*sorted(NATIVES), LAUNCHER]):
            path = self.archives / f"fixture-{index}.tgz"
            path.write_bytes(("synthetic reviewed archive " + name).encode())
            sha256, integrity = regular_digest(path)
            records.append(
                {
                    "name": name,
                    "version": "0.1.0-alpha.10",
                    "filename": path.name,
                    "sha256": sha256,
                    "integrity": integrity,
                }
            )
        self.plan = {
            "scope": "signed-prerelease-distribution",
            "source_commit": "b" * 40,
            "dist_tag": "auth-review",
            "version": "0.1.0-alpha.10",
            "packages": records,
        }
        self.plan_file = self.root / "approved.json"
        self.plan_file.write_text(json.dumps(self.plan))
        self.plan_hash = regular_digest(self.plan_file)[0]
        self.registry = FakeRegistry(records, self.archives)
        mocked = patch.object(publisher, "plan_publication", return_value=self.plan)
        self.plan_check = mocked.start()
        self.addCleanup(mocked.stop)

    def publish(self, directory="publication"):
        return publisher.publish_review(
            self.root,
            self.plan_file,
            self.plan_hash,
            self.root / directory,
            registry=self.registry,
        )

    def test_success_publishes_exact_snapshots_native_first_and_verifies_downloads(
        self,
    ):
        result = self.publish()
        self.assertTrue(result["complete"])
        self.assertTrue(result["published"])
        self.assertFalse(result["full_authentication_release_ready"])
        self.assertFalse(result["teammate_read_only_install_verified"])
        self.assertEqual(
            [entry[1].split("@0.1")[0] for entry in self.registry.mutations],
            [row["name"] for row in self.plan["packages"]],
        )
        self.assertTrue(all(row["download_verified"] for row in result["packages"]))
        self.assertEqual(self.plan_check.call_count, 2)

    def test_plan_tampering_prevents_all_registry_mutations(self):
        self.plan_file.write_text("{}")
        with self.assertRaises(ValueError):
            self.publish()
        self.assertEqual(self.registry.mutations, [])

    def test_any_existing_collision_aborts_before_publishing_other_packages(self):
        spec = LAUNCHER + "@0.1.0-alpha.10"
        self.registry.existing.add(spec)
        self.registry.wrong_integrity = spec
        with self.assertRaises(ValueError):
            self.publish()
        self.assertEqual(self.registry.mutations, [])
        result = json.loads((self.root / "publication/PUBLICATION.json").read_text())
        self.assertFalse(result["complete"])
        self.assertFalse(result["mutation_attempted"])

    def test_download_failure_retains_partial_receipt_and_retry_reuses_same_versions(
        self,
    ):
        second = self.plan["packages"][1]["name"] + "@0.1.0-alpha.10"
        self.registry.corrupt_download = second
        with self.assertRaises(ValueError):
            self.publish()
        result = json.loads((self.root / "publication/PUBLICATION.json").read_text())
        self.assertFalse(result["complete"])
        self.assertTrue(result["mutation_attempted"])
        self.assertEqual(len(result["packages"]), 1)
        self.assertEqual(len(self.registry.mutations), 2)
        self.registry.corrupt_download = None
        result = self.publish("retry")
        self.assertTrue(result["complete"])
        self.assertEqual(
            sum(kind == "publish" for kind, *_ in self.registry.mutations), 3
        )
        self.assertEqual(sum(kind == "tag" for kind, *_ in self.registry.mutations), 2)

    def test_unassociated_existing_package_is_not_silently_accepted(self):
        # Package names may already exist even when this exact version does not.
        self.registry.wrong_association = True
        with self.assertRaises(ValueError):
            self.publish()
        self.assertEqual(self.registry.mutations, [])

    def test_registry_commands_require_explicit_missing_error_and_never_run_scripts(
        self,
    ):
        registry = publisher.Registry(self.root / "cache")
        with patch.object(
            subprocess,
            "run",
            return_value=subprocess.CompletedProcess(
                [], 1, "", "E404 incidental error"
            ),
        ):
            with self.assertRaises(RuntimeError):
                registry.view("example@1")
        with patch.object(
            subprocess,
            "run",
            return_value=subprocess.CompletedProcess(
                [], 1, '{"error":{"code":"E404"}}', ""
            ),
        ):
            self.assertIsNone(registry.view("example@1"))
        with patch.object(
            subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "", "")
        ) as run:
            registry.publish(self.archives / "fixture-0.tgz", "auth-review")
            argv = run.call_args.args[0]
            self.assertIn("--ignore-scripts", argv)
            self.assertEqual(argv[argv.index("--tag") + 1], "auth-review")
            self.assertEqual(argv[argv.index("--registry") + 1], REGISTRY)
            self.assertNotIn("shell", run.call_args.kwargs)


if __name__ == "__main__":
    unittest.main()
