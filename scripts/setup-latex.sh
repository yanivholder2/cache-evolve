#!/usr/bin/env bash
# Set up the LaTeX toolchain used to compile docs/*.tex to PDF, plus
# poppler-utils so agents can render PDF pages for visual review.
#
# Idempotent and safe to re-run. Installs via apt only when run
# non-interactively in a remote container ($CLAUDE_CODE_REMOTE=true) or when
# called with --install; otherwise it reports what is missing and exits
# non-zero, so a local machine is never modified behind your back.
#
# Note for remote sessions: tectonic's package bundle host is blocked by the
# network policy, but Ubuntu's apt mirrors are allowed — hence TeX Live.
set -euo pipefail

log() { printf '%s\n' "$*" >&2; }

# texlive-science: algpseudocode; texlive-humanities/-plain-generic: nth.sty
# and friends pulled in by docs/research-proposal.tex.
APT_PACKAGES=(
  texlive-latex-base
  texlive-latex-recommended
  texlive-latex-extra
  texlive-fonts-recommended
  texlive-science
  texlive-humanities
  texlive-plain-generic
  poppler-utils
)

missing=()
command -v pdflatex >/dev/null || missing+=(pdflatex)
command -v pdftoppm >/dev/null || missing+=(pdftoppm)
if [[ ${#missing[@]} -eq 0 ]]; then
  log "LaTeX toolchain already present (pdflatex + pdftoppm)."
  exit 0
fi

if [[ "${CLAUDE_CODE_REMOTE:-}" != "true" && "${1:-}" != "--install" ]]; then
  log "Missing: ${missing[*]}. Re-run with --install to apt-get them, or install manually:"
  log "  sudo apt-get install -y --no-install-recommends ${APT_PACKAGES[*]}"
  exit 1
fi

log "Installing: ${APT_PACKAGES[*]}"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends "${APT_PACKAGES[@]}"
log "LaTeX toolchain ready."
