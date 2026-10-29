# Cycle 0004 Insight

## Cohort outcome

Three variants worked on three separate axes. v001 merged the full-bucket evict and quarter-depth insert into one partial rotation, aiming at put latency. v002 replaced BIP's period with a hashed ghost bit set sized to the bucket, aiming at hit rate. v003 narrowed `Item`'s bookkeeping fields, aiming at metadata-size. v003 is the Winner because it is the first variant in four cycles to move metadata-size, with hit rate exactly unchanged.

## Patterns

- Per-op-time could not be trusted this cycle. All three variants got slower on get, including v001, whose get path is byte-identical to its parent. That points to a shift in the benchmark environment rather than to any one change. As a result, no speed hypothesis was actually tested: v001's predicted memmove saving and v003's predicted cache-line gain are both still open.
- Rewrites that keep the layout are reliable on hit rate. v001 (fused rotation) and v003 (narrow fields) both matched every hit-rate cell exactly, so the parent's policy can now be treated as a stable base to build on.
- New hot-path costs keep coming in with changes meant to be cheap. Examples: a dynamic policy call on every miss (v001), SipHash twice per miss plus periodic full clears (v002), and a global atomic clock on every put (v003). Each one plausibly cancels the gain it sits next to. Any work added to the put path must be per-bucket and non-atomic, or decided once at construction.
- A reuse filter has to be sparse to be useful. At about one bit per tracked eviction, v002's filter fills up and acts like a high-rate random MRU promoter. That hurt hm_0 at both widths and web_0 on wide buckets, worse than BIP's blind 1/32. Its all-at-once reset also starves shifting working sets. This is the second reuse-signal design to fail on the two-workload tradeoff.
- Metadata-size can move after all: it measures `Item` layout, not per-bucket state. Shrinking the item is the one lever that reaches this dimension.

## Open questions

- With the benchmark rerun against its parent in the same session, does the fused rotation (without the dynamic victim check, and using `ptr::copy`) actually cut put latency?
- With the 24-byte item kept but the shared atomic clock removed (a per-bucket tick, or no write at all), does the denser layout speed up the scan?
- Is any reuse filter worth another attempt? It would need several bits per key, gradual aging and a cheap integer hash, and it is still not clear it can beat BIP on both workloads.

## Continuation

Build on v003's 24-byte `Item`, but replace the global atomic tick with a per-bucket counter, or skip the write when the policy never reads it. Then layer v001's fused rotation on top, cleaned up (victim-is-index-0 cached at construction, `ptr::copy` shift). Judge both against a same-session parent baseline, since this cycle's per-op-time numbers are unreliable. Deprioritize hashed ghost filters. The inline fingerprint/tag pre-filter for get p50, recommended for two cycles now, is still untested and should be one of the next hypotheses.
