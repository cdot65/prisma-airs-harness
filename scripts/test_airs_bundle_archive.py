import base64
import gzip
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import Mock, patch

from airs_bundle_archive import NoRedirect, download, unpack


def archive(entries, root="package"):
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as tar:
        data = json.dumps({"name": "fixture", "version": "1.0.0"}).encode()
        for name, content, kind in [
            (root + "/package.json", data, tarfile.REGTYPE),
            *entries,
        ]:
            member = tarfile.TarInfo(name)
            member.type = kind
            member.size = len(content)
            if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
                member.linkname = "../../outside"
            tar.addfile(member, io.BytesIO(content))
    blob = output.getvalue()
    return blob, "sha512-" + base64.b64encode(hashlib.sha512(blob).digest()).decode()


class ArchiveValidation(unittest.TestCase):
    def extract(self, entries, **overrides):
        blob, integrity = archive(entries, root=overrides.pop("root", "package"))
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "dependency"
            try:
                return unpack(
                    blob,
                    overrides.pop("integrity", integrity),
                    destination,
                    overrides.pop("name", "fixture"),
                    overrides.pop("version", "1.0.0"),
                    overrides.pop("remaining", 1024 * 1024),
                    **overrides,
                )
            except ValueError:
                self.assertFalse(
                    destination.exists(), "Rejected archive allocated dependency files"
                )
                raise

    def test_verified_nonstandard_single_root_retains_license_and_runtime_hashes(self):
        result = self.extract(
            [
                ("node v25.9/LICENSE", b"license", tarfile.REGTYPE),
                ("node v25.9/lib/index.js", b"code", tarfile.REGTYPE),
            ],
            root="node v25.9",
        )
        self.assertEqual(result["license_files"], ["LICENSE"])
        self.assertEqual(
            result["files"]["lib/index.js"], hashlib.sha256(b"code").hexdigest()
        )

    def test_paths_rejected_before_extraction_on_all_platforms(self):
        for name in [
            "/package/evil",
            "package/../escape",
            "package/./file",
            "package//file",
            "package\\evil",
            "C:/evil",
            "package/a:stream",
            "package/NUL.txt",
            "package/NUL .txt",
            "package/COM1",
            "package/LPT².txt",
            "package/file.",
            "package/file ",
            "package/a?b",
            "package/a\nb",
            "//host/share/evil",
        ]:
            with self.subTest(path=name), self.assertRaises(ValueError):
                self.extract([(name, b"evil", tarfile.REGTYPE)])

    def test_duplicates_case_collisions_and_file_directory_conflicts_are_rejected(self):
        cases = [
            [("package/package.json", b"duplicate", tarfile.REGTYPE)],
            [
                ("package/A", b"one", tarfile.REGTYPE),
                ("package/a", b"two", tarfile.REGTYPE),
            ],
            [
                ("package/A/file", b"one", tarfile.REGTYPE),
                ("package/a", b"two", tarfile.REGTYPE),
            ],
            [
                ("package/A", b"one", tarfile.REGTYPE),
                ("package/a/file", b"two", tarfile.REGTYPE),
            ],
            [
                ("package/é", b"one", tarfile.REGTYPE),
                ("package/e\u0301", b"two", tarfile.REGTYPE),
            ],
        ]
        for entries in cases:
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                self.extract(entries)

    def test_links_devices_and_multiple_roots_are_rejected(self):
        for kind in [
            tarfile.SYMTYPE,
            tarfile.LNKTYPE,
            tarfile.CHRTYPE,
            tarfile.BLKTYPE,
            tarfile.FIFOTYPE,
        ]:
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                self.extract([("package/entry", b"", kind)])
        with self.assertRaises(ValueError):
            self.extract([("different/file", b"value", tarfile.REGTYPE)])

    def test_integrity_identity_member_and_aggregate_bounds_fail_before_writes(self):
        for values in [
            {"integrity": "sha512-AAAA"},
            {"integrity": "sha1-YQ=="},
            {"name": "wrong-name"},
            {"version": "wrong-version"},
            {"remaining": 1},
            {"remaining_path_bytes": 1},
            {"remaining_members": 0},
        ]:
            with self.subTest(values=values), self.assertRaises(ValueError):
                self.extract([], **values)
        with patch("airs_bundle_archive.MAX_MEMBERS", 1), self.assertRaises(ValueError):
            self.extract([("package/second", b"x", tarfile.REGTYPE)])
        with (
            patch("airs_bundle_archive.MAX_METADATA", 128),
            self.assertRaisesRegex(ValueError, "decompression exceeds"),
        ):
            self.extract(
                [("package/large", b"x" * 20000, tarfile.REGTYPE)], remaining=512
            )


class DownloadValidation(unittest.TestCase):
    def test_untrusted_urls_never_open_network(self):
        with patch("airs_bundle_archive.build_opener") as opener:
            for url in [
                "http://registry.npmjs.org/p.tgz",
                "https://evil.test/p.tgz",
                "https://registry.npmjs.org:443/p.tgz",
                "https://user@registry.npmjs.org/p.tgz",
                "https://registry.npmjs.org/p.tgz?token=secret",
                "https://registry.npmjs.org/p.tgz#x",
                "https://registry.npmjs.org/../p.tgz",
                "https://registry.npmjs.org/%2e%2e/p.tgz",
                "\nhttps://registry.npmjs.org/p.tgz",
            ]:
                with self.subTest(url=url), self.assertRaises(ValueError):
                    download(url)
            opener.assert_not_called()

    def test_redirect_and_download_bounds(self):
        with self.assertRaisesRegex(ValueError, "redirects are prohibited"):
            NoRedirect().redirect_request(None, None, 302, "", {}, "https://evil.test/")
        response = Mock()
        response.geturl.return_value = "https://registry.npmjs.org/p.tgz"
        response.read.return_value = b"x" * 9
        context = Mock()
        context.__enter__ = Mock(return_value=response)
        context.__exit__ = Mock(return_value=False)
        with (
            patch("airs_bundle_archive.build_opener") as opener,
            patch("airs_bundle_archive.MAX_ARCHIVE", 8),
        ):
            opener.return_value.open.return_value = context
            with self.assertRaisesRegex(ValueError, "download exceeds"):
                download("https://registry.npmjs.org/p.tgz")
            response.read.assert_called_once_with(9)


if __name__ == "__main__":
    unittest.main()
