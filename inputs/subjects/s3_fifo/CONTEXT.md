# S3-FIFO Cache

> **This is a domain language file.** It records the canonical vocabulary for this Subject — preferred terms, terms to avoid, relationships, and flagged ambiguities.

The S3-FIFO policy from "FIFO Queues are ALL You Need for Cache Eviction"
(Juncheng Yang et al.). Three FIFO queues — a **Small queue** admission buffer,
a **Main queue** resident set, and a **Ghost queue** of recently evicted keys —
cooperate so that one-hit-wonders age out of the small queue while reused keys
graduate into the main queue. A second-chance frequency bit governs main-queue
eviction.

Framework vocabulary lives in [`/CONTEXT.md`](../../../CONTEXT.md).

## Authoring scope

`s3_fifo` exposes **zero extension points**: its only knob is `capacity`. The
small/main/ghost partition is derived from capacity, not separately tunable.
Variants therefore operate in **Full-autonomy scope only** — a variant may
rewrite the cache implementation freely but must never overwrite the JNI
boundary (`src/s3_fifo/s3_fifo_jni.rs`), which carries the ABI the Java
simulator binds to.

## Language

### Cache structure

**S3Fifo**:
The cache implementation of the s3_fifo Subject; satisfies the framework's
**Cache contract** (see [`/CONTEXT.md`](../../../CONTEXT.md)). Owns the small,
main, and ghost queues.
_Avoid_: Map, dictionary, two-queue (S3-FIFO is three queues)

**Capacity**:
Maximum number of distinct keys held across the **Small queue** and **Main
queue**. The sole configuration knob.
_Avoid_: Size, length

**Small queue**:
The FIFO admission buffer (~a quarter of capacity). New, non-ghosted keys enter
here; on overflow each is either graduated to the main queue or demoted to the
ghost queue.
_Avoid_: Probation, window, S1

**Main queue**:
The FIFO resident set holding graduated keys (the remainder of capacity).
Evicted with a second-chance pass over the **Frequency**.
_Avoid_: Protected, segment, S3

**Ghost queue**:
A FIFO of recently evicted keys only — no values. A **Put** of a ghosted key is
admitted straight into the main queue. Sized like the main queue.
_Avoid_: Shadow, history, S2

**Entry**:
A resident record in the small or main queue: key, value, and **Frequency**.
_Avoid_: Node, cell

**Frequency**:
A per-entry second-chance counter, saturating at `MAX_FREQ` (3). Bumped on
**Get**, decremented during main-queue eviction.
_Avoid_: Recency, age, hit count

### Item lifecycle and movement

**Get**:
Look up a key in the small or main queue; on hit, bump its **Frequency**
(saturating). Position in the queue is unchanged.
_Avoid_: Fetch, read, promote (Get does not move the entry)

**Put**:
Insert or update a key. An existing entry updates its value in place. A ghosted
key is admitted to the **Main queue** front. Any other new key enters the
**Small queue** front.
_Avoid_: Insert, add

**Graduate**:
Move a small-queue entry into the main queue on small-queue eviction when its
**Frequency** exceeds one (it was reused while in the small queue).
_Avoid_: Promote (reserve for ghost re-admission), upgrade

**Demote**:
On small-queue eviction of a cold entry (frequency ≤ 1), record its key in the
**Ghost queue** instead of keeping its value.
_Avoid_: Evict (demotion preserves the key in the ghost queue)

**Evict (main)**:
Second-chance eviction: pop the main-queue tail; if its **Frequency** is
positive, decrement and reinsert it at the front; repeat until a
zero-frequency tail is dropped.
_Avoid_: Kick, second-chance clock (describe, don't name a different policy)

**Victim**:
The tail of the queue being evicted — the main-queue tail under second chance,
or the small-queue tail on admission overflow.
_Avoid_: Loser, target

## Relationships

- An **S3Fifo** owns one **Small queue**, one **Main queue**, and one **Ghost
  queue**; the small + main capacities sum to **Capacity**.
- A new key enters the **Small queue** unless its key is in the **Ghost
  queue**, in which case **Put** admits it directly to the **Main queue**.
- **Small queue** overflow either **Graduate**s a reused entry to the **Main
  queue** or **Demote**s a cold one to the **Ghost queue**.
- **Main queue** overflow runs **Evict (main)**, a second-chance pass that
  decrements **Frequency** before dropping a **Victim**.
- **Get** mutates only an **Entry**'s **Frequency**; it never moves entries
  between queues.
