# Concepts

The vocabulary of the evolve framework, in one place. This page is self-contained; the canonical glossary is [`CONTEXT.md`](https://github.com/yanivholder2/cache-evolve/blob/main/CONTEXT.md) in the repo, and each subject carries its own glossary under `inputs/subjects/<name>/CONTEXT.md`.

## Subject

The cache implementation under evolution. A subject is a self-contained Rust crate plus a `manifest.toml` that declares its editable seams, its correctness tests, its protected files, and how it plugs into the simulator. You pick the subject with `--subject`. See [Subjects](Subjects).

## Cycle

One full iteration of the evolution loop. A cycle proposes a cohort, evaluates every variant, records insights, computes a Pareto front, appends a ledger row, and is then *sealed* (finalized and made read-only). Cycles run sequentially; a failing cycle aborts the run.

## Cohort

The set of variants produced in a single cycle. `--cohort-size` sets how many. The agent proposes one *hypothesis* per intended cohort member.

## Hypothesis

A proposed idea for improving the subject, authored by the AI agent at the start of a cycle. Each hypothesis names the change it intends to make. On continuation cycles, at least one hypothesis must build on a current Pareto champion so the search keeps exploiting what already works.

## Variant

The concrete code realization of one hypothesis. A variant is stored as an **overlay** — a small set of files that override the subject's source — so the subject crate itself is never mutated. Multiple implementation attempts may be made per variant (see *Self-repair* below); each attempt's overlay is preserved.

## Extension points and authoring scope

A subject declares **extension points**: the specific code seams a variant is allowed to edit (for example, an eviction-policy seam or a bucket seam). The `--authoring-scope` flag controls the write surface:

- **`extension-points`** — the agent may only edit files belonging to declared extension points.
- **`full-autonomy`** — the agent may edit any overlay file, still bound by the cache contract and the subject's invariants.

Certain files are **protected** (e.g. the JNI boundary) and can never be overlaid, in either scope — they host the very checks that keep a variant honest.

## Gates

Deterministic correctness checks every variant must pass, run in a fixed, fail-fast order. The three framework gates are:

1. **Build gate** — the overlaid workspace must compile.
2. **Contract gate** — the variant must satisfy the cache `Cache<K,V>` contract test.
3. **Universal invariant gate** — the variant must uphold the framework's universal cache invariants.

A subject may append its own gates after these three. The first gate to fail stops the sequence and records why.

## Self-repair

When a gate (or the implementer's own validation) rejects an attempt, the framework re-prompts the agent with the exact failure and lets it try again, up to a bounded attempt budget (`implementation_attempts`, default 3 = 1 initial + 2 repairs). If the budget is exhausted the variant is **softly rejected**: it is dropped from the cohort while the other variants continue. A cycle with zero survivors aborts.

## Fitness

How a surviving variant is scored. Each variant is run through the cache simulator, and one or more **fitness plugins** turn the raw metrics into named **dimensions** (e.g. hit rate). The per-variant result is a **fitness vector** — a list of dimensions, each with a single comparable summary number and an orientation (higher-is-better or not). See [Evaluation and Fitness](Evaluation-and-Fitness).

## Pareto front and champions

Because fitness can be multi-dimensional, "best" is not a single number. A variant **dominates** another if it is at least as good on every dimension and strictly better on one. The **Pareto front** is the set of non-dominated variants:

- The **cohort Pareto front** (`winners.json`) is computed over the survivors of one cycle.
- The **global champions** (`champions.json`) are the Pareto front over every variant ever evaluated, plus the baseline — recomputed each cycle.

## Insights and the ledger

The framework keeps a permanent, append-only record:

- A **variant insight** (agent-authored prose) per surviving variant.
- A **cycle insight** summarizing each cohort.
- A **cross-cycle insight** narrating progress across cycles.
- The **ledger** (`cycles-ledger.jsonl`) — one immutable row per cycle.
- The **leaderboard** (`leaderboard.md`) — regenerated each cycle from the winners and champions.

The append-only design is an explicit architecture decision; see [ADR 0001](https://github.com/yanivholder2/cache-evolve/blob/main/docs/adr/0001-append-only-insight-ledger.md).

## Baseline

A subject's checked-in measurement anchor: the subject as-is, with its measured fitness vector and a short prose description. Cycle-1 hypotheses ground their expectations in the baseline numbers. See [Baseline](Baseline).
