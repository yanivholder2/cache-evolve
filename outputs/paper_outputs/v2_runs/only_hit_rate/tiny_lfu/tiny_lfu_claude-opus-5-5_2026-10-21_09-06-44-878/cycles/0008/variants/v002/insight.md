# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Letting a ghost-hit newcomer seen at least 3 times win against a victim up to 2 counts higher widens the admission path well beyond v001's exact-tie rule. The drop on workload-a was tiny, close to noise. It suggests that the extra ghost keys it admitted were, on net, slightly worse to keep than the victims they pushed out.

## Takeaway

Moving the ghost-hit rule from an exact tie to a margin of 2 (with a minimum frequency of 3) did not turn recency evidence into hits. It nudged workload-a's hit-rate slightly down. The sketch never ages its counters, so a victim that is 1–2 counts ahead of a returning ghost is usually a key with real long-term reuse. Overriding that gap based on "evicted recently and came back" swaps a proven key for a weaker one about as often as it helps. Ghost recency alone is not a better signal than unaged frequency here. Before tuning the admission contest further, the counts it compares probably need to become recency-aware (for example through aging). Note: the eval files and the hypothesis were readable, but the overlay source could not be listed in this session. This reading assumes the predicate was implemented as H02 specifies.
