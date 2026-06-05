# Evolve

**Evolve** is an agentic framework that automatically improves caches. Starting from an existing design, it searches for better variants under two regimes: **refinement** over a fixed workload suite, and **workload specialization** against a deployment-specific trace distribution.

The search runs in **cycles**. Each cycle proposes a **cohort** of LLM-implemented variants, validates them through deterministic correctness gates, scores the survivors on a multi-dimensional fitness vector, and retains the Pareto front of the best trade-offs. Insights generated along the way are kept in a knowledge ledger that guides later exploration.

For the concepts, the end-to-end flow, the simulator setup, and per-subject details, see the **[project wiki](../../wiki)**.

## Environment setup

### With uv (recommended)

Dependencies are managed by [uv](https://github.com/astral-sh/uv) and installed into a local `.venv/`:

```bash
uv sync
```

This creates `.venv/` and installs the project declared in `pyproject.toml`.

### Without uv

Any standard virtual environment works:

```bash
python -m venv .venv
source .venv/bin/activate      # Windows: .venv\Scripts\activate
pip install -e .
```

### Caffeine simulator (optional)

Subjects are scored by the [Caffeine](https://github.com/ben-manes/caffeine) cache simulator over JNI. To set up its prerequisites — a Caffeine checkout (a `caffeine/` sibling of this repo, or `$CAFFEINE_DIR`) and a JDK 25 (Caffeine's Gradle toolchain floor) — run:

```bash
scripts/setup-caffeine.sh
```

The script is idempotent: an existing checkout and JDK 25 are reused. If no JDK 25 is found it prints how to install one; pass `--install-jdk` to let it `apt-get install openjdk-25` for you. In a Claude Code web session this runs automatically via the SessionStart hook (`.claude/hooks/session-start.sh`), so a fresh remote session is ready to run without any manual setup.


## Usage

The framework entry point is the `evolve` console script:

```bash
uv run evolve            # with uv
evolve                   # inside an activated venv
```

Equivalent: `python -m evolve.cli`.

Run with `--help` to see every flag and what it does:

```bash
evolve --help
```

A couple of examples:

```bash
# Single cycle
evolve --cycles 1

# Three cycles, cohort of four
evolve --cycles 3 --cohort-size 4
```

Each cycle ends with a `cycle <NNNN> complete` line naming the sealed cycle directory.

### One-shot runs via the `evolve-run` skill

Inside an AI agent session, the **`evolve-run`** skill kicks off a run from a plain-language description, then reads the whole output folder and reports what happened. Describe the run in natural language — the skill translates it to non-persistent CLI flags and never edits `config.toml`:

> evolve dash for 3 cycles, four variants each, full autonomy

It runs pre-flight checks (venv, Caffeine, JDK 25, `cargo build`), launches the run in the background, then summarises the winners and hit-rate trend, any failures with their root cause, and logging or quality issues worth a look.

## Inputs and outputs

Evolve reads a **subject** — the cache implementation to evolve — from `inputs/subjects/<name>/`. Which subject to run is a setting like any other (see [Configuration](#configuration)). The repo ships several (`dash`, `associative_cache`, `lru_cache`); each is a self-contained crate with its own manifest and measured baseline. See the [wiki](../../wiki/Subjects) for what a subject is and how to point at your own.

Everything a run produces lands under the **outputs** directory (default `outputs/<timestamp>/`, gitignored): the per-cycle artifacts, the cross-cycle ledger and leaderboard, and the logs. The subject and the rest of the repository are never modified by a run.

## Configuration

Defaults for every setting live in [`config.toml`](config.toml) — edit it to change them persistently, or pass the matching flag to override on a single run. Precedence is **flag > `config.toml`**: a setting with no flag and no config value (for example, the AI-agent backend, which has no default) aborts the run with an instructive message rather than guessing.

## License

Released under the [MIT License](LICENSE).

## Acknowledgements

Built as part of a master's thesis at the Technion — Israel Institute of Technology, under Prof. Roy Friedman.
