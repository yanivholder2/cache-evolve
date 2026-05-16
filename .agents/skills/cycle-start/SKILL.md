---
name: cycle-start
description: Start a new evolve research cycle. Use when the user says "start a cycle", "run a cycle", "run evolve", "new cycle", or wants to scaffold a cycle workspace, run fitness plugins, compute the Pareto front, append a ledger row, and rewrite the leaderboard.
---

# cycle-start skill

Runs `evolve/scripts/cycle-start.sh` to scaffold one research cycle, copy the `good` fixture into `v001`, run G1+G2 gates, execute three real fitness plugins, aggregate a FitnessVector, compute the Pareto front, write per-variant + per-cycle insights, seal the cycle, atomically append a ledger row, and regenerate `evolve/leaderboard.md`.

> v1 still copies a single fixture variant (`v001` from `evolve/fixtures/good/`). The Pareto front is therefore trivially `["v001"]`. Real cohort generation arrives in a later cycle; the walking-skeleton fitness stub has been removed and fitness is now wired to real plugins.

## Prerequisites

- **Rust toolchain.** First run triggers `cargo build` for `evolve-hit-rate`, `evolve-metadata-size`, and the `evolve/bench` harness. Cold builds can take several minutes.
- **Python 3.** The script prefers `evolve/.venv/bin/python3` if present, otherwise falls back to the system `python3`.
- **`evolve/fixtures/good/impl/`** must contain `segment.rs` and `manifest.toml` (used as the v001 source).
- No external trace files are required — the hit-rate plugin uses synthetic workloads (`EVENTS=2000`, `SEED=42`).

## Run it

From the repo root:

```bash
bash evolve/scripts/cycle-start.sh
```

With options:

```bash
HYPOTHESIS="Test if LFU performs better than LRU for Zipfian traces." bash evolve/scripts/cycle-start.sh
WORKER_ID="worker-01" bash evolve/scripts/cycle-start.sh
HYPOTHESIS="..." WORKER_ID="worker-01" bash evolve/scripts/cycle-start.sh
```

The script auto-detects `EVOLVE_ROOT` relative to its own location, so it works regardless of the shell's current directory.

## What it does

1. Detects the next cycle ID (`0002` if `0001` exists, etc.) and creates `evolve/cycles/<NNNN>/variants/v001/{impl,eval/fitness}/`.
2. Copies `evolve/fixtures/good/impl/{segment.rs,manifest.toml}` into `variants/v001/impl/`.
3. Runs `gate-check.sh` (G1 build + G2 trait contract; G3 is still a stub). Failure aborts the cycle.
4. Writes `cohort.md` with the hypothesis text.
5. Runs each fitness plugin and writes its JSON under `variants/v001/eval/fitness/`:
   - `hit-rate.py` → `eval/fitness/hit-rate.json` (`higher_is_better: true`)
   - `per-op-time.py` → `eval/fitness/per-op-time.json` (`higher_is_better: false`)
   - `metadata-size.py` → `eval/fitness/metadata-size.json` (`higher_is_better: false`)
6. Aggregates with `fitness-aggregate.py` → `variants/v001/eval/fitness-vector.json`.
7. Computes the Pareto front with `pareto.py` over the variant array (trivially `["v001"]` while there is only one variant).
8. Writes `variants/v001/insight.md` with the real per-dimension summary, and `cycle-insight.md` with the Pareto front.
9. Writes `winners.json` with `pareto_front` and the full `fitness_vectors`.
10. Seals the cycle (`SEALED` marker, `chmod -R a-w`).
11. Atomically appends one row to `evolve/ledger/cycles.jsonl`.
12. Regenerates `evolve/leaderboard.md` from the ledger.

## Verify success

Expected final line: `cycle <NNNN> complete — sealed at <path>`

```bash
ls evolve/cycles/<NNNN>/                                      # SEALED marker present
cat evolve/cycles/<NNNN>/variants/v001/eval/fitness-vector.json
cat evolve/cycles/<NNNN>/winners.json                         # pareto_front + fitness_vectors populated
tail -1 evolve/ledger/cycles.jsonl                            # new row, matching cycle ID, computed pareto_front
head -20 evolve/leaderboard.md                                # rewritten, includes this cycle
```

## Gotchas

- **Cold build is slow.** First invocation typically takes 1–3 minutes while `cargo` builds `evolve-hit-rate` (debug), `evolve-metadata-size` (debug), and the `evolve/bench` harness (release). Warm runs finish in seconds to tens of seconds. Pre-warm with: `cargo build --bin evolve-hit-rate --bin evolve-metadata-size && (cd evolve/bench && cargo build --release)`.
- **`evolve/leaderboard.md` is regenerated every cycle.** Do not hand-edit; changes will be overwritten on the next run.
- **Sealed cycles are immutable.** To delete or overwrite: `chmod -R u+w evolve/cycles/<NNNN>`, then `rm -rf` it.
- **The ledger is append-only.** Never edit `evolve/ledger/cycles.jsonl` by hand. A partial row from a crash must be verified before the next run.
- **Not idempotent.** The script aborts if `evolve/cycles/<NNNN>` already exists. Two recovery paths:
  - **Unsealed leftover** (plugin / aggregator / Pareto / ledger step failed before sealing): `rm -rf evolve/cycles/<NNNN>` directly, then rerun.
  - **Sealed but no ledger row** (rare — failure between seal and append): `chmod -R u+w evolve/cycles/<NNNN>`, delete the `SEALED` file, then `rm -rf evolve/cycles/<NNNN>` and rerun.

## v2-ready invariants

See `docs/adr/0003-v1-single-machine-v2-ready-invariants.md`. In brief:

- All paths in ledger rows are repo-relative (no leading `/`).
- Every artifact carries `worker_id` (defaults to `"local"`).
- Ledger append is atomic: flock + atomic rename in `scripts/ledger-append.py`.
