# Managed Prisma AIRS CLI and skills

The alpha.9 release candidate requires `@cdot65/prisma-airs-cli@5.2.0`, whose
manifest pins `@cdot65/prisma-airs-sdk@0.28.0`. npm installs the CLI with the
harness. An existing global `airs` installation is left alone. The harness
checks the exact CLI version before startup and reports a reinstall error if
it is missing or mismatched; it never falls back to a different global CLI.

Use Node.js 22.13 or newer in the 22.x line, or 23.5 or newer. Keep npm optional
dependencies enabled: they include the harness executable and the CLI's image
and document generators. The supported native release targets remain Linux x64
and Apple Silicon macOS. The JavaScript integration is also tested on Windows;
a Windows native harness package is a separate release prerequisite.

## Run and diagnose

These commands work before gateway setup or Keycloak login:

```sh
airs-harness airs --version
airs-harness airs doctor --output json
airs-harness airs runtime --help
```

Inside the harness, ask:

> Use $prisma-airs-cli to check the managed CLI version and run doctor. Report
> missing credential variable names and affected capabilities without revealing
> their values. Do not modify configuration.

Eight built-in skills cover setup/diagnosis, Runtime Security, Guardrail
Generation, Red Teaming, AI Gateway, Model Security, DLP Testing, and DLP
Management. The setup skill routes to the relevant capability so unrelated
instructions do not consume the session context. The skills are embedded in
the native executable and installed through the existing system-skill mechanism;
user-created skills outside the system directory are preserved.

Agent shell commands use the absolute `AIRS_MANAGED_CLI` path supplied by the npm
launcher: `"$AIRS_MANAGED_CLI" doctor --output json` on POSIX or
`& $env:AIRS_MANAGED_CLI doctor --output json` in PowerShell. Login shell startup
files can reorder PATH, so a bare `airs` is not reliable proof of the managed
version. Direct native-archive users must install CLI 5.2.0 separately and verify
its version; the npm distribution manages the requirement automatically.

## Credentials

| Capability | Environment variables |
| --- | --- |
| Runtime scanner | `PANW_AI_SEC_API_KEY` (or `PANW_AI_SEC_API_TOKEN` for token-capable scan commands) |
| Management OAuth | `PANW_MGMT_CLIENT_ID`, `PANW_MGMT_CLIENT_SECRET`, `PANW_MGMT_TSG_ID` |
| Alternate trusted config file | `PRISMA_AIRS_CONFIG_PATH` |

Existing protected `~/.prisma-airs/config.json` is supported. Nonempty environment
values override file values. Users can provision these through their existing
secret manager or login environment before starting the harness. Keycloak user
JWTs and gateway workspace inference keys do not replace management service
account credentials. This phase does not add a management credential store or
write plaintext secrets on behalf of the user.

The managed wrapper disables automatic loading of a working directory's `.env`.
This prevents repository-controlled endpoint settings from redirecting requests
that use trusted credentials. Explicit environment settings and a trusted config
path still work. CLI/library explicit overrides retain their normal behavior.
Local shell tools inherit environment credentials; this is not a secret broker.
Avoid secret-valued command arguments, raw config output and debug/reveal flags.
Doctor exception text is not universally redacted; summarize statuses before
sharing results. Network calls follow the harness's existing permission policy.

## Known CLI 5.2.0 limits

Doctor probes scanner, management and gateway APIs; a green result is not proof
of every permission. Its scanner credential check expects a key even when a
scan command can use a token. Validate the requested operation separately.

Scan/evaluation summaries can hide normalized failures behind an Allow result.
The runtime and guardrail skills require complete successful scan evidence;
they must report inconclusive results when that evidence is unavailable.
Guardrail creation upserts by name, evaluation is profile-wide, and revert
performs separate detach/delete writes. The skill documents candidate isolation,
full policy capture and verified restoration before any refinement loop.

DLP generation supports PDF, PNG, JPEG, SVG and DOCX, and does not itself scan
these files. ZIP generation is unsupported. Detection requires a verified
upload-capable scanner integration. Model Security's separate Python scanner
requires additional provisioning; its installation is not an npm prerequisite.

## Upgrade procedure

1. Select an explicit CLI version and review its manifest, SDK pin, help and
   changed behavior. Update the exact harness dependency and regenerate the
   npm lockfile against the trusted registry. No caret or automatic latest pin.
2. Update only affected skills against that version's commands and schemas.
   Independently forward-test high-impact workflows, especially rollback,
   credential handling and interpretation of security results.
3. Run launcher tests and `scripts/validate_prisma_cli.py` using a clean locked
   install on Linux, Apple Silicon and Windows. It checks actual command
   availability, missing-credential diagnosis, configuration precedence and real
   document/image generation. The npm lockfile fixes the CI dependency graph;
   published installs pin the CLI/SDK exactly but may resolve newer compatible
   transitive dependencies. Candidate registry-install acceptance tests that graph.
4. Rebuild the native executable because skills are compiled assets. Reuse the
   Rust cache; run scoped Rust tests and native/npm acceptance, then exercise a
   built-in skill through the actual agent. Preserve results and independent
   review with the release. Never build Intel Mac artifacts.
5. Publish a new harness version with its validated native packages, update the
   compatibility row below and promote only after the candidate checks pass.

| Harness | CLI | SDK | Status |
| --- | --- | --- | --- |
| 0.1.0-alpha.9 | 5.2.0 | 0.28.0 | Candidate; release acceptance in progress |
| 0.1.0-alpha.8 | None required | None required | Previously published |
