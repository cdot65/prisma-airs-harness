"""Archive restoration boundaries over exact synthetic package bytes."""

import io
import json
import tarfile
import unittest

import test_plan_airs_review_publication as plan_fixture
from plan_airs_review_publication import plan_publication, LAUNCHER
from restore_airs_review_stage import extract, restore_stage


class RestoreReviewStage(unittest.TestCase):
    def setUp(self):
        self.fixture = plan_fixture.ReviewPublicationPlan()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root

    def restore(self):
        return restore_stage(
            self.root / "tarballs",
            self.root / "NPM-PACKAGES.json",
            self.root / "restored",
        )

    def archive(self, entries):
        archive = self.root / "crafted.tgz"
        with tarfile.open(archive, "w:gz") as tar:
            for name, kind in entries:
                member = tarfile.TarInfo(name)
                member.type = kind
                member.linkname = "/outside"
                payload = b"{}"
                if kind == tarfile.REGTYPE:
                    member.size = len(payload)
                tar.addfile(member, io.BytesIO(payload) if member.size else None)
        return archive

    def test_exact_restore_preserves_archives_and_full_plan(self):
        expected = plan_publication(self.root, "auth-review")
        result = self.restore()
        self.assertEqual(
            result,
            {
                "restored_packages": 3,
                "executed": False,
                "repacked": False,
                "publication_plan_still_required": True,
            },
        )
        self.assertEqual(
            plan_publication(self.root / "restored", "auth-review"), expected
        )
        for record in self.fixture.receipt["publish_order"]:
            self.assertEqual(
                (self.root / "restored/tarballs" / record["filename"]).read_bytes(),
                (self.root / "tarballs" / record["filename"]).read_bytes(),
            )

    def test_hash_and_unexpected_archive_set_reject_before_extraction(self):
        extra = self.root / "tarballs/extra.tgz"
        extra.write_bytes(b"unexpected archive")
        with self.assertRaises(ValueError):
            self.restore()
        self.assertFalse((self.root / "restored").exists())
        extra.unlink()
        record = self.fixture.receipt["publish_order"][0]
        (self.root / "tarballs" / record["filename"]).write_bytes(b"changed")
        with self.assertRaises(ValueError):
            self.restore()
        self.assertFalse((self.root / "restored").exists())

    def test_links_devices_traversal_duplicates_and_case_aliases_reject(self):
        cases = [
            [("package/package.json", tarfile.SYMTYPE)],
            [("package/package.json", tarfile.LNKTYPE)],
            [("package/package.json", tarfile.FIFOTYPE)],
            [("package/../escape", tarfile.REGTYPE)],
            [
                ("package/package.json", tarfile.REGTYPE),
                ("package/package.json", tarfile.REGTYPE),
            ],
            [("package/A", tarfile.REGTYPE), ("package/a", tarfile.REGTYPE)],
            [
                ("package/café", tarfile.REGTYPE),
                ("package/cafe\u0301", tarfile.REGTYPE),
            ],
            [("package/A/file", tarfile.REGTYPE), ("package/a/other", tarfile.REGTYPE)],
            [
                ("package/folder/file", tarfile.REGTYPE),
                ("package/folder", tarfile.REGTYPE),
            ],
        ]
        for index, entries in enumerate(cases):
            with self.subTest(index=index):
                archive = self.archive(entries)
                with self.assertRaises(ValueError):
                    extract(archive, self.root / f"case-{index}")
        self.assertFalse((self.root / "escape").exists())

    def test_private_or_changed_manifest_is_not_converted_to_release(self):
        path = self.root / LAUNCHER / "package.json"
        manifest = json.loads(path.read_text())
        manifest["private"] = True
        path.write_text(json.dumps(manifest))
        self.fixture.pack(LAUNCHER)
        self.fixture.save_receipt()
        with self.assertRaises(ValueError):
            self.restore()
        self.assertTrue(json.loads(path.read_text())["private"])


if __name__ == "__main__":
    unittest.main()
