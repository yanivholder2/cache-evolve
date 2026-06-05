# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Tier-skip (stash→probing bypass of target) skips target's filter that ensures items are proven hot in their bucket, allowing unvetted stash hits directly into probing, which contradicts the back-insertion demotion assumption that only tier-advanced items deserve MRU re-insertion.

## Takeaway

Adding stash→probing tier-skip to the proven demotion-back-insertion architecture confirms a fundamental filtering-discipline incompatibility. The demotion philosophy rewards tier-advanced items with MRU insertion because they've proven value across tiers; tier-skip undermines this by allowing stash hits (which have no such proof) to bypass target's filter and land at MRU in probing. This pollutes the probing tier with unvetted items, degrading hit-rate. Tier-skip only works when combined with front-insertion (forcing items to climb the ladder), not when demotions use back-insertion.
