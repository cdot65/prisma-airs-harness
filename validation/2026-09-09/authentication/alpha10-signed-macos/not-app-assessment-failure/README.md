# Raw CLI notarization assessment correction

The second signed acceptance run downloaded the exact asset and passed thin
ARM64, strict signature and explicit Apple Developer ID/team checks. `spctl`
then rejected the executable because it is not an application bundle. No CLI
execution or acceptance followed; the original evidence ZIP and log hash remain
in this directory.

Apple's [WWDC2019 session703](https://developer.apple.com/videos/play/wwdc2019/703/)
documents code-signing verification with an explicit `notarized` requirement for
non-app code. The repaired verifier requires that native command to exit
successfully, retaining its output without depending on version-specific
success wording. The separate Apple/team and hardened-runtime gates remain.
The receipt names this method and explicitly says app assessment is not
applicable to this raw CLI. An unknown or failed notarization requirement still
prevents a signing receipt and all downstream execution.

Five local tests pass, including failing and silent-success notarization
commands. Actual Apple verification is pending a replacement hosted run at this
checkpoint; this correction does not assert the binary is already validated.
