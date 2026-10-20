# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle probed three follow-ups to the cycle-0001 stash-vs-normal theme:
pushing the bucket split further toward the protected tier, stacking a
selective stash gate on the even split, and swapping the protected tier's
eviction policy. The winner (v001) pushed the split past even toward
normal-dominant and kept gaining hit-rate; the gate-stacking and policy-swap
variants did not improve on it.

## Patterns

- The capacity-reallocation axis from cycle 0001 has not yet saturated:
  moving budget further into the promotion-protected normal tier (even →
  normal-dominant) still climbed hit-rate, so the optimum lies beyond the
  even point — the baseline stash was over-provisioned more than one step
  could correct.
- The two stash-protection levers proved redundant, not additive: once the
  protected tier was enlarged by capacity, layering a selective-admission
  frequency gate on top yielded steeply diminishing returns. Most of the
  available benefit is captured by *how much* room the protected tier gets.
- Changing *how* the protected tier ranks residents was a dead lever:
  forcing LFU eviction onto the normal tier regressed, since the trace's
  skew is already well-served by recency/promote-on-hit and LFU retained
  stale-but-frequent keys over the live working set.
- All movement stayed budget-conserving and invariant-preserving; the only
  regression came from the policy swap, not from any capacity reshape.

## Open questions

- Where exactly does the normal-dominant split stop paying off — is there a
  point where too small a stash starves admission and regresses? The curve
  is still rising but unbounded sampling hasn't found its peak.
- Are there protected-tier levers beyond capacity and eviction-policy that
  haven't been touched (e.g. promotion position, segment count)?

## Continuation

Keep sweeping the normal/stash split toward the normal tier to bracket the
hit-rate peak — v001 confirms the curve is still climbing, so push past 3:1
until it turns over, then back off. Drop two directions as settled: stacking
selective-admission gates on top of the capacity reshape (redundant), and
swapping the protected tier's eviction policy to LFU (regresses). If the
split curve flattens, shift to an untried protected-tier lever rather than
re-running admission filters.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
