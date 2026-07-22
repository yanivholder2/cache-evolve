# Task: cycle-insight-writer

Cycle: 0003
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0003

## Winners

- v001
- v003

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0003\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0003\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0003\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0003\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0003\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0003\hypotheses\H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three baseline-derived variants each attacked a different fitness axis —
geometry (set-associativity), eviction policy (scan resistance), and item
metadata size — with no clean Winner emerging. Every hit-rate gain was
paid for with a per-op-time regression, and the only unambiguously
positive move was cutting dead metadata.

## Patterns

- Hit-rate and per-op-time moved as an antagonistic pair: both the wider-
  bucket geometry (v001) and the reuse-protection eviction tweak (v002)
  bought hit rate by lengthening a linear bucket/eviction scan, so neither
  is a free win.
- The bucket is scanned linearly, so anything that widens buckets or turns
  an O(1) eviction pick into a front-to-back scan directly taxes every
  operation — scan cost is the dominant hidden price in this cache.
- Metadata carried only by policies that are off the eval path (the
  `ClassicLRU` path never touches timestamps) is pure dead weight and can
  be cut with zero hit-rate cost, unlike the two hit-rate plays.
- Conflict-miss headroom at this working-set size appears thin: aggressively
  raising associativity barely moved hit rate.

## Open questions

- Can scan resistance be gained without converting eviction into a linear
  scan — e.g. maintaining the eviction candidate incrementally rather than
  re-scanning?
- Is there a bucket width that captures the collision headroom without the
  full scan-cost penalty, or is the trade strictly unfavorable here?
- Was v003's p50 drift real or measurement noise? Timing signals this
  cycle were small enough to warrant caution.

## Continuation

Pursue metadata trimming and other "cut dead weight off the eval path"
moves — they are cheap and monotonic. Treat both bucket-widening and
scan-based eviction protection as expensive: only revisit them with an
O(1)-maintained variant that avoids adding a linear scan. Drop plain
associativity increases at this working-set size. Be wary of small per-op
timing deltas; prefer changes with a clear structural reason to move.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle recombined the two cycle-0001 winners along disjoint axes and
probed a policy refinement. The Winners were the clean cross-breed (v001,
stacking scan-resistant reuse-protection onto the slimmed item layout) and
a deeper metadata trim (v003, peeling one more dead field off the eval
path). The policy refinement — trading permanent protection for
second-chance aging — was the sole loser.

## Patterns

- The cycle-0001 winners composed additively with no interaction: the
  hit-rate mechanism (a reuse flag) and the metadata mechanism (layout
  slimming) touch disjoint code, and the reuse signal rides on a field the
  slim layout already keeps, so stacking them cost no extra bytes.
- Metadata trimming stayed monotonic and cheap one field deeper — dead
  weight carried only by off-eval-path policies keeps peeling off with zero
  hit-rate change, only tail-latency noise.
- On this trace, permanent reuse-protection is the feature, not a bug:
  giving protection a self-clearing second chance sheds immunity too fast
  and collapses back toward plain LRU. Cold-but-once-hot "squatters" are
  not what caps hit rate here.

## Open questions

- How much further can metadata shrink before it collides with a field the
  eval path actually reads — is the frontier near its floor?
- Since second-chance aging failed, is there any way to bound protection
  (cap how long a hot item stays immune) without shedding it too fast, or
  is permanent binary protection simply the right fit for this trace?
- The recurring upward p99 tail drift under flat p50 keeps appearing on
  metadata cuts — structural or persistent measurement noise?

## Continuation

Keep stacking the two proven, disjoint mechanisms and keep trimming dead
metadata off the eval path — both remain cheap and additive. Drop
second-chance / CLOCK-style aging on this trace; permanent protection wins,
so refinements should preserve it rather than weaken it. Next cycle should
test where the metadata floor is and whether any hit-rate headroom remains
beyond the now-composed reuse-protection win.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
