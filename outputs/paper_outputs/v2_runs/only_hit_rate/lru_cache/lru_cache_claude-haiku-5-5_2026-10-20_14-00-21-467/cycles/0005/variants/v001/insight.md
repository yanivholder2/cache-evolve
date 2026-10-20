# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Splitting the recency list into probation and protected segments, with eviction taken from probation first, left fewer slots for fresh inserts to be re-hit before they are evicted, so the incumbent's single-list retention of re-referenced keys was not improved on.

## Takeaway

On top of the v002 single recency list, the probation/protected split with probation-first eviction did not raise hit-rate on workload-a at size 2048; the change lowered it. The incumbent already keeps re-referenced keys, so segmentation only helps if the probation segment is large enough for new keys to earn a second hit, and the 80 percent protected cap leaves probation small. A follow-up would need to vary the protected cap or the demotion rule rather than repeat this split as written.
