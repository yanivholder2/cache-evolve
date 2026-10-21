# Variant v001 Insight

## Helped

- (none): No dimension improved. The ghost list did not recover enough hot keys to offset its costs on this trace.

## Hurt

- hit-rate: Keys re-inserted after a probation eviction now jump straight into protected, and the demotion loop pushes the oldest protected keys back to probation. On the scan-heavy LIRS trace, many ghost-hit keys are probably one-off or long-reuse-distance. They displace established protected keys, so the net effect is slightly negative.
- metadata-size / per-op-time: These are not in the fitness vector, so they were not measured. The change adds a `ghost_set` map, a `ghost_fifo` deque (up to 2x capacity, with lazy deletion), and a generation counter. Each probation eviction and each miss also pays for a ghost insert and lookup. This is inferred from the code.

## Takeaway

Ghost-based direct-to-protected admission was not a free win on top of the segmented LRU. Treating any key re-requested after eviction as hot, with no frequency or recency check, lets it displace proven protected entries. Because the protected segment is already 80% of capacity, there is little room for this. A gentler use of the ghost list, such as inserting at the probation MRU or adapting segment sizes as ARC does, would be a more plausible next try. The result comes from one trace at one cache size.
