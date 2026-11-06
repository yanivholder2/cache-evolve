# Variant v003 Insight

## Helped

- hit-rate: The summary went up slightly. The largest gains came on web_0 at both sizes, which is the trace the hypothesis only hoped to leave alone. A one-shot reprieve inside probation keeps a recently evicted key around for one more probation cycle. It never pushes into protected and never forces a demotion, so the protected working set stays intact. This confirms that gentle use of ghost history avoids cycle 3's web_0 damage.
- hit-rate (hm_0, small cache): hm_0 at 16K improved, which is the reuse gain the hypothesis targeted. At this size, keys are often evicted from the small probation segment just before they are reused, so the second-chance bit buys them the extra time they need.

## Hurt

- hit-rate (hm_0, large cache): hm_0 at 131K slipped a little. This was the main gain the hypothesis predicted. At this size, keys that survive probation are mostly already reused and promoted. The reprieve then mainly keeps old ghost-tagged keys in probation, and the newer key evicted in their place could have been promoted. A 16-bit direct-mapped tag also gives some false positives once the ghost slots are overwritten many times.
- per-op-time (get): Get p50 regressed sharply, even though `get` never reads the ghost. The likely cause is that `on_hit` now pays for masking on `next` as well as `prev`. `next_idx()` adds an AND, and `set_next` becomes a read-modify-write that preserves `SC_BIT` on every `unlink`/`push_back`. The protected-path `push_back` also gets its own branch to clear the bit. A second likely cause is that the header grew past one cache line, and the heap ghost array competes with the arena and map for cache.
- per-op-time (put): Put p99 rose much more than put p50. Each miss now hashes the key twice more: `hash_one` on the new key and again on the evicted key. When the probation head carries the bit, the miss also does an extra `unlink`/`push_back`. The tail grows from misses that hit both costs.
- metadata-size (secondary, `LruCache` struct): The header grew by the 16-byte `Box<[u16]>` fat pointer. The ghost also adds about 2 bytes of heap per entry, which the metric does not capture. `Node<u64,u64>` stays at 24 bytes, so the summary dimension did not move.

## Takeaway

A probation-only second-chance bit driven by a 16-bit fingerprint ghost removes the web_0 risk of ghost admission. Its hm_0 benefit, however, is limited to the small cache and disappears at large sizes, so overall it gives only a marginal hit-rate gain. That gain does not pay for the get-p50 regression caused by packing a second flag into `next` and adding a ghost array. If ghost history is tried again, it should cost nothing on the `on_hit` link path, for example by keeping `next` unmasked. Its effect should also be scaled by cache size rather than applied uniformly.
