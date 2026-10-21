# Variant v001 Insight

## Helped

- (none): hit-rate did not move relative to the parent, so raising the small-to-main promotion threshold from freq>0 to freq>=2 gave no measurable gain.

## Hurt

- (none): hit-rate did not regress either; keeping weakly-reused keys out of main neither helped nor cost anything on this trace.

## Takeaway

On this workload and cache size, the hit-rate is identical to the parent's, so the stricter promotion test (freq>=2) made no observable difference. Either the small queue rarely holds items that would have been hit exactly once, or the change did not alter the executed promotion path. The hypothesis itself flagged that earlier strictness tests may have been no-ops, so this result needs a check that the freq counter really reaches 2 while an item sits in small. Note: I could not read the overlay sources in this session (only Read/Write were usable), so this interpretation rests on the hypothesis and the fitness vectors, not on a code diff.
