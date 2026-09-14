"""Validate the owner's September 14 limited alpha.14 publication scope.

This does not turn interrupted lifecycle receipts into passed test runs.
The default promoter continues to require complete lifecycle acceptance.
"""

from promote_airs_mcp_prerelease import TOOLS

HASHES = {
    "Linux": "ec0d70b86a235ca256bc027ec14ba7c9db8788a5dcf248bd7bebc81e3af79d5b",
    "Darwin": "fa511e5d075ce0d5650ad0ce16a065e9c3c80b16aab16855ab4d9f62b73526b4",
}
LIMITATIONS = [
    "Hourly frontend MCP refresh and concurrent post-expiry recovery are unverified; zero cycles completed.",
    "Silent-wait lifecycle runs failed: inference verification after MCP logout encountered an expired refresh grant; inference logout was not reached.",
    "Keycloak SSO idle timeout and observed upstream refresh-grant lifetime are 1800 seconds; inactivity can require fresh login.",
    "Full workspace validation and independent release review are not claimed.",
]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_exception(info, e2e, exception):
    platform = {"aarch64-apple-darwin": "Darwin", "x86_64-unknown-linux-musl": "Linux"}[
        info["target"]
    ]
    require(
        info["version"] == "0.1.0-alpha.14"
        and info["source_commit"] == "6195ca83e75c9d94370396b0a30cbb999c48c046",
        "Exception applies only to frozen alpha.14",
    )
    require(
        e2e["platform"] == platform and info["binary_sha256"] == HASHES[platform],
        "Exception executable mismatch",
    )
    require(
        exception.get("authorization")
        == "Owner explicitly directed release without another hour-long authentication test on 2026-09-14",
        "Owner release direction missing",
    )
    require(
        exception.get("binary_sha256") == info["binary_sha256"]
        and exception.get("started_at") == e2e["started_at"],
        "Exception must bind the actual run",
    )
    require(
        exception.get("limitations") == LIMITATIONS
        and exception.get("frontend_refresh_cycles_completed") == 0,
        "Lifecycle limitations must remain explicit",
    )
    require(
        e2e.get("passed") is False, "Preserve the original failed lifecycle receipt"
    )
    rows = {r["case"]: r for r in e2e["results"]}
    required = {
        "inference_browser_pkce",
        "mcp_browser_pkce",
        "gateway_credential_initial",
        "doctor_after_mcp",
        "all_read_tools",
        "all_read_tools_tool_results",
        "inference_history_preserved",
        "mcp_logout",
        "mcp_list_after_logout",
    }
    require(
        all(rows.get(k, {}).get("passed") is True for k in required),
        "Initial native workflow acceptance incomplete",
    )
    require(
        TOOLS <= set(rows["all_read_tools_tool_results"]["tools"]),
        "Eight model-selected tools required",
    )
    observation = exception["gateway_observation"]
    require(
        observation["native_metadata"] == rows["gateway_credential_initial"],
        "Gateway evidence must bind native login",
    )
    calls = observation["gateway_calls"]
    require(
        TOOLS
        <= {
            r["tool"]
            for r in calls
            if r["status"] == 200
            and r["client_id"] == rows["gateway_credential_initial"]["client_id"]
        },
        "Gateway calls missing",
    )
    require(
        TOOLS
        <= {
            r["tool"]
            for r in observation["correlated_requests"]
            if r["upstream_request_ids"]
            and any(
                c["trace_id"] == r["gateway_trace"] and c["tool"] == r["tool"]
                for c in calls
            )
        },
        "Upstream correlations missing",
    )
    require(
        observation["denial"]["status"] == 401
        and observation["denial"]["oauth_challenge"]
        and observation["denial"]["endpoint"] == e2e["endpoint"],
        "Measured gateway denial missing",
    )
    initial = observation["upstream_token_metadata"]
    renewed = exception["upstream_renewal"]
    require(
        renewed["iat"] >= initial["exp"]
        and renewed["access_token_sha256"] != initial["access_token_sha256"]
        and renewed["client_id"] == initial["client_id"] == "prisma-airs-gateway-mcp"
        and renewed["exp"] - renewed["iat"] == 300,
        "Gateway-held upstream renewal missing",
    )
    require(
        exception["tui"].get("passed") is True
        and exception["frontend_token_stability"].get("passed") is True,
        "Native terminal and token stability checks missing",
    )
