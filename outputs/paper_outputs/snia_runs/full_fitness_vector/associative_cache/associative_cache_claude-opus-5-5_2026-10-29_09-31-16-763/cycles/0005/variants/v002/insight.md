# Variant v002 Insight

## Helped

- per-op-time: Get and put both got faster at p50 and p99. On the ClassicLRU path, `get_position` now scans a dense `Vec<u8>` of top-byte fingerprints in 16-wide branch-free mask chunks. It does a full key compare only when a tag matches, so a miss across a wide bucket reads about 1 byte per slot instead of a 24-byte `Item`. Put gained at least as much as get, which fits the hypothesis's point that every new-key put runs a miss scan first. `put` hashes once and reuses the tag in `get_position_tagged`, which keeps that extra cost down. The hit-path `tags.remove`/`push` byte memmove and the cheap multiplicative `TagHasher` (not SipHash) evidently cost less than the item-stream reads they save. This held even though the forward scan gives up the parent's MRU-first `rposition` early exit on hits, and even though the global atomic tick from the parent was left in place.
- hit-rate: Not a gain, but the invariant held. All four cells match the parent exactly. The tag filter only skips compares that would have failed, keys are unique per bucket so the forward scan returns the same index as `rposition`, and every `items` mutation (hit promotion, the `evict_item` override, a new `remove_at` override, quarter-depth insert, BIP push) is mirrored in `tags`. So no policy decision changed.

## Hurt

- metadata-size: The measured value didn't move, because `tags` lives in the bucket and not in `Item<u64,u64>` or `AssociativeCache<u64,u64>`. Neither measured type got bigger. But the variant does use 1 extra byte per cached entry plus a `Vec` header per bucket, which this dimension doesn't count. The real per-entry footprint went up by about 1/24 even though the dimension reads unchanged.

## Takeaway

The 1-byte fingerprint pre-filter is the first change to cut the full-key-compare cost the earlier insights kept pointing to, and it did so with no hit-rate change. A `Vec<u64>` key mirror didn't work, but a 24×-denser tag stream scanned in vectorisable 16-wide chunks did, because misses dominate and the extra work on hits is a small byte memmove. Two follow-ups make sense now: remove the shared atomic tick, which is still on the put path, and decide whether metadata-size should count side arrays kept per bucket like `tags`.
