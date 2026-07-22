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
