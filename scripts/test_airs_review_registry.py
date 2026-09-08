"""Keep denied registry reads distinct from an unused release version."""

import io
import json
import unittest
from unittest.mock import patch
from urllib.error import HTTPError

import check_airs_review_registry as check


class RegistryReview(unittest.TestCase):
    def inspect(self, *, status=200, candidate=False, baseline=True, identity=True):
        requests = []

        def respond(request, timeout):
            requests.append(request)
            if status != 200:
                raise HTTPError(request.full_url, status, "synthetic", None, None)
            versions = {}
            if baseline:
                versions["0.1.0-alpha.9"] = {}
            if candidate:
                versions["0.1.0-alpha.10"] = {}
            response = io.BytesIO(
                json.dumps(
                    {
                        "name": check.PACKAGES[len(requests) - 1]
                        if identity
                        else "wrong",
                        "versions": versions,
                    }
                ).encode()
            )
            response.status = 200
            return response

        with patch.object(check, "build_opener") as builder:
            builder.return_value.open.side_effect = respond
            result = check.inspect("0.1.0-alpha.10", "synthetic-secret")
        self.assertNotIn("synthetic-secret", json.dumps(result))
        self.assertEqual(len(requests), 3)
        self.assertTrue(all(r.get_method() == "GET" for r in requests))
        return result

    def test_readable_existing_packages_and_unused_version(self):
        result = self.inspect()
        self.assertTrue(result["passed"])
        self.assertFalse(result["published"])
        self.assertFalse(result["version_reserved"])
        self.assertFalse(result["teammate_access_tested"])

    def test_denials_redirects_missing_baseline_and_existing_version_fail(self):
        for status in (302, 401, 403, 404):
            with self.subTest(status=status):
                self.assertFalse(self.inspect(status=status)["passed"])
        self.assertFalse(self.inspect(candidate=True)["passed"])
        self.assertFalse(self.inspect(baseline=False)["passed"])
        with self.assertRaisesRegex(ValueError, "identity"):
            self.inspect(identity=False)

    def test_invalid_inputs_never_open_network(self):
        with patch.object(check, "build_opener", side_effect=AssertionError("network")):
            for version, token in (("latest", "secret"), ("0.1.0-alpha.10", "")):
                with self.subTest(version=version), self.assertRaises(ValueError):
                    check.inspect(version, token)


if __name__ == "__main__":
    unittest.main()
