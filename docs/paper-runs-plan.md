# Paper runs plan

Shared plan for the Claude 5.5 paper runs. Every run session reads this first.

## Goal

Reproduce the existing `outputs/paper_outputs/` matrix — 2 modes × 5 subjects —
with the Claude 5.5 models (`claude-opus-5-5`, `claude-sonnet-5-5`,
`claude-haiku-5-5`), for comparison with the 4.x runs already there (Opus 4.8,
Sonnet 4.6, Haiku 4.5). 30 runs in total, tracked in the [status
table](#status).

**Where results go:** `outputs/paper_outputs/v2_runs/<mode>/<subject>/<run>/`
(`promote-run` puts them there). The 4.x runs stay where they are, directly
under `outputs/paper_outputs/<mode>/`; `plots/build_data.py` reads both batches
into one dataset per mode.

## How a run works

- **One run per cloud session.** A session has 4 cores; a run takes ~3–8h,
  sometimes longer.
- **Environment** is provisioned by the SessionStart hook
  (`.claude/hooks/session-start.sh`): venv, Caffeine checkout, JDK 25.
- **Launch** with the `evolve-run` skill, **promote** with `promote-run`.
- **Resume** a crashed or reclaimed run with
  `.venv/bin/evolve --continue outputs/<subject>_<timestamp>` — it reuses the
  run's config snapshot, so pass no other flags.
- **Never edit `config.toml` for a run.** Every setting is pinned by flag
  below, so the matrix is reproducible regardless of `config.toml` drift.

### Running it in a cloud session (lessons learned)

Read this before launching so you don't have to discover it yourself:

- **Background tasks are cut off at 2h**, which is shorter than a run. Don't
  launch `evolve` as a harness background task. Detach it so it survives that
  limit (and worker restarts), and log to the scratchpad:
  ```
  L=<scratchpad>/<ID>.log
  setsid nohup bash -c '.venv/bin/evolve <flags>; echo "EXIT=$?"' > $L 2>&1 < /dev/null & disown
  ```
  Then start a separate background *waiter* that wakes you when the run exits:
  `until grep -q '^EXIT=' $L; do sleep 60; done; tail -3 $L`. Start a new
  waiter if it hits the 2h limit or the worker restarts. The detached run keeps
  going either way. Each resume writes to a new log (`<ID>-resume1.log`, …).
- **Give the user the log path.** Once the run starts, tell the user the run
  dir and the full absolute log path (e.g.
  `/tmp/claude-0/.../scratchpad/F01.log`) so they can follow along. Do the same
  after each resume.
- **Usage/session limits kill the run, but it can be resumed.** When the
  account hits its limit, every agent call returns at once with
  `ZeroTokenUsageError` (0 tokens). All variants fail and the run exits 1 with
  `no variants survived in cycle NNNN`. To confirm, grep that cycle's
  `*.log.md` for `You've hit your session limit · resets <time>`. This is not
  a run failure: once the limit resets, resume with `--continue` (it erases the
  half-done cycle) and add a Notes line. Mark the row `failed` only when the
  failure is real.
- **Stopping a run:** use `pgrep -f "bin/evolve --subject"` and kill those PIDs.
  `pkill -f` on the same pattern also matches your own shell and kills it
  (exit 144).
- **Real timing:** F01 (dash, Opus 5.5) took about 7–12 min per cycle, about
  1.5h of compute for 10 cycles. Most of the 3–8h estimate is usage-limit
  waits.
- **Promote:** the container has no `rsync`. Copy with
  `tar --exclude='*/eval/caffeine/rust-logs' --exclude='*gradle*.log'`
  (the raw run is ~6–7 GB; the curated copy is ~7 MB).

## Commands

Substitute `<S>` (subject) and `<M>` (model) from the row. Both templates
reproduce the config snapshots of the 4.x runs in `outputs/paper_outputs/`.

**full_fitness_vector** (`F` rows):

```
.venv/bin/evolve --subject <S> --model <M> --ai-agent-tool claude-sdk \
  --cycles 10 --cohort-size 3 --authoring-scope full-autonomy --log-level DEBUG \
  --max-workers 0 --simulator-timeout-s 600 --digest-cycles 3 \
  --implementation-attempts 3 --hypothesis-attempts 3 --winners-count 3 \
  --dimensions hit-rate metadata-size per-op-time \
  --selection-method weighted --weights hit-rate=0.6 metadata-size=0.2 per-op-time=0.2
```

**only_hit_rate** (`H` rows):

```
.venv/bin/evolve --subject <S> --model <M> --ai-agent-tool claude-sdk \
  --cycles 10 --cohort-size 3 --authoring-scope full-autonomy --log-level DEBUG \
  --max-workers 0 --simulator-timeout-s 600 --digest-cycles 3 \
  --implementation-attempts 3 --hypothesis-attempts 3 --winners-count 3 \
  --dimensions hit-rate --selection-method pareto
```

Known deviations in the 4.x matrix: `only_hit_rate/dash` with Sonnet 4.6 ran
with `--authoring-scope extension-points`; four `only_hit_rate` Opus/Haiku runs
(dash, lru_cache) predate the flag overlay, so their snapshots lack `model` and
record `subject = "lru_cache"` — trust the folder names there.

## Session prompt

Paste into a fresh cloud session, replacing `<ID>`:

> Read docs/paper-runs-plan.md and run row `<ID>`: mark the row `running` with
> this session's link and commit it on your branch, launch the row's exact
> command via /evolve-run (detached, per "Running it in a cloud session"; give
> me the full log path), resume with `--continue` after any usage-limit stop,
> then /promote-run and mark the row `done` (or `failed` with a Notes entry).

## Updating this file

- **Start:** flip your row to `running` and fill in the session link; commit it
  on the session's branch.
- **Finish:** in the `promote-run` PR, set `done` (or `failed`) and fill in the
  run dir, global-best vs baseline, and PR link.
- Touch only your own row — rows are independent lines, so merge conflicts stay
  trivial.
- A `failed` run records its root cause under [Notes](#notes) (row ID, one
  paragraph); a rerun reuses the row.

## Status

Status is `todo` / `running` / `done` / `failed`. First batch: F01–F05.

| ID | Mode | Subject | Model | Status | Session | Run dir | Global best vs baseline | PR |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F01 | full_fitness_vector | dash | claude-opus-5-5 | done | [session](https://claude.ai/code/session_01WAowihAggUXk2Qi38UYmw7) | `v2_runs/full_fitness_vector/dash/dash_claude-opus-5-5_2026-10-19_11-52-26-211` | hit-rate 0.713 vs 0.663; per-op-time 67 vs 183; metadata-size 40 = 40 | [#234](https://github.com/yanivholder2/cache-evolve/pull/234) |
| F02 | full_fitness_vector | associative_cache | claude-opus-5-5 | done | https://claude.ai/code/session_015Jo8GAf5i1jRV8GRQRojkK | `v2_runs/full_fitness_vector/associative_cache/associative_cache_claude-opus-5-5_2026-10-19_11-53-18-215` | hit-rate 0.734 vs 0.661, metadata 16 vs 40, per-op 45 vs 69 (score 0.256) | https://github.com/yanivholder2/cache-evolve/pull/235 |
| F03 | full_fitness_vector | lru_cache | claude-opus-5-5 | todo | | | | |
| F04 | full_fitness_vector | s3_fifo | claude-opus-5-5 | todo | | | | |
| F05 | full_fitness_vector | tiny_lfu | claude-opus-5-5 | todo | | | | |
| F06 | full_fitness_vector | dash | claude-sonnet-5-5 | todo | | | | |
| F07 | full_fitness_vector | associative_cache | claude-sonnet-5-5 | todo | | | | |
| F08 | full_fitness_vector | lru_cache | claude-sonnet-5-5 | todo | | | | |
| F09 | full_fitness_vector | s3_fifo | claude-sonnet-5-5 | todo | | | | |
| F10 | full_fitness_vector | tiny_lfu | claude-sonnet-5-5 | todo | | | | |
| F11 | full_fitness_vector | dash | claude-haiku-5-5 | todo | | | | |
| F12 | full_fitness_vector | associative_cache | claude-haiku-5-5 | todo | | | | |
| F13 | full_fitness_vector | lru_cache | claude-haiku-5-5 | todo | | | | |
| F14 | full_fitness_vector | s3_fifo | claude-haiku-5-5 | todo | | | | |
| F15 | full_fitness_vector | tiny_lfu | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_01EYsUrbsS22NguLbL53fDxv) | `v2_runs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-5-5_2026-10-20_09-59-28-671` | hit-rate 0.742 vs 0.736; per-op-time 54 vs 132; metadata-size 32 vs 48 | [#237](https://github.com/yanivholder2/cache-evolve/pull/237) |
| H01 | only_hit_rate | dash | claude-opus-5-5 | todo | | | | |
| H02 | only_hit_rate | associative_cache | claude-opus-5-5 | todo | | | | |
| H03 | only_hit_rate | lru_cache | claude-opus-5-5 | todo | | | | |
| H04 | only_hit_rate | s3_fifo | claude-opus-5-5 | todo | | | | |
| H05 | only_hit_rate | tiny_lfu | claude-opus-5-5 | todo | | | | |
| H06 | only_hit_rate | dash | claude-sonnet-5-5 | todo | | | | |
| H07 | only_hit_rate | associative_cache | claude-sonnet-5-5 | todo | | | | |
| H08 | only_hit_rate | lru_cache | claude-sonnet-5-5 | todo | | | | |
| H09 | only_hit_rate | s3_fifo | claude-sonnet-5-5 | todo | | | | |
| H10 | only_hit_rate | tiny_lfu | claude-sonnet-5-5 | todo | | | | |
| H11 | only_hit_rate | dash | claude-haiku-5-5 | todo | | | | |
| H12 | only_hit_rate | associative_cache | claude-haiku-5-5 | todo | | | | |
| H13 | only_hit_rate | lru_cache | claude-haiku-5-5 | todo | | | | |
| H14 | only_hit_rate | s3_fifo | claude-haiku-5-5 | todo | | | | |
| H15 | only_hit_rate | tiny_lfu | claude-haiku-5-5 | todo | | | | |

## Notes

- **F01:** hit the account usage limit twice (cycles 5 and 9), with
  `ZeroTokenUsageError` and "You've hit your session limit". Both times it was
  resumed with `--continue` after the reset and finished all 10 cycles, every
  cycle with survivors. Cycle 9 v003 was rejected because its insight writer
  ended its turn "waiting for the agent's report" without writing `insight.md`.
  That is an agent-behaviour failure, not a limit.
