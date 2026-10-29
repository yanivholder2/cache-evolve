# Variant v003 Insight

## Helped

- hit-rate (hm_0 at the small size only): This is the one trace/size cell that moved up, and only slightly. At 16K entries the hm_0 working set churns probation hard enough that keys with repeated misses gain from protection against single-miss candidates. The gain is too small to offset the losses elsewhere.
- metadata-size: `Node<u64,u64>` stays at 24 bytes as the hypothesis required. All admission state lives in a per-cache `FreqSketch`, so the per-node summary does not change.

## Hurt

- hit-rate: The summary fell below the parent, and the worst loss is web_0 at the small size. Frequency is only recorded on `put`, which is the miss path, so a key that has been hit many times scores the same as a key seen once. The probation victim was admitted earlier, so its estimate is at least 1. A new key's first `put` also brings it to only 1, and a tie (`<=`) rejects it. As a result, every new key must miss at least twice before it can enter. web_0 at 16K has many keys that are re-referenced soon after their first appearance, and the cache throws those hits away. The `both == 0` tie-break almost never fires because the candidate is incremented before the comparison, so it does not soften this.
- per-op-time (put): Put p50 and p99 both rose. Every miss now runs SipHash (`DefaultHasher`) twice, once for the candidate and once for the victim key, plus 4 increments and 8 estimate probes at random columns of a table of 4 × next_pow2(capacity) bytes. At the large size the table is about 512 KiB. The periodic full-table `halve()` shows up in the p99.
- per-op-time (get): Get p50 rose even though `get` never touches the sketch. The likely cause is cache pressure: the sketch table competes with the arena and the `HashMap` for cache lines on the interleaved put path, so later gets load from colder memory. Measurement noise is also possible. Either way, keeping the sketch off `get` did not protect get latency as the rationale expected.
- metadata-size (secondary, `LruCache` struct): The cache header grew to hold the sketch's `Vec`, `mask`, `additions` and `reset_at`, plus the heap table it points to. These are per-cache costs, so the summary does not count them.

## Takeaway

On top of SLRU, a bare TinyLFU gate that counts only misses and rejects ties acts like a "must miss twice" doorkeeper. That gate starves keys with short reuse distance and costs more hit-rate than it saves from scans. Probation already filters one-hit keys after admission, so the gate adds little. For admission to pay off, the sketch probably needs to count hits too, and new keys need a small admission window (W-TinyLFU style) instead of being rejected before their first reuse. A cheaper hash than SipHash is also needed to keep the cost off put.
