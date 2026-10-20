# Variant v002 Insight

## Helped

- hit-rate: `DashSegment::get` now passes `InsertPosition::Back` to `promotion_get`, so a stash-to-target promotion lands at MRU in its target bucket instead of LRU. A promoted item therefore survives one more eviction round before it can be displaced, which is the mechanism the hypothesis predicted for the LIRS workload.

## Hurt

## Takeaway

The only lever changed is where stash-to-target promotions land, and on this workload that lever favors one-shot reuse: giving a promoted item MRU status protects it from the next eviction, at the cost of the cold-entry protection the baseline's Front default encoded. The per-op-time dimension also moved favorably, but this edit touches no per-op code path (only the insert position of promotions), so that movement should not be credited to the change and is worth re-measuring before it is relied on.
