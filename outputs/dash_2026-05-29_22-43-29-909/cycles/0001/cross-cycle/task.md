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
