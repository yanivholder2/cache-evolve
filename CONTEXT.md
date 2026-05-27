# Evolve Framework

> **This is a domain language file.** It records the canonical vocabulary for the evolve framework — preferred terms, terms to avoid, relationships, and flagged ambiguities. Before adding, renaming, or removing any term, read the format spec and editing rules at `~/.claude/skills/grill-with-docs/CONTEXT-FORMAT.md`. The `/grill-with-docs` skill is the primary tool for evolving this file.

Vocabulary for the **evolve** framework — the agentic research loop that studies Subject caches. Subject-level vocabulary lives in each subject's own `CONTEXT.md` (e.g. [`inputs/subjects/dash/CONTEXT.md`](inputs/subjects/dash/CONTEXT.md)).

## Language

### Roles and subjects

**Subject cache**:
The cache under study by the framework.
_Avoid_: Target cache, test cache, system under study.

**Framework / Evolve**:
The agentic research loop that proposes, validates, evaluates, and summarises
Variants of a Subject cache. Codename `evolve`.
_Avoid_: Tool, agent, pipeline.

### Cycle and cohort

**Cycle**:
One full iteration of the evolve loop. Numbered, immutable after sealing.
_Avoid_: Iteration, generation, round.

**Cohort**:
The set of Hypotheses proposed in one Cycle.
_Avoid_: Batch, group.

**Hypothesis**:
A claim about how the Subject cache might behave better, proposed by the
orchestrator and tested by one or more Variants.
_Avoid_: Idea, conjecture.

**Variant**:
A proposed alternative implementation of the Subject cache, embodying one
Hypothesis. The unit of evaluation; the leaderboard is defined over Variants,
not Hypotheses.
_Avoid_: Mutation, fork, candidate.

**Variant attempt**:
One numbered iteration within a Variant's self-repair sub-loop. Only the
final attempt's state enters the leaderboard.
_Avoid_: Revision, retry.

**Seal**:
Finalize a Cycle directory; immutable after sealing.
_Avoid_: Lock, close.

### Action space and extension points

**Action space**:
The set of changes a Variant is allowed to make.
_Avoid_: Search space, mutation space.

**Extension point**:
A stable interface in the Subject cache where Variants plug in.
_Avoid_: Hook, plugin slot.

### Correctness gates

**Gate**:
A check a Variant must pass before its fitness is admitted to the leaderboard.
Hard gates (build gate, contract gate, invariant gate) are
deterministic; soft gates (smoke gate) are AI-judged. A planned
differential gate is opt-in for refactor-shaped variants.
_Avoid_: Check, validator.

**Invariant suite**:
The codified correctness contract for the Subject cache (enforced by the
invariant gate); a Variant must satisfy it to be admitted to the leaderboard.
_Avoid_: Property tests, test suite.

**Smoke trace**:
A small canonical trace used by the smoke gate to produce a log for AI
behavioral review. Distinct from evaluation traces.
_Avoid_: Test trace, mini trace.

**Variant verdict**:
Qualitative AI judgment from the smoke gate's smoke-log review. Non-blocking
— a negative verdict triggers the self-repair sub-loop but does not suppress
fitness numbers.
_Avoid_: Review, opinion.

**Self-repair sub-loop**:
Bounded retry loop inside the smoke gate; the AI re-implements a Variant
when the smoke-log review flags a likely bug. Same Variant ID across attempts.
_Avoid_: Fix loop, retry chain.

**Rejection log**:
Per-Variant durable record of hard-gate failures.
Failed Variants are evidence the AI can learn from, not silent drops.
_Avoid_: Failures, error log.

### Insights and ledger

**Variant insight**:
Per-Variant AI-written explanation of why a Variant behaved as it did. Tags
which Fitness dimensions the Variant helped or hurt.

**Cycle insight**:
Per-Cycle AI-written summary across the Cohort's Variant insights. The
canonical record of what the framework learned in one Cycle.

**Insight ledger**:
Append-only record of Cycle insights, and the single cross-cycle source of
truth. See [ADR-0001](docs/adr/0001-append-only-insight-ledger.md).
_Avoid_: Log, history, archive.

**Cross-cycle insights**:
Recomputable narrative across cycles derived from the Insight ledger. Rewritten
freely; never the source of truth.
_Avoid_: Summary, distillation, synthesis, bare "Insights".

### Evaluation

**Trace profile**:
Description of a Trace's workload characteristics (skew, working-set size,
scan/hot-set behavior).
_Avoid_: Trace doc, workload description.

**Development suite**:
Traces visible to the AI during evaluation.
_Avoid_: Training set, dev set.

**Held-out suite**:
Additional traces hidden from the AI; run only on Pareto-front Variants to
catch overfitting to the Development suite.
_Avoid_: Test set, validation set.

**Fitness vector**:
Multi-dimensional measurement of a Variant's quality. Plural by design —
fitness is never a single scalar.
_Avoid_: Score, fitness value (singular implies scalar).

**Pareto front**:
The set of non-dominated Variants in a Cycle's Fitness vectors. The Cycle's
winners — typically several, not one.
_Avoid_: Leaderboard, top-K.

### Architectural principles

**Token-frugal principle**:
Anything that doesn't require LLM judgment is a script, not an LLM call. See
[ADR-0002](docs/adr/0002-token-frugal-principle.md).
_Avoid_: Token budget (means a numeric cap; the principle is architectural).

**Path invariant**:
Every written artifact uses repo-relative paths only, so sealed cycle
directories are reproducible across checkouts. Enforced in
`src/evolve/cycle/ledger_append.py` (`_validate`) and `src/evolve/paths.py`.
_Avoid_: Distributed prerequisites, cluster invariants.

## Flagged ambiguities

- **"Variant"** (Evolve sense) — a proposed alternative implementation of a Subject cache. Overloaded with the Dash domain's compound-noun usage (**Hot variant**, **Cold variant**) — see [`inputs/subjects/dash/CONTEXT.md`](inputs/subjects/dash/CONTEXT.md). Qualify with the domain when both are in scope.
- **"Insight"** — bare "insight" is ambiguous within the framework. Always qualify: **Variant insight** (per-variant) or **Cycle insight** (per-cycle summary). The **Insight ledger** is the durable record; **Cross-cycle insights** is the recomputable narrative across cycles.
