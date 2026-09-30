"""A Mac-first handoff must never satisfy or publish an ordinary full release."""

import copy
import json
from pathlib import Path
import tempfile
import unittest

from airs_release_acceptance import verify_acceptance_set
from airs_test_release_spec import (
    MAC_SCOPE,
    MAC_TAG,
    MAC_TARGET,
    package_order,
    validate_spec,
)
from airs_test_release_stage import stage_packages, verify_staged
from airs_test_release_publish import _publish
from test_airs_test_release_stage import sample_spec, candidates
from test_airs_release_acceptance import evidence_set
from test_airs_test_release_publish import MemoryRegistry


class MacPreview(unittest.TestCase):
    def spec(self):
        spec = sample_spec()
        spec.update(scope=MAC_SCOPE, tag=MAC_TAG)
        spec["platforms"] = [p for p in spec["platforms"] if p["target"] == MAC_TARGET]
        return validate_spec(spec)

    def test_only_explicit_mac_scope_accepts_one_signed_platform(self):
        spec = self.spec()
        self.assertEqual(
            package_order(spec), ["airs-harness-darwin-arm64", "prisma-airs-harness"]
        )
        for change in [
            {"scope": "owner-authorized-test", "tag": "mcp"},
            {
                "scope": "owner-authorized-stable",
                "tag": "stable-candidate",
                "version": "0.1.3",
            },
            {"tag": "latest"},
            {"tag": "mcp"},
            {"platforms": sample_spec()["platforms"]},
            {"platforms": [sample_spec()["platforms"][0]]},
        ]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate_spec({**spec, **change})

    def test_signed_mac_evidence_stages_and_publishes_only_two_packages(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            spec = self.spec()
            candidate = root / "candidate"
            candidates(candidate, spec)
            evidence = root / "evidence"
            evidence_set(evidence, candidate, spec)
            verified = verify_acceptance_set(spec, evidence)
            self.assertEqual([p["target"] for p in verified["platforms"]], [MAC_TARGET])
            staged = root / "staged"
            stage_packages(spec, candidate, evidence, staged)
            plan = verify_staged(spec, staged, evidence)
            registry = MemoryRegistry(plan)
            before = copy.deepcopy(registry.documents)
            result = _publish(spec, plan, staged, root / "publication", registry)
            self.assertTrue(result["published"])
            self.assertEqual(registry.published, package_order(spec))
            for name in ["airs-harness-linux-x64", "airs-harness-linux-arm64"]:
                self.assertEqual(registry.documents[name], before[name])
            for name in package_order(spec):
                self.assertEqual(
                    registry.documents[name]["dist-tags"],
                    {**before[name]["dist-tags"], MAC_TAG: spec["version"]},
                )
            # A repeated command verifies immutable bytes without republishing.
            _publish(spec, plan, staged, root / "publication", registry)
            self.assertEqual(registry.published, package_order(spec))
            signature = (
                evidence
                / MAC_TARGET
                / "files/mac-signature/INSTALLED-MAC-SIGNATURE.json"
            )
            signature.write_text(json.dumps({"passed": False}))
            with self.assertRaises(ValueError):
                verify_staged(spec, staged, evidence)


if __name__ == "__main__":
    unittest.main()
