---
name: cycle-start
description: Start a new evolve research cycle. Use when the user says "start a cycle", "run a cycle", "run evolve", "new cycle", or wants to scaffold a cycle workspace, run fitness plugins, compute the Pareto front, append a ledger row, and rewrite the leaderboard.
---

# cycle-start skill

Runs `evolve/scripts/cycle-start.sh` to scaffold one research cycle: fans out a cohort of N Variants via the `variant-implementer` subagent in parallel (P1), runs G1+G2 gates per Variant, executes three real fitness plugins per Variant, aggregates FitnessVectors, computes the Pareto front over the cohort, writes per-Variant + cycle insights, seals the cycle, atomically appends a ledger row, and regenerates `evolve/leaderboard.md`.

> Default cohort size is **6** (from `config.toml`). The default `variant-implementer` is `evolve/scripts/variant-implementer.sh`, which copies the `good` fixture into every Variant — a deterministic stand-in until the real subagent at `.claude/agents/variant-implementer.md` is wired into the orchestrator.

## Prerequisites

- **Rust toolchain.** First run triggers `cargo build` for `evolve-hit-rate`, `evolve-metadata-size`, and the `evolve/bench` harness. Cold builds can take several minutes.
- **Python 3.** The script prefers `evolve/.venv/bin/python3` if present, otherwise falls back to the system `python3`.
- **`evolve/fixtures/good/impl/`** must contain `segment.rs` and `manifest.toml` (used as the default `variant-implementer` source).
- No external trace files are required — the hit-rate plugin uses synthetic workloads (`EVENTS=2000`, `SEED=42`).

## Run it

From the repo root:

```bash
bash evolve/scripts/cycle-start.sh
```

With options:

```bash
COHORT_SIZE=3 bash evolve/scripts/cycle-start.sh
HYPOTHESES=$'h1 text\nh2 text\nh3 text' bash evolve/scripts/cycle-start.sh   # COHORT_SIZE inferred
VARIANT_IMPLEMENTER=/path/to/my-impl.sh bash evolve/scripts/cycle-start.sh
WORKER_ID="worker-01" bash evolve/scripts/cycle-start.sh
```

The script auto-detects `EVOLVE_ROOT` relative to its own location, so it works regardless of the shell's current directory.

## What it does

1. Resolves cohort size: `COHORT_SIZE` env > `config.toml` `cohort_size` > builtin `6`. `HYPOTHESES` (newline-separated) overrides the count.
2. Detects the next cycle ID (`0002` if `0001` exists, etc.) and creates `evolve/cycles/<NNNN>/`.
3. Writes `cohort.md` and one `hypotheses/H<NN>.md` per Hypothesis.
4. Fans out the `variant-implementer` in parallel (one invocation per Variant); each writes `variants/v<NNN>/impl/{segment.rs,manifest.toml}`.
5. Per Variant, runs `gate-check.sh` (G1 build + G2 trait contract; G3 is still a stub). Any failure aborts the cycle.
6. Per Variant, runs each fitness plugin → `variants/v<NNN>/eval/fitness/{hit-rate,per-op-time,metadata-size}.json`, then aggregates with `fitness-aggregate.py` → `variants/v<NNN>/eval/fitness-vector.json`.
7. Computes the Pareto front with `pareto.py` over **all** Variant fitness vectors.
8. Writes `variants/v<NNN>/insight.md` per Variant and `cycle-insight.md` (with cohort size and Pareto front).
9. Writes `winners.json` with `pareto_front` and the full `fitness_vectors` array.
10. Seals the cycle (`SEALED` marker, `chmod -R a-w`).
11. Atomically appends one row to `evolve/ledger/cycles.jsonl` (includes `cohort_size`).
12. Regenerates `evolve/leaderboard.md` from the ledger.

## Verify success

Expected final line: `cycle <NNNN> complete — sealed at <path> (cohort=N)`

```bash
ls evolve/cycles/<NNNN>/                                      # SEALED marker present
ls evolve/cycles/<NNNN>/variants/                             # v001..v<N> directories
cat evolve/cycles/<NNNN>/cohort.md                            # links to hypotheses/H01..HNN
cat evolve/cycles/<NNNN>/winners.json                         # pareto_front + fitness_vectors populated
tail -1 evolve/ledger/cycles.jsonl                            # new row, includes cohort_size + pareto_front
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
