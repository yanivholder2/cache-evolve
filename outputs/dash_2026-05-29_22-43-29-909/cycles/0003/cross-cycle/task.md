# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The first cohort spread three single-lever bets off the baseline across
orthogonal axes — promotion position (recency), lookup ordering (get
latency), and item layout (metadata size). Only the promotion-position
variant landed on the Pareto front, and it did so on latency, not on the
hit-rate axis it was aimed at.

## Patterns

- The bets were near-independent, each touching one axis, so the cohort
  reads as a probe of which axis is cheapest to move rather than a
  coordinated push — and the axis that moved was not the one intended.
- The recency-favoring retention bet backfired: on this scan/loop-heavy
  LIRS-style trace, biasing promotion toward the MRU end keeps
  soon-cold keys resident and costs hits. Recency is the wrong signal
  here; reuse-distance or frequency awareness is what the workload rewards.
- The latency gain that did land came from a structural side effect (an
  O(1) tail push replacing a whole-bucket shift), but it arrived coupled
  to one policy's bucket geometry rather than as a clean, policy-agnostic
  promotion-path change.

## Open questions

- The latency-reorder and metadata-shrink bets did not surface on the
  front — were they dominated, behavior-neutral, or simply not realized?
- Can the tail-push latency win be isolated from the recency policy that
  rode in with it, so we keep the speed without the hit-rate penalty?

## Continuation

Next cycle should pursue retention driven by frequency or reuse-distance
rather than pure recency, since recency is actively penalized on this
trace. Preserve and decouple the O(1) tail-insert latency win from policy
geometry. Drop further pure-recency promotion bets on this workload, and
revisit whether the latency and metadata levers are worth retrying as
clean, behavior-preserving changes.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This was a follow-through cohort that took up cycle-1's three open threads
directly: frequency-driven retention (replacing the recency promotion
trigger), preserving-and-gating the v001 latency win behind a reuse
filter, and the metadata lever retried as a clean field-narrowing rather
than removal. Only the frequency-promotion variant held the front, and it
did so without demonstrably improving any axis.

## Patterns

- Two independent measurement-decoupling failures dominated the cohort,
  not policy logic. The frequency variant's hit-rate effect could not be
  attributed because the evaluated parent vector has no entry for the
  scan/loop trace it targets; the narrowing variant compiled and ran yet
  the metadata-size reading never moved, because the measured Item byte
  figure is sourced from somewhere other than the overlaid struct. The
  blocking factor on both the hit-rate and metadata axes is metric
  plumbing, not the change.
- Frequency as a signal is cheap and behavior-compatible (it reuses an
  existing per-item counter, so metadata held flat), but the chosen rule —
  strict pre-hit maximum with ties never promoting — is too conservative
  on a loop workload where residents share similar counts, so the hot tier
  stays under-filled. It also taxed get latency by swapping an O(1)
  positional check for an O(n) bucket scan.

## Open questions

- Where does the metadata-size number actually come from? Editing Item
  field widths in the overlay does not change it.
- How can hit-rate on the scan/loop trace be compared when the parent's
  fitness vector lacks an entry for that trace?
- The second-hit admission gate that would keep v001's O(1) back-push while
  filtering scan keys did not surface a result — was it run?

## Continuation

Before spending more attempts on retention or metadata, fix the
measurement path first: get the scan/loop trace into the evaluated vector
so hit-rate changes can be attributed, and locate the true source of the
metadata-size figure. Keep frequency as a promotion signal but drop the
strict-max/ties-never rule and the O(n) per-hit scan in favor of a cheap
threshold (e.g. second-hit admission) that preserves the O(1) promotion
path — this reuse-gate refinement of the front-runner is the most
promising live direction. Drop pure Item field-narrowing until the metric
source is found.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three bets all aimed at the same goal — lift hit rate on the scan/loop
trace — by three different mechanisms: cycle-2's requested second-hit stash
admission gate, its requested cheap O(1) counter-threshold promotion, and a
brand-new lever, eviction-victim selection. Only the eviction-victim bet
held the front, landing as a recency-promote / frequency-evict hybrid that
kept metadata flat but cost get latency.

## Patterns

- The cohort opened a genuinely new axis: eviction-victim selection is
  independent of the promotion/position levers every prior cycle worked.
  The front-runner kept promotion strictly recency-based and changed only
  *who leaves* — choosing the least-reused resident — proving the victim
  slot is a separately tunable lever, and it stayed metadata-free by reusing
  the existing per-item counter.
- The same measurement-plumbing blocker flagged in cycle 2 is still
  unresolved and is now the binding constraint for a third straight cycle:
  every hypothesis targeted the scan/loop trace, but the parent's fitness
  vector still has no entry for it, so the entire hit-rate thesis behind the
  whole cohort remains unjudgeable. The front moved on latency/metadata
  defaults, not on the axis the cohort was actually chasing.
- The new lever bought its (unmeasurable) hit-rate intent with a latency
  tax — a per-hit counter write plus an O(n) full-bucket victim scan —
  echoing the cost structure of cycle-2's frequency promotion.

## Open questions

- When does the scan/loop trace finally enter the evaluated vector? Until it
  does, hit-rate-targeted bets cannot be attributed, and this has now
  blocked three cycles.
- The two promotion-path refinements cycle 2 called the most promising live
  direction did not surface as front results again — were they realized this
  cycle or did they fail to emit comparable output as before?
- Is the frequency-evict hybrid worth keeping while its hit-rate payoff
  cannot be measured, given it costs latency now for an unverified gain?

## Continuation

Fix the measurement path before spending more bets: get the scan/loop trace
into the evaluated vector so hit-rate is attributable — this is now a
prerequisite, not optional, because every cohort that ignores it just moves
the front by latency/metadata side effects. Keep the eviction-victim lever
as a real new direction but replace its O(n) victim scan with a cheaper
selection. The cheap-reuse-gate promotion refinement has now failed to
produce a comparable result twice; confirm it is actually being realized or
drop it.


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
