#!/usr/bin/env bash
# Cross-compile the aarch64-unknown-linux-musl candidate in a bounded GNU
# container with persistent caches, probe it under QEMU, and stage native and
# npm candidate packages. Nothing here publishes or claims installed acceptance.
set -euo pipefail
: "${GITHUB_RUN_ID:?}" "${RUNNER_TEMP:?}"
image='catthehacker/ubuntu@sha256:4f2d5083a9d10d018c1c511eb8665cd480553c11975e78fd903a46daa830768b'
name="airs-arm64-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT:-1}"
mkdir -p "$RUNNER_TEMP/airs-arm64"
cleanup() {
  status=$?
  docker cp "$name:/tmp/airs-arm64/." "$RUNNER_TEMP/airs-arm64/" || true
  docker rm -f "$name" >/dev/null || true
  exit "$status"
}
trap cleanup EXIT
docker run -d --name "$name" --cpus=4 --memory=12g \
  --security-opt seccomp=unconfined --security-opt apparmor=unconfined \
  -v airs-arm64-cargo-v1:/home/airs-ci/.cargo \
  -v airs-arm64-rustup-v1:/home/airs-ci/.rustup \
  -v airs-arm64-target-v1:/airs-target \
  -v airs-arm64-tools-v1:/airs-tools \
  -e CARGO_BUILD_JOBS=4 -e CARGO_INCREMENTAL=0 \
  -e CARGO_TARGET_DIR=/airs-target -e RUNNER_TEMP=/tmp \
  -e NPM_CONFIG_REGISTRY=https://npm.cdot.io \
  "$image" tail -f /dev/null
docker exec "$name" mkdir -p /workspace /tmp/airs-arm64
# Copy only this checkout, never the runner registration or host credentials.
tar --exclude=./codex-rs/target --exclude=./node_modules --exclude=./npm/airs-harness/node_modules -cf - . | docker cp - "$name:/workspace"
docker exec -i "$name" bash -s <<'BOOTSTRAP'
set -euo pipefail
apt-get update -qq
apt-get install -y --no-install-recommends build-essential cmake pkg-config curl ca-certificates xz-utils git python3 npm qemu-user-static
useradd --create-home --uid 10001 airs-ci
chown -R airs-ci:airs-ci /workspace /home/airs-ci /airs-target /airs-tools /tmp/airs-arm64
BOOTSTRAP
docker exec -i -w /workspace "$name" runuser -u airs-ci -- bash -s <<'BUILD'
set -euo pipefail
export PATH="$HOME/.cargo/bin:/airs-tools/zig:$PATH"
exec > >(tee /tmp/airs-arm64/build.log) 2>&1
target=aarch64-unknown-linux-musl
zig_version=0.16.0
zig_sha256=70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00
git rev-parse HEAD > /tmp/airs-arm64/source.txt
test -z "$(git status --porcelain)"
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error https://sh.rustup.rs -o /tmp/airs-rustup-init.sh
  sh /tmp/airs-rustup-init.sh -y --profile minimal --default-toolchain none
fi
rustup toolchain install 1.95.0 --profile minimal --target "$target"
rustup default 1.95.0
if [ ! -x /airs-tools/zig/zig ] || [ "$(/airs-tools/zig/zig version)" != "$zig_version" ]; then
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error \
    "https://ziglang.org/download/${zig_version}/zig-x86_64-linux-${zig_version}.tar.xz" -o /tmp/zig.tar.xz
  echo "${zig_sha256}  /tmp/zig.tar.xz" | sha256sum -c -
  rm -rf /airs-tools/zig && mkdir -p /airs-tools/zig
  tar -xJf /tmp/zig.tar.xz -C /airs-tools/zig --strip-components=1
fi
zig version
cargo install --locked --version 0.23.4 cargo-zigbuild
cargo zigbuild --version
rustc -vV
cd codex-rs
cargo fetch --locked --target "$target"
# aws-lc-sys jitter entropy does not build on musl cross toolchains (upstream release policy).
export AWS_LC_SYS_NO_JITTER_ENTROPY=1 AWS_LC_SYS_NO_JITTER_ENTROPY_aarch64_unknown_linux_musl=1
export CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
cargo zigbuild --target "$target" --config profile.release.package.codex-cli.opt-level=1 --locked --release -p codex-cli --bin airs-harness
binary="$CARGO_TARGET_DIR/$target/release/airs-harness"
file "$binary" | tee /tmp/airs-arm64/file.txt
qemu-aarch64-static "$binary" --version | tee /tmp/airs-arm64/version.txt
cargo metadata --locked --filter-platform "$target" --format-version 1 > /tmp/airs-arm64/metadata.json
cd /workspace
python3 scripts/package_airs_harness.py --binary "$binary" --metadata /tmp/airs-arm64/metadata.json \
  --target "$target" --unvalidated-candidate --emulator qemu-aarch64-static --binary-processing none \
  --profile 'release; CLI opt-level=1; lto=false; codegen-units=16; debug=0' \
  --build-command "cargo zigbuild --target $target --config profile.release.package.codex-cli.opt-level=1 --locked --release -p codex-cli --bin airs-harness" \
  --output-directory /tmp/airs-arm64/package | tee /tmp/airs-arm64/package.json
version="$(python3 -c 'import json;print(json.load(open("npm/airs-harness/package.json"))["version"])')"
tar -xzf "/tmp/airs-arm64/package/airs-harness-$version-linux-aarch64-musl.tar.gz" -C /tmp/airs-arm64/package
python3 scripts/verify_airs_release.py --directory /tmp/airs-arm64/package --emulator qemu-aarch64-static --receipt /tmp/airs-arm64/native-integrity.json
python3 scripts/package_airs_npm.py --release-directory "/tmp/airs-arm64/package/airs-harness-$version-linux-aarch64-musl" \
  --output-directory /tmp/airs-arm64/npm --registry https://npm.cdot.io
python3 - <<'RECEIPT'
import json
from pathlib import Path
root = Path('/tmp/airs-arm64')
receipt = {
    'passed': True,
    'scope': 'Zig cross-compiled aarch64-unknown-linux-musl candidate; QEMU version probe; native and npm candidate packaging',
    'installed_acceptance': False,
    'release_ready': False,
    'published': False,
    'source_commit': (root / 'source.txt').read_text().strip(),
    'version_probe': (root / 'version.txt').read_text().strip(),
    'emulated_version_probe': 'qemu-aarch64-static',
}
(root / 'ARM64-CANDIDATE.json').write_text(json.dumps(receipt, indent=2) + '\n')
RECEIPT
BUILD
