import unittest

from airs_package_access import (
    OWNER_EVIDENCE,
    REPOSITORY,
    association_accepted,
    association_from_metadata,
)


class PackageAccessTests(unittest.TestCase):
    def setUp(self):
        self.name = "@cdot65/prisma-airs-harness"
        self.data = {
            "name": "prisma-airs-harness",
            "package_type": "npm",
            "owner": {"login": "cdot65"},
            "html_url": "https://github.com/users/cdot65/packages/npm/package/prisma-airs-harness",
            "visibility": "private",
        }

    def test_omission_retains_owner_evidence(self):
        result = association_from_metadata(self.name, self.data)
        self.assertEqual(
            result,
            {
                "repository": REPOSITORY,
                "visibility": "private",
                "repository_evidence": OWNER_EVIDENCE,
            },
        )
        self.assertTrue(association_accepted(result))

    def test_explicit_absence_or_different_repository_is_not_overridden(self):
        for repository in [None, {}, {"full_name": "cdot65/different"}]:
            with self.subTest(repository=repository):
                self.assertFalse(
                    association_accepted(
                        association_from_metadata(
                            self.name, {**self.data, "repository": repository}
                        )
                    )
                )

    def test_observed_repository_accepted_without_owner_fallback(self):
        result = association_from_metadata(
            self.name, {**self.data, "repository": {"full_name": REPOSITORY}}
        )
        self.assertEqual(result, {"repository": REPOSITORY, "visibility": "private"})
        self.assertTrue(association_accepted(result))

    def test_public_visibility_and_unverified_null_rejected(self):
        self.assertFalse(
            association_accepted(
                association_from_metadata(
                    self.name, {**self.data, "visibility": "public"}
                )
            )
        )
        self.assertFalse(
            association_accepted({"repository": None, "visibility": "private"})
        )

    def test_wrong_identity_rejected(self):
        for field, value in [
            ("name", "different"),
            ("owner", {"login": "different"}),
            ("package_type", "container"),
            ("html_url", "https://example.com"),
        ]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                association_from_metadata(self.name, {**self.data, field: value})
        with self.assertRaises(ValueError):
            association_from_metadata("@cdot65/another-package", self.data)
