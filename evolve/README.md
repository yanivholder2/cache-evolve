# Evolve

The `evolve` directory contains scripts, tools, and configurations to manage the experimental lifecycle and evolution of the Dash Cache system. This includes managing research cycles, maintaining the insight ledger, tracking traces, synthesizing results, and running benchmarks.

## Environment Setup

The automation and scripts in this directory rely on Python. To ensure dependencies are managed cleanly, we use a local virtual environment (`.venv`).

To initialize the environment and install all necessary dependencies, run:

```bash
./setup.sh
```

This will automatically create a Python virtual environment at `evolve/.venv` (if it doesn't already exist) and install the packages specified in `evolve/requirements.txt`.

## Running with the Virtual Environment

To use the Python tools or run tests, call the binaries directly from the virtual environment:

```bash
.venv/bin/python scripts/ledger-append.py "your insight here"
.venv/bin/pytest tests/
```

## Directory Structure

To keep the repository clean, only the framework logic and source assets are persistent. Artifacts from a specific evolution run are ignored by Git.

```text
evolve/
  config.toml              # Framework configuration (action_space, budget, etc.)
  scripts/                 # Deterministic verbs (ledger-append, seal, etc.)
  tests/                   # Framework validation suite
  setup-evolve.sh          # Environment bootstrap script
  subject/                 # Stable evolution context
    extension-points.md    # Stable trait definitions
    invariants/            # Rust correctness gate suite
  traces/                  # Evaluation suite (Persistent)
    smoke.trace            # Small trace for G5 smoke-tests
    dev/                   # Traces visible to the AI
    holdout/               # Traces hidden from the AI
    *.profile.md           # Workload characteristics
  ledger/                  # APPEND-ONLY source of truth (Transient)
    cycles.jsonl           # One JSONL row per Cycle
    insights.jsonl         # Detailed variant insights
  cycles/NNNN/             # Immutable after sealing (Transient)
    cohort.md              # The Cohort's hypotheses and plan
    hypotheses/            # Detailed hypothesis definitions
    variants/vNNN/         # Per-variant artifacts
      impl/                # Generated Rust implementation
      attempts/NN/         # Self-repair iterations
      eval/                # Raw evaluation metrics and logs
      insight.md           # Qualitative variant insight
    cycle-insight.md       # Synthesis of the entire cohort
    winners.json           # Pareto-front variants for this cycle
  synthesis/               # RECOMPUTABLE narrative views (Transient)
    current.md             # Latest cross-cycle synthesis
    by-topic/              # Categorized research insights
  leaderboard.md           # Derived performance view (Transient)
  rejection-log.md         # G1/G2/G3 failure records (Transient)
  bench/                   # Rust micro-benchmark harness
  .claude/                 # Claude Code adapter (Planned)
    skills/                # Framework-specific skills
    agents/                # Subagent prompt definitions
```

## Usage

The `evolve/scripts/` directory provides utilities for managing your research.

- **Ledger Append:**
  Record a new insight to the active ledger:
  ```bash
  .venv/bin/python scripts/ledger-append.py "Observation about cache eviction..."
  ```
- **Seal:**
  Execute the shell script to seal a cycle:
  ```bash
  ./scripts/seal.sh
  ```

## Testing

Tests for the Python scripts and utilities are located in the `evolve/tests/` directory and use the `pytest` framework.

To execute the full test suite, use the virtual environment's `pytest`:

```bash
.venv/bin/pytest tests/
```