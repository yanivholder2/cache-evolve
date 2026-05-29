# Associative Cache

> **This is a domain language file.** It records the canonical vocabulary for this Subject — preferred terms, terms to avoid, relationships, and flagged ambiguities.

N-way associative cache: a fixed array of fixed-capacity buckets, each evicting
per its eviction policy. Keys route to exactly one bucket by hash, with no
probing, stash, or inter-bucket migration. Serves as a simple associative
baseline Subject.

Framework vocabulary lives in [`/CONTEXT.md`](../../../CONTEXT.md).

## Language

### Cache structure

**AssociativeCache**:
The cache implementation of the associative_cache Subject; an array of buckets
keyed by hash. Satisfies the framework's **Cache contract** (see
[`/CONTEXT.md`](../../../CONTEXT.md)).
_Avoid_: Map, set-associative cache

**Bucket**:
Fixed-capacity ordered collection of items; the unit of associativity.
_Avoid_: Set, way

**Item**:
Key-value pair stored in a bucket, plus policy metadata (timestamp, LFU counter).
_Avoid_: Entry, record

**Target bucket**:
The single bucket a key hashes to; the only bucket that can ever hold that key.
_Avoid_: Home bucket, slot

### Item lifecycle and movement

**Put**:
Insert a new item into its target bucket; the bucket evicts per policy if full.
_Avoid_: Insert, add

**Get**:
Look up an item in its target bucket; may mutate bucket order per eviction policy.
_Avoid_: Fetch, read

**Evict**:
Remove an item from a full bucket to make room, selected by the eviction policy.
_Avoid_: Kick, drop

**Eviction candidate**:
The bucket member that would be evicted next, per policy.
_Avoid_: Victim, LRU item

### Policy vocabulary

**Eviction policy**:
Per-bucket rule for ordering items and picking the eviction candidate.
_Avoid_: Replacement policy

**ClassicLRU**:
Hit moves item to back; evicts from front. The baseline policy.
_Avoid_: LRU (ambiguous)

**TimestampLRU**:
Hit updates item's timestamp; evicts min-timestamp item.

**LFU**:
Hit increments item's counter; evicts min-counter item.

**FIFO / LIFO / Swap**:
Secondary policies used mostly for sweep comparison.

## Relationships

- An **AssociativeCache** owns a fixed array of **Buckets**.
- An **Item**'s **Target bucket** is derived from its key by hash — it lives in
  that bucket alone, never migrating to another.
- A **Bucket** orders its items and picks its **Eviction candidate** per its
  **Eviction policy**.

## Extension points

- **eviction-policy** (`src/shared/eviction_policy.rs`): the per-bucket
  replacement algorithm.
- **bucket** (`src/associative_cache/associative_cache_bucket.rs`): the item
  container implementing the **Bucket** trait.

The JNI boundary (`src/associative_cache/associative_cache_jni.rs`) and the
variant seam (`src/factory.rs`) are contract-protected and must not be
overwritten by a variant.
