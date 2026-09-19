#!/usr/bin/env bash
set -euo pipefail
: "${RUNNER_TEMP:?}"
# Cursor import fixtures exercise one punctuated project component; the temporary
# ancestor must not add another ambiguous encoded path component.
export TMPDIR="$RUNNER_TEMP/airstests"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="$PWD/scripts/airs_rust_test_runner.py"
unset NO_COLOR SSL_CERT_FILE SSL_CERT_DIR CODEX_CA_CERTIFICATE
exec > >(tee "$RUNNER_TEMP/airs-evidence/validation.log") 2>&1
git rev-parse HEAD > "$RUNNER_TEMP/airs-evidence/source.txt"
uname -a
ldd --version
node --test npm/airs-harness/*.test.js
npm ci --prefix npm/airs-harness --ignore-scripts --no-audit --no-fund
python3 scripts/validate_prisma_cli.py
for pattern in 'test_airs_bundle*.py' test_airs_npm_registry.py test_package_airs_harness.py test_airs_review_release.py test_airs_signed_macos_artifact.py 'test_plan_airs_review*.py' test_publish_airs_review.py test_airs_package_access.py test_restore_airs_review_stage.py test_validate_airs_registry_install.py test_format.py test_evaluate_airs_upstream.py test_live_gateway_evidence.py 'test_airs_test_release*.py' test_promote_airs_stable.py test_validate_airs_test_registry_install.py; do
  python3 -m unittest discover -s scripts -p "$pattern" -v
done
CODEX_REPO_ROOT="$PWD" PYTHONPATH=scripts python3 - <<'PY' > "$RUNNER_TEMP/airs-v8.env"
import shlex
from codex_package.targets import TARGET_SPECS
from codex_package.v8 import resolve_codex_v8_cargo_env
for key, value in resolve_codex_v8_cargo_env(TARGET_SPECS['x86_64-unknown-linux-gnu']).items():
    print(f'export {key}={shlex.quote(value)}')
PY
source "$RUNNER_TEMP/airs-v8.env"
(cd codex-rs && cargo build --locked -p codex-cli -p codex-linux-sandbox -p codex-rmcp-client -p codex-code-mode-host --bins)
python3 scripts/airs_forgejo_native_deps.py
source "$RUNNER_TEMP/airs-native.env"
just test --locked --features codex-v8-poc/sandbox
