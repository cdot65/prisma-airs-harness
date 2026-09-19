#!/usr/bin/env bash
set -euo pipefail
: "${GITHUB_RUN_ID:?}" "${RUNNER_TEMP:?}"
image='catthehacker/ubuntu@sha256:4f2d5083a9d10d018c1c511eb8665cd480553c11975e78fd903a46daa830768b'
name="airs-gnu-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT:-1}"
mkdir -p "$RUNNER_TEMP/airs-evidence"
cleanup() {
  status=$?
  docker cp "$name:/tmp/airs-evidence/." "$RUNNER_TEMP/airs-evidence/" || true
  docker rm -f "$name" >/dev/null || true
  exit "$status"
}
trap cleanup EXIT
docker run -d --name "$name" --cpus=2 --memory=8g \
  --security-opt seccomp=unconfined --security-opt apparmor=unconfined \
  -v airs-gnu-cargo-v1:/home/airs-ci/.cargo \
  -v airs-gnu-rustup-v1:/home/airs-ci/.rustup \
  -v airs-gnu-target-v1:/airs-target \
  -v airs-gnu-native-v1:/airs-native \
  -e CARGO_BUILD_JOBS=2 -e CARGO_INCREMENTAL=0 \
  -e CARGO_PROFILE_DEV_DEBUG=0 -e CARGO_PROFILE_TEST_DEBUG=0 \
  -e CARGO_TARGET_DIR=/airs-target -e RUNNER_TEMP=/tmp \
  -e TERM=xterm-256color -e COLORTERM=truecolor \
  -e NPM_CONFIG_REGISTRY=https://npm.cdot.io \
  "$image" tail -f /dev/null
docker exec "$name" mkdir -p /workspace
# Copy only this checkout, never the runner registration or host credentials.
tar --exclude=./codex-rs/target --exclude=./node_modules --exclude=./npm/airs-harness/node_modules -cf - . | docker cp - "$name:/workspace"
docker exec -i "$name" bash -s <<'BOOTSTRAP'
set -euo pipefail
apt-get update -qq
apt-get install -y --no-install-recommends build-essential clang libssl-dev libglib2.0-dev libsecret-1-dev pkg-config cmake libasound2-dev libcap-dev ripgrep bubblewrap zsh dbus-x11 gnome-keyring tini
useradd --create-home --uid 10001 airs-ci
mkdir -p /tmp/airs-evidence /tmp/airstests
chown -R airs-ci:airs-ci /workspace /home/airs-ci /airs-target /airs-native /tmp/airs-evidence /tmp/airstests
runuser -u airs-ci -- unshare -Ur true
BOOTSTRAP
# A subreaper must own the test tree: PID 1 is an idle container command.
# Otherwise terminated grandchildren remain zombies and kill(pid, 0) sees them.
docker exec -i -w /workspace "$name" /usr/bin/tini -s -- runuser -u airs-ci -- bash -s <<'VALIDATE'
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error https://sh.rustup.rs -o /tmp/airs-rustup-init.sh
  sh /tmp/airs-rustup-init.sh -y --profile minimal --default-toolchain none
fi
rustup toolchain install 1.95.0 --profile minimal
rustup default 1.95.0
cargo install --locked --version 1.51.0 just
cargo install --locked --version 0.9.143 cargo-nextest
bash scripts/airs_forgejo_linux.sh
VALIDATE
