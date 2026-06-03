# Evolve Framework

> **This is a domain language file.** It records the canonical vocabulary for the evolve framework — preferred terms, terms to avoid, relationships, and flagged ambiguities. Before adding, renaming, or removing any term, read the format spec and editing rules at `~/.claude/skills/grill-with-docs/CONTEXT-FORMAT.md`. The `/grill-with-docs` skill is the primary tool for evolving this file.

Vocabulary for the **evolve** framework — the agentic research loop that studies Subject caches. Subject-level vocabulary lives in each subject's own `CONTEXT.md` under `inputs/subjects/<name>/`.

## Language

### Roles and subjects

**Subject**:
A complete cache implementation that satisfies the Cache contract, packaged
with its Subject manifest, domain docs, baseline insight, and baseline Fitness
vector under `inputs/subjects/<name>/`. The unit the framework evolves; the
framework is subject-agnostic and can host several.
_Avoid_: Target, system under study.

**Subject cache**:
The runtime cache an evolution studies — the code inside a Subject. Use
**Subject** for the packaged directory entity, **Subject cache** for the cache
itself.
_Avoid_: Target cache, test cache.

**Framework / Evolve**:
The agentic research loop that proposes, validates, evaluates, and summarises
Variants of a Subject. Codename `evolve`.
_Avoid_: Tool, agent, pipeline.

**Framework crate**:
The Rust crate at `framework-rs/` (crate name `framework`) that publishes the
Cache contract. Distinct from the Python `evolve` package under `src/evolve/`.
_Avoid_: Core crate, shared crate.

**Cache contract**:
The `Cache<K, V>` trait plus the `Invariants` trait, published by the Framework
crate. The stable surface every Subject implements and every Variant must keep
satisfying; the framework only ever calls a Variant through it.
_Avoid_: Cache API, interface.

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
A proposed alternative implementation of a Subject — a whole cache
implementation shipped as an Overlay over the Subject tree — embodying one
Hypothesis. The unit of evaluation; the leaderboard is defined over Variants,
not Hypotheses. The framework calls into it only through the Cache contract.
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
The set of changes a Variant is allowed to make, fixed per run by the run's
authoring scope (Extension-point scope or Full-autonomy scope).
_Avoid_: Search space, mutation space.

**Extension point**:
A trait-object seam below the Segment where a Variant can swap an
implementation (eviction policy, bucket, item, probing function, cascade
order, …). Each Subject declares its extension points in its Subject manifest.
A Subject may declare zero (e.g. lru_cache), in which case Variants run in
Full-autonomy scope only.
_Avoid_: Hook, plugin slot.

**Subject manifest**:
The `manifest.toml` at a Subject's root declaring its identity (name, crate
path, cdylib name), its Extension points, its Protected files (`[contract]`),
its Caffeine wiring (`[caffeine]`), an optional microbench (`[bench]`), and
where its docs and baseline evaluation live. The framework unions the Extension
points' impl files to compute the Extension-point scope whitelist.
_Avoid_: Config, spec, descriptor.

**Protected files**:
Files a Variant may never overwrite, declared in the Subject manifest
`[contract]` table — the JNI boundary and the variant factory seam. Enforced
regardless of authoring scope; even Full-autonomy scope stops at them.
_Avoid_: Frozen files, locked files.

**Extension-point scope**:
Authoring scope where a Variant edits only files declared as Extension-point
impls in the Subject manifest; trait surfaces stay fixed.
_Avoid_: Restricted scope, narrow scope.

**Full-autonomy scope**:
Authoring scope where a Variant edits any file in the Subject tree, bounded
only by the Cache contract and the invariant suite.
_Avoid_: Open scope, free scope.

**Overlay**:
A Variant's edits stored as an `overlay/` tree mirroring the Subject, holding
only added or modified files. The build harness copies the Subject tree and
overlays these on top — no deletions. A cycle Variant's overlay lives under
`attempts/NN/overlay/` (one per Variant attempt); the baseline and the `v<NNN>`
template keep a flat `overlay/`. Eval artifacts (fitness, Caffeine output) live
under the Variant root's `eval/`. Also "overlay-by-path".
_Avoid_: Patch, diff, fork.

### Correctness gates

**Gate**:
A check a Variant must pass before its fitness is admitted to the leaderboard.
Hard gates (build gate, contract gate, universal invariant gate, invariant
gate) are deterministic; soft gates (smoke gate) are AI-judged. A planned
differential gate is opt-in for refactor-shaped variants.
_Avoid_: Check, validator.

**Universal invariant gate**:
A framework-level (subject-agnostic) hard gate that runs the variant's
`make_cache` through universal cache laws known only via `Cache<K, V>` (no
observable phantom item; a put key reads back its value or `None`; a never-put
key reads `None`). Binds to the variant's actual cache, so a factory-only
overlay cannot escape it. Sees only observable violations, not structural ones.
_Avoid_: Generic gate, base gate.

**Invariant suite**:
The codified Subject-structural correctness contract for the cache (enforced by
the invariant gate); a Variant must satisfy it to be admitted to the
leaderboard. Distinct from the universal invariant gate's subject-agnostic laws.
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
Bounded re-implementation loop: when a gate rejects a Variant, the implementer
is re-invoked with the failing gate's rejection embedded in the next attempt's
task, up to `implementation_attempts` total attempts (initial + repairs). Any
gate failure triggers it — build, contract, universal invariant, or a subject
invariant gate. Same Variant ID across attempts; only the final attempt's state
enters the leaderboard. On exhaustion the Variant seals with reason
`gate:<name> (N/M attempts)`.
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
Additional traces hidden from the AI; run only on the Cycle's Winners to
catch overfitting to the Development suite.
_Avoid_: Test set, validation set.

**Fitness vector**:
Multi-dimensional measurement of a Variant's quality. Plural by design —
fitness is never a single scalar.
_Avoid_: Score, fitness value (singular implies scalar).

**Winners**:
A Cycle's selected Variants — typically several, not one — chosen by the run's
**Selection method**. Under the default `pareto` method a Variant **dominates**
another when it is at least as good on every dimension and strictly better on
one, and the Winners are the set nothing else dominates (the **Pareto front**).
Under the `weighted` method the Winners are the top-N by **Score**.
_Avoid_: Best variant (singular implies one winner).

**Selection method**:
The pluggable rule that turns a cohort's Fitness vectors into Winners, resolved
by name from the selector registry and configured under `[selection]` in
`config.toml`. Two built-ins: `pareto` (the Pareto front; the default) and
`weighted` (top-N by Score). One method serves both the cohort-local and the
cross-cycle stage.
_Avoid_: Selection strategy, ranking, objective.

**Weights**:
Per-dimension multipliers the `weighted` Selection method applies to the
min–max-normalized Fitness dimensions before summing them into a Score. Keys
must match the active Fitness dimensions exactly; absent under `pareto`.
_Avoid_: Coefficients, priorities.

**Score**:
The scalar a Selection method assigns a Variant — under `weighted`, the weighted
sum of its min–max-normalized dimensions. Cohort/union-relative, so it ranks
within one selection but is not comparable across selections. The `pareto`
method computes no Score.
_Avoid_: Fitness (Fitness is the multi-dimensional measurement; a Score is the
scalar a method derives from it), fitness value.

**Cross-cycle winners**:
The Winners recomputed over every Variant evaluated across all Cycles plus the
baseline — the global best-so-far, refreshed each Cycle. Persisted to
`cross-cycle-winners.json`; per-Cycle Winners live in `cycle-winners.json`.
_Avoid_: Champions.

### Architectural principles

**Single generic seam principle**:
The Segment trait is the only generic seam in a Subject; every other Extension
point is a trait object owned by the smallest struct that uses it, so no type
parameter propagates above the Segment. Adding an axis is a local change, not a
cross-cutting refactor. See [ADR-0005](docs/adr/0005-single-generic-seam.md).
_Avoid_: Generic-everywhere, type-parameter-per-axis.

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

- **"Subject"** vs **"Subject cache"** — **Subject** is the packaged directory entity (`inputs/subjects/<name>/`: cache code + manifest + docs + baseline eval); **Subject cache** is the runtime cache code inside it. Prefer **Subject** unless the cache artifact itself is specifically meant.
- **"Variant"** — at the framework level this always means the Evolve sense: a proposed alternative implementation of a Subject. A Subject may carry its own narrower, retired usages; those are documented in that subject's own `CONTEXT.md`.
- **"Insight"** — bare "insight" is ambiguous within the framework. Always qualify: **Variant insight** (per-variant) or **Cycle insight** (per-cycle summary). The **Insight ledger** is the durable record; **Cross-cycle insights** is the recomputable narrative across cycles.
