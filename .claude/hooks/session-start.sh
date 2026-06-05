#!/bin/bash
# SessionStart hook: provision a Claude Code on the web container so the Evolve
# framework and its Caffeine simulator integration can build, test, and run.
#
# Local checkouts are set up by hand (uv sync + scripts/setup-caffeine.sh), so
# this only runs in the remote container. Idempotent.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

repo_root="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"

# Python venv + dependencies (the .venv the framework requires).
(cd "$repo_root" && uv sync)

# Caffeine checkout + JDK 25, via the shared setup script. In a remote container
# it auto-installs the JDK and prints the resolved JDK 25 home on stdout; capture
# it for JAVA_HOME so Gradle resolves the toolchain locally instead of from
# foojay.io, which the network policy blocks.
jdk_home="$("$repo_root/scripts/setup-caffeine.sh")"
if [ -n "$jdk_home" ]; then
  echo "export JAVA_HOME=$jdk_home" >> "$CLAUDE_ENV_FILE"
fi
