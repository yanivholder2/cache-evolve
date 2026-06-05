# Variant v003 Insight

## Helped

(None)

## Hurt

- **hit-rate**: Raising the stash-tier promotion threshold from access_count ≥ 2 to ≥ 3 reduces the population of fast-tracked items in probing, requiring more hits to qualify for skip-ahead and adding friction to the promotion path.

## Takeaway

Tightening the tier-skip selectivity by requiring 3+ stash accesses (instead of 2+) before skipping to probing undermines hit rate despite the architectural intent to filter out false positives. The stricter gate apparently suppresses legitimate multi-access patterns that benefit from faster promotion; items that accessed twice in stash still had productive lives in probing but now face demotion back to target for re-qualification. This teaches that selectivity gains must be weighed against the cost of longer promotion latency—the ≥ 2 threshold in the parent better balanced filtering discipline with cache utility.

