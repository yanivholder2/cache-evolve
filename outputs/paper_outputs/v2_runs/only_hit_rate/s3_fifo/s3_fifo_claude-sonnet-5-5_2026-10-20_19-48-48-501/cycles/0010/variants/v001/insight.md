# Variant v001 Insight

## Helped

- (none) hit-rate did not move relative to the parent.

## Hurt

- (none) hit-rate did not regress; it is identical to the parent's at the reported precision.

## Takeaway

Moving the small-queue share to about 5% of capacity, with ghost length, promotion threshold and second-chance scan unchanged, produced no measurable change on this trace and cache size. Either the small-queue size is not a sensitive lever on this workload, or the change did not alter eviction behavior at this capacity (for example, the parent's size may already round to the same effective value). The hypothesis's aim of locating a sizing optimum is not served by this point, so future sizing probes should use a larger spread or confirm the effective queue lengths differ. I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness vectors.
