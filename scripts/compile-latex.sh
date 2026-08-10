#!/usr/bin/env bash
# Compile a .tex file to PDF next to it. Runs pdflatex twice so
# cross-references and the table of contents resolve.
#
# Usage: scripts/compile-latex.sh docs/research-proposal.tex
set -euo pipefail

tex_file="${1:?usage: compile-latex.sh <file.tex>}"
tex_dir="$(cd "$(dirname "$tex_file")" && pwd)"
tex_name="$(basename "$tex_file")"

# nonstopmode without -halt-on-error: recoverable errors (e.g. a missing
# figure file) still yield a PDF to review; we surface them from the log.
cd "$tex_dir"
pdf="${tex_name%.tex}.pdf"
log_file="${tex_name%.tex}.log"
rm -f "$pdf"
for pass in 1 2; do
  pdflatex -interaction=nonstopmode "$tex_name" >/dev/null || true
done

if [[ ! -f "$pdf" ]]; then
  echo "pdflatex produced no PDF; errors:" >&2
  grep -A3 '^!' "$log_file" >&2 || true
  exit 1
fi

echo "$tex_dir/$pdf"
{
  grep -A2 '^!' "$log_file" || true
  grep -E 'LaTeX Warning|Overfull|Underfull' "$log_file" | sort | uniq -c | sort -rn | head -20 || true
} >&2
