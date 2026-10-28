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
Subjects: four of the five v2 subjects (dash, associative_cache, lru_cache,
tiny_lfu); s3_fifo is skipped (see the [run matrix](#run-matrix)). The primary
matrix is 4 SH runs plus 4 SF runs. The cross-workload rows are blocked on a missing tool. See the
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
- **One `msr` profile with both traces and shared sizes, averaged (decided).**
  This halves the runs and penalises variants that overfit one volume. The hit-rate plugin's summary is
  the unweighted mean over the (trace, size) grid, and `eval/fitness/hit-rate.json`
  keeps every point, so per-trace results come free at analysis time.
- **Opus only, 10 cycles × 3 variants, same flags as v2.** Every difference from
  the v2 Opus run is then the workload. Cutting cycles to save cost would break
  that comparison.
- **Cross-workload generalisation (`SX` rows) is listed but blocked.** See
  [Notes](#notes) for what's missing.

## How a run works

- **One run per cloud session** (4 cores). The SessionStart hook
  (`.claude/hooks/session-start.sh`) provisions the venv, Caffeine checkout and
  JDK. Launch with `evolve-run`, promote with `promote-run`, passing the plan
  path and batch explicitly (see [Session prompt](#session-prompt)).
- **Detach the run.** Harness background tasks are cut off at 2h, shorter than
  a run. Launch with
  `setsid -f bash -c '.venv/bin/evolve <flags>; echo "EXIT=$?"' > $L 2>&1 < /dev/null`
  (`L=<scratchpad>/<ID>.log`; no `&`, no `sleep`), then start a background
  waiter `until grep -q '^EXIT=' $L; do sleep 60; done; tail -3 $L`, and
  restart the waiter if it is cut off. Give the user the run dir and full log
  path; each resume logs to a new file (`<ID>-resume1.log`, …).
- **Usage limits.** When the account hits its limit every agent call returns
  `ZeroTokenUsageError` and the run exits 1 with `no variants survived in cycle
  NNNN` (the cycle's `*.log.md` says `You've hit your session limit · resets
  <time>`). Not a run failure: after the reset, resume with
  `.venv/bin/evolve --continue outputs/<subject>_<timestamp>`.
- **Stopping a run:** `pgrep -f "bin/evolve --subject"` and kill those PIDs.
  `pkill -f` on the same pattern also kills your own shell.
- **Promote:** the container has no `rsync`; copy with
  `tar --exclude='*/eval/caffeine/rust-logs' --exclude='*gradle*.log'`.

Differences from the v2 Zipf runs:

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
| `<MSR_SIZES>` | `sizes` in each subject's `[evaluation.profiles.msr]` (the same list for all four, so subjects are comparable) | `[16384, 131072]` |
| `<TIMEOUT_S>` | `--simulator-timeout-s` in both command templates | `1700` |

`--simulator-timeout-s` bounds each Caffeine call, not the whole grid: one call
per trace sweeps both sizes, except dash, which makes one call per size because
its `numOfSegments` is derived from the size (`[caffeine.size-divisors]`).
`<TIMEOUT_S>` is the slowest baseline call in `measure_baseline.py
--evaluation-profile msr` × ~3, rounded up, with a floor of 600 (config's
default). The slowest call was associative_cache on web_0 (both sizes) at
544 s, so 544 × 3 ≈ 1633 → 1700. Baselines were measured at `log_level =
"DEBUG"`, the level the run templates pass. Under DEBUG, dash and
associative_cache log every get/put, writing ~5.5–6 GB of Rust logs per call
(440–520 s per call). Their `[evaluation.profiles.msr]` now sets
`simulator-log-level = "INFO"`, which caps the cdylib's log level for MSR calls
regardless of `--log-level`: one dash hm_0 @ 16384 call drops from 447 s / 6.0 GB
to 7 s / 3 KB with an identical hit-rate CSV. The commands keep `--log-level
DEBUG`, and the timeout stays as a generous bound. A variant that
times out is rejected at fitness, so a tight value turns slow-but-good variants
into losses. It doesn't cover
per-op-time, which has its own fixed 120 s limit and doesn't depend on the trace.

**Prerequisites (one-off, not per row):** commit the traces and SNIA NOTICE
under `inputs/traces/msr/`, add `[evaluation.profiles.msr]` with
`traces = ["lirs:inputs/traces/msr/web_0.trace.gz", "lirs:inputs/traces/msr/hm_0.trace.gz"]`
and `sizes = [16384, 131072]` to the four manifests, then for each subject run
`.venv/bin/python scripts/measure_baseline.py <S> --evaluation-profile msr` and
commit the anchor.

## Commands

Substitute `<S>` (subject) from the row; `<TIMEOUT_S>` from
[Parameters](#parameters) is already filled in (1700). The model is always `claude-opus-5-5`. Apart from the
profile and timeout, these are the v2 templates.

**only_hit_rate** (`SH` rows, primary):

```
.venv/bin/evolve --subject <S> --model claude-opus-5-5 --ai-agent-tool claude-sdk \
  --cycles 10 --cohort-size 3 --authoring-scope full-autonomy --log-level DEBUG \
  --max-workers 0 --simulator-timeout-s 1700 --digest-cycles 3 \
  --implementation-attempts 3 --hypothesis-attempts 3 --winners-count 3 \
  --evaluation-profile msr \
  --dimensions hit-rate --selection-method pareto
```

**full_fitness_vector** (`SF` rows, extension):

```
.venv/bin/evolve --subject <S> --model claude-opus-5-5 --ai-agent-tool claude-sdk \
  --cycles 10 --cohort-size 3 --authoring-scope full-autonomy --log-level DEBUG \
  --max-workers 0 --simulator-timeout-s 1700 --digest-cycles 3 \
  --implementation-attempts 3 --hypothesis-attempts 3 --winners-count 3 \
  --evaluation-profile msr \
  --dimensions hit-rate metadata-size per-op-time \
  --selection-method weighted --weights hit-rate=0.6 metadata-size=0.2 per-op-time=0.2
```

Launch a row with `/evolve-run <ID> from outputs/paper_outputs/snia_runs/paper-runs-plan.md`
and promote it with `/promote-run snia_runs`, which copies into
`outputs/paper_outputs/snia_runs/<mode>/<subject>/` and titles the PR
`<subject> paper_outputs snia (<mode>) — …`. It picks the mode from the
snapshot's `[fitness].dimensions`, so both templates map to the usual
`only_hit_rate` / `full_fitness_vector` folders.

## Session prompt

Paste into a fresh cloud session, replacing `<ID>`:

> Read outputs/paper_outputs/snia_runs/paper-runs-plan.md, then run
> `/evolve-run <ID> from outputs/paper_outputs/snia_runs/paper-runs-plan.md`:
> run the extra MSR pre-flight checks, launch the row's exact command detached
> (per "How a run works"; give me the full log path), resume with `--continue`
> after any usage-limit stop, then `/promote-run snia_runs`. Don't edit the
> plan doc.

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
| SH03 | only_hit_rate | lru_cache | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `only_hit_rate/lru_cache/lru_cache_claude-opus-5-5_2026-10-28_11-39-56-149` | 0.562 vs 0.528 (c10/v003) | |
| SH05 | only_hit_rate | tiny_lfu | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `only_hit_rate/tiny_lfu/tiny_lfu_claude-opus-5-5_2026-10-28_12-54-17-355` | 0.562 vs 0.504 (c09/v001) | |
| SF01 | full_fitness_vector | dash | claude-opus-5-5 | todo | | | | |
| SF02 | full_fitness_vector | associative_cache | claude-opus-5-5 | todo | | | | |
| SF03 | full_fitness_vector | lru_cache | claude-opus-5-5 | todo | | | | |
| SF05 | full_fitness_vector | tiny_lfu | claude-opus-5-5 | todo | | | | |
| SX01 | rescore Zipf→MSR | the four (v2 H01–H03, H05 winners) | claude-opus-5-5 | blocked | | | | |
| SX02 | rescore MSR→Zipf | the four (SH01–SH03, SH05 winners) | claude-opus-5-5 | blocked | | | | |

SH04/SF04 (s3_fifo) are dropped: its linear-scan eviction takes ~1 h per variant
on MSR, so it has no `msr` profile. IDs are not renumbered, so each row still
pairs with the v2 row of the same number.

Run order: SH01–SH03 and SH05 first (any order, in parallel sessions), then the SF rows
(scheduled unconditionally). SX rows need no LLM time,
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
- **Plots.** `plots/build_data.py` ignores `snia_runs/` (batch regex
  `^v\d+_runs$`). Comparing MSR to Zipf needs it to treat the evaluation
  profile as a dataset key, not a batch.
- **Resolved: one averaged profile with shared sizes.** Per-trace profiles
  (`msr_web0`, `msr_hm0`) were rejected: they would double the matrix, and
  `hit-rate.json` already keeps every (trace, size) point for per-volume
  analysis. Sizes `[16384, 131072]` are shared by both traces (≈0.9% / 6.8% of
  web_0's 1.92M distinct blocks, 2.7% / 21% of hm_0's 610k) and are multiples
  of dash's 2048-entry segment.
- **Open choice.** Whether SF rows run at all, which depends on cost after SH.
- **Known issue carried from v2: missing writer output.** A hypotheses or
  insight writer sometimes starts a background agent, says it will wait for
  its report, and ends its turn without writing `H<NN>.md` / `insight.md`
  (v2 F01, F11, H13). That is an agent-behaviour failure, not a usage limit.
  `--hypothesis-attempts` only retries scope violations, so a missing
  `H<NN>.md` ends the run; resume with `--continue`.
- **Known issue carried from v2: Gradle/Maven infra failures.** On a fresh
  checkout the cycle-1 variants run Caffeine's first Gradle build in parallel
  and one can read a half-written generated `FDW.java` (`reached end of file
  while parsing` in `:caffeine:generateNodes`) and be rejected at fitness (v2
  H12). Generated sources aren't rebuilt, so stopping and restarting the run
  from scratch avoids it. Gradle's cold dependency fetch has also hit HTTP 429
  from Maven Central (v2 F04, F13). Treat both as infra failures, not variant
  failures.
