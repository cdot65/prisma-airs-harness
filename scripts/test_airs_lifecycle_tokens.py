import unittest

from airs_lifecycle_tokens import RotatingTokens


class LifecycleTokens(unittest.TestCase):
    def setUp(self):
        self.now = 1000
        self.events = []
        self.tokens = RotatingTokens(
            "inference",
            90,
            lambda kind, **fields: self.events.append(dict(kind=kind, **fields)),
            clock=lambda: self.now,
        )

    def build(self, generation, expires):
        return {
            "access_token": f"access-{generation}",
            "refresh_token": f"refresh-{generation}",
        }

    def test_expired_access_is_rejected_and_rotation_consumes_predecessor(self):
        first = self.tokens.issue(self.build)
        self.assertTrue(self.tokens.authorize(first["access_token"], "responses"))
        self.now = 1090
        self.assertFalse(self.tokens.authorize(first["access_token"], "responses"))
        values = {"refresh_token": [first["refresh_token"]]}
        status, second = self.tokens.exchange(values, self.build)
        self.assertEqual(status, 200)
        self.assertTrue(self.tokens.authorize(second["access_token"], "responses"))
        self.assertEqual(
            self.tokens.exchange(values, self.build), (400, {"error": "invalid_grant"})
        )
        self.assertEqual(self.events[-1]["reason"], "consumed")

    def test_unknown_and_other_resource_credentials_fail(self):
        other = RotatingTokens("mcp", 90, lambda *args, **kwargs: None)
        first = other.issue(self.build)
        self.assertFalse(self.tokens.authorize(first["access_token"], "responses"))
        self.assertEqual(
            self.tokens.exchange(
                {"refresh_token": [first["refresh_token"]]}, self.build
            )[0],
            400,
        )

    def test_failed_returned_generation_never_reenables_consumed_grant(self):
        first = self.tokens.issue(self.build)

        def broken(*_args):
            raise RuntimeError("Synthetic issuance failure")

        values = {"refresh_token": [first["refresh_token"]]}
        with self.assertRaises(RuntimeError):
            self.tokens.exchange(values, broken)
        self.assertEqual(self.tokens.generation, 1)
        self.assertEqual(self.tokens.exchange(values, self.build)[0], 400)

    def test_raw_credentials_are_absent_from_events(self):
        first = self.tokens.issue(self.build)
        self.tokens.exchange({"refresh_token": [first["refresh_token"]]}, self.build)
        self.assertNotIn("access-1", str(self.events))
        self.assertNotIn("refresh-1", str(self.events))
