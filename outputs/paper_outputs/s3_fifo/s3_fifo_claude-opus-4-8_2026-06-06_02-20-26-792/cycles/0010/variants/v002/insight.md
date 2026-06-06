# Variant v002 Insight

## Helped

- hit-rate: Applying main's move-to-front discipline to the small-queue hit branch (`remove(pos)` + saturating `freq` bump + `push_front`) makes `evict_small`'s `pop_back` shed the least-recently-accessed admission candidate instead of the oldest-inserted one, so a small key re-touched on workload-a's recency trace survives quick-demotion a little longer and converts into a hit.

## Hurt

## Takeaway

Extending the proven recency-reordering lever to the small admission queue — the one queue it had never touched — does raise hit-rate, but only barely. Unlike main, small is tiny (`capacity/24`, orbiting an adaptive split) and churns fast, so most admission candidates never earn a second small hit before eviction; reordering only rearranges a short tail of cold one-hit-wonders, leaving the strict `freq > 1` promotion gate to do the real retention work. The ordering discipline is correct here but near-saturated, confirming the small filter's hit-rate ceiling is set by its size and promotion test, not by its insertion-order eviction.
