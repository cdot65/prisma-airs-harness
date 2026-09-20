# First-use reliability after 0.1.1

Published 0.1.1 reproduced the misleading missing-native guidance in a disposable
local install with optional dependencies omitted. Global omission did not reproduce
it; that observation and npm implementation evidence are retained separately.

The updated source launcher, copied into that isolated published installation,
reported the pinned release and scope/registry-preserving repair. Reinstalling the
same version with optional dependencies restored airs0.1.1 and bundledCLI7.0.0.
This is a source-launcher check, not acceptance of a newly published package.

All31 Node launcher/package tests passed. The actual packaged 0.1.1 Ubuntu helper
passed six isolated encrypted-keyring/readiness cases with no version override.
No owner login, configuration, native credential record or gateway was changed.
Formatting passed;19 unrelated baseline formatting changes were restored.
Integrated native publication remains the later release focus.
