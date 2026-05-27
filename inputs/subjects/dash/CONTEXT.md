# Dash Cache

> **This is a domain language file.** It records the canonical vocabulary for this Subject — preferred terms, terms to avoid, relationships, and flagged ambiguities. Before adding, renaming, or removing any term, read the format spec and editing rules at `~/.claude/skills/grill-with-docs/CONTEXT-FORMAT.md`. The `/grill-with-docs` skill is the primary tool for evolving this file.

Associative cache for M.Sc. thesis research on mitigating the hot-set problem in N-way associative caches using Dash-inspired bucket movement, where items migrate between adjacent buckets to reduce contention in overloaded sets.

Framework vocabulary lives in [`/CONTEXT.md`](../../../CONTEXT.md).

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

## Flagged ambiguities

- **"Target"** — in code, this is both a noun for "the home bucket of a key" (use **Target bucket** or `Slot::Target`) and an adjective in policy discussion. Prefer **Target bucket** / `Slot::Target` when the domain meaning is intended; avoid bare "target."
- **"Bucket"** — unqualified "bucket" is ambiguous between stash and normal. Use **Normal bucket** or **Stash bucket** when precision matters. `Slot` makes the intent explicit at call sites.
- **"Hot"** — overloaded: variant name of the promotion strategy (**Hot variant**) and adjective for a frequently-accessed item. Prefer **Hottest item** for the concept; leave **Hot** as the bare variant label.
- **"Insert"** vs **"Put"** — now unified: the primitive is **Put** (`SegmentCore::put(slot, item)`). Avoid **Insert** as a verb except when referring to `InsertPosition` (the enum used by `Bucket::put_at`).
- **"Demote"** — strictly means the promotion tier cascade; combined never demotes, it **places** items and drops evicted ones to stash.
- **"Promotion"** — overloaded: strategy name (**Promotion strategy**) and the act of moving an item up a tier. Prefer **Promotion strategy** when the strategy is meant and the bare verb **Promote** when the action is meant.
- **"Variant"** (Dash sense) — used only as a compound noun for promotion sub-variants (**Hot variant**, **Cold variant**). The unqualified Evolve framework sense lives in [`/CONTEXT.md`](../../../CONTEXT.md); qualify when both contexts are in scope.
