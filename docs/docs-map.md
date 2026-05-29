# Docs map

Index of reference docs. Paths are relative to the repo root. Read on demand; don't preload.

## READMEs

### [README.md](../README.md)

* Environment setup
* Usage (flags, examples)
* Baseline and per-cycle grounding
* Measuring the baseline
* Configuration
* Testing

**Read when** running the evolve framework, configuring a cycle, or running its test suite.

### [src/evolve/agents/README.md](../src/evolve/agents/README.md)

* Agent prompt convention (markdown body = system prompt)

**Read when** adding a new LLM-driven agent call site.

## Domain

### [CONTEXT.md](../CONTEXT.md)

Framework (evolve) vocabulary — subject-agnostic. Per-subject terms live in each subject's own glossary (below).

* Language
  * Roles and subjects
  * Cycle and cohort
  * Action space and extension points
  * Correctness gates
  * Insights and ledger
  * Evaluation
  * Architectural principles
* Flagged ambiguities

**Read when** encountering unfamiliar framework terms or before changing cycles, gates, scopes, manifests, or fitness.

### Subject glossaries — `inputs/subjects/<name>/CONTEXT.md`

Each Subject carries its own domain glossary:

* [dash](../inputs/subjects/dash/CONTEXT.md) — multi-segment associative cache with bucket migration.
* [associative_cache](../inputs/subjects/associative_cache/CONTEXT.md) — N-way associative baseline.
* [lru_cache](../inputs/subjects/lru_cache/CONTEXT.md) — classic LRU baseline.

**Read when** working on a specific subject crate.

### [docs/research-proposal.tex](research-proposal.tex)

Full thesis proposal: motivation, design, and initial hit-rate results.

**Read when** you need to understand research goals or justify a design decision.

## Architecture decisions

### [0001 — Append-only insight ledger](adr/0001-append-only-insight-ledger.md)

* Considered options
* Consequences

**Read when** modifying ledger writes or cycle output.

### [0002 — Token-frugal principle](adr/0002-token-frugal-principle.md)

* Considered options
* Consequences

**Read when** deciding whether to use LLM reasoning vs. deterministic computation.

### [0004 — Formalize uniqueness invariant](adr/0004-formalize-uniqueness-invariant.md)

* What was done
* Considered options
* Consequences

**Read when** modifying insert, evict, or migration logic.

### [0005 — Single generic seam](adr/0005-single-generic-seam.md)

* Considered options
* Consequences

**Read when** adding or swapping a Subject extension point, or touching `Dash`/`Segment`/JNI type parameters.

### [0006 — Overlay-by-path variants](adr/0006-overlay-by-path-variants.md)

* Considered options
* Consequences

**Read when** working on variant storage, the build harness, or how Variant edits are materialised.

### [0007 — Standalone subject crates](adr/0007-standalone-subject-crates.md)

* Considered options
* Consequences

**Read when** adding a Subject, touching `caffeine_runner.py`/manifest wiring, or wondering why `src/shared/` is duplicated per subject.

## Conventions

### [docs/CONVENTIONS.md](CONVENTIONS.md)

* Module
* Logging
* Variable naming

**Read when** writing or reviewing code.

## Agent workflow

### [docs/agents/domain.md](agents/domain.md)

* Before exploring, read these
* File structure
* Use the glossary's vocabulary
* Flag ADR conflicts

**Read when** writing a skill that navigates this repo.

### [docs/agents/issue-tracker.md](agents/issue-tracker.md)

* Conventions
* When a skill says "publish to the issue tracker"
* When a skill says "fetch the relevant ticket"

**Read when** creating, triaging, or closing issues.

### [docs/agents/pr-workflow.md](agents/pr-workflow.md)

* Leaving feedback on PRs
* Blocking a PR
* Branch protection

**Read when** reviewing or creating a PR.

### [docs/agents/triage-labels.md](agents/triage-labels.md)

Canonical triage role → label mapping.

**Read when** labelling issues or running the triage skill.
