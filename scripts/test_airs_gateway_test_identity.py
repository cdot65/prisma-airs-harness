import unittest

from airs_gateway_test_identity import GatewayTestIdentity
from promote_airs_mcp_prerelease import TOOLS


class GatewayTestIdentityTests(unittest.TestCase):
    def test_runs_using_the_same_endpoint_have_disjoint_keyring_accounts(self):
        first = GatewayTestIdentity("https://gateway.example/mcp")
        second = GatewayTestIdentity(first.endpoint)
        self.assertTrue(set(first.accounts()).isdisjoint(second.accounts()))
        for identity in [first, second]:
            self.assertTrue(all(key.startswith(identity.name + "|") for key in identity.accounts()))
            self.assertFalse(any(key.startswith("prisma-airs|") for key in identity.accounts()))
            self.assertTrue(all(len(f"mcp__{identity.name}__{tool}") <= 64 for tool in TOOLS))

    def test_readback_rejects_other_runs_and_everyday_credentials(self):
        identity = GatewayTestIdentity("https://gateway.example/mcp")
        identity.verify_record({"server_name": identity.name, "url": identity.endpoint})
        for record in [
            {"server_name": "prisma-airs", "url": identity.endpoint},
            {"server_name": GatewayTestIdentity(identity.endpoint).name, "url": identity.endpoint},
            {"server_name": identity.name, "url": "https://other.example/mcp"},
        ]:
            with self.assertRaisesRegex(ValueError, "does not belong"):
                identity.verify_record(record)


if __name__ == "__main__":
    unittest.main()
