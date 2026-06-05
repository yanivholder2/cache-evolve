# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle moved past the split-ratio sweep that dominated cycles 0001–0002
and probed three distinct geometry/promotion levers: pushing the normal/stash
split one step further, accelerating promotion via MRU insert position, and
trading bucket count for bucket width. The winner (v002) was the
promotion-position flip, but its gain was only a sliver; the split-push barely
turned over, and the width-for-count trade regressed.

## Patterns

- The capacity-reallocation axis is now confirmed saturated: pushing the split
  past the prior winner did not climb and edged fractionally lower, pinning the
  hit-rate peak at or just past the inherited ratio rather than deeper toward
  the baseline. The over-provisioned-stash story from cycle 0001 has been fully
  corrected — further starving the stash now costs admission as much as it saves.
- The protected tier's internal levers are mostly exhausted on this trace. The
  winning promotion-position change (insert at MRU so hot keys climb in one
  follow-up access) helped only marginally, because high-skew re-touch is fast
  enough that the "earn your way up" delay was barely costing hits.
- Bucket addressability beats bucket depth at conserved budget: widening
  buckets while halving their count regressed, so hot keys collided into fewer
  landing spots. Bucket count is a value to preserve, not a free associativity
  knob to spend.

## Open questions

- With both the split axis and the promotion-position lever now near-exhausted,
  where is the next real source of hit-rate on this skewed trace? Every
  geometry/promotion lever touched so far yields diminishing returns.
- Has the cohort over-fit to a single high-skew LIRS trace? The flattening of
  every lever may be a property of this workload rather than of the design.

## Continuation

Treat the normal/stash split and the protected-tier promotion mechanics as
settled — both are near their peak and further probes there will only return
slivers or regressions. Drop bucket-widening entirely (count dominates width).
Next cycle should look outside the geometry/promotion family that cycles 0001–0003
exhausted: explore admission/eviction dynamics, segment count, or validate
against a second, less-skewed trace before assuming any lever generalizes.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle finally left the saturated split/promotion-position family and
probed three new admission/eviction-dynamics levers: widening the
target→probing promotion *gate* (top-quarter instead of strict-hottest),
removing stash admission probation entirely, and re-landing demoted keys at
MRU. The winner (v001) was the widened gate, but its gain was only a sliver;
the demotion-MRU change (v003) also lifted hit-rate, while removing stash
admission (v002) regressed sharply.

## Patterns

- The stash's admission step is confirmed as warm-set protection, not dead
  probation: eliminating it let cold one-hit-wonders displace resident hot
  keys and cascaded the working set out, regressing well below baseline. The
  "over-provisioned stash" theme from cycles 0001–0002 had a floor — the
  stash filter itself is load-bearing and must not be removed.
- The promotion *gate* is a real but bottlenecked lever. Admitting more of
  the warm set into probing is directionally correct, but on this heavily
  hashed trace target buckets are usually too small for the quarter rule to
  engage, so the small-bucket fallback caps the gain.
- The demotion-path insert position is a genuinely independent lever from the
  hit-path position that prior cycles exhausted. Keeping probing-evicted keys
  at the MRU end protects the oscillating LIRS working set and gains hit-rate
  where the hit-path position had already flattened.

## Open questions

- Both winning levers (gate width, demotion MRU) gave only slivers and are
  gated by bucket occupancy — is there headroom if target buckets were
  denser, or does the trace's hashing structurally cap them?
- Are v001's gate widening and v003's demotion-MRU additive, or do they both
  chase the same small pool of boundary-oscillating keys?
- Every lever's reach now seems bounded by bucket occupancy and trace skew;
  has the cohort over-fit to this single high-skew trace?

## Continuation

Treat stash admission as settled and protective — do not probe removing or
gutting it again. The live frontier is eviction/demotion dynamics: v003's
demotion-MRU landing is the freshest real lever and worth combining with
v001's widened gate to test additivity. The promotion gate is real but
occupancy-bottlenecked, so any further gate work should target a fractional
rule that engages below small-bucket thresholds rather than re-running the
quarter rule. Before investing more, validate these slivers against a
second, less-skewed trace — the diminishing returns may be a property of
this workload.


### Cycle 0005

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
