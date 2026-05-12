---
name: cycle-start
description: Start a new evolve research cycle. Use when the user says "start a cycle", "run a cycle", "run evolve", "new cycle", "test the framework", or wants to scaffold a cycle workspace and append a ledger row.
---

# cycle-start skill

Runs `evolve/scripts/cycle-start.sh` to scaffold one research cycle, write stub artifacts, seal the cycle directory, and atomically append a ledger row.

> **Walking skeleton.** Every subagent slot is a stub: hardcoded variant, all gates pass, fixed fitness numbers. No real LLM judgment occurs yet.

## Run it

From the repo root:

```bash
bash evolve/scripts/cycle-start.sh
```

With options:

```bash
# Custom hypothesis
HYPOTHESIS="Test if LFU performs better than LRU for Zipfian traces." bash evolve/scripts/cycle-start.sh

# Custom worker ID
WORKER_ID="worker-01" bash evolve/scripts/cycle-start.sh

# Both
HYPOTHESIS="Refactor bucket movement" WORKER_ID="worker-01" bash evolve/scripts/cycle-start.sh
```

The script auto-detects `EVOLVE_ROOT` relative to its own location, so it works regardless of the shell's current directory.

## What it does

1. Detects the next available cycle ID (`0002` if `0001` exists, etc.).
2. Creates `evolve/cycles/<NNNN>/` with the required file tree.
3. Writes stub artifacts: `cohort.md`, `variants/v001/{impl/,eval/fitness.json,insight.md}`, `cycle-insight.md`, `winners.json`.
4. Seals the cycle: writes `SEALED` marker, drops write bits (`chmod -R a-w`).
5. Atomically appends one row to `evolve/ledger/cycles.jsonl`.

## Verify success

Expected script output: `cycle <NNNN> complete — sealed at <path>`

```bash
ls evolve/cycles/<NNNN>/          # SEALED file must be present
tail -1 evolve/ledger/cycles.jsonl  # new row with matching cycle ID
```

## Gotchas

- **Python 3 required** — `scripts/ledger-append.py` runs via `python3`. Verify with `python3 --version` if the append step fails.
- **Not idempotent.** If the run fails after sealing but before ledger append, the sealed cycle directory exists without a ledger row. Unseal manually (`chmod -R u+w evolve/cycles/<NNNN>`, delete `SEALED`, then `rm -rf evolve/cycles/<NNNN>`), then rerun.
- **Sealed cycles are immutable.** To delete or overwrite a sealed cycle: `chmod -R u+w evolve/cycles/<NNNN>` first.
- **The ledger is append-only.** Never edit `evolve/ledger/cycles.jsonl` by hand. A partial row from a crash must be verified before the next run.

## v2-ready invariants

See `docs/adr/0003-v1-single-machine-v2-ready-invariants.md` for the full rationale. In brief:

- All paths in ledger rows are repo-relative (no leading `/`).
- Every artifact carries `worker_id` (defaults to `"local"`).
- Ledger append is atomic: flock + atomic rename in `scripts/ledger-append.py`.
