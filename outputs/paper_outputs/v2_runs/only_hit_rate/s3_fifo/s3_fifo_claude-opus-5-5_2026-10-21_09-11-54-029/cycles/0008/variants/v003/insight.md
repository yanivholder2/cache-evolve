# Variant v003 Insight

## Helped

(No measured dimension improved. Hit-rate is the only dimension in the eval, and this change was meant to leave it unchanged. The benefits the hypothesis targets are per-op time and metadata size, and neither is measured here. On metadata, `ghost_ring` drops from 8-byte to 4-byte slots, and `ghost_map` keys shrink from `u64` to `u32`. On lookups, `ghost_map` no longer runs SipHash a second time over an already-hashed fingerprint. Instead it uses the identity hasher's `write_u32` plus a single odd-constant multiply in `finish()`. Each ghost probe still pays one `DefaultHasher` pass inside `fingerprint()`, so only the redundant in-map hash was removed.)

## Hurt

(No measured dimension regressed. Hit-rate matches the parent, which is inside the ±0.0005 band H03 set as the porting-bug tripwire. Two risks remain unmeasured. First, folding to 32 bits adds false ghost hits at odds of roughly main_size / 2^32 per lookup, which is about 1946 / 2^32 at capacity 2048. A false hit sends a cold key straight into main at freq 0 instead of into small. That is far too rare to show up on this trace. Second, the identity hasher's bucket quality depends entirely on the multiply in `finish()`. The new 10k-fingerprint round-trip test and its dense sequential-`u32` case check correctness, not probe-length distribution.)

## Takeaway

The two cycle-7 representation changes compose cleanly on top of the SIEVE main queue. The extended `matches_ring_reference_on_pseudo_random_streams` still checks small, main, hand, ghost ring and ghost head step for step against the reference, with the reference ring now projected to `u32`. Combined with the unchanged hit-rate, this confirms that 32-bit fingerprints plus an identity-hashed `ghost_map` are policy-neutral for the SIEVE lineage. That makes v003 a lean ghost base for later policy experiments. Whether it actually delivers the intended cost win cannot be judged until the eval adds per-op-time and metadata-size dimensions.
