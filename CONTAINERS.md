# Container delivery scope

Status: planned, not validated or published. The owner deferred native Windows
delivery on 2026-09-08 and selected a Linux container as the proposed alternative
for Windows hosts. The current release remains Apple Silicon and Linux x64;
container delivery is a separate follow-up.

There is currently no validated user-facing harness container image or install
command. The Dockerfiles in `mcp-scanner`, `gateway-identity` and
`gateway-compatibility` build backend services; they are not harness installers.

Before recommending this route, verify:

- A versioned image with the matching Linux agent, pinned Prisma AIRS CLI and
  project tools, running as a non-root user.
- An interactive terminal and a selected project mount that supports edits,
  tests and file ownership on the Windows host.
- A container-specific authentication and credential-storage workflow, including
  restart/resume. A Linux container does not automatically receive Windows
  Credential Manager or the host's unlocked desktop keyring.
- Persistent configuration and session history, with credentials excluded from
  image layers and build arguments.
- The harness sandbox working under the selected container runtime. Do not count
  disabling the sandbox or granting a privileged container as acceptance.
- Gateway and remote MCP connectivity through the user's network/VPN, a real
  file-edit/test/scan task, and successful continuation after restart.
- Fresh installation and upgrade on the actual Windows container host, with
  documented versions and retained evidence.

These checks establish container behavior only. They do not establish native
Windows support or resolve the reported Mac Keychain incident. See the
[authentication release plan](AUTHENTICATION-PLAN.md) for the active release gates.
