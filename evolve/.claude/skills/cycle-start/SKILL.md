---
name: cycle-start
description: >
  Stub orchestrator for one evolve cycle. Scaffolds cycles/<N>/, runs all
  subagent stubs (hardcoded in the walking skeleton), seals the cycle, and
  appends one row to the insight ledger. Use when starting a new research cycle.
triggers:
  - "start cycle"
  - "run cycle"
  - "cycle-start"
---

# cycle-start skill

Runs `evolve/scripts/cycle-start.sh` from the repo root.

## What it does (walking skeleton)

1. Creates `evolve/cycles/0001/` with the required file tree.
2. Writes stub `cohort.md`, `variants/v001/{impl/,eval/,insight.md}`, `cycle-insight.md`, `winners.json`.
3. Seals the cycle (drops write bits, writes `SEALED` marker).
4. Atomically appends one row to `evolve/ledger/cycles.jsonl`.

## Usage

```
EVOLVE_ROOT=evolve/ bash evolve/scripts/cycle-start.sh
```

Or invoke this skill: `/cycle-start`

## v2-ready invariants (ADR-0003)

- All paths in ledger rows are repo-relative.
- Every artifact carries `worker_id: "local"`.
- Ledger append is atomic (flock + rename via `scripts/ledger-append.py`).
