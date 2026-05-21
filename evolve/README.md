# Evolve

`evolve/` is the research framework that drives caches experimental loop. It runs **cycles** of cache-policy evolution: each cycle proposes a **cohort** of hypotheses, generates one Rust variant per hypothesis, runs correctness gates (G1/G2/G3) plus three fitness plugins (hit-rate, per-op time, metadata size), computes a Pareto front, and appends one row to the immutable ledger.

Subagents handle judgment work (hypothesize, implement) via per-CLI adapters. Scripts handle everything deterministic — scaffolding, gates, scoring, sealing, ledger/leaderboard maintenance. Everything a run writes lands under `evolve/outputs/` (gitignored): cycle artifacts at `outputs/cycles/NNNN/` (sealed `chmod -w` once complete), plus `outputs/ledger/cycles.jsonl` and `outputs/leaderboard.md` as the persistent across-cycle views.

## Environment Setup

The scripts rely on Python; dependencies live in a local venv.

```bash
./setup-evolve.sh
```

Creates `evolve/.venv` if missing and installs `requirements.txt`.

## Usage

Framework entry point is `evolve/scripts/evolve.py`. Run it through the project venv (all examples below assume `cd evolve/`):

```bash
.venv/bin/python scripts/evolve.py [flags]
```

Flags (defaults sourced from [`config.toml`](config.toml); flags override per-run):

| Flag                     | Config key                 | Purpose                                                                                |
| ------------------------ | -------------------------- | -------------------------------------------------------------------------------------- |
| `--cycles N`             | `framework.cycles`         | Number of cycles to run sequentially. Aborts on first failure.                         |
| `--cohort-size N`        | `framework.cohort_size`    | Number of variants per cycle.                                                          |
| `--ai-agent-tool <name>` | `framework.ai_agent_tool`  | AI agent CLI to drive subagents. Choices: `gemini`.                                    |
| `--outputs-root <path>`  | —                          | Read-write dir for cycles/ledger/logs (default: `outputs/`).                           |
| `--log-level <level>`    | `framework.log_level`      | Log verbosity. One of `DEBUG`, `INFO`, `WARNING`, `ERROR`, `CRITICAL`.                 |

Examples:

```bash
# Single cycle, defaults from config.toml
.venv/bin/python scripts/evolve.py

# Three cycles, cohort of 4
.venv/bin/python scripts/evolve.py --cycles 3 --cohort-size 4

# Override AI agent tool for one run
.venv/bin/python scripts/evolve.py --ai-agent-tool gemini
```

Each cycle ends with `cycle <NNNN> complete — sealed at <path> (cohort=N)`.

### Configuration

Defaults for every flag live in [`config.toml`](config.toml) under `[framework]`. Edit the file to change defaults persistently; pass the corresponding flag to override on a single run. Precedence: **flag > `config.toml` > built-in fallback**.
