# Managed helper relocation baseline failure

The retained source154 Linux binary was copied into disposable old/new paths containing spaces and quotes. Synthetic environment-backed inference login and file-backed managed MCP setup used no real credentials or native store. The exact saved MCP command successfully returned the expected synthetic credential through a pipe before relocation. Removing the old executable then made that saved MCP command unavailable, and launching the second copy failed because inference still invoked the absent old auth.command.

The single positive relocation regression therefore fails on this baseline, as recorded in the unchanged output. Candidate remediation has not been tested by this record. The source-hash receipt binds the new fixture bytes. Three additional fixture methods prepare normalized-revision-first fault-state replay, config-first/tool/custom-change denial and unchanged custom-helper preservation; their candidate outcomes remain pending.

MCP remains disabled in the runtime configuration to avoid introducing TLS fixture infrastructure. This establishes direct saved-helper execution/readback and the missing-executable failure, not MCP HTTP transport. Fault-state replay is not an injected process crash or power-loss test. No release or full authentication score is awarded.
