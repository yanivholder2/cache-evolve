# Variant v002 Insight

## Helped

- per-op-time: put median and tail latency dropped by roughly an order of magnitude. Replacing the separate small/main/ghost membership probes with one location-tagged index means a put does one lookup and updates the tag in place, instead of removing and inserting across several maps. This confirms the parent's put cost was mostly hash-map traffic.
- per-op-time: get p50 improved slightly, consistent with a single lookup plus a tag branch. The gain is small and could be partly noise.

## Hurt

- metadata-size: the S3Fifo struct footprint roughly doubled. The combined index and its tag state add fixed per-cache overhead that the parent did not carry. The per-entry size is unchanged, so only the fixed struct grew.
- per-op-time: get p99 got worse even though p50 improved. The unified index is probably larger and more heavily loaded, since it now also holds ghost keys, so lookups that miss cache lines or trigger probing show up in the tail.

## Takeaway

Consolidating queue membership into one tagged index fixed the put bottleneck that the earlier eviction-loop rewrite (the parent's change) could not reach. That shows the cost was in map operations rather than queue manipulation. The trade is a larger fixed metadata footprint and a worse get tail, and hit rate stayed identical, so the ordering and thresholds were preserved.
