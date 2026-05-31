# Evolve

Research framework that drives a cache-policy evolution loop. It runs **cycles**: each cycle proposes a **cohort** of hypotheses, generates one code variant per hypothesis, puts each variant through correctness **gates** and **fitness** scoring, records an insight per surviving variant and a summary insight per cycle, computes a Pareto front, and appends one row to an immutable ledger.

Subagents handle the judgment work (hypothesize, implement, write insights) through a pluggable AI-agent backend. Deterministic library code handles everything else — scaffolding, gates, scoring, sealing, and the ledger/leaderboard. Everything a run writes lands under a read-write outputs directory (gitignored); the repo itself stays unchanged.

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

The first run prompts you to select an AI-agent backend if one isn't configured — see [Configuration](#configuration).

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

Flags override the defaults; defaults live in [`config.toml`](config.toml), which documents every setting inline. A couple of examples:

```bash
# Single cycle
evolve --cycles 1

# Three cycles, cohort of four
evolve --cycles 3 --cohort-size 4
```

Each cycle ends with a `cycle <NNNN> complete` line naming the sealed cycle directory.

## Configuration

Defaults for every setting live in [`config.toml`](config.toml) — edit it to change them persistently, or pass the matching flag to override on a single run. Precedence is **flag > `config.toml`**: a setting with no flag and no config value (for example, the AI-agent backend, which has no default) aborts the run with an instructive message rather than guessing.

## License

Released under the [MIT License](LICENSE).

## Acknowledgements

Built as part of a master's thesis at the Technion — Israel Institute of Technology, under Prof. Roy Friedman.
