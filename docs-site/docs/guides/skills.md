---
title: Skills and the local agent loop
---

Skills provide task instructions to the local agent. The harness bundles Prisma
AIRS product skills with the pinned CLI and an optional TypeSafe Jev judge skill.
A skill does not grant a credential or bypass the local approval policy.

```mermaid
sequenceDiagram
  actor User
  participant Agent as Harness
  participant Skill as Local skill instructions
  participant Gateway as AI Gateway
  participant Model
  participant Local as Local tool / bundled CLI
  participant MCP as Gateway MCP listener
  User->>Agent: Request a task
  Agent->>Skill: Load relevant bounded instructions
  Agent->>Gateway: Prompt, context and tool declarations
  Gateway->>Model: Policy-approved inference
  Model-->>Agent: Proposed tool call through gateway
  Agent->>Agent: Enforce sandbox and approval policy
  alt Local operation
    Agent->>Local: Execute approved command
    Local-->>Agent: Result
  else Remote MCP operation
    Agent->>MCP: Call permitted tool
    MCP-->>Agent: Upstream result through gateway
  end
  Agent->>Gateway: Add tool result to continuing context
  Gateway->>Model: Continue inference
  Model-->>User: Answer through gateway and harness
```

The bundled product CLI calls Prisma AIRS management APIs using its own tenant
credentials. Those administrative API calls are distinct from the agent's model
inference and remote MCP traffic. Use an explicit tenant, begin with read-only
inspection, and review writes before approving them.

The optional Jev judge sends selected scan records to TypeSafe using a separate
native-stored key and an approved command. It is an explicit specialized service
integration, not an alternate route for the agent's model conversation. Dry-run
and replay modes let you inspect the workflow before live scoring. Follow the
[judge guide](judge.md) for the actual commands and output contract.

TypeSafe supplies typed judgments and probabilities; code controls execution,
thresholds and reporting. Evaluate the resulting decisions on representative
records. A typed answer is not proof that a semantic security assessment is true.
See [TypeSafe's System One model](https://docs.typesafe.ai/concepts/system-one)
for that distinction.

Files remain in the local workspace until a command or selected context sends
their contents elsewhere. Tool output may enter the next model request. Keep
secrets out of skill text, command arguments and transcripts, and use the
harness's native credential prompts for supported credentials.
