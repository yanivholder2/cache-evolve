# Cycle 0010 Insight

## Cohort outcome

All three variants forked from cycle-0007 incumbents and attacked the
"stash coupling" frontier cycle 0009 flagged from three angles: decouple the
put-side cascade by dropping its deepest victim outright instead of re-homing
it in the stash (v001), shorten the median stash get by scanning the bucket
MRU-first (v002), and gate the get-path stash→target promotion when the target
is full to cut a read-side cascade (v003). All three entered the Winners, but
none delivered its premise — every latency hypothesis was refuted, and the one
measurable effect across the cohort was on hit-rate.

## Patterns

- The decouple-by-dropping move is strictly worse, not a trade. v001 was the
  cleanest test of cycle 0009's "resolve victims without growing the stash"
  ask: drop the terminal cascade victim rather than re-insert it. It paid the
  full predicted hit-rate cost (real residency leaves the cache) yet produced
  no observable put-p99 or get-p50 win — the saved stash insert never surfaced.
  Removing a stash insert outright does not beat the parent's re-insert; the
  conceded residency is real while the latency saving is not.
- Cascade/stash latency is now below the noise floor. Two variants moved per-op
  metrics on paths they never touched — v001 raised all four op metrics editing
  only a rare second-order branch, and v002 shifted put-p50/p99 while changing
  only the get-side stash probe. These deltas read as run-to-run variance, not
  cost signal. The cascade events are too rare on this workload to register
  above measurement noise, so structural surgery on them cannot be scored on
  latency at all.
- The read-path cascade is a hit-rate lever pointing the *opposite* way. v003
  gated the stash→target promotion expecting a get-tail-for-residency trade;
  instead hit-rate *rose* (skipping the promotion avoids evicting a target LRU
  resident, retaining more useful keys) and get latency regressed from the
  added per-get branch. The full-target cascade almost never fires here, so its
  removal buys retention while costing only the branch that guards it.

## Open questions

- Both surviving hit-rate effects (v001's loss, v003's gain) come from whether
  a cascade victim is retained or evicted, and both fire rarely. Is residency on
  this workload governed by these rare cascade decisions, or are they too
  infrequent to move the long-run cap meaningfully either?
- The latency frontier is now indistinguishable from run variance on every
  cascade/stash path probed. Is there any op-latency lever left that clears the
  noise floor, or is per-op-time fully converged for this segment?

## Continuation

The stash-coupling latency frontier is closed: three independent routes this
cycle showed cascade and stash-probe surgery moves per-op-time only within
run-to-run noise, and the decouple-by-dropping move (v001) is strictly worse
than the parent, not the clean win cycle 0009 hoped for. Stop scoring cascade
changes on latency — the events are too rare on this workload to register.
What *does* move is hit-rate, and v003 inverted its sign: gating read-path
promotion *retains* keys by not evicting target residents. If the next cycle
pursues residency, treat retention-on-cascade as the lever and expect
counterintuitive signs; do not expect any accompanying latency payoff. Drop
the "shorten the stash to win get-p50" thesis entirely — the median get is at
its noise floor regardless of stash length or scan direction.
