#!/bin/bash
# SessionStart hook: provision a Claude Code on the web container so the Evolve
# framework and its Caffeine simulator integration can build, test, and run.
#
# Local checkouts already have the venv, a sibling caffeine, and a JDK, so this
# is gated on the remote flag. Every step is idempotent (safe to re-run).
set -euo pipefail

# Only the remote (web) container needs provisioning.
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

repo_root="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"

# 1. Python venv + dependencies (the .venv the framework requires).
(cd "$repo_root" && uv sync)

# 2. Caffeine checkout — a sibling of the repo, matching caffeine_runner's
#    default resolution (the CAFFEINE_DIR env var overrides it).
caffeine_dir="${CAFFEINE_DIR:-$(dirname "$repo_root")/caffeine}"
if [ ! -d "$caffeine_dir/.git" ]; then
  git clone --depth 1 https://github.com/yanivholder2/caffeine.git "$caffeine_dir"
fi

# 3. JDK 25 — Caffeine's Gradle build pins its toolchain to a 25 floor. Install
#    it from the distro so Gradle resolves the toolchain locally instead of
#    downloading from foojay.io, which the remote network policy blocks.
jdk25="/usr/lib/jvm/java-25-openjdk-amd64"
if [ ! -d "$jdk25" ]; then
  sudo apt-get update -qq || true
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y openjdk-25-jdk-headless
fi

# Point Gradle (and the wrapper's daemon JVM) at the local JDK 25 so no
# toolchain download is attempted.
if [ -d "$jdk25" ]; then
  echo "export JAVA_HOME=$jdk25" >> "$CLAUDE_ENV_FILE"
fi
