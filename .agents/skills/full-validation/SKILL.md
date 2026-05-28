---
name: full-validation
description: Run the entire pytest suite including every opt-in marker. Use when the user wants full validation, to run all tests, or to check everything before merging — not just the fast suite.
---

## Steps

Markers can't be combined in one run, so run each in its own invocation, in order. Use the venv python (`.venv/bin/python`), never system python. Run `uv sync` first if `.venv` is missing.

1. Fast suite (no opt-in flags):
   ```
   .venv/bin/python -m pytest
   ```

2. Slow suite (real cargo builds, adds ~2min):
   ```
   .venv/bin/python -m pytest --slow
   ```

3. Caffeine suite (requires `~/projects/caffeine` and a working JDK):
   ```
   .venv/bin/python -m pytest --caffeine-needed
   ```
   If caffeine or a JDK isn't available, skip this run and say so — don't fail the validation over a missing prerequisite.

Report each run's result. If any run fails, surface the failures rather than continuing silently.
