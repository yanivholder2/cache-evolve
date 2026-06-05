# Variant v002 Insight

## Helped

- hit-rate: Stash-pressure-aware promotion strategy bypasses tier filtering when stash is over 75% occupied, reducing cascade evictions and enabling faster tier climbing for high-value items during congested periods.

## Hurt

(none)

## Takeaway

The adaptive stash-hit promotion strategy successfully improves hit rate by switching between aggressive tier-skipping (high occupancy) and careful tier filtering (low occupancy). When the stash bucket is congested, direct promotion to probing avoids eviction cascades; when capacity permits, tier-by-tier filtering selects better promotion candidates. This validates the hypothesis that optimal promotion depth should depend on cache pressure, not be fixed.
