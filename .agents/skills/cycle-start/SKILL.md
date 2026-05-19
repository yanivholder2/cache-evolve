---
name: cycle-start
description: Start a new evolve research cycle. Use when the user says "start a cycle", "run a cycle", "run evolve", "new cycle", or otherwise wants to launch the next research iteration over a cohort of Variants and record the result on the leaderboard.
---

# cycle-start skill

Run `evolve/scripts/cycle-start.py`. The script self-documents its env vars (`COHORT_SIZE`, `HYPOTHESES`, `VARIANT_IMPLEMENTER`, `WORKER_ID`) in its header and errors on missing prerequisites. Success ends with `cycle <NNNN> complete — sealed at <path> (cohort=N)`.

## Pre-warm (optional)

Cold builds take 1–3 minutes. To warm the cargo cache first:

```bash
cargo build --bin evolve-hit-rate --bin evolve-metadata-size && (cd evolve/bench && cargo build --release)
```

## Recovery — the script is not idempotent

If a run aborts, `evolve/cycles/<NNNN>` is left behind and the next run will refuse to start.

- **Unsealed leftover** (failed before sealing): `rm -rf evolve/cycles/<NNNN>`, then rerun.
- **Sealed but no ledger row** (rare — failure between seal and append): `chmod -R u+w evolve/cycles/<NNNN> && rm -rf evolve/cycles/<NNNN>`, then rerun.

Never hand-edit `evolve/ledger/cycles.jsonl` or `evolve/leaderboard.md` — both are regenerated/appended by the script.
