# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Weighting the controller step by ghost-match freshness — jumping straight to the step-4 cap on a fresh agreeing hit and committing a 2-step move on a fresh reversal — slightly degraded hit-rate versus the parent's gradual doubling/reset, so the freshness amplification overshoots rather than sharpens.

## Takeaway

On workload-a a ghost record's queue position carries no useful steering information beyond its `from_main` direction: skipping the ramp to hit max step immediately on "fresh" hits made the split move faster but not better, landing marginally below the parent. This reinforces cycle-0009's read that the controller is already well-aimed and converges fast enough — the remaining headroom is not in how hard or how soon to push, so modulating step magnitude by signal recency is a dead end for this trace.
