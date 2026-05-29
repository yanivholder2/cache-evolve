# Evolve

Research framework that drives the cache-experiment loop. It runs **cycles** of cache-policy evolution: each cycle proposes a **cohort** of hypotheses, generates one Rust variant per hypothesis, runs correctness gates (build gate, contract gate, invariant gate) plus fitness plugins (e.g. hit-rate), writes one LLM-authored Variant insight per surviving Variant, writes one Cycle insight summarising the cohort, computes a Pareto front, and appends one row to the immutable ledger.

Subagents handle judgment work (hypothesize, implement, write variant insight, write cycle insight) via in-process adapters. Library modules under `src/evolve/` handle everything deterministic — scaffolding, gates, scoring, sealing, ledger/leaderboard maintenance. Everything a run writes lands under `outputs/` (gitignored): cycle artifacts at `outputs/cycles/NNNN/` (sealed once complete), plus `outputs/cycles-ledger.jsonl` and `outputs/leaderboard.md` as the persistent across-cycle views.

## Environment Setup

Dependencies are managed by [uv](https://github.com/astral-sh/uv) and installed into a local `.venv/`:

```bash
uv sync
```

This creates `.venv/` and installs the project (declared in `pyproject.toml`).

## Usage

Framework entry point is the `evolve` console script (installed by `uv sync`):

```bash
uv run evolve [flags]
```

Equivalent: `uv run python -m evolve.cli [flags]` (or `.venv/bin/python -m evolve.cli`).

Flags (defaults sourced from [`config.toml`](config.toml); flags override per-run):

| Flag                      | Config key                | Purpose                                                                        |
| ------------------------- | ------------------------- | ------------------------------------------------------------------------------ |
| `--cycles N`              | `framework.cycles`        | Number of cycles to run sequentially. Aborts on first failure.                 |
| `--cohort-size N`         | `framework.cohort_size`   | Number of variants per cycle.                                                  |
| `--ai-agent-tool <name>`  | `framework.ai_agent_tool` | AI agent backend to drive subagents. Choices: `antigravity-sdk`, `claude-sdk`. |
| `--outputs-root <path>`   | —                         | Read-write dir for cycles, ledger, and logs (default: `outputs/`).             |
| `--log-level <level>`     | `framework.log_level`     | Log verbosity. One of `DEBUG`, `INFO`, `WARNING`, `ERROR`, `CRITICAL`.         |
| `--simulator-timeout-s N` | `simulator.timeout_s`     | Per-variant simulator invocation timeout (seconds). Surfaces as `FitnessError`.|
| `--max-workers N`         | `framework.max_workers`   | Worker threads for per-variant fan-out (0 = match cohort size).                |

Examples:

```bash
# Single cycle, defaults from config.toml
uv run evolve

# Three cycles, cohort of 4
uv run evolve --cycles 3 --cohort-size 4

# Override AI agent tool for one run
uv run evolve --ai-agent-tool antigravity-sdk
```

Each cycle ends with `cycle <NNNN> complete — sealed at <path> (cohort=N)`.

### Baseline and per-cycle grounding

Each subject carries a checked-in permanent research baseline under `inputs/subjects/<name>/baseline/`. The baseline implementation is the subject crate's own source; the baseline directory holds its measured grounding:

```
inputs/subjects/<name>/baseline/
  eval/fitness-vector.json  # measured fitness, produced by measure_baseline.py
  insight.md                # short prose describing the baseline
```

### Measuring the baseline

`fitness-vector.json` is a checked-in artefact. Re-measure (and commit the diff) whenever bench config or fitness plugins change:

```bash
uv run python scripts/measure_baseline.py
```

### Configuration

Defaults for every flag live in [`config.toml`](config.toml) under `[framework]`. Edit the file to change defaults persistently; pass the corresponding flag to override on a single run. Precedence: **flag > `config.toml`** — if neither is set, the run aborts.

## Testing

Tests live in `tests/`. Run from the repo root:

```bash
# Fast suite
uv run pytest
```

Markers (defined in `tests/conftest.py`):

| Marker            | Skipped by default unless  | Use for                                                         |
| ----------------- | -------------------------- | --------------------------------------------------------------- |
| `slow`            | `--slow` passed            | End-to-end tests that drive a real `cargo` build.               |
| `caffeine_needed` | `--caffeine-needed` passed | Tests requiring ~/projects/caffeine and a JDK. Never run in CI. |

A session-scoped fixture puts `tests/_mocks/` first on `PATH` so the fast suite never invokes a real `cargo`.

`--slow` and `--caffeine-needed` cannot be combined: slow tests assume the caffeine stub is active, which `--caffeine-needed` disables. Passing both aborts collection.

## License

Released under the [MIT License](LICENSE).
