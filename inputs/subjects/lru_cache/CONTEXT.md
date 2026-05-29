# LRU Cache

> **This is a domain language file.** It records the canonical vocabulary for this Subject — preferred terms, terms to avoid, relationships, and flagged ambiguities.

Classic least-recently-used cache used as a simple, dependency-free baseline
Subject. Strict global LRU ordering with no hash routing, segments, or stash.

Framework vocabulary lives in [`/CONTEXT.md`](../../../CONTEXT.md).

## Authoring scope

`lru_cache` exposes **zero extension points**: its only knob is `capacity`.
Variants therefore operate in **Full-autonomy scope only** — a variant may
rewrite the cache implementation freely but must never overwrite the JNI
boundary (`src/lru_cache/lru_cache_jni.rs`), which carries the ABI the Java
simulator binds to.

## Language

### Cache structure

**LruCache**:
The cache implementation of the lru_cache Subject; satisfies the framework's
**Cache contract** (see [`/CONTEXT.md`](../../../CONTEXT.md)).
_Avoid_: Map, dictionary

**Capacity**:
Maximum number of distinct keys the cache holds. The sole configuration knob.
_Avoid_: Size, length

**Arena**:
The `Vec<Node>` backing store. Node slots are reused in place on eviction, so no
free list is maintained.
_Avoid_: Pool, heap

**Node**:
An arena entry holding a key, value, and the `prev`/`next` links of the
doubly-linked recency list.
_Avoid_: Cell, slot (slot refers to the arena index)

**Recency list**:
The doubly-linked list threading nodes from LRU (head) to MRU (tail).
_Avoid_: Queue, order list

### Item lifecycle and movement

**Put**:
Insert or update a key. A new key at capacity evicts the LRU entry (head) and
reuses its arena slot. An existing key updates in place and is promoted to MRU.
_Avoid_: Insert, add

**Get**:
Look up a key; on hit, promote it to MRU (tail).
_Avoid_: Fetch, read

**Evict**:
Remove the LRU entry (head of the recency list) to make room.
_Avoid_: Kick, drop

**Promote**:
Move a node to the MRU end (tail) of the recency list on access or update.
_Avoid_: Bump, touch

**LRU (head)**:
The least-recently-used node; the next eviction candidate.

**MRU (tail)**:
The most-recently-used node; the most protected from eviction.

## Relationships

- An **LruCache** owns one **Arena** and one **Recency list** over that arena.
- Every resident key maps to exactly one **Node** (arena index) via the cache's
  `HashMap`.
- **Get** and **Put** both **Promote** the touched node to **MRU (tail)**.
- **Evict** always removes the **LRU (head)** node, whose slot is then reused.
