# Final autonomous handoff

All four planned focuses passed independent review: first-use reliability 9.4/10,
diagnostic reporting 9.4/10, lifecycle recovery 9.5/10 and release reliability 9.5/10.
The owner review remains pending. The final release review and independent audit
receipts here refer to evidence commit `0b171dd54fd724634ec696c5a6ea60a9d83ef054`;
they do not create a self-referential audit claim.

Install the published test version:

```sh
npm install -g airs-harness@0.1.2-alpha.1.mcp.1 --registry=https://npm.cdot.io
```

Stable latest remains `0.1.1`. Exact candidates and fresh anonymous registry installs
passed on Ubuntu x64, native Linux ARM64 and signed/notarized Apple Silicon,
including stable upgrade/rollback and preserved credentials/conversation history.
The full GNU run remains failed: 18,331 passed, 3 disabled experimental remote shell
snapshot V2 failures, 34 skipped. This is neither stable promotion nor a new
attended production SSO/ServiceNow claim.

The getting-started guide is deployed at `0235219441a75f9aa28c67312e47e6be1037ca90`:
https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/
All 22 local/CI browser checks passed and all 16 live pages matched the artifact.

Future work is separate: owner feedback on the new report, an explicit stable
promotion decision, experimental remote snapshot defects, and gateway support
for native MCP device authorization. No new authentication service was added.
