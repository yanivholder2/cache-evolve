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

Model: `claude-opus-5-5` only, the same `--model` value as the v2 Opus runs.
Subjects: four of the five v2 subjects (dash, associative_cache, lru_cache,
tiny_lfu); s3_fifo is skipped (see the [run matrix](#run-matrix)). The matrix
is one `SH` and one `SF` row per subject; the cross-workload `SX` rows are
blocked on a missing tool.

**Where results go:** `outputs/paper_outputs/snia_runs/<mode>/<subject>/<run>/`,
with the same `<mode>` folders and run-dir names as v2. `plots/build_data.py`
reads every `*_runs` batch, `snia_runs/` included, and splits by the run
snapshot's `fitness.evaluation_profile`: these runs go to
`plots/paper-plots/data_<group>_msr.json` (e.g. `data_only_hit_rate_msr.json`),
never into the default-grid (Zipf) `data_<group>.json`.

**v2 counterparts.** Each row's Zipf counterpart is the `claude-opus-5-5` run
dir with the same mode and subject under
`outputs/paper_outputs/v2_runs/<mode>/<subject>/`. The retired v2 plan (row ids
`H<NN>`/`F<NN>`) is in git: `git show 40dc0a52^:outputs/paper_outputs/v2_runs/paper-runs-plan.md`.

### Matrix design (one line per choice)

- **only_hit_rate (`SH` rows).** metadata-size and per-op-time don't depend on
  the trace (they come from a `size_of` bin and a fixed microbench), so
  hit-rate is the only signal MSR adds. Hit-rate-only isolates it and compares
  directly with the v2 Opus only_hit_rate runs.
- **full_fitness_vector (`SF` rows).** This answers whether the 0.6/0.2/0.2
  trade-off still wins when real-trace hit-rate gains are harder to get.
- **One `msr` profile with both traces and shared sizes, averaged.** This
  halves the runs and penalises variants that overfit one volume. The hit-rate
  plugin's summary is the unweighted mean over the (trace, size) grid, and
  `eval/fitness/hit-rate.json` keeps every point, so per-trace results come
  free at analysis time.
- **Opus only, 10 cycles × 3 variants, same flags as v2.** Every difference from
  the v2 Opus run is then the workload.
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

| Parameter | Where it goes | Value |
| --- | --- | --- |
| MSR sizes | `sizes` in each subject's `[evaluation.profiles.msr]` (the same list for all four, so subjects are comparable) | `[16384, 131072]` |
| Timeout | `--simulator-timeout-s` in both command templates | `1700` |

`--simulator-timeout-s` bounds each trace's sweep over both sizes. dash makes
one Caffeine call per size (its `numOfSegments` is derived from the size via
`[caffeine.size-divisors]`), and those calls share one deadline, so every
subject gets the same budget per trace. The value is the slowest baseline trace
in `measure_baseline.py --evaluation-profile msr` × ~3, rounded up, floor 600
(config's default): associative_cache on web_0 took 544 s, so 544 × 3 ≈ 1633 →
1700. A variant that times out is rejected at fitness, so a tight value turns
slow-but-good variants into losses. per-op-time has its own fixed 120 s limit
and doesn't depend on the trace.

dash and associative_cache log every get/put at DEBUG (~5.5–6 GB of Rust logs
per call). Their `[evaluation.profiles.msr]` sets `simulator-log-level =
"INFO"`, which caps the cdylib's log level on MSR calls whatever `--log-level`
is (one dash hm_0 @ 16384 call: 447 s / 6.0 GB → 7 s / 3 KB, identical
hit-rate CSV). The commands keep `--log-level DEBUG`.

**Prerequisites (done in PR 264):** the traces and SNIA NOTICE are committed
under `inputs/traces/msr/`; the four manifests declare
`[evaluation.profiles.msr]` with
`traces = ["lirs:inputs/traces/msr/web_0.trace.gz", "lirs:inputs/traces/msr/hm_0.trace.gz"]`
and `sizes = [16384, 131072]`; each subject's anchor is committed at
`inputs/subjects/<S>/baseline/eval/profiles/msr/fitness-vector.json` (measured
with `.venv/bin/python scripts/measure_baseline.py <S> --evaluation-profile msr`).
The dash anchor is re-measured after the segment-hash fix (see
[Run matrix](#run-matrix)).

## Commands

Substitute `<S>` (subject) from the row. Apart from `--evaluation-profile msr`
and the [timeout](#parameters), these are the v2 templates.

**only_hit_rate** (`SH` rows):

```
.venv/bin/evolve --subject <S> --model claude-opus-5-5 --ai-agent-tool claude-sdk \
  --cycles 10 --cohort-size 3 --authoring-scope full-autonomy --log-level DEBUG \
  --max-workers 0 --simulator-timeout-s 1700 --digest-cycles 3 \
  --implementation-attempts 3 --hypothesis-attempts 3 --winners-count 3 \
  --evaluation-profile msr \
  --dimensions hit-rate --selection-method pareto
```

**full_fitness_vector** (`SF` rows):

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

**Don't edit this file from a run session**: concurrent PRs conflict on a
shared table. The [run matrix](#run-matrix) only
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

Merge `promote-run` PRs without waiting for CI. Record incidents and
the root cause of a failed run in the PR description.

## Run matrix

Status columns below are a frozen snapshot. See [Checking run
status](#checking-run-status) for the live state. v2 counterparts: see
[Goal](#goal).

| ID | Mode | Subject | Model | Status | Session | Run dir | Global best vs baseline | PR |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SH01 | only_hit_rate | dash | claude-opus-5-5 | invalidated — re-run needed | | | | |
| SH02 | only_hit_rate | associative_cache | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `only_hit_rate/associative_cache/associative_cache_claude-opus-5-5_2026-10-28_14-10-09-632` | 0.549 vs 0.527 (c10/v002) | |
| SH03 | only_hit_rate | lru_cache | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `only_hit_rate/lru_cache/lru_cache_claude-opus-5-5_2026-10-28_11-39-56-149` | 0.562 vs 0.528 (c10/v003) | |
| SH05 | only_hit_rate | tiny_lfu | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `only_hit_rate/tiny_lfu/tiny_lfu_claude-opus-5-5_2026-10-28_12-54-17-355` | 0.562 vs 0.504 (c09/v001) | |
| SF01 | full_fitness_vector | dash | claude-opus-5-5 | done | session_01Hio5C5m2zG4qiZfgqm2ddj | `full_fitness_vector/dash/dash_claude-opus-5-5_2026-11-05_22-15-04-763` | score 0.075 (c10/v002): hit-rate 0.535 vs 0.510, metadata 40 vs 40 B, per-op 58 vs 75 ns | |
| SF02 | full_fitness_vector | associative_cache | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `full_fitness_vector/associative_cache/associative_cache_claude-opus-5-5_2026-10-29_09-31-16-763` | score 0.176 (c10/v002): hit-rate 0.541 vs 0.527, metadata 24 vs 40 B, per-op 33 vs 55 ns | |
| SF03 | full_fitness_vector | lru_cache | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `full_fitness_vector/lru_cache/lru_cache_claude-opus-5-5_2026-10-28_19-54-39-536` | score 0.139 (c10/v003): hit-rate 0.538 vs 0.528, metadata 16 vs 48 B, per-op 35 vs 34 ns | |
| SF05 | full_fitness_vector | tiny_lfu | claude-opus-5-5 | done | session_01Hio5C3pM7tnk71WAbEFUbo | `full_fitness_vector/tiny_lfu/tiny_lfu_claude-opus-5-5_2026-10-29_08-15-12-815` | score 0.297 (c09/v002): hit-rate 0.559 vs 0.504, metadata 24 vs 48 B, per-op 35 vs 102 ns | |
| SX01 | rescore Zipf→MSR | the four (winners of their v2 Opus only_hit_rate runs) | claude-opus-5-5 | blocked | | | | |
| SX02 | rescore MSR→Zipf | the four (SH01–SH03, SH05 winners) | claude-opus-5-5 | blocked | | | | |

**SH01/SF01 (dash) are invalidated — re-run needed.** Dash derived the
segment index and the in-segment bucket from the same hash bits, so on the MSR
grid (8/64 segments) only ~5 of 32 buckets per segment were reachable (fixed in
this branch: the segment index now comes from the hash's high 32 bits; the
single-segment default grid is unaffected). The old runs (SH01 0.405 vs 0.388,
SF01 score 0.151) and the dash msr baseline of 0.388 measured that broken cache.
The old run dirs are removed from the tree and remain in git at `230de476`. The
re-measured dash msr baseline hit-rate is **0.510** (per-op get p50 75 ns, metadata 40 B; see
`inputs/subjects/dash/baseline/eval/profiles/msr/fitness-vector.json`). Re-run
both rows with the unchanged commands.

SH04/SF04 (s3_fifo) are dropped: its linear-scan eviction takes ~1 h per variant
on MSR, so it has no `msr` profile. IDs are not renumbered, so the number still
matches the subject order dash, associative_cache, lru_cache, s3_fifo, tiny_lfu.

SX rows need no LLM time, only simulator time, once the tool below exists.

## Notes

v2 run dirs cited below are relative to `outputs/paper_outputs/v2_runs/`.

- **Cross-workload rescoring (SX01/SX02) is blocked.** The idea is to score the
  v2 Zipf winners on the MSR grid, and the SH winners on the default grid,
  without evolving. That separates "evolution found MSR-specific gains" from
  "v2 winners were already better everywhere". The ingredients exist: promoted
  runs keep each variant's `attempts/NN/overlay/`, both anchors (default and
  `msr`) are committed, and `run_gates` + `compute_fitness` take any `Variant`. What's
  missing is an entry point. `evolve` has no rescore flag, and
  `scripts/measure_baseline.py` only scores the empty-overlay baseline. Needed:
  a script (e.g. `scripts/score_overlay.py <subject> <overlay dir>
  --evaluation-profile NAME --out DIR`) that wraps an overlay dir as a
  `VariantDir`, runs the gates, and writes `eval/fitness-vector.json` outside
  the run dir, plus a `cross_workload/` output folder under `snia_runs/`. Until
  that lands, don't invent commands for these rows.
- **Plots.** `plots/build_data.py` picks up `snia_runs/` like any `*_runs`
  batch and keys datasets by evaluation profile: MSR runs land in
  `plots/paper-plots/data_<group>_msr.json`, Zipf runs in `data_<group>.json`.
  A side-by-side MSR-vs-Zipf figure still has to load both files.
- **Why one averaged profile with shared sizes.** Per-trace profiles
  (`msr_web0`, `msr_hm0`) were rejected: they would double the matrix, and
  `hit-rate.json` already keeps every (trace, size) point for per-volume
  analysis. Sizes `[16384, 131072]` are shared by both traces (≈0.9% / 6.8% of
  web_0's 1.92M distinct blocks, 2.7% / 21% of hm_0's 610k) and are multiples
  of dash's 2048-entry segment.
- **Known issue carried from v2: missing writer output.** A hypotheses or
  insight writer sometimes starts a background agent, says it will wait for
  its report, and ends its turn without writing `H<NN>.md` / `insight.md`
  (v2 `full_fitness_vector/dash/dash_claude-opus-5-5_2026-10-19_11-52-26-211`,
  `full_fitness_vector/dash/dash_claude-haiku-5-5_2026-10-20_11-18-16-635`,
  `only_hit_rate/lru_cache/lru_cache_claude-haiku-5-5_2026-10-20_14-00-21-467`). That is an agent-behaviour failure, not a usage limit.
  `--hypothesis-attempts` only retries scope violations, so a missing
  `H<NN>.md` ends the run; resume with `--continue`.
- **Known issue carried from v2: Gradle/Maven infra failures.** On a fresh
  checkout the cycle-1 variants run Caffeine's first Gradle build in parallel
  and one can read a half-written generated `FDW.java` (`reached end of file
  while parsing` in `:caffeine:generateNodes`) and be rejected at fitness (v2
  `only_hit_rate/associative_cache/associative_cache_claude-haiku-5-5_2026-10-20_14-11-42-199`). Generated sources aren't rebuilt, so stopping and restarting the run
  from scratch avoids it. Gradle's cold dependency fetch has also hit HTTP 429
  from Maven Central (v2
  `full_fitness_vector/s3_fifo/s3_fifo_claude-opus-5-5_2026-10-20_12-08-19-021`,
  `full_fitness_vector/lru_cache/lru_cache_claude-haiku-5-5_2026-10-20_11-17-42-329`). Treat both as infra failures, not variant
  failures.
