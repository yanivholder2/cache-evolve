# Ubiquitous Language

## Cache structure

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **Dash** | Multi-segment associative cache that lets items migrate between adjacent buckets to mitigate the hot-set problem | Cache, map |
| **Segment** | Independent partition of the Dash; each segment owns its own normal buckets and stash | Shard, partition |
| **SegmentCore** | Shared state (normal buckets + stash buckets) inside every segment variant | — |
| **Bucket** | Fixed-capacity ordered collection of items within a segment | Set, way |
| **Normal bucket** | A non-stash bucket; plays the role of target for some keys and probing for their neighbors | Data bucket |
| **Stash** | Secondary bucket group used as overflow / second-chance storage | Overflow, L2 |
| **Item** | Key-value pair stored in a bucket, plus policy metadata (timestamp, LFU counter) | Entry, record |
| **Slot** | Logical destination for an item: one of *Stash*, *Target*, *Probing* — derived from the item's key, never an arbitrary bucket index | Destination, bucket index |
| **Target bucket** | The normal bucket an item hashes to | Home bucket |
| **Probing bucket** | The neighbor of an item's target bucket: `(target + 1) % n`; overflow destination | Probe bucket, neighbor |

## Strategies and variants

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **Promotion strategy** | Strategy 1: items earn their way into the probing bucket only after proving they're the hottest in their target | Position-based LRU, tiered strategy |
| **Combined strategy** | Strategy 2: target and probing are peer storage; eviction picks the weakest candidate across both | Combined eviction, peer strategy |
| **Cold variant** | Promotion sub-variant where stash→target insert lands at the LRU end (item must climb) | — |
| **Hot variant** | Promotion sub-variant where stash→target insert lands at the MRU end (one more access promotes) | — |

## Item lifecycle and movement

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **Put** | Insert a new item into the cache (default routes to stash) | Insert, add |
| **Get** | Look up an item; may mutate state per eviction policy and per strategy (promotion / touch-in-place) | Fetch, read |
| **Evict** | Remove an item from a bucket to make room, selected by the eviction policy | Kick, drop |
| **Hottest item** | The bucket member most protected from eviction, per policy (index returned by `promotion_candidate_index`) | Top item, MRU item |
| **Eviction candidate** | The bucket member that would be evicted next, per policy | Victim, LRU item |
| **Promote** | Move an item to a higher tier (stash→target or target→probing); promotion strategy only | — |
| **Demote** | Move an item to a lower tier (probing→target→stash); promotion strategy only | — |
| **Cascade** | Chain of evictions triggered by a single put (e.g., promote_to_probing evicts probing victim → cascades into target → may cascade into stash) | — |
| **Demotion chain** | Promotion strategy's tier order: `probing → target → stash → out` | — |
| **Peer storage** | Combined strategy's view of target and probing: non-tiered; evictions go straight to stash | — |
| **Tier drop** | Single step down the demotion chain (probing→target or target→stash) | — |

## Domain verbs (API level)

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **promote_to_target** | Promotion verb: move an item into its target bucket at a given position; evicted item drops to stash | — |
| **promote_to_probing** | Promotion verb: move an item into its probing bucket; evicted item cascades through the demotion chain | — |
| **place_in_normal_bucket** | Combined verb: place an item into its target or probing (caller's choice); evicted item drops straight to stash | — |
| **put_in_stash** (implicit, via `SegmentCore::put(Slot::Stash, …)`) | Route a new item into its stash bucket; stash overflow is terminal (item permanently lost) | — |

## Policy vocabulary

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **Eviction policy** | Per-bucket rule for ordering items and picking the eviction candidate | Replacement policy |
| **ClassicLRU** | Hit moves item to back; evicts from front | LRU (ambiguous) |
| **TimestampLRU** | Hit updates item's timestamp; evicts min-timestamp item | Timestamp LRU |
| **LFU** | Hit increments item's counter; evicts min-counter item | — |
| **FIFO / LIFO / Swap** | Secondary policies used mostly for sweep comparison | — |
| **MRU position** | The end of a bucket's order that holds the hottest item under position-based policies (ClassicLRU/Fifo/Swap) | Top |
| **LRU position** | The end of a bucket's order that holds the next eviction candidate under position-based policies | Bottom, front |

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

- **"Target"** — in code, this is both:
  - a noun for "the home bucket of a key" (use **Target bucket** or `Slot::Target`), and
  - an adjective in policy discussion (e.g. "the target of combined's placement decision").
  Prefer **Target bucket** / `Slot::Target` when the domain meaning is intended; avoid bare "target."
- **"Bucket"** — unqualified "bucket" is ambiguous between stash and normal. Use **Normal bucket** or **Stash bucket** when precision matters. `Slot` makes the intent explicit at call sites.
- **"Hot"** — overloaded:
  - variant name of the promotion strategy (**Hot variant**), and
  - adjective for a frequently-accessed item.
  Prefer **Hottest item** for the concept; leave **Hot** as the bare variant label.
- **"Insert"** vs **"Put"** — historically synonymous in this codebase (`SegmentCore::insert` and `SegmentCore::put_to_stash` both existed). Now unified: the primitive is **Put** (`SegmentCore::put(slot, item)`). Avoid **Insert** as a verb except when referring to `InsertPosition` (the enum used by `Bucket::put_at`).
- **"Demote"** — historically used for both the promotion tier cascade and combined's "route to stash." These are distinct operations. **Demote** now strictly means the promotion tier cascade; combined never demotes, it **places** items and drops evicted ones to stash.
- **"Promotion"** — overloaded:
  - strategy name (**Promotion strategy**), and
  - the act of moving an item up a tier within that strategy.
  Context usually resolves it; prefer **Promotion strategy** when the strategy is meant and the bare verb **Promote** when the action is meant.
