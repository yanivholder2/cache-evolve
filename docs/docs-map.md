# Docs map

Index of reference docs. Paths are relative to the repo root. Read on demand; don't preload.

## Domain

### [CONTEXT.md](../CONTEXT.md)

* Language
  * Cache structure
  * Strategies and variants
  * Item lifecycle and movement
  * Domain verbs (API level)
  * Policy vocabulary
* Relationships
* Framework vocabulary (evolve)
* Flagged ambiguities

**Read when** encountering unfamiliar domain terms or before any cache-structure change.

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

### [0003 — v1 single-machine, v2-ready invariants](adr/0003-v1-single-machine-v2-ready-invariants.md)

* Considered options
* Consequences

**Read when** adding parallelism or distributed features.

### [0004 — Formalize uniqueness invariant](adr/0004-formalize-uniqueness-invariant.md)

* What was done
* Considered options
* Consequences

**Read when** modifying insert, evict, or migration logic.

### [0005 — Agent-agnostic LLM CLI contract](adr/0005-agent-agnostic-llm-cli-contract.md)

* Considered options
* Consequences

**Read when** adding or changing any framework script that invokes an LLM (hypothesis-writer, variant-implementer, smoke-reviewer, insight-writer).

## Conventions

### [docs/CONVENTIONS.md](CONVENTIONS.md)

* Module
* Logging
* Variable naming

**Read when** writing or reviewing code style or commenting conventions.

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
