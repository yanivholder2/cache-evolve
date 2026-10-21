# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants tweaked promotion logic in `DashSegment::get`/`promotion_get` against baseline: insertion position (v001), promotion eligibility into probing (v002), and probing→target back-promotion (v003). Only v001, the one-argument change to MRU insertion, won.

## Patterns

- Only hit-rate was measured. Metadata-size and per-op-time are unevaluated for every variant, so no cost claims exist yet.
- The winning lever changed where a promoted item lands, not how many items move. Variants that added movement (v002 more promotions, v003 bidirectional swaps) gained little or lost.
- Extra movement tends to cause demotion cascades toward stash and eviction. This offsets or exceeds the benefit of relocating items.
- The recency-skew rationale (don't make just-hit items the next victim) was supported by v001. Spare probing capacity was not a binding constraint (v002, marginal gain).

## Open questions

- Does MRU placement generalize beyond workload-a, and does it cost per-op time?
- Would combining v001 with v002 add anything, or is v002's marginal gain noise?
- Do other insertion positions (demotion path, stash→target vs. probing retries separately) matter?

## Continuation

Build on v001: explore insertion-position variants on other paths (demotion, cascade to stash) and test across more workloads, ideally with per-op-time and metadata in the fitness vector. Drop bidirectional probing→target swaps (v003) and don't expect more from eligibility relaxation alone (v002).


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three placement/eligibility tweaks were tried: MRU placement on the insert path (v001, parent cycle-1 v001), stash→probing promotion eligibility stacked on the MRU winner (v002, same parent), and cold-end demotion of displaced victims from baseline (v003). Hit-rate was unchanged from the parent in all three. v001 and v002 are listed as Winners, but only by matching the incumbent, not by improving on it.

## Patterns

- Every variant reproduced its parent's hit-rate exactly. That points to no-ops on workload-a at this cache size, or to overlays that never changed the executed path. It does not show the levers are harmful.
- The insert-path and eligibility levers add nothing beyond the hit-promotion position change. This supports the view that cycle 1's marginal eligibility gain was noise or already subsumed by MRU promotion.
- The cascade-eviction theory behind cold-end demotion got no support. Demotion position did not change which useful entries were lost.
- Only hit-rate was measured again. Metadata-size and per-op-time are still unevaluated, so there are no cost claims.
- Authors could not read the overlay sources, so the "overlay may not have taken effect" explanation could not be ruled out.

## Open questions

- Do these overlays actually alter the executed code path, or are they dead edits?
- Is workload-a or the cache size insensitive to placement on insert and demotion paths? Other traces or sizes might separate them.
- Does any variant differ on per-op-time or metadata, even with equal hit-rate?

## Continuation

The placement-position family (insert, demotion, eligibility) looks exhausted on workload-a. Drop further variants of it unless they are checked against a different workload or cache size. Next cycle should use a different lever, such as eviction victim selection, bucket and probing structure, or admission, branching from cycle/0001/v001. It should also add per-op-time and metadata-size to the fitness vector so ties can be broken.


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
