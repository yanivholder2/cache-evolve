# Variant v002 Insight

## Helped

- hit-rate: Widening the probationary scan from k=4 to k=6 produced a marginal positive movement, suggesting the two extra candidate positions occasionally surface a colder victim that the 4-item scan would miss.

## Hurt

## Takeaway

Extending `EVICTION_SCAN_K` from 4 to 6 under the accurate-sketch regime (8× width + conservative update) yields only a negligible hit-rate gain, confirming that the vast majority of eviction decisions are not improved by looking two positions deeper into the LRU tail. While positions 5–6 do contain a colder candidate often enough to register a positive signal, the effect is far too small to suggest the victim-scan window is a meaningful lever at this cache size. Further improvements should target a different axis — segment sizing, PROMO_THRESHOLD gating, or admission policy — rather than continuing to widen k.
