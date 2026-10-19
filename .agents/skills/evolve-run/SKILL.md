---
name: evolve-run
description: Launch an evolve run from a natural-language description, validate it, then read the whole output folder and report what happened. Use when the user wants to kick off an evolve run — described in plain language like "run dash for 3 cycles with full autonomy" — let it run, and be told the outcome: winners, failures, and any logging issues.
---

Kick off one evolve run, gate it with cheap pre-flight checks, then — once it
finishes — read its entire output folder and report the outcome in chat.

## Translate the request into flags

The user describes the run in **natural language**, not strict flags (e.g.
"evolve the dash subject for 3 cycles, four variants each, full autonomy, debug
logging"). Map that description onto the `evolve` CLI flags below; anything the
user doesn't mention falls back to the repo's `config.toml` default (don't pass
a flag for it). With no description at all, run with config defaults only.

| Says something like… | Flag |
| --- | --- |
| which cache / subject | `--subject <name>` (a dir under `inputs/subjects/`) |
| how many cycles / generations | `--cycles <n>` |
| variants/candidates per cycle, cohort/population size | `--cohort-size <n>` |
| which agent backend | `--ai-agent-tool {claude-sdk,antigravity-sdk,ollama-local}` |
| verbosity / logging | `--log-level {DEBUG,INFO,WARNING,ERROR,CRITICAL}` |
| simulator timeout | `--simulator-timeout-s <s>` |
| parallelism / worker threads | `--max-workers <n>` |
| "full autonomy" vs "extension points only" | `--authoring-scope {full-autonomy,extension-points}` |
| which model | `--model <id>` |
| prior insights in the insight-writer prompt | `--digest-cycles <n>` |
| implementation / self-repair attempts per variant | `--implementation-attempts <n>` |
| hypothesis attempts per cycle | `--hypothesis-attempts <n>` |
| fitness dimensions ("hit-rate only", "full vector") | `--dimensions DIM [DIM ...]` (under weighted, also pass matching `--weights`, or use pareto) |
| winner selection | `--selection-method {pareto,weighted}` (pareto drops config.toml's weights) |
| how many winners | `--winners-count <n>` |
| dimension weights | `--weights DIM=W [DIM=W ...]` (replaces the whole table) |

Every `config.toml` setting has a flag; `"$EVOLVE" --help` is authoritative.
Under `claude-sdk`, `--model` takes any `claude-<family>-<version>` id (e.g. `claude-opus-5-5`,
`claude-sonnet-5-5`, `claude-haiku-5-5`) — don't reject an id for being absent
from a list.

**Paper runs.** If the request references the paper run plan or a plan row id
(e.g. `F01`), read `docs/paper-runs-plan.md` first and launch that row's exact
command — every setting pinned, no natural-language translation — and update
the row as that doc's rules say.

Confirm the assembled command line back to the user in one line before
launching. Flags are non-persistent overrides — never edit `config.toml` or any
committed input, so the run leaves no persistent change.

## Setup

```
REPO_ROOT="$(git rev-parse --show-toplevel)"
EVOLVE="$REPO_ROOT/.venv/bin/evolve"
```

If `$EVOLVE` is missing, run `uv sync` at `$REPO_ROOT` first. Always use the
venv, never system python.

## 1. Pre-flight (abort before launching if any fail)

These are seconds of work guarding a run that can take ~8 min per cycle, so
stop and report rather than launching into a broken environment:

1. **Venv** resolves (above).
2. **Caffeine checkout** is reachable: the `$CAFFEINE_DIR` dir, else a
   `caffeine/` sibling of `$REPO_ROOT`. If absent, abort — the fitness stage
   needs it.
3. **JDK 25** is available (Caffeine's Gradle toolchain floor): `$JAVA_HOME`
   points at a 25 JDK, or `/usr/lib/jvm/java-25-openjdk-amd64` exists, or
   `java -version` reports >= 25. If absent, abort and say so (on the web the
   SessionStart hook provisions it; locally the user installs it).
4. **Workspace builds**: `cargo build --release` at `$REPO_ROOT`. A broken main
   build means every variant fails — catch it now, not 8 minutes in.

If any check fails, report which one and stop. Do not launch.

## 2. Launch (in the background)

Runs are long, so launch in the background and let the turn end; the harness
wakes you when it exits. Capture stdout+stderr and the exit code.

```
"$EVOLVE" <user flag overrides>
```

Record the run's output dir — the newest `outputs/<subject>_<timestamp>/` under
`$REPO_ROOT/outputs/` created by this launch.

## 3. Validate (after it exits)

- **Exit code** is 0.
- **Every cycle had a survivor.** Each `cycles/NNNN/cycle-winners.json` lists
  winners; the run log also prints `cycle NNNN survivors=X/Y rejected={…}`. Flag
  any cycle with 0 survivors and surface the rejection reason.
- **Caffeine produced results**: `cycles/*/variants/*/eval/caffeine/*.csv` exist.
- **Ledger + leaderboard advanced**: `cycles-ledger.jsonl` and `leaderboard.md`
  are present and non-empty, with one ledger line per completed cycle.
- **No truncated/empty logs**: the `*-writer.log.md` and per-variant attempt
  logs are non-zero and not cut off mid-stream.

## 4. Report (chat only)

Read the **entire** output folder — `leaderboard.md`, `cross-cycle-insights.md`,
`cross-cycle-winners.json`, each `cycles/NNNN/` (task, insight, winners, writer
logs), and the per-variant `attempts/` and `eval/` trees — then report in chat
(do not write a file, do not commit):

1. **What happened** — global best + per-cycle hit-rate trend, the winning
   variant(s), and whether fitness improved over baseline.
2. **Failures + root cause** — rejected variants and why (build gate, invariant
   gate, simulator error, timeout), with the smallest telling excerpt.
3. **Logging / quality issues** — exceptions or tracebacks in any log, agents
   that finished `status != ok`, empty/malformed JSON or CSV, simulator
   timeouts, and cost/usage anomalies. This is the "is something off?" pass the
   user explicitly wants.
4. **Improvement suggestions** — concrete, repo-specific follow-ups (a gate that
   keeps tripping, a noisy or missing log, a flag worth changing next run).

Keep it scannable. If the run failed outright (non-zero exit, or 0 survivors in
every cycle), lead with that and the root cause.
