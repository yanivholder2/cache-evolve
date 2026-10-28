# Paper runs plan — SNIA (MSR Cambridge) traces

Shared plan for the real-workload paper runs. Every run session reads this first.

## Goal

Ask whether the v2 result (Opus 5.5 beats every subject's baseline on the
synthetic Zipf trace `workload-a`) carries over to real storage workloads, as
the research proposal requires ("perform well on known workloads … a wider
range of workloads"). The workloads are two MSR Cambridge volumes from SNIA
IOTTA, converted to lirs by `scripts/convert_msr_trace.py`: 4 KiB block keys,
reads and writes both count as accesses.

| Trace | Events | Distinct blocks | File |
| --- | --- | --- | --- |
| web_0 | 9.64M | 1.92M | `inputs/traces/msr/web_0.trace.gz` |
| hm_0 | 8.99M | 610k | `inputs/traces/msr/hm_0.trace.gz` |

Model: `claude-opus-5-5` only, the same `--model` value as the v2 Opus rows.
Subjects: the five v2 subjects. The primary matrix is 5 runs, with an optional
5-run extension. The cross-workload rows are blocked on a missing tool. See the
[run matrix](#run-matrix).

**Where results go:** `outputs/paper_outputs/snia_runs/<mode>/<subject>/<run>/`,
with the same `<mode>` folders and run-dir names as v2. This batch is kept out
of `v<N>_runs/` on purpose: `plots/build_data.py` only picks up folders that
match `^v\d+_runs$`, so MSR results can't end up in the Zipf datasets. MSR
plots need a follow-up (see [Notes](#notes)).

### Matrix design (one line per choice)

- **Primary = only_hit_rate (`SH` rows).** metadata-size and per-op-time don't
  depend on the trace (they come from a `size_of` bin and a fixed microbench), so
  hit-rate is the only signal MSR adds. Hit-rate-only isolates it and compares
  directly with v2's Opus H01–H05 (all done).
- **Extension = full_fitness_vector (`SF` rows).** This answers whether the
  0.6/0.2/0.2 trade-off still wins when real-trace hit-rate gains are harder
  to get. Run it only after SH rows show MSR hit-rate is movable.
- **One `msr` profile with both traces, averaged.** This halves the runs and
  penalises variants that overfit one volume. The hit-rate plugin's summary is
  the unweighted mean over the (trace, size) grid, and `eval/fitness/hit-rate.json`
  keeps every point, so per-trace results come free at analysis time.
- **Opus only, 10 cycles × 3 variants, same flags as v2.** Every difference from
  the v2 Opus run is then the workload. Cutting cycles to save cost would break
  that comparison.
- **Cross-workload generalisation (`SX` rows) is listed but blocked.** See
  [Notes](#notes) for what's missing.

## How a run works

Everything in the v2 plan's "How a run works" and "Running it in a cloud
session (lessons learned)" applies unchanged: one run per cloud session,
SessionStart hook environment, `evolve-run` to launch, `promote-run` to
promote, the detached `setsid -f` launch plus a waiter, resuming with
`--continue` after a usage limit, stopping with `pgrep -f "bin/evolve --subject"`,
and copying with `tar` for promotion. Read that section first. Differences:

- **Slower variants.** Caffeine on MSR takes about 18× longer per variant than
  on Zipf (two traces of ~9M events each, against one small trace). Expect
  cycles to be dominated by simulation instead of agent time, so plan on the
  long end of the v2 3–8h range or beyond. `--max-workers 0` (3 workers on 4
  cores) is kept. Don't raise it.
- **Resume keeps the profile.** The config snapshot records
  `fitness.evaluation_profile`, so `.venv/bin/evolve --continue <run dir>`
  stays on MSR. Pass no other flags.
- **Pre-flight, in addition to `evolve-run`'s checks:** the two trace files
  exist, the subject's `manifest.toml` declares `[evaluation.profiles.msr]`, and
  `inputs/subjects/<S>/baseline/eval/profiles/msr/fitness-vector.json` is
  committed. `evolve` refuses a profile whose anchor is missing or was scored
  on a different grid, but check before launching anyway.
- **Never edit `config.toml` or a manifest for a run.** The MSR grid lives in
  the committed manifests and everything else is pinned by flag below.

### Parameters

Fill these in once measuring is done, **before any row runs**. Each row takes
them from here.

| Placeholder | Where it goes | Value |
| --- | --- | --- |
| `<MSR_SIZES>` | `sizes` in each subject's `[evaluation.profiles.msr]` (the same list for all five, so subjects are comparable) | `<MSR_SIZES>` |
| `<TIMEOUT_S>` | `--simulator-timeout-s` in both command templates | `<TIMEOUT_S>` |

`<TIMEOUT_S>` must cover one variant's whole Caffeine grid (2 traces × the size
count) with headroom. The slowest baseline (`measure_baseline.py --evaluation-profile msr`)
× ~1.5 is a reasonable floor. A variant that times out is rejected at fitness,
so a tight value turns slow-but-good variants into losses. It doesn't cover
per-op-time, which has its own fixed 120 s limit and doesn't depend on the trace.

**Prerequisites (one-off, not per row):** commit the traces and SNIA NOTICE
under `inputs/traces/msr/`, add `[evaluation.profiles.msr]` with
`traces = ["lirs:inputs/traces/msr/web_0.trace.gz", "lirs:inputs/traces/msr/hm_0.trace.gz"]`
and `sizes = <MSR_SIZES>` to all five manifests, then for each subject run
`.venv/bin/python scripts/measure_baseline.py <S> --evaluation-profile msr` and
commit the anchor.

## Commands

Substitute `<S>` (subject) from the row and `<TIMEOUT_S>` from
[Parameters](#parameters). The model is always `claude-opus-5-5`. Apart from the
profile and timeout, these are the v2 templates.

**only_hit_rate** (`SH` rows, primary):

```
.venv/bin/evolve --subject <S> --model claude-opus-5-5 --ai-agent-tool claude-sdk \
  --cycles 10 --cohort-size 3 --authoring-scope full-autonomy --log-level DEBUG \
  --max-workers 0 --simulator-timeout-s <TIMEOUT_S> --digest-cycles 3 \
  --implementation-attempts 3 --hypothesis-attempts 3 --winners-count 3 \
  --evaluation-profile msr \
  --dimensions hit-rate --selection-method pareto
```

**full_fitness_vector** (`SF` rows, extension):

```
.venv/bin/evolve --subject <S> --model claude-opus-5-5 --ai-agent-tool claude-sdk \
  --cycles 10 --cohort-size 3 --authoring-scope full-autonomy --log-level DEBUG \
  --max-workers 0 --simulator-timeout-s <TIMEOUT_S> --digest-cycles 3 \
  --implementation-attempts 3 --hypothesis-attempts 3 --winners-count 3 \
  --evaluation-profile msr \
  --dimensions hit-rate metadata-size per-op-time \
  --selection-method weighted --weights hit-rate=0.6 metadata-size=0.2 per-op-time=0.2
```

`promote-run` picks the mode from the snapshot's `[fitness].dimensions` exactly
as in v2, so both templates map to the usual `only_hit_rate` /
`full_fitness_vector` folders.

## Session prompt

Paste into a fresh cloud session, replacing `<ID>`:

> Read outputs/paper_outputs/snia_runs/paper-runs-plan.md (and the "Running it
> in a cloud session" section of outputs/paper_outputs/v2_runs/paper-runs-plan.md)
> and run row `<ID>`: run the extra MSR pre-flight checks, launch the row's exact
> command via /evolve-run (detached; give me the full log path), resume with
> `--continue` after any usage-limit stop, then /promote-run, overriding its v2
> defaults: copy into `outputs/paper_outputs/snia_runs/<mode>/<subject>/`, and
> title the PR `<subject> paper_outputs snia (<mode>) — claude-opus-5-5, …`.
> Don't edit either plan doc.

## Checking run status

**Don't edit this file from a run session**, for the same reason as in v2:
concurrent PRs conflict on a shared table. The [run matrix](#run-matrix) only
defines the rows. Its status columns are a frozen snapshot.

The repo is the source of truth. A row is **done** when its promoted run dir
exists on `main`:

```
git fetch origin main
git ls-tree -d --name-only origin/main \
  outputs/paper_outputs/snia_runs/<mode>/<subject>/ | grep "_claude-opus-5-5_"
```

- A match means the row is done (the dir name carries the run timestamp).
- No match, but an open PR titled `<subject> paper_outputs snia (<mode>) — claude-opus-5-5, …`,
  means it's promoted and waiting to be merged.
- Neither means `todo` (or a run still in progress in some session).

Merge `promote-run` PRs without waiting for CI, as in v2. Record incidents and
the root cause of a failed run in the PR description.

## Run matrix

Status columns below are a frozen snapshot. See [Checking run
status](#checking-run-status) for the live state. The v2 Zipf counterpart of each
`SH` row is v2 H01–H05, and of each `SF` row v2 F01–F05 (same subject order).

| ID | Mode | Subject | Model | Status | Session | Run dir | Global best vs baseline | PR |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SH01 | only_hit_rate | dash | claude-opus-5-5 | todo | | | | |
| SH02 | only_hit_rate | associative_cache | claude-opus-5-5 | todo | | | | |
| SH03 | only_hit_rate | lru_cache | claude-opus-5-5 | todo | | | | |
| SH04 | only_hit_rate | s3_fifo | claude-opus-5-5 | todo | | | | |
| SH05 | only_hit_rate | tiny_lfu | claude-opus-5-5 | todo | | | | |
| SF01 | full_fitness_vector | dash | claude-opus-5-5 | optional | | | | |
| SF02 | full_fitness_vector | associative_cache | claude-opus-5-5 | optional | | | | |
| SF03 | full_fitness_vector | lru_cache | claude-opus-5-5 | optional | | | | |
| SF04 | full_fitness_vector | s3_fifo | claude-opus-5-5 | optional | | | | |
| SF05 | full_fitness_vector | tiny_lfu | claude-opus-5-5 | optional | | | | |
| SX01 | rescore Zipf→MSR | all five (v2 H01–H05 winners) | claude-opus-5-5 | blocked | | | | |
| SX02 | rescore MSR→Zipf | all five (SH01–SH05 winners) | claude-opus-5-5 | blocked | | | | |

Run order: SH01–SH05 first (any order, in parallel sessions). Start SF rows only
once at least three SH rows beat their MSR baseline. SX rows need no LLM time,
only simulator time, once the tool below exists.

## Notes

- **Cross-workload rescoring (SX01/SX02) is blocked.** The idea is to score the
  v2 Zipf winners on the MSR grid, and the SH winners on the default grid,
  without evolving. That separates "evolution found MSR-specific gains" from
  "v2 winners were already better everywhere". The ingredients exist: promoted
  runs keep each variant's `attempts/NN/overlay/`, both anchors will be
  committed, and `run_gates` + `compute_fitness` take any `Variant`. What's
  missing is an entry point. `evolve` has no rescore flag, and
  `scripts/measure_baseline.py` only scores the empty-overlay baseline. Needed:
  a script (e.g. `scripts/score_overlay.py <subject> <overlay dir>
  --evaluation-profile NAME --out DIR`) that wraps an overlay dir as a
  `VariantDir`, runs the gates, and writes `eval/fitness-vector.json` outside
  the run dir, plus a `cross_workload/` output folder under `snia_runs/`. Until
  that lands, don't invent commands for these rows.
- **Skills assume v2.** `evolve-run` points plan row ids at the v2 plan, and
  `promote-run` hard-codes `v2_runs/` and the `v2` PR title. The session prompt
  overrides both. A small skill update to take the batch from the plan would
  remove the override.
- **Plots.** `plots/build_data.py` ignores `snia_runs/` (batch regex
  `^v\d+_runs$`). Comparing MSR to Zipf needs it to treat the evaluation
  profile as a dataset key, not a batch.
- **Open choices.** (1) Per-trace profiles (`msr_web0`, `msr_hm0`) instead of
  the averaged `msr`. These need only extra manifest tables and anchors, no
  code, and would double the matrix. Worth it if SH results show one volume
  dominating the mean. (2) Whether SF rows run at all, which depends on cost
  after SH. (3) `<MSR_SIZES>` should probably span each trace's working set
  (e.g. a small and a large fraction of 610k / 1.92M distinct blocks), since
  one size can't do that for both volumes.
- **Inherited hazards:** the first-cycle Gradle race in a fresh session (v2 H12)
  and the Maven Central HTTP 429 (v2 F04/F13) still apply. Treat both as infra
  failures, not variant failures.
