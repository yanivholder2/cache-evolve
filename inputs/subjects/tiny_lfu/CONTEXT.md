# TinyLFU Cache

> **This is a domain language file.** It records the canonical vocabulary for this Subject — preferred terms, terms to avoid, relationships, and flagged ambiguities.

A frequency-gated cache: an LRU resident set fronted by a TinyLFU **admission
filter**. When the cache is full, a new key is admitted only if a compact
frequency **sketch** estimates it as more frequent than the LRU victim it would
displace. Otherwise the victim stays and the newcomer is dropped.

Framework vocabulary lives in [`/CONTEXT.md`](../../../CONTEXT.md).

## Authoring scope

`tiny_lfu` exposes **zero extension points**: its only knob is `capacity`
(the sketch width is derived from capacity, not separately tunable). Variants
therefore operate in **Full-autonomy scope only** — a variant may rewrite the
cache implementation freely but must never overwrite the JNI boundary
(`src/tiny_lfu/tiny_lfu_jni.rs`), which carries the ABI the Java simulator
binds to.

## Language

### Cache structure

**TinyLfuCache**:
The cache implementation of the tiny_lfu Subject; satisfies the framework's
**Cache contract** (see [`/CONTEXT.md`](../../../CONTEXT.md)). Couples a
**Recency list** resident set with an admission **Sketch**.
_Avoid_: Map, dictionary, filter (the filter is one part, not the whole cache)

**Capacity**:
Maximum number of distinct keys the cache holds. The sole configuration knob.
_Avoid_: Size, length

**Arena**:
The `Vec<Node>` backing store. Node slots are reused in place when a victim is
displaced on admission, so no free list is maintained.
_Avoid_: Pool, heap

**Node**:
An arena entry holding a key, value, and the `prev`/`next` links of the
doubly-linked recency list.
_Avoid_: Cell, slot (slot refers to the arena index)

**Recency list**:
The doubly-linked list threading the resident nodes from LRU (head) to MRU
(tail). Holds the admitted keys; admission, not recency alone, decides what
enters it.
_Avoid_: Queue, order list

### Frequency estimation

**Sketch**:
The `CountMinSketch` — a `depth`-row by `width`-column counter table that
estimates per-key access frequency in sublinear space. `width` is derived from
`capacity`; `depth` is fixed.
_Avoid_: Counter, histogram, map

**Increment**:
Bump every row's counter for a key by one. Done on **every** access (both
**Get** and **Put**), before the admission decision.
_Avoid_: Add, record

**Estimate**:
The frequency a key is credited with: the **minimum** counter across the
sketch's rows (the count-min estimate).
_Avoid_: Count, frequency value (reserve "frequency" for the concept)

### Item lifecycle and movement

**Get**:
Look up a key; **Increment** its sketch counters and, on hit, **Promote** it to
MRU (tail).
_Avoid_: Fetch, read

**Put**:
Insert or update a key. Always **Increment** first. An existing key updates in
place and is promoted. A new key with spare capacity is inserted at MRU. A new
key at capacity triggers the **Admission contest**.
_Avoid_: Insert, add

**Admission contest**:
At capacity, compare the incoming key's **Estimate** against the **Victim**'s.
The incoming key is admitted (reusing the victim's slot) only if its estimate
is strictly greater; otherwise the victim is retained and the newcomer dropped.
_Avoid_: Eviction (the contest may evict nothing), TinyLFU filter

**Victim**:
The LRU (head) resident node — the candidate the admission contest weighs an
incoming key against.
_Avoid_: Loser, target

**Evict**:
Displace the **Victim** when the admission contest admits a newcomer; its slot
is reused in place.
_Avoid_: Kick, drop (drop refers to rejecting the newcomer)

**Promote**:
Move a node to the MRU end (tail) of the recency list on access or update.
_Avoid_: Bump, touch

**LRU (head)** / **MRU (tail)**:
The least- / most-recently-used ends of the recency list. The head is the
standing **Victim**; the tail is most protected from displacement.

## Relationships

- A **TinyLfuCache** owns one **Arena**, one **Recency list** over that arena,
  and one **Sketch**.
- Every resident key maps to exactly one **Node** (arena index) via the cache's
  `HashMap`.
- Every **Get** and **Put** **Increment**s the **Sketch** before any residency
  change.
- **Get** and an update-**Put** both **Promote** the touched node to **MRU
  (tail)**.
- At capacity, **Put** runs the **Admission contest**: the incoming **Estimate**
  versus the **Victim**'s. Admission **Evict**s the **Victim** and reuses its
  slot; rejection leaves residency unchanged.
