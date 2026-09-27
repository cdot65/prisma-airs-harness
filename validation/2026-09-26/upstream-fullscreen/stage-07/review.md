# Image-only user history — fullscreen prerequisite

Adapted the applicable user-image portion of `321c50fc2c`. Local and remote image-only messages retain a visible numbered attachment label even when text is empty. Existing composer placeholders suppress duplicate labels; raw output uses the same missing-label selection. No local paths, image payloads or remote URLs are added to history text.

Preserved AIRS user styling, URL annotation, forced URL wrapping, source metadata and right-side padding. The upstream voice/prompt-style refactor is not needed. The test merge keeps the prior repeated-line/URL source regression alongside the new local-only and mixed-attachment snapshots. Added explicit raw output coverage for an image-only message.

History/replay subset:214/214 passed, zero retries. New snapshots inspected; existing snapshots pass unchanged. This is a prerequisite, not a standalone feature score or native release claim.

Scoped config/core/TUI lint passed without warnings or fixes; formatting passed with unrelated Python churn restored.
