#!/bin/bash
# Reclaim disk on the Apple Silicon Forgejo runner without touching release evidence.
#
#   scripts/airs_mac_runner_cleanup.sh            # dry run: report what would be removed
#   scripts/airs_mac_runner_cleanup.sh --apply    # remove it
#   scripts/airs_mac_runner_cleanup.sh --install  # run --apply daily at 04:30 via launchd
#
# Removed, only when no job is running:
#   - bulky files in ~/.cache/airs-* scratch folders older than KEEP_DAYS (default 14);
#     small .json/.md/.log/.txt evidence is kept
#   - ~/.cache/airs-* archives (.tar, .tar.gz, .tgz, .zip) older than KEEP_DAYS
#   - runner job work directories older than 2 days
#   - runner cache blobs unused for CACHE_DAYS (default 21); a pruned entry is a cache miss
# Written for macOS /bin/bash 3.2.
set -euo pipefail

KEEP_DAYS="${KEEP_DAYS:-14}"
CACHE_DAYS="${CACHE_DAYS:-21}"
RUNNER_HOME="${RUNNER_HOME:-$HOME/.local/share/airs-forgejo-runner}"
CONFIG="$RUNNER_HOME/config.yaml"
LABEL="io.cdot.airs-runner-cleanup"
mode="dry-run"

case "${1:-}" in
  "") ;;
  --apply) mode="apply" ;;
  --install) mode="install" ;;
  *) echo "usage: $0 [--apply|--install]" >&2; exit 2 ;;
esac

free_gib() {
  df -k "$HOME" | awk 'NR==2 { printf "%.1f", $4 / 1048576 }'
}

if [ "$mode" = "install" ]; then
  installed="$RUNNER_HOME/airs-runner-cleanup.sh"
  plist="$HOME/Library/LaunchAgents/$LABEL.plist"
  mkdir -p "$RUNNER_HOME" "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"
  cp "$0" "$installed"
  chmod 700 "$installed"
  cat > "$plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>$LABEL</string>
  <key>ProgramArguments</key>
  <array><string>/bin/bash</string><string>$installed</string><string>--apply</string></array>
  <key>StartCalendarInterval</key>
  <dict><key>Hour</key><integer>4</integer><key>Minute</key><integer>30</integer></dict>
  <key>StandardOutPath</key><string>$HOME/Library/Logs/airs-runner-cleanup.log</string>
  <key>StandardErrorPath</key><string>$HOME/Library/Logs/airs-runner-cleanup.log</string>
</dict>
</plist>
PLIST
  launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
  launchctl bootstrap "gui/$(id -u)" "$plist"
  echo "Installed $LABEL: daily at 04:30, log ~/Library/Logs/airs-runner-cleanup.log"
  echo "Run now with: /bin/bash $installed --apply"
  exit 0
fi

# Read a two-level key such as "cache: dir:" from the runner's YAML configuration.
config_value() {
  [ -f "$CONFIG" ] || return 0
  awk -v section="$1:" -v key="$2:" '
    /^[^ #]/ { inside = ($1 == section) }
    inside && $1 == key { $1 = ""; sub(/^ +/, ""); gsub(/["\047]/, ""); print; exit }
  ' "$CONFIG"
}

runner_busy() {
  local pid
  for pid in $(pgrep -f 'forgejo-runner daemon' || true); do
    if pgrep -P "$pid" >/dev/null 2>&1; then
      return 0
    fi
  done
  return 1
}

remove() {
  if [ "$mode" = "apply" ]; then
    rm -rf -- "$@"
  else
    printf 'would remove %s\n' "$@"
  fi
}

echo "$(date '+%Y-%m-%d %H:%M:%S') $mode: $(free_gib) GiB free"
if runner_busy; then
  echo "A runner job is in progress; nothing removed."
  exit 0
fi

# 1. Dated scratch folders and archives from earlier release runs.
for entry in "$HOME"/.cache/airs-*; do
  [ -e "$entry" ] || continue
  if [ -z "$(find "$entry" -maxdepth 0 -mtime +"$KEEP_DAYS" 2>/dev/null || true)" ]; then
    continue
  fi
  if [ -f "$entry" ]; then
    case "$entry" in
      *.tar|*.tar.gz|*.tgz|*.zip) remove "$entry" ;;
    esac
    continue
  fi
  find "$entry" -type d \( -name target -o -name node_modules -o -name _cacache \) -prune -print 2>/dev/null |
    while IFS= read -r path; do remove "$path"; done || true
  find "$entry" -type f -size +10240 ! -name '*.json' ! -name '*.md' ! -name '*.log' ! -name '*.txt' -print 2>/dev/null |
    while IFS= read -r path; do remove "$path"; done || true
done

# 2. Job work directories left behind by host-mode jobs.
workdir="$(config_value host workdir_parent)"
workdir="${workdir:-$HOME/.cache/act}"
workdir="${workdir/#\~/$HOME}"
if [ -d "$workdir" ]; then
  find "$workdir" -mindepth 1 -maxdepth 1 -mtime +2 -print 2>/dev/null |
    while IFS= read -r path; do remove "$path"; done || true
fi

# 3. Action cache blobs unused for CACHE_DAYS. The index is left in place; a pruned
#    entry restores as a miss and the next successful build saves a fresh one.
cache="$(config_value cache dir)"
cache="${cache:-$HOME/.cache/actcache}"
cache="${cache/#\~/$HOME}"
if [ -d "$cache" ]; then
  find "$cache" -type f -atime +"$CACHE_DAYS" -mtime +"$CACHE_DAYS" ! -name '*.db' -print 2>/dev/null |
    while IFS= read -r path; do remove "$path"; done || true
fi

[ "$mode" = "apply" ] && find "$HOME"/.cache/airs-* -type d -empty -delete 2>/dev/null || true
echo "$(date '+%Y-%m-%d %H:%M:%S') $mode done: $(free_gib) GiB free"
