# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/hypotheses/H03.md

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
