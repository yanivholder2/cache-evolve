# Variant v003 Insight

## Helped

(none)

## Hurt

- hit-rate: Expanding the hot zone from 1/3 to 1/2 reduced selectivity in item promotion, allowing less-frequently-accessed items to occupy protected space that could have been reserved for truly hot items, and the additional complexity of balancing zone sizes competed with effective cold-zone LRU eviction.

## Takeaway

Doubling the hot zone's size and adding explicit zone-balance logic backfired—the original 1/3 hot / 2/3 cold partition from v002 was actually better tuned to this workload. The v002 approach was more selective: items had to demonstrate reuse (freq ≥ 2) to earn protection in a smaller hot zone, allowing the larger cold zone to quickly shed one-time accesses. The bigger hot zone here diluted that selectivity, and the new eviction strategy's check for zone overflow interfered with the priority of cold-zone LRU eviction.
