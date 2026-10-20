# Variant v002 Insight

## Helped

- hit-rate: Held steady as the hypothesis required. Only the key→slot index changed. `find_victim`, `push_back` and the hand/hand_prev logic are byte-for-byte the parent's, so eviction order is the same. This confirms the custom table (including backward-shift deletion) is functionally correct on the real trace, not just in the new randomized tests.

## Hurt

- per-op-time: The `get` median and tail both regressed, which is the opposite of what was predicted. The table stores no keys, so every occupied bucket on a probe path costs a dependent load into the 24-byte `nodes` arena (`self.nodes[s].key == *key`) to compare keys. That includes buckets that merely collide. hashbrown instead filters 16 buckets at once with SIMD tag bytes and touches the inline key only on a tag match. A `get` miss in hashbrown usually touches no key at all. Here it touches one arena line per occupied bucket until it reaches `EMPTY`.
- per-op-time: The `put` tail got much worse while the `put` median stayed flat. The "one hash per miss-path put" claim only holds for the new key. On eviction, the code re-hashes the victim's key, runs `bucket_of_slot` (a second probe), and then `remove_bucket` re-hashes the key of *every* entry in the rest of the cluster to find its home bucket. Each of those is another random arena load. Finally `find_empty` probes a third time. Long linear-probing clusters (Fx plus a single `h ^ (h >> 32)` fold is a weak mix for linear probing) turn into exactly the rare, expensive puts that p99 measures. Short-cluster and not-yet-full puts, which set the median, are about as cheap as before.

metadata-size: The summary (per-`Node` bytes) didn't move, because `Node` is unchanged. `LruCache` also stayed the same size: `Vec<u32>` plus `mask` takes the same inline space as a `HashMap` with a zero-sized hasher. The real memory win is per index entry: 4-byte buckets at load factor ≤ 0.5, against hashbrown's 16-byte `(u64,u32)` buckets plus control bytes. No current dimension measures this, so it doesn't show up as a gain.

## Takeaway

A key-less `u32` open-addressing index trades hashbrown's in-bucket key and tag filtering for an arena dereference on every probe step. On this SIEVE that indirection costs more than the saved re-hashes, and backward-shift deletion (which re-hashes each shifted key through the arena) makes the eviction path's tail worse. Hand-rolled linear probing loses to hashbrown on latency unless the table can reject non-matching buckets without touching `nodes`. Storing a hash fingerprint or the key itself in the bucket would do this, as would storing the bucket's home index so deletion never re-hashes. The single-hash idea is only worth pursuing on top of such a layout. The cheaper path is probably hashbrown's own entry-style or raw-table API, not a replacement for it.
