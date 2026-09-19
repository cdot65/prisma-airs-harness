#!/usr/bin/env bash
# Prepare an Ubuntu SSH test host. Run as the ordinary login user, not with sudo.
# Usage: bash prepare_airs_ubuntu.sh [--check | --unlock]
# Override AIRS_TEST_VERSION to install another already-published exact version.
set +x
set +a
unset keyring_password keyring_confirmation
set -euo pipefail
umask 077
mode=${1:-install}
case "$mode" in install|--check|--unlock) ;; *) echo 'Usage: bash prepare_airs_ubuntu.sh [--check | --unlock]' >&2; exit 2;; esac
[[ $# -le 1 ]] || exit 2
[[ $EUID -ne 0 ]] || { echo 'Run as your SSH login user; the script calls sudo for apt only.' >&2; exit 2; }
# shellcheck source=/dev/null
source /etc/os-release
[[ ${ID:-} == ubuntu && $(uname -m) == x86_64 ]] || { echo 'This preparation script targets Ubuntu x86_64.' >&2; exit 2; }
version=${AIRS_TEST_VERSION:-0.1.1}
[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$ ]] || { echo 'Use an exact published version, not an npm tag.' >&2; exit 2; }
registry=https://npm.cdot.io
state=$HOME/.local/state/airs-test-host
prefix=$HOME/.local/share/airs-test-host/npm
config=$HOME/.config/airs-test-host
mkdir -p "$state" "$config"
report=$state/readiness-$(date -u +%Y%m%dT%H%M%S)-$$.txt
exec > >(tee "$report") 2>&1
failures=0
pass() { printf 'PASS %s\n' "$*"; }
fail() { printf 'FAIL %s\n' "$*"; failures=$((failures + 1)); }
work=$(mktemp -d "$state/work.XXXXXX")
probe_id=
cleanup() {
    if [[ -n $probe_id ]] && command -v secret-tool >/dev/null; then
        timeout 10 secret-tool clear application airs-host-readiness probe-id "$probe_id" >/dev/null 2>&1 || true
    fi
    rm -rf -- "$work"
}
trap cleanup EXIT
printf 'AIRS host preparation: %s / %s / user %s\n' "$PRETTY_NAME" "$(uname -m)" "$(id -un)"
printf 'Requested package: airs-harness@%s\nReport: %s\n' "$version" "$report"
if [[ $mode == install ]]; then
    sudo -v
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends \
        ca-certificates curl git ripgrep jq nodejs npm bubblewrap \
        dbus-user-session dbus-bin libglib2.0-bin gnome-keyring libsecret-tools \
        python3 python3-venv openssl xdg-utils
fi
for tool in node npm curl git rg jq bwrap gdbus gnome-keyring-daemon secret-tool timeout python3; do
    command -v "$tool" >/dev/null || fail "Missing $tool; run the script without --check to install prerequisites."
done
if (( failures )); then exit 1; fi
if node -e 'const [a,b]=process.versions.node.split(".").map(Number);process.exit((a===22&&b>=13)||(a===23&&b>=5)||a>=24?0:1)'; then
    pass "Node $(node --version), npm $(npm --version) satisfy the harness engine requirement."
else
    fail "Node $(node --version) is unsupported. Use Node 22.13+ in 22.x, or 23.5+. Ubuntu 26.04 supplies a compatible nodejs package."
    exit 1
fi
# Keep npm installs in this user's dedicated prefix; preserve existing npm configuration.
if [[ $mode == install ]]; then
    mkdir -p "$prefix"
    npm install --global --prefix "$prefix" --include=optional --engine-strict \
        --registry="$registry" "airs-harness@$version"
    cat > "$config/env.sh" <<'ENV'
# AIRS test-host tools and the existing per-user D-Bus session.
export PATH="$HOME/.local/share/airs-test-host/npm/bin:$PATH"
if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ] && [ -S "/run/user/$(id -u)/bus" ]; then
    export DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$(id -u)/bus"
fi
ENV
    # Bash login shells read the first of these files that exists.
    startup=$HOME/.profile
    if [[ -e $HOME/.bash_profile ]]; then startup=$HOME/.bash_profile
    elif [[ -e $HOME/.bash_login ]]; then startup=$HOME/.bash_login; fi
    line='[ ! -r "$HOME/.config/airs-test-host/env.sh" ] || . "$HOME/.config/airs-test-host/env.sh"'
    if ! grep -Fqx "$line" "$startup" 2>/dev/null; then printf '\n%s\n' "$line" >> "$startup"; fi
fi
export PATH="$prefix/bin:$PATH"
if [[ -z ${DBUS_SESSION_BUS_ADDRESS:-} && -S /run/user/$UID/bus ]]; then
    export DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$UID/bus"
fi
if [[ ! -x $prefix/bin/airs ]]; then fail 'The dedicated airs installation is missing.'
else
    if "$prefix/bin/airs" --version; then pass 'Installed airs launches.'; else fail 'Installed airs failed to launch.'; fi
    if node -e 'const p=require(process.argv[1]);process.exit(p.version===process.argv[2]?0:1)' "$prefix/lib/node_modules/airs-harness/package.json" "$version"; then
        pass "Installed package matches $version."
    else fail 'Installed package version does not match the requested version.'; fi
    if "$prefix/bin/airs" cli --version; then pass 'Bundled product CLI launches.'; else fail 'Bundled product CLI failed.'; fi
    # A disposable environment avoids the owner's saved permission/profile settings.
    mkdir -p "$work/airs" "$work/core"
    printf 'sandbox-readiness-marker' > "$work/canary"
    if (
        export AIRS_HARNESS_HOME="$work/airs" CODEX_HOME="$work/core"
        unset AIRS_READINESS_UNUSED_CREDENTIAL
        timeout 20 "$prefix/bin/airs" env create readiness \
            --gateway-url https://gateway.redtail.cdot.io/v1 \
            --credential-env AIRS_READINESS_UNUSED_CREDENTIAL &&
        timeout 20 "$prefix/bin/airs" sandbox -c 'sandbox_mode="read-only"' -- \
            /bin/sh -c 'test "$(cat "$1")" = sandbox-readiness-marker && ! (printf forbidden >> "$1")' \
            airs-readiness "$work/canary"
    ) > "$work/sandbox.log" 2>&1 && [[ $(cat "$work/canary") == sandbox-readiness-marker ]]; then
        pass 'Harness sandbox reads an allowed file and denies modification.' 
    else
        fail 'The harness sandbox could not start; preserve AppArmor/user-namespace policy for diagnosis.'
        cat "$work/sandbox.log"
    fi
fi
if curl --proto '=https' --tlsv1.2 -fsS --connect-timeout 10 --max-time 30 \
    "$registry/airs-harness/$version" -o "$work/package.json" && \
    jq -e --arg version "$version" '.version == $version' "$work/package.json" >/dev/null; then
    pass 'The exact npm package is reachable over verified HTTPS.'
else fail 'Registry TLS/connectivity or package availability failed.'; fi
if code=$(curl --proto '=https' --tlsv1.2 -sS -o /dev/null -w '%{http_code}' \
    --connect-timeout 10 --max-time 20 https://gateway.redtail.cdot.io/v1); then
    case "$code" in 2??|3??|400|401|403|404|405) pass "Gateway TLS/connectivity works (HTTP $code; authentication is not tested).";;
        *) fail "Gateway returned HTTP $code; investigate before login.";; esac
else fail 'Gateway DNS/TLS/connectivity failed.'; fi

# Probe only a unique disposable record, never enumerate/read existing credentials.
if ! timeout 10 gdbus call --session --dest org.freedesktop.DBus \
    --object-path /org/freedesktop/DBus --method org.freedesktop.DBus.GetId >/dev/null 2>&1; then
    fail 'No usable user D-Bus session. Log out and SSH back in after package installation, then rerun.'
else
    pass 'User D-Bus session is reachable.'
    # Read collection metadata only; D-Bus may activate the user's Secret Service.
    read_collection_state() {
        collection=$(timeout 10 gdbus call --session --dest org.freedesktop.secrets \
            --object-path /org/freedesktop/secrets --method org.freedesktop.Secret.Service.ReadAlias default 2>/dev/null || true)
        collection=$(printf '%s' "$collection" | sed -n "s/.*objectpath '\([^']*\)'.*/\1/p")
        unlocked=false
        if [[ $collection == /org/freedesktop/secrets/collection/* ]]; then
            locked=$(timeout 10 gdbus call --session --dest org.freedesktop.secrets \
                --object-path "$collection" --method org.freedesktop.DBus.Properties.Get \
                org.freedesktop.Secret.Collection Locked 2>/dev/null || true)
            [[ $locked != *false* ]] || unlocked=true
        fi
    }
    read_collection_state
    if [[ $unlocked == false && $mode != --check ]]; then
        printf '\nUnlock the existing login keyring, or choose a nonempty password for a new one.\n'
        printf 'This is a local keyring password, not your SSO password or API key.\n'
        if [[ -t 0 ]]; then
            IFS= read -r -s -p 'Keyring password (hidden): ' keyring_password
            printf '\n'
            if [[ -n $keyring_password ]]; then
                if [[ ! -e ${XDG_DATA_HOME:-$HOME/.local/share}/keyrings/login.keyring ]]; then
                    IFS= read -r -s -p 'Confirm new keyring password: ' keyring_confirmation
                    printf '\n'
                    [[ $keyring_confirmation == "$keyring_password" ]] || { unset keyring_password keyring_confirmation; echo 'Passwords did not match.'; exit 1; }
                    unset keyring_confirmation
                fi
                # --unlock alone starts a competing daemon after D-Bus activation.
                # Replace the locked daemon so Secret Service clients reach the
                # unlocked instance. Password stays on stdin, never in logs/argv.
                printf 'Restarting the locked keyring service to apply the unlock.\n'
                if ! printf '%s' "$keyring_password" | timeout 20 gnome-keyring-daemon \
                    --replace --unlock --daemonize --components=secrets >/dev/null 2>&1; then
                    fail 'The keyring daemon could not restart and unlock.'
                fi
            fi
            unset keyring_password
        else printf 'Use an interactive SSH terminal (ssh -t) to unlock the keyring.\n'; fi
        # Daemon startup can finish after its launcher returns. Verify the
        # collection through D-Bus rather than trusting the launcher exit code.
        for attempt in {1..10}; do
            read_collection_state
            [[ $unlocked == false ]] || break
            sleep 0.2
        done
    fi
    if [[ $unlocked == false ]]; then
        if [[ $mode == --check ]]; then
            fail 'Credential store is locked/unavailable. Run this script with --unlock in your SSH terminal.'
        else
            fail 'The default keyring is still locked/unavailable. Use the password originally chosen for the login keyring; a different password cannot unlock it.'
            printf 'If that password was accepted, inspect the user keyring service and default collection; do not reset or delete the keyring.\n'
        fi
    else
        probe_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
        probe_value="airs-disposable-readiness-$probe_id"
        if printf '%s' "$probe_value" | timeout 15 secret-tool store --label='AIRS disposable host readiness check' \
            application airs-host-readiness probe-id "$probe_id" >/dev/null 2>&1; then
            retrieved=$(timeout 10 secret-tool lookup application airs-host-readiness probe-id "$probe_id" 2>/dev/null || true)
            if [[ $retrieved == "$probe_value" ]]; then pass 'Native credential store write/read passed.'
            else fail 'Native credential store readback did not match.'; fi
            unset retrieved
            if timeout 10 secret-tool clear application airs-host-readiness probe-id "$probe_id" >/dev/null 2>&1; then
                probe_id=
                pass 'Disposable credential removed.'
            else fail 'Disposable credential cleanup failed; this run is not ready.'; fi
        else fail 'The default keyring is unlocked, but the disposable credential write failed.'; fi
    fi
fi
printf '\nReport: %s\n' "$report"
if (( failures )); then printf 'NOT READY: %s check(s) failed. Authentication has not been attempted.\n' "$failures"; exit 1; fi
printf '\nREADY for attended AIRS testing. Production SSO, API-key access and MCP login remain to be tested.\n'
printf 'In this terminal run: source ~/.config/airs-test-host/env.sh\n'
printf 'Then: command airs env create\n'
printf 'After login: command airs doctor --verify-access\n'
printf 'Inside airs, use /mcp for gateway MCP connections.\n'
printf 'After reboot or keyring lock, rerun: bash %q --unlock\n' "$0"
