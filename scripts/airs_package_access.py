"""Resolve package association without interpreting omitted metadata as absence.

On 2026-09-09 the repository owner confirmed all three existing packages were
already bound to cdot65/airs-harness. GitHub's workflow-token REST response omits
repository entirely. Preserve that evidence distinction; explicit contradictory
metadata, wrong package identity, or public visibility still fail closed.
"""

REPOSITORY = "cdot65/airs-harness"
PACKAGES = {
    "@cdot65/prisma-airs-harness",
    "@cdot65/prisma-airs-harness-linux-x64",
    "@cdot65/prisma-airs-harness-darwin-arm64",
}
OWNER_EVIDENCE = "owner-confirmed-2026-09-09; REST repository field omitted"


def association_from_metadata(name, data):
    if name not in PACKAGES:
        raise ValueError("Unexpected package identity")
    short = name.split("/")[1]
    if (
        data.get("name") != short
        or data.get("package_type") != "npm"
        or (data.get("owner") or {}).get("login") != "cdot65"
        or data.get("html_url")
        != "https://github.com/users/cdot65/packages/npm/package/" + short
    ):
        raise ValueError("GitHub package identity differs from requested package")
    observed = data.get("repository")
    result = {
        "repository": observed.get("full_name") if isinstance(observed, dict) else None,
        "visibility": data.get("visibility"),
    }
    if "repository" not in data:
        result.update(repository=REPOSITORY, repository_evidence=OWNER_EVIDENCE)
    return result


def association_accepted(value):
    expected = {"repository": REPOSITORY, "visibility": "private"}
    return value in (expected, {**expected, "repository_evidence": OWNER_EVIDENCE})
