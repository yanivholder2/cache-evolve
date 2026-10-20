# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three hypotheses tried to raise hit rate by letting a key live in more than one bucket: two-choice placement (H01), cuckoo-style displacement (H02), and paired home/partner buckets (H03). Only v003 (H03) passed the gates. It won because it got faster on both get and put, especially put's tail. Its hit rate came in slightly below baseline, so the hit-rate goal of this cycle was not met.

## Patterns

- The P2 slot_legality invariant blocks any design where a key can sit outside its single hashed bucket. v001 and v002 failed on this one invariant in all three attempts. v003 got through only because it departed from its hypothesis: it merged each pair into one physical bucket, so every key still sits in its one hashed bucket.
- Within a bucket, splitting LRU into home and partner halves with demotion and promotion rules did not improve hit rate. The extra flexibility was cancelled by the bookkeeping, and hot halves pushing out their partner's native items may make it slightly worse.
- v003's speedup seems to come from how it was built, not from the pairing policy: it hashes each key once, keeps each bucket in one contiguous allocation, and updates the eviction policy in place. That speedup should carry over to other variants on its own.

## Open questions

- Is a larger, merged bucket with ordinary single LRU (no split halves) as fast as v003, and does its hit rate match or beat baseline? This would separate the speedup from the policy.
- Can hit rate be gained inside the single-bucket rule by changing the eviction policy, for example frequency-aware, scan-resistant, or with insertion-position tuning?
- Does the `Instant::now()` timestamp refresh on every hit, added by all three hypotheses, cost per-op time that a cheaper counter would avoid?

## Continuation

Drop every hypothesis that places a key outside its hashed bucket (two-choice, cuckoo, cross-bucket victim areas). They cannot pass P2 slot_legality. Keep v003's mechanics (one hash per op, contiguous bucket storage, in-place policy updates) as the new baseline. Spend the next cycle on eviction and admission policy inside the bucket to recover hit rate. Also test plain LRU over v003's merged buckets, to check whether the split-half logic is hurting hit rate.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three hypotheses started from cycle 0001's v003 and each changed something different:
- v001 replaced the split halves with plain LRU over each merged bucket.
- v002 turned the two segments into a shared SLRU (probation and protected).
- v003 shrank `Item` metadata by replacing the wall-clock timestamp with logical 32-bit fields.

All three are Winners. v001 and v002 recovered hit rate: v001 rose above baseline, and v002 rose clearly above its parent. v003 moved metadata-size for the first time and made put much faster, with hit rate flat.

## Patterns

- Cycle 0001's suspicion is confirmed: the split-half policy was what held hit rate down. Both versions that let every key use the whole merged bucket gained hit rate, and both also removed the partner-path tail. A merged bucket with one shared policy is now the right base.
- Scan resistance matters on this trace. v002's SLRU gained hit rate without adding any per-item metadata, because segment membership is stored in the existing `len0` boundary.
- Get p50 got worse in all three variants, for different reasons. v001 and v002 scan the full bucket and memmove large ranges of 40-byte items on hits and evictions. In v003 the cause is unclear (thread-local clock access or noise). Policy and layout changes keep moving cost onto the median read.
- The three changes are independent and should combine. v003's 24-byte `Item` reduces bytes moved per shift, and that is exactly the cost v001 and v002 added. Under position-ordered policies (plain LRU, SLRU), the timestamp is not read at all.
- Taking `Instant::now()` out of the hot path helped put and tail latency. It was not the source of the get median, so cycle 0001's hypothesis on that point was wrong.

## Open questions

- Does SLRU beat plain LRU on hit rate? Each was compared only against its own parent, so they have never been measured side by side.
- Is the 3/4 protected fraction close to optimal?
- Can a compact key or tag array scanned before the full items, or a ring/offset layout that avoids front-of-Vec shifts, bring back the get median without giving up the hit-rate gains?
- Under SLRU or LRU, can the timestamp field be removed entirely to shrink metadata further, or do other policy kinds still need it?

## Continuation

Use the merged bucket with a shared policy as the new base, combined with v003's 24-byte logical-tick `Item`. Run plain LRU against SLRU on that combined base to settle which policy wins on hit rate, and tune SLRU's protected fraction. Put the main effort into making the get path cheap: fingerprint or tag scans, and layouts that avoid large memmoves on hit and evict. Drop split-half designs and any wall-clock reads on the hot path.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
