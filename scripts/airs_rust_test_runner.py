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
for name in ("SSL_CERT_FILE", "SSL_CERT_DIR", "CODEX_CA_CERTIFICATE"):
    child_env.pop(name, None)
os.execve(sys.argv[1], sys.argv[1:], child_env)
