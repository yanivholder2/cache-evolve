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
