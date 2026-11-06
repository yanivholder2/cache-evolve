## Recurring wins

- **Scan-resistant insertion is the only hit-rate lever that paid off cleanly** (cycle 0001: LIP quarter-depth and the CLOCK cleared reference bit). Its gain is biggest on wide buckets. Every later policy tweak has been marginal or has traded one workload for another, so fixed quarter-depth LIP with BIP 1/32 is still the best policy.
- **Cut the number of full-key compares, not the scan order.** The dense byte-tag fingerprint pre-filter (0005) was the first change to move get p50. It also helped put, because every new-key put runs a miss scan. A side array helps only when it is much denser than the items and is scanned in chunks.
- **Remove bulk memmoves on the full-bucket put-miss path.** The fused evict+insert partial rotation, proposed in 0004 and measured cleanly in 0006, cut put p50 and p99.
- **Removing shared atomics from the put path helps.** The global atomic tick caused the 0004 slowdown, and taking it out in 0005 recovered the loss. This was clearest on the base without tags.
- **Shrinking `Item` is the only thing that moves metadata-size** (0004, 24-byte item).
- **Mechanical rewrites that keep layout and semantics reproduce every hit-rate cell exactly** (0004 through 0006). The current policy is a stable base for speed work.

## Recurring failures

- **Reuse and adaptation signals keep failing the hm_0 vs web_0 tradeoff.** Five have failed so far: BIP's blind period on web_0, a 16-key ghost ring, a hashed ghost bit set, hit-position adaptive depth, and a 1-bit second chance with no decay. Each one either saturates and turns into random MRU promotion, or misses reuse at the workload's reuse distance.
- **Scan reordering and hit-time reordering** (reverse scan, swap-halfway self-organization) leave get p50 flat or make it worse. Hit-path writes must stay a single store.
- **Splitting hot data across allocations** (the separate `u64` key mirror in 0002) adds a second memory stream to every hit.
- **New per-op costs keep slipping in alongside changes meant to be cheap:** per-op string compares, dynamic policy calls, SipHash on every miss, global atomics, unbounded sweeps. Anything added to the put path has to be per-bucket, non-atomic, or decided once at construction.
- **Changes that bound worst-case work** (sweep cap, dense absence scan) improve p99 but tend to make p50 worse.
- **Flat metadata-size proves nothing about per-bucket state.** The metric missed real footprint four times: key mirror, ghost ring, tags, and extra bucket fields.
- **Per-op-time is noisy across sessions.** This showed in the 0004 environment shift and in the get p99 moves of 0006, where diffs did not touch the get path. Speed claims without a same-session parent baseline have misled before.

## Open questions

- Does removing the clock stack on the tag+rotation base if the bucket stays the same size (`clock` and `bip_tick` folded into one shared `u32`)? Bucket stride may be a hidden cost.
- Can the remaining quarter-length shift and tag-mirror movement be cut further, for example with a ring offset for the probationary region or tags-only movement on the hot path?
- Do wider tags, or SIMD on the 16-wide mask, remove the remaining false-positive compares? Can tags be extended to the CLOCK path?
- Is any reuse signal still worth a try? The only design not yet ruled out is a per-item probationary bit with independent, symmetric decay, with spared items placed outside the admission region.
- Does the hit-rate harness run the overlaid eviction path? The 0001 frequency-window result was bit-identical to the baseline and was never re-checked.

## Per-policy notes

- **ClassicLRU / LIP (quarter-depth):** This is the main base. Its hit rate is stable, and the cost sits in the put path's mid-bucket shift, which the fused rotation has now partly removed. Adapting the depth from hit position failed, and whether depth should scale with bucket width is still untested.
- **BIP (1/32 MRU admission):** It improves hm_0 on small buckets but gives back web_0 scan resistance on wide buckets. Every attempt to replace its blind period with a reuse filter has done worse overall.
- **CLOCK / GCLOCK:** Hits cost a single bit store, so it has the cheapest hit path. A 2-bit GCLOCK with a 16-slot bounded sweep and clear-on-insert gained a little hit rate at every point and helped the put tail, at some cost to put p50. Self-organizing swaps break its latency advantage. Tags have not been applied here yet.
- **Frequency-aware victim window (LFU-ish):** Tested once (0001) with a result identical to the baseline. It is unresolved until the harness is confirmed to run that path.
- **Second chance at the LRU end:** Without decay, the bit saturates on large caches. It helped only small hm_0 and hurt the put tail.
