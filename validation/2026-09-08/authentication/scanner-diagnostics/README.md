# Scanner diagnostic rollout

A live Keycloak/MCP acceptance attempt reached the scanner tool but received an
unavailable/incomplete result. The backend previously discarded the cause.
Source `c92a7079f` adds only bounded, allowlisted failure metadata; raw errors,
prompts and credentials remain excluded, and uncertain scans are not retried.

The [test summary](tests.json) records seven passing MCP SDK test groups. The
[candidate receipt](candidate.json) identifies an incremental OCI layer built on
the exact prior image, preserving dependencies and runtime configuration. It is
not a Dockerfile rebuild. Harbor reported zero Critical/High, thirteen Medium
and seven Low findings. The [isolated smoke test](smoke.json) used placeholder
credentials and no scanner requests.

The [subsequent rollout](rollout.json) changed only the production Deployment
image digest and verified both replicas. Earlier receipt fields retain their
original pre-deployment state; rollout.json is the later state. The previous
image is retained for rollback. A rollout alone does not explain the original
scanner failure or pass the full authentication release. Fresh authenticated
acceptance is recorded separately against its exact harness binary.
