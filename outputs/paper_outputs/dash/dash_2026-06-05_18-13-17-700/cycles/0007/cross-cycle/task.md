# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-lever reshapes of the baseline DASH tiering,
each targeting hit-rate on a skewed trace without touching capacity. The
winner (v002) repartitioned fixed budget toward the promotion-protected
normal tier; the two stash-admission tweaks (v001 back-insertion, v003
frequency gate) also moved hit-rate favorably but did not top it.

## Patterns

- Every variant converged on the same lever family: protect the warm working
  set in the normal tier by reducing how easily transient keys disturb it,
  whether via capacity reallocation, insertion position, or a promotion gate.
- The baseline appears to over-provision the cold stash admission area; both
  shrinking it (v002) and making it more selective (v001, v003) helped, which
  points to a single underlying weakness rather than three independent ones.
- All wins were budget-conserving and invariant-preserving — no dimension
  regressed in any variant, so the cohort moved along one axis (hit-rate)
  with no observed trade-off cost.

## Open questions

- Are these levers additive or redundant? They each attack the same
  stash-vs-normal imbalance, so stacking v002 with v001/v003 may yield
  diminishing or conflicting returns.
- Both v002's split ratio and v003's promote threshold are unswept knobs; the
  reported wins are single points, not optima.
- Results rest on one skewed trace; the stash-protection bias may not
  generalize to less skewed or churn-heavy workloads.

## Continuation

Pursue the stash-vs-normal balance as the dominant theme — the winning move
was reallocating budget to the protected tier. Next cycle should sweep the
v002 split ratio and v003 threshold for optima, and test whether combining
the split shift with a selective stash gate compounds or cancels. Drop pure
re-runs of single-lever baselines; the cohort has established the direction.


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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle ran two converging tests on the promotion gate and opened one
brand-new seam on the stash. The promotion-side probes — a density-independent
per-key touch counter (v001) and loosening the len-2 hottest-only boundary
(v003) — came back flat and slightly negative respectively. The genuinely new
lever, a cuckoo-style stash relocation hop from a clean baseline (v002), was
the only clean positive move. The winner was v001, but its gain was
noise-level; v002 is the more meaningful result.

## Patterns

- The promotion gate is now closed from two independent directions. The touch
  counter replaced *rank* with a density-independent warmth signal and still
  produced nothing — proving the small-bucket ceiling is structural, not a
  rank artifact. The len-2 loosening over-admitted and edged down — proving the
  withheld second resident is genuinely cold, not a warm key held back. Both
  reads point the same way: no promotion-side signal, of any shape, beats the
  occupancy ceiling on this trace.
- The stash's load-bearing property remains *dwell time*, and it is the only
  axis still yielding gains. A single relocation hop into a salt-derived
  alternate bucket lets a few more cold keys catch their second reference
  before dropping — positive, no regression, capacity conserved. This is the
  freshest live lever in the whole search.
- The relocation gain is marginal for a structural reason, not a logic flaw:
  the alternate stash bucket is usually full too, so one hop rescues only a
  sliver of the dwell-time headroom the stash holds.

## Open questions

- The stash relocation is the first new positive seam in several cycles — does
  it deepen (multi-hop cascade, smarter overflow-victim selection, larger
  alternate set) or does it saturate as fast as every prior lever once the
  alternate buckets fill?
- With the promotion gate now exhausted by both rank and touch-count signals,
  is *every* hit-rate lever on this trace ultimately gated by the same hashing
  skew, or is the stash dwell-time axis structurally different?

## Continuation

Stop all promotion-gate work — rank, fraction, and now density-independent
touch-counting are all confirmed dead against the small-bucket ceiling, and
the len-2 boundary is doing real work that must not be loosened. The live
frontier is stash dwell-time: v002's relocation hop is the only fresh positive
result, so next cycle should deepen it (better victim selection, a second hop,
or a larger alternate set) rather than chase more promotion variants. The
recurring skew/occupancy bottleneck — now blocking levers across five cycles —
makes validating against a second, less-skewed trace overdue before investing
further.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle drilled entirely into cycle 0006's one fresh seam — the stash
relocation hop — testing three ways to deepen it: grafting the single hop
onto the proven protected-tier winner (v001), balancing admission across both
homes via power-of-two-choices (v002), and extending the single hop into a
bounded multi-hop cuckoo cascade (v003). The declared winner was v001, but its
gain was noise-level; v003's cascade was the only clean, regression-free
positive and is the meaningful result.

## Patterns

- The stash dwell-time seam responds to *depth*, not breadth. A bounded
  multi-hop cascade — evicting an LRU resident at each full bucket and chasing
  free space a few buckets deep — converts would-be drops into survivors and
  lifts hit-rate with capacity conserved. The single hop left this headroom on
  the table precisely because the lone alternate home was usually full too.
- Pre-emptive load-balancing is the wrong frame for the same seam. Spreading
  cold keys evenly across both homes at admission time scatters keys away from
  the primary-first locality the get path expects, deferring rather than
  preventing drops, and regressed. Reactive rescue of genuine collision victims
  beats proactive evenness — the value is dwell time, not balanced occupancy.
- The late slivers do not compound. Stacking the single-hop relocation onto the
  protected-tier/two-speed-promotion winner came back flat: the two seams chase
  the same boundary key pool, and keys the relocation would rescue are already
  reclaimed through promotion. Independently-discovered gains overlap rather
  than add.

## Open questions

- Does the cascade keep paying out if deepened further (larger hop budget,
  smarter victim selection than LRU), or does it saturate once the reachable
  bucket neighborhood fills — the same occupancy ceiling in a new guise?
- With combination shown non-additive here, is there *any* pair of discovered
  seams on this trace that stacks, or is every lever ultimately competing for
  the same thin pool of boundary-oscillating keys?

## Continuation

The live frontier is the stash cascade depth (v003), not the single hop or
balanced admission — drop power-of-two-choices admission (it regressed; primary
locality matters more than even fill) and stop trying to graft the single hop
onto the protected-tier winner (confirmed overlapping, not additive). Next
cycle should deepen v003: test victim-selection policy and hop-budget
sensitivity to find where the cascade saturates. The recurring finding that
seams overlap rather than compound, plus the five-cycle skew/occupancy
bottleneck, makes validating against a second, less-skewed trace overdue before
investing further in this single workload.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
