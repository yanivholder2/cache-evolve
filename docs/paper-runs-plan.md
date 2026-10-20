# Paper runs plan

Shared plan for the Claude 5.5 paper runs. Every run session reads this first.

## Goal

Reproduce the existing `outputs/paper_outputs/` matrix — 2 modes × 5 subjects —
with the Claude 5.5 models (`claude-opus-5-5`, `claude-sonnet-5-5`,
`claude-haiku-5-5`), for comparison with the 4.x runs already there (Opus 4.8,
Sonnet 4.6, Haiku 4.5). 30 runs in total, listed in the [run
matrix](#run-matrix).

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

> Read docs/paper-runs-plan.md and run row `<ID>`: launch the row's exact
> command via /evolve-run (detached, per "Running it in a cloud session"; give
> me the full log path), resume with `--continue` after any usage-limit stop,
> then /promote-run. Don't edit docs/paper-runs-plan.md.

## Checking run status

**Don't edit this file from a run session.** Run PRs used to update a shared
status table here, and every pair of concurrent PRs conflicted on it. The
[run matrix](#run-matrix) below only defines the rows; its status columns are a
frozen snapshot, not maintained.

The repo is the source of truth. A row is **done** when its promoted run dir
exists on `main`:

```
git fetch origin main
git ls-tree -d --name-only origin/main \
  outputs/paper_outputs/v2_runs/<mode>/<subject>/ | grep "_<model>_"
```

- A match means the row is done (the dir name carries the run timestamp).
- No match but an open PR titled `<subject> paper_outputs v2 (<mode>) — <model>, …`
  means it is promoted and awaiting merge.
- Neither means `todo` (or a run still in progress in some session).

A `promote-run` PR only adds run outputs (no code), so merge it without
waiting for CI. Record run incidents (usage-limit resumes, agent failures,
infra errors) and the root cause of a failed run in the `promote-run` PR
description, not in this file; a rerun just promotes a new run dir.

## Run matrix

Status columns below are a frozen snapshot — see [Checking run
status](#checking-run-status) for the live state.

| ID | Mode | Subject | Model | Status | Session | Run dir | Global best vs baseline | PR |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F01 | full_fitness_vector | dash | claude-opus-5-5 | done | [session](https://claude.ai/code/session_01WAowihAggUXk2Qi38UYmw7) | `v2_runs/full_fitness_vector/dash/dash_claude-opus-5-5_2026-10-19_11-52-26-211` | hit-rate 0.713 vs 0.663; per-op-time 67 vs 183; metadata-size 40 = 40 | [#234](https://github.com/yanivholder2/cache-evolve/pull/234) |
| F02 | full_fitness_vector | associative_cache | claude-opus-5-5 | done | https://claude.ai/code/session_015Jo8GAf5i1jRV8GRQRojkK | `v2_runs/full_fitness_vector/associative_cache/associative_cache_claude-opus-5-5_2026-10-19_11-53-18-215` | hit-rate 0.734 vs 0.661, metadata 16 vs 40, per-op 45 vs 69 (score 0.256) | https://github.com/yanivholder2/cache-evolve/pull/235 |
| F03 | full_fitness_vector | lru_cache | claude-opus-5-5 | done | https://claude.ai/code/session_01JccMG2g1aPbYViN43Hg9hf | `v2_runs/full_fitness_vector/lru_cache/lru_cache_claude-opus-5-5_2026-10-19_11-55-52-682` | hit-rate 0.731 vs 0.665; metadata 8 vs 48; per-op-time 26 vs 41 | [#236](https://github.com/yanivholder2/cache-evolve/pull/236) |
| F04 | full_fitness_vector | s3_fifo | claude-opus-5-5 | done | [session](https://claude.ai/code/session_011TQ2JdPeyTHk15EvP95FpY) | `v2_runs/full_fitness_vector/s3_fifo/s3_fifo_claude-opus-5-5_2026-10-20_12-08-19-021` | hit-rate 0.733 vs 0.716; per-op-time 22 vs 248; metadata-size 8 vs 24 | [#246](https://github.com/yanivholder2/cache-evolve/pull/246) |
| F05 | full_fitness_vector | tiny_lfu | claude-opus-5-5 | todo | | | | |
| F06 | full_fitness_vector | dash | claude-sonnet-5-5 | todo | | | | |
| F07 | full_fitness_vector | associative_cache | claude-sonnet-5-5 | todo | | | | |
| F08 | full_fitness_vector | lru_cache | claude-sonnet-5-5 | todo | | | | |
| F09 | full_fitness_vector | s3_fifo | claude-sonnet-5-5 | todo | | | | |
| F10 | full_fitness_vector | tiny_lfu | claude-sonnet-5-5 | todo | | | | |
| F11 | full_fitness_vector | dash | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_013hP3e4eiFKeHTZ1j2ZR38e) | `v2_runs/full_fitness_vector/dash/dash_claude-haiku-5-5_2026-10-20_11-18-16-635` | hit-rate 0.691 vs 0.663; per-op-time 75 vs 183; metadata-size 32 vs 40 | [#238](https://github.com/yanivholder2/cache-evolve/pull/238) |
| F12 | full_fitness_vector | associative_cache | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_016kRPvF7L8PHK3phJHi1jKZ) | `v2_runs/full_fitness_vector/associative_cache/associative_cache_claude-haiku-5-5_2026-10-20_11-17-39-256` | hit-rate 0.728 vs 0.661; per-op-time 51 vs 69; metadata-size 40 = 40 | [#240](https://github.com/yanivholder2/cache-evolve/pull/240) |
| F13 | full_fitness_vector | lru_cache | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_01UB8SgNBhVia7sv4HnZkAjN) | `v2_runs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-5-5_2026-10-20_11-17-42-329` | hit-rate 0.724 vs 0.665; per-op-time 35 vs 41; metadata-size 16 vs 48 | [#241](https://github.com/yanivholder2/cache-evolve/pull/241) |
| F14 | full_fitness_vector | s3_fifo | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_017536x5FADQvAAG92zzSgmx) | `v2_runs/full_fitness_vector/s3_fifo/s3_fifo_claude-haiku-5-5_2026-10-20_11-17-18-755` | hit-rate 0.722 vs 0.716; per-op-time 41 vs 248; metadata-size 16 vs 24 | [#239](https://github.com/yanivholder2/cache-evolve/pull/239) |
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
| H12 | only_hit_rate | associative_cache | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_018dhG2LdphVcdxguz3MMQqT) | `v2_runs/only_hit_rate/associative_cache/associative_cache_claude-haiku-5-5_2026-10-20_14-11-42-199` | hit-rate 0.743 vs 0.661 | [#243](https://github.com/yanivholder2/cache-evolve/pull/243) |
| H13 | only_hit_rate | lru_cache | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_01Lor6rbEG64XV6jKWZWh6hg) | `v2_runs/only_hit_rate/lru_cache/lru_cache_claude-haiku-5-5_2026-10-20_14-00-21-467` | hit-rate 0.737 vs 0.665 | [#245](https://github.com/yanivholder2/cache-evolve/pull/245) |
| H14 | only_hit_rate | s3_fifo | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_01MdVbig9j526V8JoEwAkmQk) | `v2_runs/only_hit_rate/s3_fifo/s3_fifo_claude-haiku-5-5_2026-10-20_14-00-19-782` | hit-rate 0.717 vs 0.716 | [#244](https://github.com/yanivholder2/cache-evolve/pull/244) |
| H15 | only_hit_rate | tiny_lfu | claude-haiku-5-5 | done | [session](https://claude.ai/code/session_01Bmw4udWzXC5MBfeYBkWGfV) | `v2_runs/only_hit_rate/tiny_lfu/tiny_lfu_claude-haiku-5-5_2026-10-20_13-08-10-178` | hit-rate 0.743 vs 0.736 | [#242](https://github.com/yanivholder2/cache-evolve/pull/242) |

## Notes

Frozen like the run matrix: new incidents go in the run's `promote-run` PR.

- **F01:** hit the account usage limit twice (cycles 5 and 9), with
  `ZeroTokenUsageError` and "You've hit your session limit". Both times it was
  resumed with `--continue` after the reset and finished all 10 cycles, every
  cycle with survivors. Cycle 9 v003 was rejected because its insight writer
  ended its turn "waiting for the agent's report" without writing `insight.md`.
  That is an agent-behaviour failure, not a limit.
- **F03:** hit the account session limit twice, in cycle 6 (insight writer,
  `ZeroTokenUsageError`) and at the start of cycle 8 (hypotheses writer wrote no
  `H<NN>.md` files). Both times it was resumed with `--continue` after the reset
  and finished all 10 cycles, all 30 variants surviving.
- **F04:** the detached `setsid nohup` launch did not survive. When the
  launcher's Bash call hit its background time limit (~30 min), the harness
  killed the run with it, mid-cycle 5 with no `EXIT=` line (same as F13). It
  was resumed with `--continue` and finished all 10 cycles. In cycle 1, v001
  and v002 were rejected at fitness because Gradle's cold dependency fetch got
  HTTP 429 from Maven Central. That is an environment failure, not a variant
  failure. Every later cycle had 3/3 survivors.
- **F11:** exited 1 twice (cycles 4 and 8), both before any variant ran. Each
  time the hypotheses-writer started a background Explore agent, said it would
  wait for the agent's report, and ended its turn without writing any
  `H<NN>.md`. This is the same agent behaviour as F01 cycle 9, not a usage
  limit. `--hypothesis-attempts` only retries scope violations, not a
  wrong file count, so each one ended the run. Both were resumed with
  `--continue` and the run finished all 10 cycles, 3/3 survivors in every
  cycle.
- **F13:** the detached `setsid nohup` launch did not survive: the harness
  killed the run with its launcher shell when that Bash call hit its timeout,
  in cycle 7 with no `EXIT=` line. It was resumed with `--continue` as a
  tracked background task (2h limit, enough for the ~5 min/cycle left) and
  finished all 10 cycles, every cycle with survivors. Cycle 1 v003 was
  rejected at fitness because Maven Central returned HTTP 429 while Gradle
  fetched `caffeine-3.2.3.jar`, an infra failure rather than the variant's.
- **H12:** the first launch was stopped and restarted from scratch in
  cycle 2. In cycle 1 the variants ran Caffeine's first Gradle build in
  parallel in the shared checkout, and v001 read a half-written generated
  `FDW.java` (`reached end of file while parsing` in
  `:caffeine:generateNodes`), so it was rejected at fitness. Once
  generated, the sources are not rebuilt, so the restarted run did not
  hit the race and finished all 10 cycles with 3/3 survivors in every
  cycle. Any fresh session can hit this race on its first cycle.
- **H13:** hit the account session limit within 10 minutes of launch, while
  cycle 1 was still running. Cycle 1 lost v001 and v002 at the insight stage
  (`ZeroTokenUsageError`), so only v003 survived, at baseline hit-rate. Cycle 2
  then failed outright (0/3, exit 1). After the 14:10 UTC reset the run was
  resumed with `--continue` and finished all 10 cycles, every cycle with at
  least one survivor. Two later insight rejections (cycle 2 v002, cycle 4 v002)
  are the F11 behaviour: the writer started a background agent and ended its
  turn without writing `insight.md`. Cycle 2 v003 was rejected because its
  hypothesis (pre-size the index) was already true of the baseline, so the
  implementer refused to write an empty change three times.
- **H14:** hit the account session limit in cycle 2: all three variant
  implementers returned "You've hit your session limit · resets 2:10pm (UTC)"
  and the run exited 1 with `no variants survived in cycle 0002`. It was
  resumed with `--continue` after the reset and finished all 10 cycles, 3/3
  survivors in every cycle (about 3.5 min per cycle). The first launch's
  `setsid nohup … & disown; sleep 20; tail` Bash call hung and was moved into
  a harness background task, as in F13. The run was already in its own
  session, so killing only the stuck launcher shells left it running.
  Launching with `setsid -f bash -c '…' > $L 2>&1 < /dev/null` (no `&`, no
  sleep) returned immediately on the resume.
