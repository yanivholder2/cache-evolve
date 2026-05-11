#!/bin/bash
# SessionStart hook (web-only): clone/pull the Caffeine fork and pre-assemble the
# simulator in the background so :simulator:run is fast later in the session.
# Idempotent. Failures are logged but never bubble up to the session.
set -uo pipefail

CAFFEINE_DIR="${HOME}/caffeine"
CAFFEINE_REPO="https://github.com/yanivholder2/caffeine.git"
LOG_DIR="${HOME}/.claude/hook-logs"
LOG_FILE="${LOG_DIR}/caffeine-warmup.log"

mkdir -p "${LOG_DIR}"

{
  echo "=== caffeine-warmup.sh $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="

  if [ ! -d "${CAFFEINE_DIR}" ]; then
    echo "Cloning ${CAFFEINE_REPO} -> ${CAFFEINE_DIR}"
    git clone "${CAFFEINE_REPO}" "${CAFFEINE_DIR}" --branch master || true
  else
    echo "Updating ${CAFFEINE_DIR}"
    git -C "${CAFFEINE_DIR}" pull || true
  fi
} >> "${LOG_FILE}" 2>&1

# Pre-assemble the simulator in the background so the session is interactive
# immediately. The log will keep growing after the hook returns.
if [ -x "${CAFFEINE_DIR}/gradlew" ]; then
  (
    cd "${CAFFEINE_DIR}" && ./gradlew :simulator:assemble || true
  ) >> "${LOG_FILE}" 2>&1 &
  disown || true
fi

exit 0
