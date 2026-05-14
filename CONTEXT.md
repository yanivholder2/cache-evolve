# Dash Cache

> **This is a domain language file.** It records the canonical vocabulary for this codebase — preferred terms, terms to avoid, relationships, and flagged ambiguities. Before adding, renaming, or removing any term, read the format spec and editing rules at `~/.claude/skills/grill-with-docs/CONTEXT-FORMAT.md`. The `/grill-with-docs` skill is the primary tool for evolving this file.

Associative cache for M.Sc. thesis research on mitigating the hot-set problem in N-way associative caches using Dash-inspired bucket movement, where items migrate between adjacent buckets to reduce contention in overloaded sets.

## Language

### Cache structure

**Dash**:
Multi-segment associative cache that lets items migrate between adjacent buckets to mitigate the hot-set problem.
_Avoid_: Cache, map

**Segment**:
Independent partition of the Dash; each segment owns its own normal buckets and stash.
_Avoid_: Shard, partition

**SegmentCore**:
Shared state (normal buckets + stash buckets) inside every segment variant.

**Bucket**:
Fixed-capacity ordered collection of items within a segment.
_Avoid_: Set, way

**Normal bucket**:
A non-stash bucket; plays the role of target for some keys and probing for their neighbors.
_Avoid_: Data bucket

**Stash**:
Secondary bucket group used as overflow / second-chance storage.
_Avoid_: Overflow, L2

**Item**:
Key-value pair stored in a bucket, plus policy metadata (timestamp, LFU counter).
_Avoid_: Entry, record

**Slot**:
Logical destination for an item: one of *Stash*, *Target*, *Probing* — derived from the item's key, never an arbitrary bucket index.
_Avoid_: Destination, bucket index

**Target bucket**:
The normal bucket an item hashes to.
_Avoid_: Home bucket

**Probing bucket**:
The neighbor of an item's target bucket: `(target + 1) % n`; overflow destination.
_Avoid_: Probe bucket, neighbor

### Strategies and variants

**Promotion strategy**:
Items earn their way into the probing bucket only after proving they're the hottest in their target.
_Avoid_: Position-based LRU, tiered strategy

**Combined strategy**:
Target and probing are peer storage; eviction picks the weakest candidate across both.
_Avoid_: Combined eviction, peer strategy

**Cold variant**:
Promotion sub-variant where stash→target insert lands at the LRU end (item must climb).

**Hot variant**:
Promotion sub-variant where stash→target insert lands at the MRU end (one more access promotes).

### Item lifecycle and movement

**Put**:
Insert a new item into the cache (default routes to stash).
_Avoid_: Insert, add

**Get**:
Look up an item; may mutate state per eviction policy and per strategy (promotion / touch-in-place).
_Avoid_: Fetch, read

**Evict**:
Remove an item from a bucket to make room, selected by the eviction policy.
_Avoid_: Kick, drop

**Hottest item**:
The bucket member most protected from eviction, per policy (index returned by `promotion_candidate_index`).
_Avoid_: Top item, MRU item

**Eviction candidate**:
The bucket member that would be evicted next, per policy.
_Avoid_: Victim, LRU item

**Promote**:
Move an item to a higher tier (stash→target or target→probing); promotion strategy only.

**Demote**:
Move an item to a lower tier (probing→target→stash); promotion strategy only.

**Cascade**:
Chain of evictions triggered by a single put (e.g., promote_to_probing evicts probing victim → cascades into target → may cascade into stash).

**Demotion chain**:
Promotion strategy's tier order: `probing → target → stash → out`.

**Peer storage**:
Combined strategy's view of target and probing: non-tiered; evictions go straight to stash.

**Tier drop**:
Single step down the demotion chain (probing→target or target→stash).

### Domain verbs (API level)

**promote_to_target**:
Promotion verb: move an item into its target bucket at a given position; evicted item drops to stash.

**promote_to_probing**:
Promotion verb: move an item into its probing bucket; evicted item cascades through the demotion chain.

**place_in_normal_bucket**:
Combined verb: place an item into its target or probing (caller's choice); evicted item drops straight to stash.

**put_in_stash** (implicit, via `SegmentCore::put(Slot::Stash, …)`):
Route a new item into its stash bucket; stash overflow is terminal (item permanently lost).

### Policy vocabulary

**Eviction policy**:
Per-bucket rule for ordering items and picking the eviction candidate.
_Avoid_: Replacement policy

**ClassicLRU**:
Hit moves item to back; evicts from front.
_Avoid_: LRU (ambiguous)

**TimestampLRU**:
Hit updates item's timestamp; evicts min-timestamp item.
_Avoid_: Timestamp LRU

**LFU**:
Hit increments item's counter; evicts min-counter item.

**FIFO / LIFO / Swap**:
Secondary policies used mostly for sweep comparison.

**MRU position**:
The end of a bucket's order that holds the hottest item under position-based policies (ClassicLRU/Fifo/Swap).
_Avoid_: Top

**LRU position**:
The end of a bucket's order that holds the next eviction candidate under position-based policies.
_Avoid_: Bottom, front

## Relationships

- A **Dash** contains one or more **Segments**.
- A **Segment** contains a fixed number of **Normal buckets** and a fixed number of **Stash buckets**, all held in its **SegmentCore**.
- An **Item**'s **Slot** (*Stash*, *Target*, *Probing*) is derived from its key — never chosen by the caller as a raw index.
- A **Target bucket** for key `k` in bucket `i` is simultaneously the **Probing bucket** for keys hashing to `i-1`. Ownership is resolved by re-hashing the victim's key on eviction.
- In the **Promotion strategy**, target and probing form a **tier** (probing > target > stash). A **Cascade** obeys the **Demotion chain**.
- In the **Combined strategy**, target and probing are **Peer storage**. There is no tier; evictions go straight to stash.
- Only **Promotion** uses the verbs **Promote** / **Demote**. Only **Combined** uses the verb **Place** (peer placement).

## Example dialogue

> **Dev:** "If an **Item** hits in its **Target bucket** and it's the **Hottest item**, what happens?"
>
> **Domain expert:** "Under the **Promotion strategy**, you call **promote_to_probing**. The item moves into its **Probing bucket**. If that bucket was full, its **Eviction candidate** becomes a victim and starts a **Cascade** down the **Demotion chain**."
>
> **Dev:** "So that victim lands in its own **Target bucket**?"
>
> **Domain expert:** "Right — a **Tier drop** from probing to target. If the target bucket is also full, another **Tier drop** sends that item to its **Stash**. Stash overflow is terminal."
>
> **Dev:** "And for the **Combined strategy**?"
>
> **Domain expert:** "No **Cascade**. You call **place_in_normal_bucket**. Target and probing are **Peer storage**, so whatever gets evicted goes straight to stash. Cascading would undo the weakness comparison that combined just did."
>
> **Dev:** "Which **Slot** does the caller pass to **place_in_normal_bucket**?"
>
> **Domain expert:** "*Target* or *Probing* — never *Stash*. Combined never places new items into the stash directly; that's only for **Put** of brand-new items via `Segment::put`."

## Framework vocabulary (evolve)

Vocabulary for the **evolve** framework — the agentic research loop that studies
Subject caches. Distinct from the Dash domain above. Framework is not yet
implemented; this section exists ahead of code so the upcoming work speaks a
consistent language. When `evolve/` is scaffolded, this section should migrate
to `evolve/CONTEXT.md` and the root will become `CONTEXT-MAP.md`.

### Roles and subjects

**Subject cache**:
The cache under study by the framework. v1: the Dash crate.
_Avoid_: Target cache, test cache, system under study.

**Framework / Evolve**:
The agentic research loop that proposes, validates, evaluates, and synthesizes
Variants of a Subject cache. Codename `evolve`.
_Avoid_: Tool, agent, pipeline.

### Cycle and cohort

**Cycle**:
One full iteration: read ledger digest, propose Cohort, implement Variants,
gate, evaluate, synthesize, append. Numbered, immutable after sealing.
_Avoid_: Iteration, generation, round.

**Cohort**:
The set of Hypotheses proposed in one Cycle. v1 default size: 6.
_Avoid_: Batch, group.

**Hypothesis**:
A claim about how the Subject cache might behave better, proposed by the
orchestrator and tested by one or more Variants.
_Avoid_: Idea, conjecture.

**Variant**:
A proposed alternative implementation of the Subject cache, embodying one
Hypothesis. Variants are the unit of evaluation; the leaderboard is defined
over Variants, not Hypotheses.
_Avoid_: Mutation, fork, candidate.

**Variant attempt**:
One numbered iteration within a Variant's self-repair sub-loop (G5). Up to 3
attempts per Variant; only the final attempt's state enters the leaderboard.
_Avoid_: Revision, retry.

**Seal**:
Finalize a Cycle directory; immutable after sealing.
_Avoid_: Lock, close.

### Action space and extension points

**Action space**:
The set of changes a Variant is allowed to make. Configurable: Level 3
(bounded code mutation through stable Extension points) or Level 4 (open-ended
code rewriting). Levels 1–2 (hyperparameter sweep) are explicitly out of scope.
_Avoid_: Search space, mutation space.

**Extension point**:
A stable interface in the Subject cache where Variants plug in. Initial set
for Dash: EvictionPolicy trait, Segment trait, probing function,
cascade/demotion-chain order.
_Avoid_: Hook, plugin slot.

### Correctness gates

**Gate**:
A check a Variant must pass (G1 build, G2 trait contract, G3 invariant suite)
or pass-with-judgment (G5 smoke-log review) before its fitness is admitted to
the leaderboard. Hard gates are deterministic; soft gates are AI-judged.
_Avoid_: Check, validator.

**Invariant suite**:
The codified correctness contract for the Subject cache (G3). Encodes
properties such as no-phantom-items, capacity bound, Slot legality,
single-eviction-per-insert modulo cascade, and stash-terminal.
_Avoid_: Property tests, test suite.

**Smoke trace**:
A small, canonical, framework-owned trace used by G5 to produce a manageable
log for AI behavioral review. Distinct from dev/held-out evaluation traces.
_Avoid_: Test trace, mini trace.

**Variant verdict**:
Qualitative AI judgment from G5 smoke-log review. Non-blocking — a "looks
buggy" verdict triggers the self-repair sub-loop but does not suppress fitness
numbers.
_Avoid_: Review, opinion.

**Self-repair sub-loop**:
Bounded retry loop inside G5; the AI re-implements a Variant up to 3 times
when the smoke-log review flags a likely bug. Same Variant ID across attempts.
_Avoid_: Fix loop, retry chain.

**Rejection log**:
Durable record of Variants that failed a hard gate (G1, G2, G3). Failed
Variants are evidence the AI can learn from, not silent drops.
_Avoid_: Failures, error log.

### Insights and ledger

**Variant insight**:
Per-Variant AI-written explanation of why a Variant behaved as it did. Tags
which Fitness dimensions the Variant helped or hurt.

**Cycle insight**:
Per-Cycle AI-written synthesis across the Cohort's Variant insights. The
canonical record of what the framework learned in one Cycle.

**Insight ledger**:
Append-only JSONL of Cycle insights. Single cross-cycle source of truth.
Atomic-append enforced; never rewritten.
_Avoid_: Log, history, archive.

**Synthesis**:
Recomputable cross-cycle narrative; rewritten freely by the cross-cycle
synthesizer subagent. Derived from the Insight ledger; never the source of
truth.
_Avoid_: Summary, distillation.

### Evaluation

**Trace profile**:
Per-trace human-and-AI-authored Markdown description of workload
characteristics (skew, working-set size, scan/hot-set behavior). Created once
per Trace, signed off by the maintainer, then immutable for the run.
_Avoid_: Trace doc, workload description.

**Development suite**:
Traces visible to the AI during evaluation. v1: ~4–6 traces, 3 cache sizes
each.
_Avoid_: Training set, dev set.

**Held-out suite**:
Additional traces hidden from the AI. Run only on Pareto-front Variants.
Catches AI overfitting to the development suite.
_Avoid_: Test set, validation set.

**Fitness vector**:
Multi-dimensional measurement of a Variant's quality. v1: hit rate (per
trace × size), per-op time (p50/p99 from microbench), metadata size (static
via `mem::size_of`).
_Avoid_: Score, fitness value (singular implies scalar).

**Pareto front**:
The set of non-dominated Variants in a Cycle's Fitness vectors. The Cycle's
winners — typically several, not one.
_Avoid_: Leaderboard, top-K.

**Fingerprint**:
Stable hash of a Variant's `impl/` directory. Used for cross-cycle dedup
(catches the AI re-proposing the same Variant) and for v2 multi-machine
worker assignment.
_Avoid_: Hash, signature.

### Architectural principles

**Token-frugal principle**:
Anything that doesn't require LLM judgment is a script. The LLM is for
hypothesizing, code-writing, smoke-log judgment, and insight synthesis;
everything else (ledger reads, leaderboard computation, log filtering, context
bundling) is mechanical and lives in `evolve/scripts/`.
_Avoid_: Token budget (means a numeric cap; the principle is architectural).

**v2-ready invariants**:
Five v1 architectural commitments that make v2 multi-machine execution a
storage-layer swap rather than a refactor: repo-relative paths only, atomic
ledger append, stable Variant fingerprint, `worker_id` on every artifact,
ledger as the only synchronization point.
_Avoid_: Distributed prerequisites, cluster invariants.

## Flagged ambiguities

- **"Target"** — in code, this is both a noun for "the home bucket of a key" (use **Target bucket** or `Slot::Target`) and an adjective in policy discussion. Prefer **Target bucket** / `Slot::Target` when the domain meaning is intended; avoid bare "target."
- **"Bucket"** — unqualified "bucket" is ambiguous between stash and normal. Use **Normal bucket** or **Stash bucket** when precision matters. `Slot` makes the intent explicit at call sites.
- **"Hot"** — overloaded: variant name of the promotion strategy (**Hot variant**) and adjective for a frequently-accessed item. Prefer **Hottest item** for the concept; leave **Hot** as the bare variant label.
- **"Insert"** vs **"Put"** — now unified: the primitive is **Put** (`SegmentCore::put(slot, item)`). Avoid **Insert** as a verb except when referring to `InsertPosition` (the enum used by `Bucket::put_at`).
- **"Demote"** — strictly means the promotion tier cascade; combined never demotes, it **places** items and drops evicted ones to stash.
- **"Promotion"** — overloaded: strategy name (**Promotion strategy**) and the act of moving an item up a tier. Prefer **Promotion strategy** when the strategy is meant and the bare verb **Promote** when the action is meant.
- **"Variant"** — overloaded across contexts. In the **Dash** domain, used only as a compound noun for promotion sub-variants (**Hot variant**, **Cold variant**). In the **Evolve framework** domain, **Variant** alone is a proposed alternative implementation of a Subject cache. Bare "variant" is ambiguous when both contexts are in scope; qualify with the domain.
- **"Insight"** — bare "insight" is ambiguous within the framework. Always qualify: **Variant insight** (per-variant) or **Cycle insight** (per-cycle synthesis). The **Insight ledger** is the durable record; **Synthesis** is the recomputable narrative.
