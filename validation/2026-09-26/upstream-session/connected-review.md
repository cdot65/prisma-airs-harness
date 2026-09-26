# Session recovery adversarial review — in progress

This is a self-review with reproducible tests, not an independent certification. Scores remain pending until the connected source passes its required gates; earlier transport receipts do not validate this patch.

| Dimension | Required evidence before 9/10 | Current state |
| --- | --- | --- |
| Implementation | Atomic persistence; equivalent bounded reconstruction; surviving/discarded rollback checkpoints; actual resumed request and owning gateway config | Targeted cases pass; connected package/native runs pending |
| Code quality | Small shared metadata/helper APIs; reviewed schema/snapshots; scoped lint and formatting; no unrelated refactor | Data prerequisite lint passed; connected lint/format pending |
| Design | Canonical thread ownership; explicit-empty versus absent metadata; no parent continuation identity; current route/effort overrides; old-server fallback | Source reviewed and adversarial regressions added; native/full-workspace confirmation pending |
| Feature completeness | Storage, core, app-server and TUI agree on resumed state; first prompt and disconnected draft behavior; native executables rebuilt | Connected source implemented; tests, native build and workspace comparison pending |

## Challenges that changed the implementation

- Upstream's simplified storage scanner omitted legacy rollback handling that AIRS still needs. Both storage selection and core bounded replay now reject a shortcut across a newer rollback marker.
- Full replay initially lost previous-turn settings on a surviving current checkpoint after rollback. A failing regression reproduced the loss. Reconstruction now honors that checkpoint's baseline, including intentionally empty values, before consulting older turns.
- A genuine cold-resume integration reached the new checkpoint but lost Plan mode. The missing `91d54f1667` prerequisite is adapted end-to-end. Saved mode/instructions do not override current model/reasoning or permission settings; only the owning thread's snapshot wins.
- The integration initially used an OpenAI-authenticated test provider by accident. Its gateway fixture now matches existing AIRS test conventions and fails immediately on inference errors.
- An old collaboration test implicitly changed models on restart because its hardcoded model differed from the builder default. It now uses the actual initial model and still asserts exactly one instruction occurrence; a separate regression validates deliberate current-model overrides.

## Boundaries retained

No managed-daemon continuation, voice activation, remote exec-server defaults, direct upstream MCP route, new authentication service, hosted account fallback or sandbox-environment bypass is introduced. Existing Guardian context-mode selection and checkpoint provenance remain intact. The gateway owns inference and upstream MCP OAuth; the new history metadata contains no credential material. Canonical thread metadata is reattached by the storage owner rather than copied from a parent segment.

A signed/notarized npm Mac preview is still a later delivery gate. No package or latest-source full-workspace pass is claimed by this review. The Python SDK remains tied to its separately pinned runtime; this harness change regenerates its authoritative Rust/TypeScript/JSON app-server schema fixtures.


## Rejected prerequisite behavior

A diagnostic adoption of the resume-time checkpoint write from `6515a72db7` failed four existing AIRS lifecycle assertions: quiet resume must not rewrite rollout timestamps or replace persisted cwd with a transient override. That write was removed. The mode response still restores saved Plan instructions and applies current model/effort overrides, while the accepted persisted snapshot remains unchanged until a turn or explicit settings update. Tests assert both effective response values and original durable values. The broader workspace-root migration is not imported merely to satisfy an upstream test expectation.

The connected initial Linux run passed 5,633/5,636; one new reviewed TUI snapshot and two adapted assertions remained. The diagnostic checkpoint write then passed 5,632/5,636 and demonstrated the four incompatibilities above; it is not retained. Native CLI executables built, but native test compilation exhausted disk. The failed run is retained. Only stale changed-package test executable outputs and subsequently idle task incremental caches were reclaimed; all compiled library dependencies, production binaries and user files were retained. Native validation must finish before any score.

## Compatibility closure

The retained production source matches the initial connected run (5,633 passes) byte for byte. Its three remaining failures were two imported fixture assumptions and the first new TUI golden. After adapting those expectations to AIRS and accepting the reviewed golden, all 75 targeted resume/reconnect tests passed in 17.305 seconds. These include the initial three failures and the four lifecycle checks that rejected the temporary persistence write. This is a targeted closure, not a claim that the entire workspace passed. Connected lint, latest-source native validation and full GNU workspace comparison remain required.

The native diagnostic still in flight contains the rejected persistence write. Preserve its results as diagnostics and synchronize the final source before counting native acceptance. Neither that diagnostic nor the existing transport workspace receipt can satisfy this feature's gate.

Scoped lint completed in 6m57s without warnings. Formatting and `git diff --check` passed; 17 unrelated formatter changes were restored. No Rust dependency changed. The patch is committed in two review slices: storage/core reconstruction and mode restoration, followed by app-server/TUI propagation with generated protocol fixtures. Native and full-workspace gates remain pending.
