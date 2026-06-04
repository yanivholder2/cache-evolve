# Variant v002 Insight

## Helped

- hit-rate: Replacing list-relinking with a per-slot reference bit preserved enough recency signal for the cache to register hits on the LIRS workload — the CLOCK sweep keeps the hot working set resident across cold scans even without strict LRU ordering.

## Hurt

- hit-rate: CLOCK's second-chance approximation discards strict recency order, so victims chosen by the hand-sweep can be slightly "fresher" than the true LRU tail; on a recency-skewed trace this caps how close CLOCK can get to LRU's score.

## Takeaway

Swapping the doubly-linked list for a fixed ring plus reference bits is a viable structural simplification on LIRS-style traffic: it removes four pointer rewrites per hit while keeping eviction decisions close enough to LRU that hits still land. The hypothesis's main bet — trade a small hit-rate risk for a substantial per-op time win — is only half-visible here, since the fitness vector currently scores only hit-rate; the per-op gain that motivated the swap will stay invisible until a time dimension is added.
