# Cross-Cycle Insights

_Projection of 1 completed cycle (0001). Each pattern below comes from a single cohort, so treat it as provisional until a later cycle confirms it._

## Recurring wins

- Retention decisions gated on reuse evidence (e.g., an `lfu_counter`-driven stash admission gate) improved hit-rate and get tail together. This is the strongest candidate main line so far.
- Changes that cut multi-bucket moves per hit (promotions plus demotion cascades) were faster. In this design, hit-rate and per-op latency share that one cost, so cutting moves tends to help both.
- Reuse-aware gains were largest on hm_0 and at the large cache size, where the reused working set fits once churn is suppressed.

## Recurring failures

- Placement driven only by free capacity (balanced / free-slot placement) left hit-rate flat under pressure and kept cold items when there was slack. Its latency gain is unconfirmed.
- Changes that add promotion churn (aggressive MRU landing on every promotion) lost time overall, even though each insert was cheaper.
- Fixed per-hit bookkeeping (counter writes, comparisons with the victim) adds a small median-get cost, even when it cuts the tail.

## Open questions

- Where does the reuse gate's median-get penalty come from: per-hit counter increments in target/probing, or the victim comparison at admission?
- Does protective MRU landing applied only to demotions (`demote_pos = Back`, `promote_pos` left at Front) raise hit-rate without extra churn? Is it additive with the reuse gate?
- Should promotion from target to probing also require reuse evidence?
- Does the chain-shortening latency gain from free-slot moves hold when re-measured, and can it be kept for demotions only, so cold evictees don't fill probing?
- Should the gate threshold grow with cache size?

## Per-policy notes

- Dash segment (target/probing/stash tiers): where an item lands and the promotion rules are coupled. Landing position sets how fast an item climbs tiers on its next hit, not only its eviction order. The hottest-in-target → probing rule makes this coupling stronger.
- Stash: admission gating by reuse is the only stash-level lever tried so far that raised hit-rate.
- Item layout has not been explored yet. All variants so far changed only promotion and placement.
