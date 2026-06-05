#!/usr/bin/env bash
# Set up the Caffeine simulator prerequisites for this repo:
#   1. a Caffeine checkout (a `caffeine/` sibling of the repo, or $CAFFEINE_DIR)
#   2. a JDK 25 (Caffeine's Gradle toolchain floor)
#
# Idempotent and safe to re-run, on a laptop or in a fresh CI/web container.
#
# Logs go to stderr; on success the resolved JDK 25 home is printed to stdout as
# the final line, so a caller (e.g. the SessionStart hook) can capture it for
# JAVA_HOME:   jdk_home="$(scripts/setup-caffeine.sh)"
#
# JDK install policy: an existing JDK 25 is always reused. If none is found the
# script installs openjdk-25 only when it can do so non-interactively — in a
# remote container ($CLAUDE_CODE_REMOTE=true) or when called with --install-jdk.
# Otherwise it prints how to install one and exits non-zero, so a local machine
# is never modified behind your back.
set -euo pipefail

log() { printf '%s\n' "$*" >&2; }

install_jdk=0
[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] && install_jdk=1
for arg in "$@"; do
  case "$arg" in
    --install-jdk) install_jdk=1 ;;
    -h|--help) log "usage: setup-caffeine.sh [--install-jdk]"; exit 0 ;;
    *) log "unknown argument: $arg"; exit 2 ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 1. Caffeine checkout — a sibling of the repo, or $CAFFEINE_DIR. Matches the
#    resolution in scripts/caffeine_runner.py.
#
# An existing checkout is advanced to origin/master, not left as-is: a reused
# warm container keeps whatever caffeine it was provisioned with, which can lag
# behind a freshly-merged subject's simulator policy. Without this, a session
# refreshes cache-evolve but runs against a stale caffeine — the new policy is
# unregistered, caffeine emits no CSV, and every variant fails fitness. Fetch +
# hard-reset (not pull --ff-only) because the clone is shallow and the checkout
# is a disposable build dependency.
caffeine_dir="${CAFFEINE_DIR:-$(dirname "$repo_root")/caffeine}"
if [ -d "$caffeine_dir/.git" ]; then
  log "caffeine: updating $caffeine_dir to origin/master"
  git -C "$caffeine_dir" fetch --depth 1 origin master >&2
  git -C "$caffeine_dir" reset --hard FETCH_HEAD >&2
else
  log "caffeine: cloning into $caffeine_dir"
  git clone --depth 1 https://github.com/yanivholder2/caffeine.git "$caffeine_dir" >&2
fi

# 2. JDK 25 — reuse any existing install before considering a new one.
find_jdk25() {
  # Honour an explicit JAVA_HOME if it is a 25 JDK.
  if [ -n "${JAVA_HOME:-}" ] && [ -x "${JAVA_HOME}/bin/javac" ] \
     && "${JAVA_HOME}/bin/javac" -version 2>&1 | grep -q ' 25'; then
    printf '%s\n' "$JAVA_HOME"; return 0
  fi
  # Any java-25 toolchain in a standard location.
  local d
  for d in /usr/lib/jvm/java-25-openjdk-* /usr/lib/jvm/*-25-openjdk-* \
           /Library/Java/JavaVirtualMachines/*-25*/Contents/Home; do
    [ -x "$d/bin/javac" ] && { printf '%s\n' "$d"; return 0; }
  done
  return 1
}

if jdk_home="$(find_jdk25)"; then
  log "jdk 25: found at $jdk_home"
elif [ "$install_jdk" = "1" ]; then
  log "jdk 25: not found — installing openjdk-25-jdk-headless"
  sudo apt-get update -qq >&2 || true
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y openjdk-25-jdk-headless >&2
  jdk_home="$(find_jdk25)" || { log "jdk 25: install did not yield a usable toolchain"; exit 1; }
  log "jdk 25: installed at $jdk_home"
else
  log "jdk 25: not found."
  log "Caffeine's Gradle build needs a JDK 25. Install one, e.g.:"
  log "    sudo apt-get install -y openjdk-25-jdk-headless    # Debian/Ubuntu"
  log "or set JAVA_HOME to an existing JDK 25 and re-run. (Or re-run with"
  log "--install-jdk to let this script apt-install it for you.)"
  exit 1
fi

log "caffeine simulator ready — CAFFEINE_DIR=$caffeine_dir JAVA_HOME=$jdk_home"
# Final stdout line: the JDK 25 home, for callers to capture into JAVA_HOME.
printf '%s\n' "$jdk_home"
