#!/usr/bin/env bash
# Cross-compile a supported Linux musl candidate in a bounded GNU
# container with persistent caches, probe it under QEMU, and stage native and
# npm candidate packages. Nothing here publishes or claims installed acceptance.
set -euo pipefail
: "${GITHUB_RUN_ID:?}" "${RUNNER_TEMP:?}" "${AIRS_LINUX_TARGET:?}"
case "$AIRS_LINUX_TARGET" in
  aarch64-unknown-linux-musl|x86_64-unknown-linux-musl) ;;
  *) echo "Unsupported Linux candidate target" >&2; exit 1 ;;
esac
image='catthehacker/ubuntu@sha256:4f2d5083a9d10d018c1c511eb8665cd480553c11975e78fd903a46daa830768b'
name="airs-musl-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT:-1}"
mkdir -p "$RUNNER_TEMP/airs-musl"
cleanup() {
  status=$?
  docker cp "$name:/tmp/airs-musl/." "$RUNNER_TEMP/airs-musl/" || true
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
  -e CARGO_BUILD_JOBS=4 -e CARGO_INCREMENTAL=0 -e AIRS_LINUX_TARGET="$AIRS_LINUX_TARGET" \
  -e CARGO_TARGET_DIR=/airs-target -e RUNNER_TEMP=/tmp \
  -e NPM_CONFIG_REGISTRY=https://npm.cdot.io \
  "$image" tail -f /dev/null
docker exec "$name" mkdir -p /workspace /tmp/airs-musl
# Copy only this checkout, never the runner registration or host credentials.
tar --exclude=./codex-rs/target --exclude=./node_modules --exclude=./npm/airs-harness/node_modules -cf - . | docker cp - "$name:/workspace"
docker exec -i "$name" bash -s <<'BOOTSTRAP'
set -euo pipefail
apt-get update -qq
apt-get install -y --no-install-recommends build-essential cmake pkg-config curl ca-certificates xz-utils git file python3 npm qemu-user-static
useradd --create-home --uid 10001 airs-ci
chown -R airs-ci:airs-ci /workspace /home/airs-ci /airs-target /airs-tools /tmp/airs-musl
BOOTSTRAP
docker exec -i -w /workspace "$name" runuser -u airs-ci -- bash -s <<'BUILD'
set -euo pipefail
export PATH="$HOME/.cargo/bin:/airs-tools/zig:$PATH"
exec > >(tee /tmp/airs-musl/build.log) 2>&1
target="$AIRS_LINUX_TARGET"
case "$target" in
  aarch64-unknown-linux-musl) emulator=qemu-aarch64-static; suffix=linux-aarch64-musl ;;
  x86_64-unknown-linux-musl) emulator=qemu-x86_64-static; suffix=linux-x86_64-musl ;;
esac
export AIRS_CANDIDATE_EMULATOR="$emulator"
zig_version=0.16.0
zig_sha256=70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00
git rev-parse HEAD > /tmp/airs-musl/source.txt
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
cargo-zigbuild --version
rustc -vV
cd codex-rs
cargo fetch --locked --target "$target"
# aws-lc-sys jitter entropy does not build on musl cross toolchains (upstream release policy).
export AWS_LC_SYS_NO_JITTER_ENTROPY=1 AWS_LC_SYS_NO_JITTER_ENTROPY_aarch64_unknown_linux_musl=1 AWS_LC_SYS_NO_JITTER_ENTROPY_x86_64_unknown_linux_musl=1
export CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
cargo zigbuild --target "$target" --config profile.release.package.codex-cli.opt-level=1 --locked --release -p codex-cli --bin airs-harness -p codex-http-client --bin custom_ca_probe
binary="$CARGO_TARGET_DIR/$target/release/airs-harness"
file "$binary" | tee /tmp/airs-musl/file.txt
"$emulator" "$binary" --version | tee /tmp/airs-musl/version.txt
python3 ../scripts/validate_airs_musl_dns.py --probe "$CARGO_TARGET_DIR/$target/release/custom_ca_probe" \
  --emulator "$emulator" --output /tmp/airs-musl/dns
cargo metadata --locked --filter-platform "$target" --format-version 1 > /tmp/airs-musl/metadata.json
cd /workspace
python3 scripts/package_airs_harness.py --binary "$binary" --metadata /tmp/airs-musl/metadata.json \
  --target "$target" --unvalidated-candidate --emulator "$emulator" --binary-processing none \
  --profile 'release; CLI opt-level=1; lto=false; codegen-units=16; debug=0' \
  --build-command "cargo zigbuild --target $target --config profile.release.package.codex-cli.opt-level=1 --locked --release -p codex-cli --bin airs-harness" \
  --output-directory /tmp/airs-musl/package | tee /tmp/airs-musl/package.json
version="$(python3 -c 'import json;print(json.load(open("npm/airs-harness/package.json"))["version"])')"
tar -xzf "/tmp/airs-musl/package/airs-harness-$version-$suffix.tar.gz" -C /tmp/airs-musl/package
python3 scripts/verify_airs_release.py --directory /tmp/airs-musl/package --emulator "$emulator" --receipt /tmp/airs-musl/native-integrity.json
python3 scripts/package_airs_npm.py --release-directory "/tmp/airs-musl/package/airs-harness-$version-$suffix" \
  --output-directory /tmp/airs-musl/npm --registry https://npm.cdot.io
python3 - <<'RECEIPT'
import json, os
from pathlib import Path
root = Path('/tmp/airs-musl')
receipt = {
    'passed': True,
    'scope': 'Zig cross-compiled Linux musl candidate; QEMU version probe; native and npm candidate packaging',
    'target': os.environ['AIRS_LINUX_TARGET'],
    'installed_acceptance': False,
    'release_ready': False,
    'published': False,
    'source_commit': (root / 'source.txt').read_text().strip(),
    'version_probe': (root / 'version.txt').read_text().strip(),
    'emulated_version_probe': os.environ['AIRS_CANDIDATE_EMULATOR'],
}
(root / 'LINUX-CANDIDATE.json').write_text(json.dumps(receipt, indent=2) + '\n')
RECEIPT
BUILD
