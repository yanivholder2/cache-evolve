# Variant v002 Insight

## Helped

- per-op-time (`get` p50): The median get got clearly faster. That supports the hypothesis that the stash filter's false-positive rate was the p50 lever. With k=2 bits per key in the same 64-bit word, a stash bucket holding 4–8 keys rejects roughly 4–5× more absent keys, so more normal-tier hits skip the `get_position` stash scan. The reject test still costs one load and an AND/compare against `stash_mask_bits(h)`, which is computed in registers from the existing `hash_key` value. Fewer avoidable scans therefore came with no added fixed cost.
- per-op-time (`get` p99): The tail eased slightly. The slow path (stash-hit promotion plus a `refresh_stash_mask` rebuild) is structurally unchanged, so the small gain is consistent with fewer false-positive scans on the common path, not a fix to the tail itself.
- per-op-time (`put` p50, p99): Both put percentiles dropped noticeably, though the hypothesis targeted only reads. Writing a mask now costs one extra shift and OR per key, which should be neutral or slightly worse. The likely explanation is that put's existence check / pre-insert lookup goes through the same stash reject test, so it also skips more false-positive stash scans. Run-to-run noise may account for part of the drop. I could not inspect the put path to confirm which.

## Hurt

(None. hit-rate is bit-identical to the parent, as the no-false-negative invariant requires: every live stash key's bits are all set, and lookup order stays stash-first. metadata-size did not change because the filter reuses the existing `stash_masks` u64 word.)

## Takeaway

Under a fixed one-word budget, the stash presence filter is limited by its false-positive rate, not its size or probe cost. Moving from k=1 to k=2 inside the same u64 (low hash bits 0..6 and 6..12, independent of the multiply-shift bucket and segment reductions) gave a policy-neutral speedup on both get and put. Combined with 0008 v001's finding that shrinking the filter hurt, this means the filter is now worth tuning on the FP/k axis. k=3 is the obvious next probe, but at n≈8 the optimum is near k≈5, and that gain shrinks as stash buckets fill.

(Note: I reviewed the overlay's `src/shared/utils.rs`, which contains `stash_mask_bits` and its unit tests for no false negatives and fewer false positives than k=1. I could not list or open the other overlay files in this session, so the claims about how `get`, `promotion_get`, `refresh_stash_mask` and the put path call the mask rest on the hypothesis's described changes.)
