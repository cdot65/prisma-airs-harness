#!/usr/bin/env bash
set -euo pipefail
: "${GITHUB_RUN_ID:?}" "${RUNNER_TEMP:?}"
image='catthehacker/ubuntu@sha256:4f2d5083a9d10d018c1c511eb8665cd480553c11975e78fd903a46daa830768b'
name="airs-package-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT:-1}"
mkdir -p "$RUNNER_TEMP/linux-evidence"
cleanup() {
  status=$?
  docker cp "$name:/tmp/linux-evidence/." "$RUNNER_TEMP/linux-evidence/" || true
  docker rm -f "$name" >/dev/null || true
  exit "$status"
}
trap cleanup EXIT
docker run -d --name "$name" --cpus=2 --memory=4g \
  --security-opt seccomp=unconfined --security-opt apparmor=unconfined \
  "$image" tail -f /dev/null
docker exec "$name" mkdir -p /workspace /tmp/signed-npm /tmp/linux-evidence
# Copy only acceptance scripts and the exact package artifact from this run.
docker cp scripts "$name:/workspace/"
docker cp "$RUNNER_TEMP/signed-npm/." "$name:/tmp/signed-npm/"
docker exec -i "$name" bash -s <<'BOOTSTRAP'
set -euo pipefail
apt-get update -qq
apt-get install -y --no-install-recommends ripgrep bubblewrap zsh
useradd --create-home --uid 10001 airs-ci
chown -R airs-ci:airs-ci /workspace /tmp/signed-npm /tmp/linux-evidence
runuser -u airs-ci -- unshare -Ur true
BOOTSTRAP
docker exec -i -w /workspace "$name" runuser -u airs-ci -- bash -s <<'VALIDATE'
set -euo pipefail
prefix='/home/airs-ci/Signed Candidate Install'
python3 scripts/validate_airs_npm.py --packages /tmp/signed-npm --prefix "$prefix"
cp "$prefix/INSTALL-VERIFICATION.json" "$prefix/INSTALL-NETWORK.json" /tmp/linux-evidence/
python3 scripts/validate_prisma_cli.py --launcher "$prefix/lib/node_modules/airs-harness/bin/airs-harness.js" --receipt /tmp/linux-evidence/managed-prisma-cli.json
AIRS_MANAGED_CLI_ACCEPTANCE=1 AIRS_HARNESS_BIN="$prefix/bin/airs-harness" python3 -m unittest discover -s scripts -p test_airs_harness.py -v > /tmp/linux-evidence/installed-tests.log 2>&1
VALIDATE
