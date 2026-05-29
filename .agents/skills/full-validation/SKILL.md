---
name: full-validation
description: Run the entire pytest suite including every opt-in marker. Use when the user wants full validation, to run all tests, or to check everything before merging — not just the fast suite.
---

## Setup (handles worktrees)

The venv lives at the main repo root and its editable install of `evolve` points at the main repo's `src`. When running from a worktree, that means tests would import stale main-repo code unless overridden. Resolve both paths up front, then use them for every run below:

```
REPO_ROOT="$(git rev-parse --path-format=absolute --git-common-dir)"; REPO_ROOT="${REPO_ROOT%/.git}"
VENV_PY="$REPO_ROOT/.venv/bin/python"
WT_SRC="$(git rev-parse --show-toplevel)/src"
```

`$VENV_PY` is the venv python (at the main repo root, shared by all worktrees). `$WT_SRC` is the current working tree's `src`, prepended via `PYTHONPATH` so pytest exercises *this* checkout's code, not the editable install's. In the main checkout these collapse to the same paths, so the commands are safe everywhere.

If `$VENV_PY` is missing, run `uv sync` at `$REPO_ROOT` first. Never use system python.

## Steps

Markers can't be combined in one run, so run each in its own invocation, in order.

1. Fast suite (no opt-in flags):
   ```
   PYTHONPATH="$WT_SRC" "$VENV_PY" -m pytest
   ```

2. Slow suite (real cargo builds, adds ~2min):
   ```
   PYTHONPATH="$WT_SRC" "$VENV_PY" -m pytest --slow
   ```

3. Caffeine suite (requires `~/projects/caffeine` and a working JDK):
   ```
   PYTHONPATH="$WT_SRC" "$VENV_PY" -m pytest --caffeine-needed
   ```
   If caffeine or a JDK isn't available, skip this run and say so — don't fail the validation over a missing prerequisite.

Report each run's result. If any run fails, surface the failures rather than continuing silently.
