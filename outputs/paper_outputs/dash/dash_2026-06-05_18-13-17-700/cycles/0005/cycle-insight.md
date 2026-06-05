# Cycle 0005 Insight

## Cohort outcome

This cycle stayed inside the admission/promotion-dynamics frontier that
cycle 0004 opened, probing three follow-ups against the same parent: a
fractional top-*half* promotion gate that engages on small buckets, cold-vs-warm
differentiation by stash insert position, and a two-speed promotion that slows
the stash→target hop. The winner (v003) was the two-speed promotion, but its
gain was a sliver; the half-gate (v001) also edged up marginally, while the
stash position-differentiation (v002) regressed.

## Patterns

- The promotion gate's occupancy bottleneck is now confirmed as a property of
  the trace, not the rule shape. Replacing the quarter-gate's `len < 4` cliff
  with a fractional half-gate did unlock promotion on the len-3/len-4 buckets
  the parent flagged as starved — so gate *direction* was right — but target
  buckets skew so hard toward len-1/len-2 that the widened branch reaches only
  a thin slice. Gate-fraction widening has nearly exhausted its headroom.
- Two-speed promotion is a genuinely distinct, working lever: landing the
  first (stash→target) hop slow while keeping the second (target→probing) hop
  fast reserves the protected tier for keys with two demonstrated accesses and
  blocks one-hit cold spikes from displacing the hot set. It gains where uniform
  promotion had flattened — but the same small-bucket occupancy caps it, since a
  slow-landed key climbs to promotable rank in an access or two anyway.
- Differentiating cold admissions from warm demotions by stash insert position
  is the wrong lever and re-confirms the cycle-0004 finding that the stash's
  value is giving *new* keys dwell time to prove themselves. Front-inserting
  cold keys makes them the immediate eviction victim, denying that residency and
  gutting the load-bearing admission step — a regression, as in 0004 v002.

## Open questions

- Both upward levers this cycle (half-gate, two-speed promotion) are throttled
  by the same root cause: target buckets rarely hold enough residents for any
  rank/recency rule to have daylight. Is there any path to denser target buckets,
  or does the trace's hashing structurally cap every promotion-side lever?
- The stash insert-position axis is now closed in *both* directions
  (front-inserting cold keys hurts; the protective MRU default holds) — is any
  remaining stash-dynamics lever left untouched, or is the stash fully settled?

## Continuation

Treat the promotion gate as occupancy-saturated — stop widening its fraction;
every rank rule degenerates on len-1/len-2 buckets regardless of shape. Drop
stash insert-position differentiation entirely: cold keys need dwell time, and
front-landing them regresses (now shown twice). The freshest live lever is
v003's two-speed promotion — worth combining with a denser-bucket angle, since
both winning levers are gated by the same small-bucket ceiling rather than by
their own logic. Before spending more cycles chasing slivers, the recurring
occupancy/skew bottleneck strongly suggests validating against a second,
less-skewed trace to learn whether any of these levers generalize.
