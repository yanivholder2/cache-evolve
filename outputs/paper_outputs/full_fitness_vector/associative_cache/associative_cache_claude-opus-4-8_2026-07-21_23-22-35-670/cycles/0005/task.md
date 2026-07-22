# Task: cycle-insight-writer

Cycle: 0005
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005

## Winners

(none)

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0005\hypotheses\H03.md

## Prior cycles digest

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort chased one prize from two directions — recover scan-resistant
hit-rate while holding the 16-byte metadata floor — plus a frequency-based
victim probe. Both floor-preserving paths landed as Winners: relocating the
reuse bit into the unmeasured bucket struct (v001) and buying scan
resistance from insertion geometry alone (v003). The LFU-tiebreak probe
(v002) was the sole loser.

## Patterns

- Scan resistance at the metadata floor is reachable two independent ways:
  explicit reuse state parked off the measured `Item` (v001) and pure
  probationary-front insertion with no per-item state (v003). Both hit
  hit-rate targets; the stateless geometric path was the clean one, while
  the stateful path recovered the same hit-rate but carried an
  implementation tax.
- The size metric weighs only `Item`, so state kept in the bucket is free
  on that axis — the exploit holds, and hit-rate and metadata-size stopped
  being antagonistic for these two Winners.
- The tax on v001 was implementation, not mechanism: a key-matched rebuild
  of the reuse bits on every hit is quadratic and blew out the tails, a cost
  an index-preserving swap would avoid.
- Frequency lost to recency again: evicting the least-frequently-reused item
  on saturated buckets was a worse victim signal than plain LRU-front,
  reaffirming that permanent binary protection fits this trace.
- Overlay hygiene bit v002: touching only the eviction file silently
  forfeited the parent's slim layout and regressed metadata — a win can be
  lost by omission, not just by logic.

## Open questions

- Can the bucket-bit path (v001) shed its quadratic reuse-bit rebuild via
  index-preserving swaps and then match or beat the stateless geometric
  path's clean tails?
- Does stateless probation (v003) leave any hit-rate on the table versus
  explicit reuse tracking, or is insertion geometry alone sufficient here?
- How to keep single-file variants from silently dropping prior-cycle wins
  carried in files they don't touch?

## Continuation

Treat v003's stateless probationary-front insertion as the cheap incumbent
to build on — it lifts hit-rate at the floor with better tails and no per-item
state. If pursuing v001's bucket-bit path, first replace the key-matched
rebuild with an index-preserving swap so the bits move for free. Drop
frequency/LFU-based victim choice; recency-front wins. Guard variants against
inheriting only part of the parent's layout.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried to push past the established hit-rate ceiling at the
16-byte floor from three angles: repairing the stateful reuse-bit path's
quadratic tax (v001), compounding explicit protection onto stateless
front-admission (v002), and widening the probation window via midpoint
insertion (v003). The two Winners each held the line rather than advancing
it — v001 recovered clean tails without moving hit-rate, v002 held the
metadata floor but gained nothing — while the midpoint geometry probe
regressed and lost.

## Patterns

- The index-preserving swap the prior cycle called for works exactly as
  predicted: mirroring each policy's single-element relocation onto the
  parallel reuse vector collapses the p99 tails and put p50 that the
  quadratic key-matched rebuild had inflated, with hit-rate held bit-for-bit
  identical. The parent's only real tax was the rebuild algorithm, not the
  scan-resistance mechanism.
- Explicit reuse protection and stateless front-admission are the same lever
  expressed twice, not compounding ones. Under move-to-back-on-hit, the
  frontmost slot is *always* the probationary one, so an added reuse bit
  resolves to the eviction index the position ordering already selects —
  v002 came out behaviorally indistinguishable from its parent. Protection
  adds value only over a policy that does not positionally segregate reused
  items.
- The admission-geometry axis has a sharp optimum at the front slot.
  Midpoint admission is strictly worse: the longer proving window it grants
  hot-but-unrehit items is dwarfed by the cost of letting never-reused scan
  items linger above the eviction slot, dropping hit-rate and nudging tails
  up. Front's value is precisely that a fresh unproven miss is the next
  thing evicted.
- The hit-rate ceiling held against every angle this cycle. Two independent
  attempts to exceed it at the floor produced no gain — evidence the current
  scan-resistance mechanisms are saturated, not that a better encoding of
  the same signal remains to be found.

## Open questions

- v001 flagged a metadata regression off the 16-byte floor that its own diff
  cannot explain (`Item` is still key+value; the reuse vector lives on the
  unmeasured bucket). Is this an overlay/inheritance artifact rather than a
  real layout change?
- If explicit protection and front-admission are redundant under the current
  policy, is there hit-rate headroom only reachable by changing the base
  policy's ordering itself — or is this trace's ceiling genuinely structural?
- Every stateless and stateful path now converges on the same hit-rate.
  Where does the remaining headroom, if any, actually live?

## Continuation

Adopt v001's index-preserving swap as the clean stateful incumbent — it
delivers the floor-level hit-rate with tails matching the stateless path.
Stop stacking explicit reuse protection onto front-admission: the two levers
are redundant under move-to-back-on-hit, so future protection ideas must
override a policy that does *not* positionally segregate reused items to
have any effect. Treat the front slot as the settled optimum on the
admission-geometry axis and drop midpoint/wider-probation variants. Next
cycle should first resolve v001's unexplained metadata regression (likely an
overlay artifact), then, since same-signal re-encodings have plateaued,
probe whether any hit-rate headroom requires changing the base policy
ordering rather than layering state on top of it.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
