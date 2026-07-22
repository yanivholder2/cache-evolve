# Task: cycle-insight-writer

Cycle: 0008
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008

## Winners

- v002
- v003
- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0008\hypotheses\H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort probed for hit-rate headroom *above* the recovered SLRU
incumbent from three directions — widening per-set associativity (v001),
giving the bucket cross-eviction ghost memory in 2Q style (v002), and
retuning the protected cap toward near-total protection (v003). None gained
on any dimension; the two Winners (v001, v002) qualified only by holding the
metadata floor while every hit-rate bet regressed. This cycle is a cluster
of falsifications: it maps three walls around the current ceiling rather
than breaking through it.

## Patterns

- Set *diversity* beats per-set *depth* at this size. Trading bucket count
  for bucket width (v001) gave each SLRU set more promotion slots but
  coarsened the hash spread, and hit-rate fell — so the ceiling is not a
  bucket-width limit. Widening also surfaced the predicted scan cost as a
  doubled `get` tail without moving the median.
- Cross-eviction memory does not lift hit-rate here. Admitting ghost-hit
  keys as pre-protected (v002) over-inflates the protected set: a
  re-reference is not proof of long-term heat, and blanket protection erodes
  the very reuse-bit discrimination that earned the incumbent its ceiling.
  It also taxed every `put` with linear ghost bookkeeping.
- The protected-cap knob has now been falsified at both extremes. Cycle 6
  ruled out `max_size/2` (too aggressive); v003 rules out `max_size - 1`
  (near-total protection degenerates the two-segment order into LRU, killing
  scan-resistance and lengthening protected-MRU rotations). The optimum sits
  strictly between the ends, not at either wall.
- Off-`Item` metadata hygiene held everywhere this cycle — ghost rings and
  cap knobs all stayed off the measured struct, so the 16-byte floor was
  never at risk. The overlay hazard that recurred in prior cycles did not
  reappear.

## Open questions

- If neither more associativity, more memory, nor more protection lifts
  hit-rate, is two-segment SLRU at diverse small buckets simply the
  structural ceiling for this trace?
- Where does the protected cap actually optimize, now that both extremes are
  falsified — is there a mid-range setting that turns a hold into a gain, or
  is the default already near-optimal?
- Is any remaining headroom a hit-rate question at all, or has the frontier
  fully shifted to reclaiming the incumbent's per-access rotation latency?

## Continuation

Treat the segmented incumbent's hit-rate as at or near its structural
ceiling for this trace: three independent headroom bets (wider
associativity, 2Q ghost memory, near-total protection cap) all regressed, so
stop chasing hit-rate gains through more depth, more memory, or more
protection. Keep the metadata floor and hash-spread diversity as settled —
do not trade bucket count for bucket width again. If the protected cap is
revisited, search the mid-range only; both extremes are now falsified. The
most promising unclaimed lever is latency: the per-access protected rotation
remains pure overhead outside the saturated-bucket case, so next cycle
should pursue lazy/conditional demotion to shed rotation cost while holding
the ceiling, rather than another hit-rate probe.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
