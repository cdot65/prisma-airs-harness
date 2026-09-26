#!/usr/bin/env python3
"""Run isolated Rust test binaries without inherited enterprise CA overrides.

Cargo/nextest startup can repopulate SSL_CERT_FILE after the parent shell clears
it. Clear these values immediately before the test process starts so native-TLS
fixtures exercise the intended backend. Custom-CA tests still set their own
fixture values inside the test. This runner is never used by product commands.
"""

import os
import sys

child_env = dict(os.environ)
# The wrapper belongs to nextest, not helper processes spawned by a test.
# assert_cmd otherwise returns this wrapper as the helper executable, and
# callers retaining only get_program() lose the actual binary argument.
child_env.pop("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER", None)
child_env.pop("CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUNNER", None)
child_env.pop("CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER", None)
for name in ("SSL_CERT_FILE", "SSL_CERT_DIR", "CODEX_CA_CERTIFICATE"):
    child_env.pop(name, None)
os.execve(sys.argv[1], sys.argv[1:], child_env)
