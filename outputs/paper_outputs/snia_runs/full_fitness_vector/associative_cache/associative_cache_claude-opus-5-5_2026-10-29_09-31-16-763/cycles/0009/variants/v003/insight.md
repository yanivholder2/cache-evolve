# Variant v003 Insight

## Helped

- per-op-time: Get and put both got faster at p50 and p99. This is the expected result of removing a whole SipHash-1-3 (`DefaultHasher` in `utils::get_index`) from every op. The variant notes confirm the parent picked a bucket with SipHash and then hashed the key a second time with `TagHasher`. Now one multiply-based `tag_hash` feeds both the bucket index (bits 24–55, masked for power-of-two counts) and the tag (bits 56–63). The parent's latency changes were small and touched paths its diff didn't change. This drop is different: it appears on both ops and both percentiles, and both ops share the removed hash, so it can be credited to the change.
- hit-rate: Re-mapping keys to buckets moved every cell by a tiny amount, all upward, and the summary stayed well inside the hypothesis's ±0.0005 band. The overlay test shows the multiplicative hash on bits 24–55 spreads sequential, stride-8 and stride-4096 keys across 64 and 1024 buckets with a max/min occupancy ratio under 1.5. So dropping SipHash did not cause the bucket clustering that could have hurt MSR block-address traces.

## Hurt

- (none) metadata-size is unchanged: `tag_hash`/`get_hashed`/`put_hashed` add code, not fields, and the 80-byte bucket stride test still passes.

## Takeaway

The answer to cycle 8's open question is "bucket hashing". A SipHash per op, which none of the in-bucket work in cycles 5–8 could reach, was a large share of the per-op time. A single multiplicative hash whose bucket bits and tag bits don't overlap replaced it with no measurable effect on hit rate. When the remaining cost sits above the bucket, removing a redundant pass over the key pays off far more than tuning the scan or rotation loops further.
