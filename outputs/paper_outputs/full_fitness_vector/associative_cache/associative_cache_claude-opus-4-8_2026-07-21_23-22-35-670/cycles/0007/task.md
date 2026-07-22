# Task: cycle-insight-writer

Cycle: 0007
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007

## Winners

- v002
- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0007\hypotheses\H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort attacked the plateaued hit-rate ceiling exactly where cycle 4
pointed — the base policy's promotion ordering — from three angles:
gradual single-step promotion (v001), a two-hit reuse-bit probation gate
(v002), and a genuine two-segment SLRU bucket (v003). No variant qualified
as a Winner, but the cohort produced its first real evidence that hit-rate
headroom exists and is reachable only by adding true segment structure, not
by re-timing a single-list promotion.

## Patterns

- Re-timing promotion on a single list does not buy hit-rate on these
  short associative buckets. Slowing promotion (single-step, v001) actively
  regressed — a hot key gets evicted before it can ratchet out of the front
  zone — and gating promotion behind a reuse bit (v002) left the resident
  set unchanged, buying latency but not accuracy. Gradual/frequency-like
  promotion needs more slots than a small bucket provides.
- Genuine segment separation is the lever that moved hit-rate. Only v003,
  which gives twice-hit keys a reserved protected segment a one-hit scan
  burst cannot evict, crossed the front-admission ceiling. This confirms
  cycle 4's read that remaining headroom requires changing the base
  ordering itself, and identifies *segmentation* (2Q/SLRU-style) as the
  specific structural change that unlocks it.
- The hit-rate gain is bought with latency, not metadata. v003 keeps its
  segment boundary off-`Item` (16-byte floor holds), but every access now
  rotates and rebalances two segments, pushing get/put tails up. The new
  frontier is a hit-rate/latency trade, no longer a hit-rate/metadata one.
- The index-preserving swap keeps paying off (v002 collapsed the quadratic
  p99 tails as predicted), but the overlay-hygiene hazard recurred: v002
  carried only the bucket file and silently reverted `Item` off the 16-byte
  floor to the 40-byte base — a metadata regression by omission, not logic.

## Open questions

- Can v003's SLRU win keep its hit-rate gain while shedding the per-access
  rotate/rebalance latency — e.g. cheaper boundary bookkeeping or lazy
  demotion?
- What protected-segment cap is optimal for these short buckets, and does
  the gain survive at the smallest bucket sizes?
- How much headroom is left above the SLRU ceiling, or is two-segment
  separation itself near the structural limit for this trace?

## Continuation

Promote v003's segmented (SLRU) bucket to the incumbent worth building on —
it is the only path this cycle that broke the hit-rate ceiling, and it does
so at the metadata floor. Next cycle should focus on reclaiming its latency:
reduce the per-access rotate/rebalance work and tune the protected cap,
rather than chasing further hit-rate. Drop single-list promotion re-timing
(single-step under-protects; reuse-bit gating is redundant with position) —
the evidence is now firm that segmentation, not promotion timing, is the
lever. Carry v002's index-preserving swap into any stateful path, and
enforce full parent-layout inheritance so wins like the 16-byte `Item` are
not dropped by single-file overlays.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort worked to consolidate cycle 5's segmentation win rather than
chase new hit-rate: reclaim the metadata floor the SLRU incumbent had lost
by omission (v001), shed its per-access rotation latency (v002), and probe a
bounded protected cap to keep a live probationary zone (v003). The clean
win was v001 — the segmented bucket now delivers its ceiling-breaking
hit-rate at the 16-byte floor. v003 held the floor but its cap bet
regressed hit-rate; v002's lazy reorder recovered latency but leaked a hair
of hit-rate.

## Patterns

- The cycle-5 "40-byte" metadata cost of segmentation was never a cost of
  segmentation — it was pure overlay-hygiene debt. Trimming the dead
  `Item` fields the SLRU bucket never reads was behaviourally inert on
  hit-rate and reclaimed the full floor, and incidentally sped the
  byte-shifting put path. The recurring overlay hazard, once repaired,
  cost nothing.
- The per-hit protected rotation is confirmed pure overhead in the
  mixed-segment case — but it is not free to drop. It stops mattering only
  while a probationary item shields index 0; once a bucket saturates to
  all-protected, that same rotation was silently maintaining the intra-
  protected eviction order, so removing it trades eviction quality for the
  latency win. Latency reclamation here is entangled with the saturated-
  bucket edge case.
- The protected-cap knob is real but sharply mis-tuned at `max_size/2`.
  Capping that aggressively on short associative buckets demotes genuine
  reuse winners back to probationary and evicts them as scan-fodder,
  lowering hit-rate rather than lifting it. The "protection saturates to
  LRU" concern is genuine, but the corrective cap must sit close to
  `max_size` and demote only when protection is near-total.

## Open questions

- Can the protected rotation be shed for latency without the saturated-
  bucket hit-rate leak — e.g. lazy demotion that reorders only when the
  bucket is fully protected?
- Where does the optimal protected cap actually sit for these short
  buckets — near `max_size`, and does a near-total-only demotion trigger
  turn v003's regression into a gain?
- With the metadata floor now recovered on the segmented incumbent, is
  there any hit-rate headroom left above the SLRU ceiling, or is two-
  segment separation the structural limit for this trace?

## Continuation

Adopt v001 as the incumbent: it is the segmented (SLRU) bucket delivering
the ceiling-breaking hit-rate at the 16-byte floor, with the overlay-
hygiene debt finally cleared — build on it, not on cycle-5/v003. Do not
re-run the aggressive `max_size/2` cap; if pursuing a protected cap at all,
set it near `max_size` and demote only when protection is near-total, and
verify it against v003's regression. Treat v002's latency play as
conditional, not settled — the rotation may only be safely dropped when the
bucket is not fully protected, so any lazy-reorder retry must handle the
saturated case explicitly. Enforce full parent-layout inheritance on every
overlay so the reclaimed 16-byte `Item` is never dropped again.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
